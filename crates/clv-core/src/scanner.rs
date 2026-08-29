use crate::locale::{
    resolve_language, scan_phase_agent_sessions, scan_phase_discovering, scan_phase_preparing,
    scan_phase_scanning_path, scan_phase_scanning_projects,
};
use crate::messages::AgentReasonPart;
use crate::models::{RiskLevel, ScanItem, ScanProgress, ScanReport, TechStack};
use crate::settings::{
    agent_marker_files, agent_name_patterns, global_cache_rules, is_protected_system_path,
    project_marker_files, project_rules, resolve_global_path, AppSettings, CleanupRule,
};
use chrono::{DateTime, Utc};
use walkdir::WalkDir;
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Ignore scan hits smaller than this threshold to reduce list noise.
pub const MIN_SCAN_ITEM_BYTES: u64 = 1024 * 1024;

struct ProgressThrottle<F> {
    inner: F,
    last_emit: Instant,
    interval: Duration,
}

impl<F: FnMut(ScanProgress)> ProgressThrottle<F> {
    fn new(inner: F) -> Self {
        Self {
            inner,
            last_emit: Instant::now() - Duration::from_secs(1),
            interval: Duration::from_millis(300),
        }
    }

    fn emit(&mut self, progress: ScanProgress, force: bool) {
        if force || self.last_emit.elapsed() >= self.interval {
            (self.inner)(progress);
            self.last_emit = Instant::now();
        }
    }
}

pub struct Scanner {
    settings: AppSettings,
}

impl Scanner {
    pub fn new(settings: AppSettings) -> Self {
        Self { settings }
    }

    pub fn scan<F>(&self, on_progress: F) -> ScanReport
    where
        F: FnMut(ScanProgress),
    {
        self.scan_cancellable(on_progress, &AtomicBool::new(false))
    }

    pub fn scan_cancellable<F>(&self, on_progress: F, cancel: &AtomicBool) -> ScanReport
    where
        F: FnMut(ScanProgress),
    {
        let lang = resolve_language(self.settings.language);
        let mut on_progress = ProgressThrottle::new(on_progress);
        let started = Instant::now();
        let mut items = Vec::new();
        let mut roots_scanned = Vec::new();
        let mut seen_paths: HashSet<PathBuf> = HashSet::new();
        let mut bytes_found = 0u64;
        let mut sizes_truncated = false;
        let mut agent_roots: HashSet<PathBuf> = HashSet::new();
        let mut large_files: Vec<crate::large_files::LargeFileEntry> = Vec::new();
        let mut cancelled = false;

        on_progress.emit(
            ScanProgress {
                phase: scan_phase_preparing(lang),
                current_path: None,
                items_found: 0,
                bytes_found: 0,
            },
            true,
        );

        // Global caches under home / known env locations
        for rule in global_cache_rules() {
            if cancel.load(Ordering::Relaxed) {
                cancelled = true;
                break;
            }
            let Some(path) = resolve_global_path(rule.relative) else {
                continue;
            };
            self.try_add_rule_path(
                &path,
                None,
                rule,
                &mut items,
                &mut seen_paths,
                None,
                &mut bytes_found,
                &mut sizes_truncated,
                &mut on_progress,
                cancel,
            );
        }

        if !cancelled && self.settings.include_agent_heuristics {
            for target in crate::agent_sessions::discover_agent_session_targets() {
                if cancel.load(Ordering::Relaxed) {
                    cancelled = true;
                    break;
                }
                self.try_add_agent_session(
                    &target,
                    &mut items,
                    &mut seen_paths,
                    &mut bytes_found,
                    &mut sizes_truncated,
                    &mut on_progress,
                    cancel,
                );
            }
        }

        // User-configured scan roots
        if !cancelled {
            for root in &self.settings.scan_paths {
                if cancel.load(Ordering::Relaxed) {
                    cancelled = true;
                    break;
                }
                if !root.exists() || is_protected_system_path(root) {
                    continue;
                }
                roots_scanned.push(root.clone());

                on_progress.emit(
                    ScanProgress {
                        phase: scan_phase_scanning_path(lang, root),
                        current_path: Some(root.clone()),
                        items_found: items.len(),
                        bytes_found,
                    },
                    true,
                );

                if !self.scan_tree(
                    root,
                    lang,
                    &mut items,
                    &mut seen_paths,
                    &mut bytes_found,
                    &mut sizes_truncated,
                    &mut agent_roots,
                    &mut large_files,
                    &mut on_progress,
                    cancel,
                ) {
                    cancelled = true;
                    break;
                }
            }
        }

        items = drop_nested_items(items);

        let mut report = ScanReport {
            items,
            agent_projects: Vec::new(),
            large_files: crate::large_files::finalize_large_files(large_files),
            scanned_at: Some(Utc::now()),
            scan_duration_ms: started.elapsed().as_millis() as u64,
            roots_scanned,
            cancelled,
            sizes_truncated,
        };

        if self.settings.include_agent_heuristics {
            let extra: Vec<PathBuf> = agent_roots.into_iter().collect();
            report.agent_projects =
                crate::agent::detect_agent_projects(&report.items, &extra);
            self.tag_agent_items(&mut report);
        }

        report
    }

