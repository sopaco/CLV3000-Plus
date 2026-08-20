use crate::models::{RiskLevel, ScanItem, ScanProgress, ScanReport, TechStack};
use crate::settings::{
    agent_marker_files, agent_name_patterns, global_cache_rules, is_protected_system_path,
    project_marker_files, project_rules, AppSettings,
};
use chrono::{DateTime, Utc};
use walkdir::WalkDir;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use uuid::Uuid;

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
        let mut on_progress = ProgressThrottle::new(on_progress);
        let started = Instant::now();
        let mut items = Vec::new();
        let mut roots_scanned = Vec::new();
        let mut seen_paths: HashSet<PathBuf> = HashSet::new();

        on_progress.emit(
            ScanProgress {
                phase: "准备扫描".into(),
                current_path: None,
                items_found: 0,
                bytes_found: 0,
            },
            true,
        );

        // Global caches under home
        if let Some(home) = directories::UserDirs::new().map(|d| d.home_dir().to_path_buf()) {
            for rule in global_cache_rules() {
                let path = home.join(rule.relative);
                self.try_add_rule_path(
                    &path,
                    None,
                    rule,
                    &mut items,
                    &mut seen_paths,
                    &mut on_progress,
                );
            }
        }

        // User-configured scan roots
        for root in &self.settings.scan_paths {
            if !root.exists() || is_protected_system_path(root) {
                continue;
            }
            roots_scanned.push(root.clone());

            on_progress.emit(
                ScanProgress {
                    phase: format!("扫描 {}", root.display()),
                    current_path: Some(root.clone()),
                    items_found: items.len(),
                    bytes_found: items.iter().map(|i| i.size_bytes).sum(),
                },
                true,
            );

            self.scan_tree(root, &mut items, &mut seen_paths, &mut on_progress);
        }

        let mut report = ScanReport {
            items,
            agent_projects: Vec::new(),
            scanned_at: Some(Utc::now()),
            scan_duration_ms: started.elapsed().as_millis() as u64,
            roots_scanned,
        };

        if self.settings.include_agent_heuristics {
            report.agent_projects =
                crate::agent::detect_agent_projects(&report.items, &self.settings.scan_paths);
            self.tag_agent_items(&mut report);
        }

        // Default selection: safe items in simple mode
        for item in &mut report.items {
            item.selected = item.risk == RiskLevel::Safe;
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
        items: &mut Vec<ScanItem>,
        seen: &mut HashSet<PathBuf>,
        on_progress: &mut ProgressThrottle<F>,
    ) where
        F: FnMut(ScanProgress),
    {
        let rules = project_rules();
        let max_depth = 8;

        for entry in WalkDir::new(root)
            .follow_links(false)
            .min_depth(1)
            .max_depth(max_depth)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if is_protected_system_path(path) || is_inaccessible_path(path) {
                continue;
            }

            if entry.depth() > 0 && entry.depth() % 5 == 0 {
                on_progress.emit(
                    ScanProgress {
                        phase: "扫描项目目录".into(),
                        current_path: Some(path.to_path_buf()),
                        items_found: items.len(),
                        bytes_found: items.iter().map(|i| i.size_bytes).sum(),
                    },
                    false,
                );
            }

            let file_name = entry
                .file_name()
                .to_str()
                .unwrap_or_default()
                .to_string();

            for rule in rules {
                if file_name == rule.relative {
                    let project_root = find_project_root(&path);
                    self.try_add_rule_path(
                        &path,
                        project_root,
                        rule,
                        items,
                        seen,
                        on_progress,
                    );
                }
            }

            // CMake build dirs pattern cmake-build-*
            if file_name.starts_with("cmake-build-") {
                let project_root = find_project_root(&path);
                let rule = project_rules()
                    .iter()
                    .find(|r| r.relative == "cmake-build-debug")
                    .unwrap();
                self.try_add_rule_path(&path, project_root, rule, items, seen, on_progress);
            }
        }
    }

    fn try_add_rule_path<F>(
        &self,
        path: &Path,
        project_root: Option<PathBuf>,
        rule: &crate::settings::CleanupRule,
        items: &mut Vec<ScanItem>,
        seen: &mut HashSet<PathBuf>,
        on_progress: &mut ProgressThrottle<F>,
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

        let size = dir_size(path);
        if size == 0 {
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
        items.push(ScanItem {
            id: Uuid::new_v4().to_string(),
            path: key,
            name,
            size_bytes: size,
            stack: rule.stack,
            risk,
            category: rule.category.to_string(),
            description: rule.description.to_string(),
            project_root,
            last_modified,
            selected: false,
        });

        on_progress.emit(
            ScanProgress {
                phase: "发现可清理项".into(),
                current_path: Some(path.to_path_buf()),
                items_found: items.len(),
                bytes_found: items.iter().map(|i| i.size_bytes).sum(),
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

fn dir_size(path: &Path) -> u64 {
    if path.is_file() {
        return path.metadata().map(|m| m.len()).unwrap_or(0);
    }
    dir_size_dir(path)
}

const DIR_SIZE_MAX_ENTRIES: usize = 100_000;
const SKIP_DIR_NAMES: &[&str] = &[".git", ".svn", ".hg"];

fn dir_size_dir(root: &Path) -> u64 {
    let mut total = 0u64;
    let mut entries_seen = 0usize;
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let read_dir = match std::fs::read_dir(&dir) {
            Ok(read_dir) => read_dir,
            Err(_) => continue,
        };

        for entry in read_dir.filter_map(|e| e.ok()) {
            entries_seen += 1;
            if entries_seen > DIR_SIZE_MAX_ENTRIES {
                return total;
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

    total
}

fn should_skip_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| SKIP_DIR_NAMES.contains(&name))
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

pub fn is_agent_project_path(path: &Path) -> (bool, String) {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    for pattern in agent_name_patterns() {
        if name.contains(pattern) {
            return (true, format!("目录名包含「{pattern}」"));
        }
    }

    for marker in agent_marker_files() {
        if path.join(marker).exists() {
            return (true, format!("存在 Agent 标记 {marker}"));
        }
    }

    (false, String::new())
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