    fn tag_agent_items(&self, report: &mut ScanReport) {
        let agent_roots: HashSet<PathBuf> = report
            .agent_projects
            .iter()
            .map(|p| p.path.clone())
            .collect();
        for item in &mut report.items {
            if let Some(root) = &item.project_root {
                if agent_roots.contains(root) {
                    item.stack = TechStack::Agent;
                }
            }
        }
    }

    fn scan_tree<F>(
        &self,
        root: &Path,
        lang: crate::locale::Language,
        items: &mut Vec<ScanItem>,
        seen: &mut HashSet<PathBuf>,
        bytes_found: &mut u64,
        sizes_truncated: &mut bool,
        agent_roots: &mut HashSet<PathBuf>,
        large_files: &mut Vec<crate::large_files::LargeFileEntry>,
        on_progress: &mut ProgressThrottle<F>,
        cancel: &AtomicBool,
    ) -> bool
    where
        F: FnMut(ScanProgress),
    {
        let rules = project_rules();
        let max_depth = 8;
        let prune_roots: RefCell<HashSet<PathBuf>> = RefCell::new(HashSet::new());

        for entry in WalkDir::new(root)
            .follow_links(false)
            .min_depth(1)
            .max_depth(max_depth)
            .into_iter()
            .filter_entry(|e| {
                let path = e.path();
                !should_skip_dir(path) && !is_under_prune_root(path, &prune_roots.borrow())
            })
            .filter_map(|e| e.ok())
        {
            if cancel.load(Ordering::Relaxed) {
                return false;
            }

            let path = entry.path();
            if is_protected_system_path(path) || is_inaccessible_path(path) {
                continue;
            }

            let file_name = entry.file_name().to_str().unwrap_or_default();
            maybe_record_agent_root(path, file_name, agent_roots);

            if entry.file_type().is_file() {
                if let Ok(meta) = entry.metadata() {
                    crate::large_files::consider_large_file(
                        path,
                        &meta,
                        entry.depth(),
                        large_files,
                    );
                }
            }

            if entry.depth() > 0 && entry.depth() % 5 == 0 {
                on_progress.emit(
                    ScanProgress {
                        phase: scan_phase_scanning_projects(lang),
                        current_path: Some(path.to_path_buf()),
                        items_found: items.len(),
                        bytes_found: *bytes_found,
                    },
                    false,
                );
            }

            for rule in rules {
                if !rule_matches_dir_name(file_name, rule) {
                    continue;
                }
                if !rule_matches_parent(entry.path(), rule) {
                    continue;
                }
                let project_root = find_project_root(&path);
                if !rule_matches_marker(project_root.as_deref(), rule) {
                    continue;
                }
                self.try_add_rule_path(
                    &path,
                    project_root,
                    rule,
                    items,
                    seen,
                    Some(&prune_roots),
                    bytes_found,
                    sizes_truncated,
                    on_progress,
                    cancel,
                );
            }
        }
        true
    }

    fn try_add_agent_session<F>(
        &self,
        target: &crate::agent_sessions::AgentSessionTarget,
        items: &mut Vec<ScanItem>,
        seen: &mut HashSet<PathBuf>,
        bytes_found: &mut u64,
        sizes_truncated: &mut bool,
        on_progress: &mut ProgressThrottle<F>,
        cancel: &AtomicBool,
    ) where
        F: FnMut(ScanProgress),
    {
        let path = &target.path;
        if !path.exists() {
            return;
        }
        let key = path.to_path_buf();
        if seen.contains(&key) || is_protected_system_path(path) {
            return;
        }

        let (size, truncated) = dir_size(path, cancel);
        *sizes_truncated |= truncated;
        if size == 0 {
            return;
        }

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        seen.insert(key.clone());
        *bytes_found += size;
        items.push(ScanItem {
            id: Uuid::new_v4().to_string(),
            path: key,
            name,
            size_bytes: size,
            stack: target.stack,
            risk: target.risk,
            category: target.category,
            description: target.description,
            project_root: None,
            last_modified: last_modified(path),
        });

        on_progress.emit(
            ScanProgress {
                phase: scan_phase_agent_sessions(resolve_language(self.settings.language)),
                current_path: Some(path.to_path_buf()),
                items_found: items.len(),
                bytes_found: *bytes_found,
            },
            false,
        );
    }

    fn try_add_rule_path<F>(
        &self,
        path: &Path,
        project_root: Option<PathBuf>,
        rule: &crate::settings::CleanupRule,
        items: &mut Vec<ScanItem>,
        seen: &mut HashSet<PathBuf>,
        prune_roots: Option<&RefCell<HashSet<PathBuf>>>,
        bytes_found: &mut u64,
        sizes_truncated: &mut bool,
        on_progress: &mut ProgressThrottle<F>,
        cancel: &AtomicBool,
    ) where
        F: FnMut(ScanProgress),
    {
        if !path.exists() {
            return;
        }
        // Avoid canonicalize() — on macOS it can block on iCloud / network paths.
        let key = path.to_path_buf();
        if seen.contains(&key) || is_protected_system_path(path) {
            return;
        }

        let (size, truncated) = dir_size(path, cancel);
        *sizes_truncated |= truncated;
        if size < MIN_SCAN_ITEM_BYTES {
            return;
        }

        let mut risk = rule.risk;
        if let Some(root) = &project_root {
            if is_likely_active_project(root) {
                if risk == RiskLevel::Safe {
                    risk = RiskLevel::Caution;
                }
            }
        }

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        let last_modified = last_modified(path);

        seen.insert(key.clone());
        if path.is_dir() {
            if let Some(prune_roots) = prune_roots {
                prune_roots.borrow_mut().insert(key.clone());
            }
        }
        *bytes_found += size;
        items.push(ScanItem {
            id: Uuid::new_v4().to_string(),
            path: key,
            name,
            size_bytes: size,
            stack: rule.stack,
            risk,
            category: rule.category,
            description: rule.description,
            project_root,
            last_modified,
        });

        on_progress.emit(
            ScanProgress {
                phase: scan_phase_discovering(resolve_language(self.settings.language)),
                current_path: Some(path.to_path_buf()),
                items_found: items.len(),
                bytes_found: *bytes_found,
            },
            false,
        );
    }
}

fn is_inaccessible_path(path: &Path) -> bool {
    #[cfg(target_os = "macos")]
    {
        let s = path.to_string_lossy();
        if s.contains(".icloud") || s.contains("com~apple~CloudDocs") {
            return true;
        }
    }
    let _ = path;
    false
}

fn dir_size(path: &Path, cancel: &AtomicBool) -> (u64, bool) {
    if path.is_file() {
        return (path.metadata().map(|m| m.len()).unwrap_or(0), false);
    }
    dir_size_dir(path, cancel)
}

const DIR_SIZE_MAX_ENTRIES: usize = 100_000;

fn dir_size_dir(root: &Path, cancel: &AtomicBool) -> (u64, bool) {
    let mut total = 0u64;
    let mut entries_seen = 0usize;
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        if cancel.load(Ordering::Relaxed) {
            return (total, false);
        }
        let read_dir = match std::fs::read_dir(&dir) {
            Ok(read_dir) => read_dir,
            Err(_) => continue,
        };

        for entry in read_dir.filter_map(|e| e.ok()) {
            entries_seen += 1;
            if entries_seen > DIR_SIZE_MAX_ENTRIES {
                return (total, true);
            }

            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(_) => continue,
            };

            if file_type.is_dir() {
                if should_skip_dir(&path) {
                    continue;
                }
                stack.push(path);
            } else if file_type.is_file() {
                total += entry.metadata().map(|m| m.len()).unwrap_or(0);
            } else if file_type.is_symlink() {
                if let Ok(meta) = entry.metadata() {
                    total += meta.len();
                }
            }
        }
    }

    (total, false)
}

fn should_skip_dir(path: &Path) -> bool {
    crate::paths::is_scan_skip_dir(path)
}

/// True when `path` is a strict descendant of a directory already matched as a cleanup item.
fn is_under_prune_root(path: &Path, prune_roots: &HashSet<PathBuf>) -> bool {
    prune_roots
        .iter()
        .any(|root| path != root.as_path() && path.starts_with(root))
}

/// Drop items whose path is nested inside another item (e.g. `node_modules/.cache` under `node_modules`).
fn drop_nested_items(mut items: Vec<ScanItem>) -> Vec<ScanItem> {
    items.sort_by(|a, b| a.path.cmp(&b.path));
    let mut kept: Vec<ScanItem> = Vec::with_capacity(items.len());
    for item in items {
        if kept
            .last()
            .is_some_and(|parent| item.path.starts_with(&parent.path) && item.path != parent.path)
        {
            continue;
        }
        kept.push(item);
    }
    kept
}

fn maybe_record_agent_root(path: &Path, file_name: &str, agent_roots: &mut HashSet<PathBuf>) {
    for marker in agent_marker_files() {
        if file_name == *marker {
            if let Some(parent) = path.parent() {
                agent_roots.insert(parent.to_path_buf());
            }
            return;
        }
    }
}

pub fn rule_matches_dir_name(file_name: &str, rule: &CleanupRule) -> bool {
    if let Some(pattern) = rule.relative_prefix {
        if let Some(suffix) = pattern.strip_prefix('*') {
            return file_name.ends_with(suffix);
        }
        return file_name.starts_with(pattern);
    }
    !rule.relative.is_empty() && file_name == rule.relative
}

pub fn rule_matches_parent(path: &Path, rule: &CleanupRule) -> bool {
    let Some(required_parent) = rule.requires_parent else {
        return true;
    };
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        == Some(required_parent)
}

pub fn rule_matches_marker(project_root: Option<&Path>, rule: &CleanupRule) -> bool {
    let Some(marker) = rule.requires_marker else {
        return true;
    };
    let Some(root) = project_root else {
        return false;
    };
    if marker.contains('*') {
        has_glob(root, marker)
    } else {
        root.join(marker).exists()
    }
}

fn last_modified(path: &Path) -> Option<DateTime<Utc>> {
    let meta = path.metadata().ok()?;
    let modified = meta.modified().ok()?;
    Some(DateTime::<Utc>::from(modified))
}

fn find_project_root(path: &Path) -> Option<PathBuf> {
    let mut current = path.parent()?;
    for _ in 0..6 {
        if is_project_root(current) {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
    path.parent().map(|p| p.to_path_buf())
}

fn is_project_root(path: &Path) -> bool {
    for (marker, _) in project_marker_files() {
        if marker.contains('*') {
            continue;
        }
        if path.join(marker).exists() {
            return true;
        }
    }
    false
}

fn is_likely_active_project(root: &Path) -> bool {
    if let Some(modified) = last_modified(root) {
        let days = (Utc::now() - modified).num_days();
        return days < 3;
    }
    false
}

pub fn agent_path_signals(path: &Path) -> (bool, bool, Vec<AgentReasonPart>) {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let mut reasons = Vec::new();
    let mut name_hit = false;
    for pattern in agent_name_patterns() {
        if name.contains(pattern) {
            name_hit = true;
            reasons.push(AgentReasonPart::NameContainsPattern(pattern.to_string()));
            break;
        }
    }

    let mut marker_hit = false;
    for marker in agent_marker_files() {
        if path.join(marker).exists() {
            marker_hit = true;
            reasons.push(AgentReasonPart::HasAgentMarker(marker.to_string()));
            break;
        }
    }

    (name_hit, marker_hit, reasons)
}

pub fn is_agent_project_path(path: &Path) -> (bool, Vec<AgentReasonPart>) {
    let (name_hit, marker_hit, reasons) = agent_path_signals(path);
    (name_hit || marker_hit, reasons)
}

pub fn detect_project_stacks(root: &Path) -> Vec<TechStack> {
    let mut stacks = HashSet::new();
    for (marker, stack) in project_marker_files() {
        if marker.ends_with(".csproj") {
            if has_glob(root, "*.csproj") {
                stacks.insert(*stack);
            }
        } else if root.join(marker).exists() {
            stacks.insert(*stack);
        }
    }
    if root.join("settings.gradle.kts").exists() || root.join("shared/build.gradle.kts").exists() {
        stacks.insert(TechStack::Kmp);
    }
    stacks.into_iter().collect()
}

fn has_glob(dir: &Path, pattern: &str) -> bool {
    let ext = pattern.trim_start_matches('*');
    std::fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .any(|e| e.path().extension().is_some_and(|x| ext.trim_start_matches('.') == x.to_string_lossy()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::CleanupCategory;
    use crate::messages::RuleDescription;

    fn item(path: &str) -> ScanItem {
        ScanItem {
            id: path.to_string(),
            path: PathBuf::from(path),
            name: path.to_string(),
            size_bytes: 1,
            stack: TechStack::Rust,
            risk: RiskLevel::Safe,
            category: CleanupCategory::CompileCache,
            description: RuleDescription::R001,
            project_root: None,
            last_modified: None,
        }
    }

    #[test]
    fn drop_nested_items_is_linear_and_keeps_parents() {
        let kept = drop_nested_items(vec![
            item("/proj/target/debug"),
            item("/proj/target"),
            item("/other/node_modules"),
            item("/other/node_modules/.cache"),
        ]);
        let paths: Vec<_> = kept.iter().map(|i| i.path.to_string_lossy().to_string()).collect();
        assert_eq!(
            paths,
            vec!["/other/node_modules".to_string(), "/proj/target".to_string()]
        );
    }
}
