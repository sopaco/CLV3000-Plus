//! Application uninstaller domain.
//!
//! Enumerates installed and stray `.app` bundles, matches each app's residue in
//! the fixed `~/Library` subdirectories, and surfaces broken CLI symlinks.
//! Related entries are expressed as [`ScanItem`]s so cleanup flows through the
//! existing [`crate::cleanup::CleanupExecutor`] recycle-area pipeline without
//! a parallel executor.
//!
//! The module stays platform-agnostic: standard roots, bin dirs and the
//! Library layout are injected via [`AppScanOptions`] so tests can run
//! against temp directories on any OS.

use crate::category::CleanupCategory;
use crate::messages::RuleDescription;
use crate::models::{RiskLevel, ScanItem, TechStack};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use walkdir::WalkDir;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Where an app bundle was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppSource {
    /// `/Applications` — standard system install location.
    Applications,
    /// `~/Applications` — per-user install location.
    UserApplications,
    /// `.app` bundle outside the standard locations (home shallow scan).
    StrayBundle,
    /// No bundle left; virtual entry aggregated from orphaned residue (M1.5).
    Orphan,
    /// Windows: discovered from the registry uninstall keys (M2).
    Registry,
}

/// How confidently a Library entry was attributed to an app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchQuality {
    /// Entry name equals the app's bundle id (or the group-container suffix).
    BundleId,
    /// Entry name equals the app's normalized display name.
    Name,
    /// Entry name merely contains the app name — expert-mode display only.
    Fuzzy,
}

/// One Library-residue entry bound to its match confidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedItem {
    pub item: ScanItem,
    pub quality: MatchQuality,
}

impl RelatedItem {
    /// Default checkbox state per PRD §5.4: precise matches with `Safe` risk
    /// are pre-checked; fuzzy matches never are.
    pub fn is_default_selected(&self) -> bool {
        self.quality != MatchQuality::Fuzzy && self.item.risk == RiskLevel::Safe
    }
}

/// An app-centric cleanup target: the bundle itself plus its Library residue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    /// Stable id: bundle id when known, otherwise the bundle path.
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    /// Windows: `Publisher` from the uninstall key; unused on macOS.
    #[serde(default)]
    pub publisher: Option<String>,
    pub bundle_id: Option<String>,
    /// `None` for `Orphan` virtual entries.
    pub app_path: Option<PathBuf>,
    /// Size of the `.app` bundle alone.
    pub binary_size_bytes: u64,
    /// Bundle + related residue.
    pub size_bytes: u64,
    pub source: AppSource,
    /// Risk of removing the bundle; drives the related items' risk too.
    pub risk: RiskLevel,
    pub is_running: bool,
    pub related_items: Vec<RelatedItem>,
    /// Windows: the registry `UninstallString` used to launch the official
    /// uninstaller (M2). `None` on macOS and for orphan entities.
    #[serde(default)]
    pub uninstall_string: Option<String>,
}

impl AppEntry {
    /// The bundle itself as a cleanable item. `None` for orphan entries.
    pub fn binary_scan_item(&self) -> Option<ScanItem> {
        let path = self.app_path.clone()?;
        Some(ScanItem {
            id: format!("{}:binary", self.id),
            path,
            name: self.name.clone(),
            size_bytes: self.binary_size_bytes,
            stack: TechStack::Other,
            risk: self.risk,
            category: CleanupCategory::AppBinary,
            description: RuleDescription::R141,
            project_root: None,
            last_modified: None,
        })
    }

    /// Ids of related items that should be pre-checked after a scan.
    pub fn default_selected_related_ids(&self) -> HashSet<String> {
        self.related_items
            .iter()
            .filter(|r| r.is_default_selected())
            .map(|r| r.item.id.clone())
            .collect()
    }
}

/// Result of an on-demand app scan. Kept separate from the rule-driven
/// `ScanReport` so dashboard figures never absorb app-residue bytes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppScanReport {
    pub apps: Vec<AppEntry>,
    pub stray_executables: Vec<ScanItem>,
    pub scanned_at: Option<DateTime<Utc>>,
    pub scan_duration_ms: u64,
    #[serde(default)]
    pub cancelled: bool,
}

impl AppScanReport {
    /// Bytes of Library residue across all apps (excludes bundles).
    pub fn related_residue_bytes(&self) -> u64 {
        self.apps.iter().map(|a| a.residue_bytes()).sum()
    }

    /// Whether the cached report is recent enough to skip a rescan.
    pub fn is_fresh_within(&self, hours: i64) -> bool {
        self.scanned_at
            .is_some_and(|at| Utc::now() - at < chrono::Duration::hours(hours))
    }

    /// Reconciles a cached report with the filesystem after a cleanup.
    ///
    /// A successfully removed bundle leaves its entry behind when residue is
    /// still pending (residue items are `Caution` and not pre-checked). Left
    /// alone the row keeps claiming to be an installed app and keeps summing
    /// the deleted bundle into `size_bytes`, which reads as "the app was not
    /// deleted". Demoting such entries to residue-only (`AppSource::Orphan`,
    /// no `app_path`, no bundle bytes) makes the list reflect reality while
    /// keeping the leftover data reachable for cleanup.
    pub fn normalize_install_state(&mut self) {
        for app in &mut self.apps {
            if app.app_path.as_ref().is_some_and(|path| !path.exists()) {
                app.app_path = None;
                app.binary_size_bytes = 0;
                app.is_running = false;
                app.uninstall_string = None;
                app.source = AppSource::Orphan;
            }
            app.size_bytes = app.binary_size_bytes + app.residue_bytes();
        }
        // Drop entries that no longer represent anything on disk.
        self.apps
            .retain(|app| app.app_path.is_some() || !app.related_items.is_empty());
    }
}

impl AppEntry {
    pub fn residue_bytes(&self) -> u64 {
        self.related_items.iter().map(|r| r.item.size_bytes).sum()
    }

    pub fn size_human(&self) -> String {
        crate::models::format_bytes(self.size_bytes)
    }
}

#[derive(Debug, Clone)]
pub struct AppScanProgress {
    pub current_path: Option<PathBuf>,
    pub apps_found: usize,
}

/// Inputs for [`scan_apps`]. Every field has a real-machine default; tests
/// override them with temp directories.
#[derive(Debug, Clone, Default)]
pub struct AppScanOptions {
    /// Roots enumerated as "installed apps" (depth 2). Empty → platform
    /// defaults (`/Applications`, `/System/Applications`, `~/Applications`).
    pub app_roots: Vec<PathBuf>,
    /// Home dir used for the stray-bundle scan and to locate `~/Library`.
    pub home_dir: Option<PathBuf>,
    /// Overrides the Library root (tests point this at a temp dir).
    pub library_dir: Option<PathBuf>,
    /// Bin dirs scanned for broken CLI symlinks. Empty → platform defaults.
    pub bin_dirs: Vec<PathBuf>,
    /// Executable paths of currently running processes; an app is running
    /// when any of these starts with its bundle path.
    pub running_paths: Vec<PathBuf>,
    /// Windows registry / shortcut / AppData inputs, collected by
    /// `clv-platform` (`cfg(windows)`). Empty on macOS, which keeps the
    /// matching logic platform-agnostic and unit-testable everywhere.
    pub windows: WindowsScanInputs,
}

// ── Windows inputs (M2) ──────────────────────────────────────────────────────

/// Registry hive a Windows entry belongs to. Machine-wide (`HKLM`) residue is
/// display-only; only per-user (`HKCU`) keys are ever deleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistryHive {
    Machine,
    User,
}

impl RegistryHive {
    /// Token used at the head of the synthetic registry path.
    pub fn token(self) -> &'static str {
        match self {
            Self::Machine => "HKLM",
            Self::User => "HKCU",
        }
    }
}

/// One installed app discovered from
/// `...\CurrentVersion\Uninstall` (HKLM, HKLM\WOW6432Node or HKCU).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryAppRecord {
    pub display_name: String,
    pub display_version: Option<String>,
    pub publisher: Option<String>,
    pub install_location: Option<PathBuf>,
    pub uninstall_string: Option<String>,
    /// `EstimatedSize` from the registry, already converted to bytes.
    pub estimated_size_bytes: u64,
    pub hive: RegistryHive,
    /// Full key path, used as a stable id (`HKLM\SOFTWARE\...\App`).
    pub key_path: String,
}

/// A `Software\<vendor>` subkey that may belong to an uninstalled app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryResidueCandidate {
    pub name: String,
    pub key_path: String,
    pub hive: RegistryHive,
}

/// A Start Menu / Desktop `.lnk`. `target` is `None` when the platform uses
/// the filename heuristic instead of COM `IShellLink` resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutCandidate {
    pub name: String,
    pub path: PathBuf,
    pub target: Option<PathBuf>,
    pub size_bytes: u64,
}

/// One AppData / ProgramData root whose child directories are matched
/// against installed apps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidueRoot {
    pub path: PathBuf,
    /// Category assigned to matched child directories.
    pub category: CleanupCategory,
}

/// All Windows-specific scan inputs, collected once by the platform layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowsScanInputs {
    pub registry_apps: Vec<RegistryAppRecord>,
    pub registry_residue: Vec<RegistryResidueCandidate>,
    pub shortcuts: Vec<ShortcutCandidate>,
    pub residue_roots: Vec<ResidueRoot>,
}

impl WindowsScanInputs {
    pub fn is_empty(&self) -> bool {
        self.registry_apps.is_empty()
            && self.registry_residue.is_empty()
            && self.shortcuts.is_empty()
            && self.residue_roots.is_empty()
    }
}

// ── Scan ─────────────────────────────────────────────────────────────────────

/// Runs the full app scan: installed bundles → stray bundles → Library
/// matching → broken symlinks. Cancellable between apps and inside size
/// walks.
pub fn scan_apps(
    options: &AppScanOptions,
    cancel: &AtomicBool,
    mut on_progress: impl FnMut(AppScanProgress),
) -> AppScanReport {
    let started = Instant::now();
    let mut report = AppScanReport::default();

    let home = options
        .home_dir
        .clone()
        .or_else(crate::paths::user_home_dir);
    let library = options
        .library_dir
        .clone()
        .or_else(|| home.as_ref().map(|h| h.join("Library")));

    let roots: Vec<(PathBuf, AppSource)> = if options.app_roots.is_empty() {
        default_app_roots(home.as_deref())
    } else {
        options
            .app_roots
            .iter()
            .map(|r| (r.clone(), AppSource::Applications))
            .collect()
    };

    // 1. Installed bundles from standard roots.
    let mut apps: Vec<AppEntry> = Vec::new();
    for (root, source) in roots {
        if cancel.load(Ordering::Relaxed) {
            report.cancelled = true;
        }
        if report.cancelled {
            break;
        }
        for bundle in collect_bundles_in_root(&root) {
            if cancel.load(Ordering::Relaxed) {
                report.cancelled = true;
                break;
            }
            if let Some(entry) = build_app_entry(&bundle, source, options) {
                on_progress(AppScanProgress {
                    current_path: Some(bundle.clone()),
                    apps_found: apps.len() + 1,
                });
                apps.push(entry);
            }
        }
    }

    // 1b. Windows: registry-discovered apps (injected by clv-platform).
    //     macOS leaves `windows` empty, so this branch is a no-op there.
    if !report.cancelled {
        for record in &options.windows.registry_apps {
            if cancel.load(Ordering::Relaxed) {
                report.cancelled = true;
                break;
            }
            let entry = build_registry_app_entry(record, options);
            on_progress(AppScanProgress {
                current_path: entry.app_path.clone(),
                apps_found: apps.len() + 1,
            });
            apps.push(entry);
        }
    }

    // 2. Stray bundles: shallow home walk outside the standard roots.
    if !report.cancelled {
        if let Some(home) = &home {
            let standard: Vec<PathBuf> = default_app_roots(Some(home))
                .into_iter()
                .map(|(p, _)| p)
                .collect();
            for bundle in scan_stray_bundles(home, &standard, cancel) {
                if cancel.load(Ordering::Relaxed) {
                    report.cancelled = true;
                    break;
                }
                if let Some(entry) = build_app_entry(&bundle, AppSource::StrayBundle, options) {
                    on_progress(AppScanProgress {
                        current_path: Some(bundle),
                        apps_found: apps.len() + 1,
                    });
                    apps.push(entry);
                }
            }
        }
    }

    // 3. Attribute Library residue to the apps found above; unmatched
    //    entries come back as orphan candidates.
    let mut orphan_candidates: Vec<OrphanCandidate> = Vec::new();
    if !report.cancelled {
        if let Some(library) = &library {
            orphan_candidates = match_library_residue(library, &mut apps, cancel);
        }
    }

    // 3.5 Orphaned residue (M1.5): unmatched entries past the triple gate
    //     (Apple blacklist + ≥10MB + 90 days untouched) surface as virtual
    //     `Orphan` entities with no bundle of their own.
    if !report.cancelled {
        on_progress(AppScanProgress {
            current_path: None,
            apps_found: apps.len(),
        });
        apps.extend(build_orphan_entries(&orphan_candidates, cancel));
    }

    // 3.6 Windows residue (M2): registry keys, shortcuts and AppData
    //     directories attributed to the apps above.
    if !report.cancelled && !options.windows.is_empty() {
        match_windows_residue(&options.windows, &mut apps, cancel);
    }

    // 4. Broken CLI symlinks.
    if !report.cancelled {
        let bin_dirs: Vec<PathBuf> = if options.bin_dirs.is_empty() {
            default_bin_dirs(home.as_deref())
        } else {
            options.bin_dirs.clone()
        };
        report.stray_executables = scan_broken_symlinks(&bin_dirs);
    }

    for app in &mut apps {
        app.size_bytes = app.binary_size_bytes + app.residue_bytes();
    }
    apps.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes).then(a.name.cmp(&b.name)));
    report.apps = apps;
    report.scan_duration_ms = started.elapsed().as_millis() as u64;
    report.scanned_at = Some(Utc::now());
    report
}

fn default_app_roots(home: Option<&Path>) -> Vec<(PathBuf, AppSource)> {
    #[cfg(target_os = "macos")]
    {
        // PRD §5.3: the app list covers /Applications (nested folders such as
        // /Applications/Utilities are enumerated one level deep) and
        // ~/Applications. /System/Applications is deliberately excluded, and
        // /Library is never enumerated.
        let mut roots = vec![(PathBuf::from("/Applications"), AppSource::Applications)];
        if let Some(home) = home {
            roots.push((home.join("Applications"), AppSource::UserApplications));
        }
        roots
    }
    #[cfg(not(target_os = "macos"))]
    {
        // Windows registry-driven enumeration lands in M2.
        let _ = home;
        Vec::new()
    }
}

fn default_bin_dirs(home: Option<&Path>) -> Vec<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let mut dirs = vec![
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/opt/homebrew/bin"),
        ];
        if let Some(home) = home {
            dirs.push(home.join(".cargo/bin"));
            dirs.push(home.join("bin"));
        }
        dirs
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = home;
        Vec::new()
    }
}

// ── Bundle discovery ──────────────────────────────────────────────────────────

/// Enumerate `.app` bundles directly inside `root` and one level deeper
/// (e.g. `/Applications/Utilities/Terminal.app`, `/Applications/Adobe/…`).
fn collect_bundles_in_root(root: &Path) -> Vec<PathBuf> {
    let mut bundles = Vec::new();
    let Ok(read_dir) = std::fs::read_dir(root) else {
        return bundles;
    };
    for entry in read_dir.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if !entry.file_type().is_ok_and(|t| t.is_dir()) {
            continue;
        }
        if name.ends_with(".app") {
            bundles.push(path);
        } else {
            // One level of nesting for app folders.
            if let Ok(nested) = std::fs::read_dir(&path) {
                for child in nested.filter_map(|e| e.ok()) {
                    let child_name = child.file_name().to_string_lossy().to_string();
                    if child_name.ends_with(".app") && child.file_type().is_ok_and(|t| t.is_dir()) {
                        bundles.push(child.path());
                    }
                }
            }
        }
    }
    bundles.sort();
    bundles
}

/// Maximum depth of the home stray-bundle scan (PRD: 家目录浅层扫描).
const STRAY_SCAN_MAX_DEPTH: usize = 3;
/// Safety valve so a pathological home tree cannot stall the scan.
const STRAY_SCAN_MAX_ENTRIES: usize = 80_000;

fn scan_stray_bundles(
    home: &Path,
    standard_roots: &[PathBuf],
    cancel: &AtomicBool,
) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut visited = 0usize;
    for entry in WalkDir::new(home)
        .max_depth(STRAY_SCAN_MAX_DEPTH)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || !is_stray_skip_entry(e))
    {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        visited += 1;
        if visited > STRAY_SCAN_MAX_ENTRIES {
            break;
        }
        let Ok(entry) = entry else {
            continue;
        };
        if !entry.file_type().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if name.ends_with(".app") {
            let path = entry.into_path();
            if standard_roots.iter().any(|root| path.starts_with(root)) {
                continue;
            }
            found.push(path);
        }
    }
    found
}

/// Directories the stray scan never descends into: heavy media/library trees,
/// package-manager caches and hidden folders. Symlinks are never followed.
fn is_stray_skip_entry(entry: &walkdir::DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy().to_string();
    if name.starts_with('.') {
        return true;
    }
    matches!(
        name.as_str(),
        "Applications"
            | "Library"
            | "Movies"
            | "Music"
            | "Pictures"
            | "Public"
            | "node_modules"
            | "target"
            | "venv"
            | ".venv"
            | "site-packages"
            | "__pycache__"
    )
}

// ── Bundle metadata ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
struct BundleInfo {
    bundle_id: Option<String>,
    name: Option<String>,
    version: Option<String>,
}

/// Reads `Contents/Info.plist` bundle metadata. Missing or unreadable plists
/// degrade to the file name.
fn read_bundle_info(app_path: &Path) -> BundleInfo {
    let Ok(value) = plist::Value::from_file(app_path.join("Contents/Info.plist")) else {
        return BundleInfo::default();
    };
    let dict = value.as_dictionary();
    let get = |key: &str| {
        dict.and_then(|d| d.get(key))
            .and_then(|v| v.as_string())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    };
    BundleInfo {
        bundle_id: get("CFBundleIdentifier"),
        name: get("CFBundleDisplayName").or_else(|| get("CFBundleName")),
        version: get("CFBundleShortVersionString").or_else(|| get("CFBundleVersion")),
    }
}

fn build_app_entry(bundle: &Path, source: AppSource, options: &AppScanOptions) -> Option<AppEntry> {
    let info = read_bundle_info(bundle);
    let file_stem = bundle
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "App".to_string());
    let name = info.name.clone().unwrap_or(file_stem);
    let bundle_id = info.bundle_id.clone();

    let (size_bytes, _truncated) = crate::scanner::dir_size(bundle, &AtomicBool::new(false));
    let risk = bundle_risk(bundle, bundle_id.as_deref());
    let is_running = bundle_running(bundle, &options.running_paths);

    let id = bundle_id
        .clone()
        .unwrap_or_else(|| format!("path:{}", bundle.display()));

    Some(AppEntry {
        id,
        name,
        version: info.version,
        publisher: None,
        bundle_id,
        app_path: Some(bundle.to_path_buf()),
        binary_size_bytes: size_bytes,
        size_bytes,
        source,
        risk,
        is_running,
        related_items: Vec::new(),
        uninstall_string: None,
    })
}

/// System apps and system volumes are `Protected`; everything user-installed
/// is `Caution` (the uninstall CTA is always an explicit user action).
fn bundle_risk(bundle: &Path, bundle_id: Option<&str>) -> RiskLevel {
    if bundle.starts_with("/System") {
        return RiskLevel::Protected;
    }
    if bundle_id.is_some_and(|id| id.starts_with("com.apple.")) {
        return RiskLevel::Protected;
    }
    RiskLevel::Caution
}

fn bundle_running(bundle: &Path, running_paths: &[PathBuf]) -> bool {
    running_paths
        .iter()
        .any(|exe| exe.starts_with(bundle) && bundle.is_dir())
}

// ── Windows residue matching (M2) ────────────────────────────────────────────

/// Builds an [`AppEntry`] from a registry uninstall record. `app_path` is the
/// install directory when it still exists; the estimated size comes straight
/// from the registry (walking `Program Files` for every app would make the
/// scan I/O-bound).
fn build_registry_app_entry(record: &RegistryAppRecord, options: &AppScanOptions) -> AppEntry {
    let install_location = record.install_location.clone().filter(|p| p.is_dir());
    let is_running = install_location.as_ref().is_some_and(|path| {
        options
            .running_paths
            .iter()
            .any(|exe| exe.starts_with(path))
    });
    AppEntry {
        id: format!("registry:{}", record.key_path),
        name: record.display_name.clone(),
        version: record.display_version.clone(),
        publisher: record.publisher.clone(),
        bundle_id: None,
        app_path: install_location,
        binary_size_bytes: record.estimated_size_bytes,
        size_bytes: record.estimated_size_bytes,
        source: AppSource::Registry,
        risk: RiskLevel::Caution,
        is_running,
        related_items: Vec::new(),
        uninstall_string: record.uninstall_string.clone(),
    }
}

/// Attributes Windows residue tracks to the registry-discovered apps:
/// `HKCU\Software\<vendor>` keys, Start Menu / Desktop shortcuts, and
/// AppData / ProgramData directories. Everything is `Caution` (never
/// pre-checked); machine-wide registry keys are `Protected` (display only).
fn match_windows_residue(inputs: &WindowsScanInputs, apps: &mut [AppEntry], cancel: &AtomicBool) {
    if apps.is_empty() {
        return;
    }

    // 1. Registry residue keys.
    for candidate in &inputs.registry_residue {
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        let Some((index, quality)) = best_windows_app(&candidate.name, apps) else {
            continue;
        };
        let risk = match candidate.hive {
            RegistryHive::User => RiskLevel::Caution,
            RegistryHive::Machine => RiskLevel::Protected,
        };
        let app_id = apps[index].id.clone();
        apps[index].related_items.push(RelatedItem {
            quality,
            item: registry_item(&app_id, candidate, risk),
        });
    }

    // 2. Shortcuts (Start Menu / Desktop `.lnk`).
    for shortcut in &inputs.shortcuts {
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        let Some((index, quality)) = best_shortcut_app(shortcut, apps) else {
            continue;
        };
        let app_id = apps[index].id.clone();
        apps[index].related_items.push(RelatedItem {
            quality,
            item: ScanItem {
                id: format!("{app_id}:shortcut:{}", shortcut.path.display()),
                path: shortcut.path.clone(),
                name: shortcut.name.clone(),
                size_bytes: shortcut.size_bytes,
                stack: TechStack::Other,
                risk: RiskLevel::Caution,
                category: CleanupCategory::AppShortcut,
                description: RuleDescription::R154,
                project_root: None,
                last_modified: None,
            },
        });
    }

    // 3. AppData / ProgramData child directories.
    for root in &inputs.residue_roots {
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        let Ok(read_dir) = std::fs::read_dir(&root.path) else {
            continue;
        };
        for entry in read_dir.filter_map(|e| e.ok()) {
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            let Some((index, quality)) = best_windows_app(&name, apps) else {
                continue;
            };
            let path = entry.path();
            let (size_bytes, _truncated) = crate::scanner::dir_size(&path, cancel);
            let last_modified = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .map(DateTime::<Utc>::from);
            let app_id = apps[index].id.clone();
            apps[index].related_items.push(RelatedItem {
                quality,
                item: ScanItem {
                    id: format!("{app_id}:{}", path.display()),
                    path,
                    name,
                    size_bytes,
                    stack: TechStack::Other,
                    risk: RiskLevel::Caution,
                    category: root.category,
                    description: category_description(root.category),
                    project_root: None,
                    last_modified,
                },
            });
        }
    }
}

/// Synthetic [`ScanItem`] for a registry key. `path` carries the key path
/// (`HKCU\Software\Vendor`) so the UI can display it; `CleanupExecutor` never
/// touches it — `spawn_cleanup` routes `RegistryLeftover` items through the
/// platform's export-then-delete path instead.
fn registry_item(app_id: &str, candidate: &RegistryResidueCandidate, risk: RiskLevel) -> ScanItem {
    ScanItem {
        id: format!("{app_id}:registry:{}", candidate.key_path),
        path: PathBuf::from(&candidate.key_path),
        name: candidate.name.clone(),
        size_bytes: 0,
        stack: TechStack::Other,
        risk,
        category: CleanupCategory::RegistryLeftover,
        description: RuleDescription::R153,
        project_root: None,
        last_modified: None,
    }
}

/// Name-based attribution shared by registry keys and AppData directories:
/// exact normalized match, then forward containment (the entry name contains
/// the app name, e.g. `SpotifyPremium` → `Spotify`).
///
/// Reverse vendor-prefix matching is deliberately **not** used here: it
/// exists on macOS to stop live vendor directories from surfacing as orphans,
/// but Windows has no orphan pipeline, so attributing `AppData\Local\Microsoft`
/// to "Microsoft Edge" would only add a dangerous oversized item.
fn best_windows_app(name: &str, apps: &[AppEntry]) -> Option<(usize, MatchQuality)> {
    let norm_entry = normalized(name);
    if norm_entry.len() < 4 {
        return None;
    }
    let mut fuzzy: Option<(usize, MatchQuality)> = None;
    for (index, app) in apps.iter().enumerate() {
        let target = normalized(&app.name);
        if target.is_empty() {
            continue;
        }
        if norm_entry == target {
            return Some((index, MatchQuality::Name));
        }
        if fuzzy.is_none() && target.len() >= 4 && norm_entry.contains(&target) {
            fuzzy = Some((index, MatchQuality::Fuzzy));
        }
    }
    fuzzy
}

/// Shortcuts prefer a resolved target path; the filename heuristic (target
/// `None`) falls back to name matching.
fn best_shortcut_app(
    shortcut: &ShortcutCandidate,
    apps: &[AppEntry],
) -> Option<(usize, MatchQuality)> {
    if let Some(target) = &shortcut.target {
        for (index, app) in apps.iter().enumerate() {
            if let Some(install) = &app.app_path {
                if target.starts_with(install) {
                    return Some((index, MatchQuality::Name));
                }
            }
        }
    }
    best_windows_app(&shortcut.name, apps)
}

// ── Library residue matching ──────────────────────────────────────────────────

struct LibrarySlot {
    dir: &'static str,
    category: CleanupCategory,
    risk: RiskLevel,
}

/// The fixed set of `~/Library` subdirectories searched for app residue
/// (PRD §7.1). Group containers use bundle-id-suffix matching only.
const LIBRARY_SLOTS: &[LibrarySlot] = &[
    LibrarySlot {
        dir: "Application Support",
        category: CleanupCategory::AppSupport,
        risk: RiskLevel::Caution,
    },
    LibrarySlot {
        dir: "Caches",
        category: CleanupCategory::AppCache,
        risk: RiskLevel::Safe,
    },
    LibrarySlot {
        dir: "Preferences",
        category: CleanupCategory::AppPreference,
        risk: RiskLevel::Caution,
    },
    LibrarySlot {
        dir: "Logs",
        category: CleanupCategory::AppLog,
        risk: RiskLevel::Safe,
    },
    LibrarySlot {
        dir: "Saved Application State",
        category: CleanupCategory::AppSavedState,
        risk: RiskLevel::Safe,
    },
    LibrarySlot {
        dir: "Containers",
        category: CleanupCategory::AppContainer,
        risk: RiskLevel::Caution,
    },
    LibrarySlot {
        dir: "Group Containers",
        category: CleanupCategory::AppGroupContainer,
        risk: RiskLevel::Caution,
    },
    LibrarySlot {
        dir: "WebKit",
        category: CleanupCategory::AppWebData,
        risk: RiskLevel::Caution,
    },
    LibrarySlot {
        dir: "HTTPStorages",
        category: CleanupCategory::AppWebData,
        risk: RiskLevel::Caution,
    },
    LibrarySlot {
        dir: "Application Scripts",
        category: CleanupCategory::AppSupport,
        risk: RiskLevel::Caution,
    },
    LibrarySlot {
        dir: "Cookies",
        category: CleanupCategory::AppWebData,
        risk: RiskLevel::Caution,
    },
    LibrarySlot {
        dir: "LaunchAgents",
        category: CleanupCategory::AppLaunchItem,
        risk: RiskLevel::Caution,
    },
];

/// Lightweight matching view of an [`AppEntry`].
struct AppTarget {
    index: usize,
    bundle_id: Option<String>,
    norm_name: String,
}

/// An unmatched `~/Library` slot entry — the input list for orphan
/// detection (M1.5).
struct OrphanCandidate {
    name: String,
    path: PathBuf,
    last_modified: Option<DateTime<Utc>>,
}

/// For every Library slot entry, attribute it to the best-matching app.
/// Returns the entries that matched no app (and are not Apple-owned) for
/// downstream orphan detection.
fn match_library_residue(
    library: &Path,
    apps: &mut [AppEntry],
    cancel: &AtomicBool,
) -> Vec<OrphanCandidate> {
    let targets: Vec<AppTarget> = apps
        .iter()
        .enumerate()
        .map(|(index, app)| AppTarget {
            index,
            bundle_id: app.bundle_id.clone(),
            norm_name: normalized(&app.name),
        })
        .collect();
    let mut unmatched: Vec<OrphanCandidate> = Vec::new();

    for slot in LIBRARY_SLOTS {
        if cancel.load(Ordering::Relaxed) {
            // Partial results are still sound: every visited entry was
            // matched (or not) against the complete app list.
            return unmatched;
        }
        let slot_dir = library.join(slot.dir);
        let Ok(read_dir) = std::fs::read_dir(&slot_dir) else {
            continue;
        };
        for entry in read_dir.filter_map(|e| e.ok()) {
            let entry_name = entry.file_name().to_string_lossy().to_string();
            if entry_name.starts_with('.') || is_apple_owned_entry(&entry_name) {
                continue;
            }
            let path = entry.path();
            let last_modified = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .map(DateTime::<Utc>::from);
            let Some((target, quality)) = best_match_for(&entry_name, &targets, slot) else {
                unmatched.push(OrphanCandidate {
                    name: entry_name,
                    path,
                    last_modified,
                });
                continue;
            };
            let (size_bytes, _truncated) = crate::scanner::dir_size(&path, cancel);
            apps[target.index].related_items.push(RelatedItem {
                quality,
                item: ScanItem {
                    id: format!("{}:{}", apps[target.index].id, path.display()),
                    path,
                    name: entry_name,
                    size_bytes,
                    stack: TechStack::Other,
                    // Protected apps (Safari, …) get protected residue too.
                    risk: if apps[target.index].risk == RiskLevel::Protected {
                        RiskLevel::Protected
                    } else {
                        slot.risk
                    },
                    category: slot.category,
                    description: category_description(slot.category),
                    project_root: None,
                    last_modified,
                },
            });
        }
    }
    unmatched
}

/// Well-known Apple service/entry names that carry neither a `com.apple.`
/// prefix nor an `Apple` prefix but are still system-owned.
const APPLE_ENTRY_NAMES: &[&str] = &[
    "MobileSync",  // iPhone/iPad backups under Application Support
    "AddressBook", // Contacts database
    "CloudDocs",   // iCloud Drive content
    "Knowledge",   // Apple Knowledge feature data
    "Safari",
    "iCloud",
    "iCloudDrive",
    "Metadata", // Caches/Metadata — Spotlight-related
];

/// Apple-owned entries never attribute to third-party apps, never become
/// orphan candidates, and are skipped entirely (they also dominate false
/// positives for name matching).
fn is_apple_owned_entry(name: &str) -> bool {
    name == "Apple"
        || name.starts_with("com.apple.")
        || name.starts_with("group.com.apple.")
        || name.starts_with("apple.")
        || name.starts_with("Apple")
        || APPLE_ENTRY_NAMES.contains(&name)
}

/// Orphaned-residue gates (PRD §9, M1.5 decision). Apple-prefixed entries
/// are blacklisted during matching; a surviving candidate must additionally
/// pass the size and staleness gates below.
const ORPHAN_MIN_BYTES: u64 = 10 * 1024 * 1024;
const ORPHAN_STALE_DAYS: i64 = 90;

/// Turns unmatched Library entries past the triple gate into virtual
/// `Orphan` entities, aggregated by their normalized stem — one entity per
/// suspected uninstalled app. Orphans have no bundle (`app_path == None`),
/// carry `Caution` risk and are therefore never pre-checked.
fn build_orphan_entries(candidates: &[OrphanCandidate], cancel: &AtomicBool) -> Vec<AppEntry> {
    if candidates.is_empty() {
        return Vec::new();
    }
    let cutoff = Utc::now() - chrono::Duration::days(ORPHAN_STALE_DAYS);

    // (normalized stem, display name, items)
    let mut groups: Vec<(String, String, Vec<ScanItem>)> = Vec::new();
    for candidate in candidates {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        // Gate 2: minimum size — keeps plist-level noise out of the list.
        let (size_bytes, _truncated) = crate::scanner::dir_size(&candidate.path, cancel);
        if size_bytes < ORPHAN_MIN_BYTES {
            continue;
        }
        // Gate 3: untouched for ≥90 days — apps on external drives may come
        // back; the grace period absorbs that false-positive source.
        let Some(modified) = candidate.last_modified else {
            continue;
        };
        if modified > cutoff {
            continue;
        }
        let stem = orphan_stem(&candidate.name);
        let item = ScanItem {
            id: format!("orphan:{}", candidate.path.display()),
            path: candidate.path.clone(),
            name: candidate.name.clone(),
            size_bytes,
            stack: TechStack::Other,
            risk: RiskLevel::Caution,
            category: CleanupCategory::OrphanResidue,
            description: RuleDescription::R152,
            project_root: None,
            last_modified: candidate.last_modified,
        };
        match groups.iter_mut().find(|(key, _, _)| *key == stem) {
            Some((_, _, items)) => items.push(item),
            None => groups.push((stem, orphan_display_name(&candidate.name), vec![item])),
        }
    }

    groups
        .into_iter()
        .map(|(key, display, items)| {
            let size_bytes: u64 = items.iter().map(|i| i.size_bytes).sum();
            AppEntry {
                id: format!("orphan:{key}"),
                name: display,
                version: None,
                publisher: None,
                bundle_id: None,
                app_path: None,
                binary_size_bytes: 0,
                size_bytes,
                source: AppSource::Orphan,
                risk: RiskLevel::Caution,
                is_running: false,
                related_items: items
                    .into_iter()
                    .map(|item| RelatedItem {
                        item,
                        // The unmatched fact itself is certain; the Caution
                        // risk (not the quality) keeps orphans unchecked.
                        quality: MatchQuality::BundleId,
                    })
                    .collect(),
                uninstall_string: None,
            }
        })
        .collect()
}

/// Normalized aggregation key: the extension-stripped stem of an entry
/// name (`com.example.gone.plist` and `com.example.gone` share a stem).
fn orphan_stem(name: &str) -> String {
    normalized(entry_match_names(name)[1])
}

/// Display form of a candidate: the extension-stripped stem.
fn orphan_display_name(name: &str) -> String {
    entry_match_names(name)[1].to_string()
}

fn best_match_for<'a>(
    entry_name: &str,
    targets: &'a [AppTarget],
    slot: &LibrarySlot,
) -> Option<(&'a AppTarget, MatchQuality)> {
    // Group containers may be shared across apps: only an exact bundle-id
    // suffix claims them (PRD §9.4 fallback).
    if slot.category == CleanupCategory::AppGroupContainer {
        return targets.iter().find_map(|t| {
            let bundle_id = t.bundle_id.as_deref()?;
            let suffix = format!(".{bundle_id}");
            if entry_name == bundle_id || entry_name.ends_with(&suffix) {
                Some((t, MatchQuality::BundleId))
            } else {
                None
            }
        });
    }

    let mut best: Option<(&AppTarget, MatchQuality)> = None;
    for target in targets {
        if let Some(quality) = match_quality_for(entry_name, target) {
            if best.is_none_or(|(_, b)| quality < b) {
                best = Some((target, quality));
            }
        }
    }
    best
}

/// Match confidence of one entry name against one app.
fn match_quality_for(entry_name: &str, target: &AppTarget) -> Option<MatchQuality> {
    for candidate in entry_match_names(entry_name) {
        if let Some(bundle_id) = &target.bundle_id {
            if candidate == bundle_id {
                return Some(MatchQuality::BundleId);
            }
        }
        if normalized(candidate) == target.norm_name {
            return Some(MatchQuality::Name);
        }
    }
    // Fuzzy: containment only, and only for names long enough to be meaningful.
    if target.norm_name.len() >= 4 {
        for candidate in entry_match_names(entry_name) {
            let norm_entry = normalized(candidate);
            if norm_entry.contains(&target.norm_name) {
                return Some(MatchQuality::Fuzzy);
            }
            // Reverse containment: vendor directories such as
            // `Application Support/Microsoft` prefix-match installed apps
            // like "Microsoft Edge". Attributing them to the (fuzzy, expert
            // only) app beats flagging live vendor data as an orphan.
            if norm_entry.len() >= 4 && target.norm_name.starts_with(&norm_entry) {
                return Some(MatchQuality::Fuzzy);
            }
        }
    }
    None
}

/// File names carry their "kind" extension (`com.app.id.plist`,
/// `com.app.id.savedState`, `com.app.id.binarycookies`); directories usually
/// do not. Compare against both the raw name and the stripped stem.
fn entry_match_names(entry_name: &str) -> [&str; 2] {
    const EXTENSIONS: [&str; 3] = [".plist", ".savedState", ".binarycookies"];
    for ext in EXTENSIONS {
        if entry_name.ends_with(ext) {
            return [entry_name, &entry_name[..entry_name.len() - ext.len()]];
        }
    }
    [entry_name, entry_name]
}

/// Lowercased alphanumeric-only form for name comparisons.
fn normalized(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

fn category_description(category: CleanupCategory) -> RuleDescription {
    match category {
        CleanupCategory::AppBinary => RuleDescription::R141,
        CleanupCategory::AppSupport => RuleDescription::R142,
        CleanupCategory::AppCache => RuleDescription::R143,
        CleanupCategory::AppPreference => RuleDescription::R144,
        CleanupCategory::AppContainer => RuleDescription::R145,
        CleanupCategory::AppGroupContainer => RuleDescription::R146,
        CleanupCategory::AppSavedState => RuleDescription::R147,
        CleanupCategory::AppWebData => RuleDescription::R148,
        CleanupCategory::AppLaunchItem => RuleDescription::R149,
        CleanupCategory::AppLog => RuleDescription::R150,
        CleanupCategory::StrayExecutable => RuleDescription::R151,
        CleanupCategory::OrphanResidue => RuleDescription::R152,
        CleanupCategory::RegistryLeftover => RuleDescription::R153,
        CleanupCategory::AppShortcut => RuleDescription::R154,
        _ => RuleDescription::R141,
    }
}

// ── Broken CLI symlinks ──────────────────────────────────────────────────────

/// Dead symlinks in well-known bin dirs. The link itself is size ~0; the
/// value is decluttering the PATH.
fn scan_broken_symlinks(bin_dirs: &[PathBuf]) -> Vec<ScanItem> {
    let mut items = Vec::new();
    for dir in bin_dirs {
        let Ok(read_dir) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in read_dir.filter_map(|e| e.ok()) {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_symlink() {
                continue;
            }
            // Broken = the link target cannot be resolved.
            if std::fs::metadata(entry.path()).is_ok() {
                continue;
            }
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            items.push(ScanItem {
                id: format!("stray:{}", path.display()),
                path,
                name,
                size_bytes: 0,
                stack: TechStack::Other,
                risk: RiskLevel::Safe,
                category: CleanupCategory::StrayExecutable,
                description: RuleDescription::R151,
                project_root: None,
                last_modified: None,
            });
        }
    }
    items.sort_by(|a, b| a.name.cmp(&b.name));
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_bundle(dir: &Path, bundle_id: Option<&str>, name: Option<&str>) -> PathBuf {
        let stem = bundle_id
            .map(|id| id.split('.').next_back().unwrap_or("App"))
            .unwrap_or("UnknownApp")
            .to_string();
        let bundle = dir.join(format!("{stem}.app"));
        std::fs::create_dir_all(bundle.join("Contents/MacOS")).unwrap();
        std::fs::write(bundle.join("Contents/MacOS/binary"), "x".repeat(4096)).unwrap();
        let mut plist = plist::Dictionary::new();
        if let Some(id) = bundle_id {
            plist.insert("CFBundleIdentifier".into(), id.into());
        }
        if let Some(name) = name {
            plist.insert("CFBundleDisplayName".into(), name.into());
        }
        plist.insert("CFBundleShortVersionString".into(), "1.2.3".into());
        plist::Value::Dictionary(plist)
            .to_file_binary(bundle.join("Contents/Info.plist"))
            .unwrap();
        bundle
    }

    fn scan_opts(roots: &[PathBuf], library: &Path) -> AppScanOptions {
        AppScanOptions {
            app_roots: roots.to_vec(),
            home_dir: None,
            library_dir: Some(library.to_path_buf()),
            bin_dirs: vec![],
            running_paths: vec![],
            windows: WindowsScanInputs::default(),
        }
    }

    #[test]
    fn scans_installed_apps_with_bundle_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let apps = temp.path().join("Applications");
        let library = temp.path().join("Library");
        let bundle = write_bundle(&apps, Some("com.example.demo"), Some("Demo App"));
        assert!(bundle.exists());

        let report = scan_apps(
            &scan_opts(&[apps], &library),
            &AtomicBool::new(false),
            |_| {},
        );
        assert_eq!(report.apps.len(), 1);
        let app = &report.apps[0];
        assert_eq!(app.name, "Demo App");
        assert_eq!(app.bundle_id.as_deref(), Some("com.example.demo"));
        assert_eq!(app.version.as_deref(), Some("1.2.3"));
        assert_eq!(app.source, AppSource::Applications);
        assert_eq!(app.risk, RiskLevel::Caution);
        assert!(app.binary_size_bytes >= 4096);
    }

    #[test]
    fn apple_and_system_apps_are_protected() {
        let temp = tempfile::tempdir().unwrap();
        let apps = temp.path().join("Applications");
        let library = temp.path().join("Library");
        write_bundle(&apps, Some("com.apple.Safari"), Some("Safari"));
        write_bundle(&apps, Some("com.example.demo"), Some("Demo App"));

        let report = scan_apps(
            &scan_opts(&[apps], &library),
            &AtomicBool::new(false),
            |_| {},
        );
        let safari = report
            .apps
            .iter()
            .find(|a| a.bundle_id.as_deref() == Some("com.apple.Safari"))
            .unwrap();
        assert_eq!(safari.risk, RiskLevel::Protected);
        let demo = report
            .apps
            .iter()
            .find(|a| a.bundle_id.as_deref() == Some("com.example.demo"))
            .unwrap();
        assert_eq!(demo.risk, RiskLevel::Caution);
    }

    #[test]
    fn matches_library_residue_by_bundle_id() {
        let temp = tempfile::tempdir().unwrap();
        let apps = temp.path().join("Applications");
        let library = temp.path().join("Library");
        write_bundle(&apps, Some("com.example.demo"), Some("Demo App"));
        std::fs::create_dir_all(library.join("Caches/com.example.demo")).unwrap();
        std::fs::write(
            library.join("Caches/com.example.demo/blob"),
            "x".repeat(8192),
        )
        .unwrap();
        std::fs::create_dir_all(library.join("Application Support/com.example.demo")).unwrap();
        std::fs::create_dir_all(library.join("Preferences")).unwrap();
        std::fs::write(
            library.join("Preferences/com.example.demo.plist"),
            "plist-bytes",
        )
        .unwrap();
        std::fs::create_dir_all(library.join("Saved Application State")).unwrap();
        std::fs::create_dir_all(
            library.join("Saved Application State/com.example.demo.savedState"),
        )
        .unwrap();
        std::fs::create_dir_all(library.join("Group Containers")).unwrap();
        std::fs::create_dir_all(library.join("Group Containers/TEAM123.com.example.demo")).unwrap();

        let report = scan_apps(
            &scan_opts(&[apps], &library),
            &AtomicBool::new(false),
            |_| {},
        );
        let app = &report.apps[0];
        let categories: Vec<CleanupCategory> =
            app.related_items.iter().map(|r| r.item.category).collect();
        assert!(
            categories.contains(&CleanupCategory::AppCache),
            "{categories:?}"
        );
        assert!(
            categories.contains(&CleanupCategory::AppSupport),
            "{categories:?}"
        );
        assert!(
            categories.contains(&CleanupCategory::AppPreference),
            "{categories:?}"
        );
        assert!(
            categories.contains(&CleanupCategory::AppSavedState),
            "{categories:?}"
        );
        assert!(
            categories.contains(&CleanupCategory::AppGroupContainer),
            "{categories:?}"
        );
        for related in &app.related_items {
            assert_eq!(related.quality, MatchQuality::BundleId);
        }
        assert!(app.residue_bytes() >= 8192);
    }

    #[test]
    fn apple_owned_library_entries_are_never_matched() {
        let temp = tempfile::tempdir().unwrap();
        let apps = temp.path().join("Applications");
        let library = temp.path().join("Library");
        // An app whose fuzzy name could collide with Apple entries.
        write_bundle(&apps, Some("com.example.musicexport"), Some("Music Export"));
        std::fs::create_dir_all(library.join("Caches/com.apple.Music")).unwrap();

        let report = scan_apps(
            &scan_opts(&[apps], &library),
            &AtomicBool::new(false),
            |_| {},
        );
        assert!(report.apps[0].related_items.is_empty());
    }

    #[test]
    fn fuzzy_matches_need_long_names_and_stay_unchecked() {
        let temp = tempfile::tempdir().unwrap();
        let apps = temp.path().join("Applications");
        let library = temp.path().join("Library");
        write_bundle(&apps, Some("com.example.demo"), Some("Demo Application"));
        std::fs::create_dir_all(library.join("Caches/demoapplicationtools")).unwrap();

        let report = scan_apps(
            &scan_opts(&[apps], &library),
            &AtomicBool::new(false),
            |_| {},
        );
        let app = &report.apps[0];
        assert_eq!(app.related_items.len(), 1);
        let related = &app.related_items[0];
        assert_eq!(related.quality, MatchQuality::Fuzzy);
        assert!(!related.is_default_selected());
    }

    #[test]
    fn group_containers_only_match_bundle_id_suffix() {
        let temp = tempfile::tempdir().unwrap();
        let apps = temp.path().join("Applications");
        let library = temp.path().join("Library");
        write_bundle(&apps, Some("com.example.demo"), Some("Demo App"));
        let groups = library.join("Group Containers");
        std::fs::create_dir_all(groups.join("TEAM123.com.example.demo")).unwrap();
        std::fs::create_dir_all(groups.join("TEAM999.com.example.other")).unwrap();

        let report = scan_apps(
            &scan_opts(&[apps], &library),
            &AtomicBool::new(false),
            |_| {},
        );
        let matched: Vec<&str> = report.apps[0]
            .related_items
            .iter()
            .map(|r| r.item.name.as_str())
            .collect();
        assert_eq!(matched, vec!["TEAM123.com.example.demo"]);
    }

    #[test]
    fn stray_bundles_are_found_outside_standard_roots() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("home");
        let library = home.join("Library");
        std::fs::create_dir_all(home.join("Downloads")).unwrap();
        let stray = write_bundle(
            &home.join("Downloads"),
            Some("com.example.stray"),
            Some("Stray"),
        );
        // Standard roots (for exclusion) + noise dirs that must be skipped.
        std::fs::create_dir_all(home.join("Applications")).unwrap();
        write_bundle(
            &home.join("Applications"),
            Some("com.example.installed"),
            Some("Installed"),
        );
        std::fs::create_dir_all(home.join("Movies/Deep")).unwrap();
        std::fs::create_dir_all(home.join(".hidden")).unwrap();
        write_bundle(
            &home.join(".hidden"),
            Some("com.example.hidden"),
            Some("Hidden"),
        );

        let options = AppScanOptions {
            app_roots: vec![home.join("Applications")],
            home_dir: Some(home.clone()),
            library_dir: Some(library),
            bin_dirs: vec![],
            running_paths: vec![],
            windows: WindowsScanInputs::default(),
        };
        let report = scan_apps(&options, &AtomicBool::new(false), |_| {});
        let sources: Vec<(String, AppSource)> = report
            .apps
            .iter()
            .map(|a| (a.bundle_id.clone().unwrap_or_default(), a.source))
            .collect();
        assert!(
            sources
                .iter()
                .any(|(id, s)| id == "com.example.stray" && *s == AppSource::StrayBundle),
            "{sources:?}"
        );
        assert!(
            sources
                .iter()
                .any(|(id, s)| id == "com.example.installed" && *s == AppSource::Applications),
            "{sources:?}"
        );
        assert!(
            !sources.iter().any(|(id, _)| id == "com.example.hidden"),
            "hidden dirs must be skipped: {sources:?}"
        );
        assert_eq!(report.apps.len(), 2, "{sources:?}");
        assert!(stray.exists());
    }

    #[test]
    fn running_apps_are_flagged() {
        let temp = tempfile::tempdir().unwrap();
        let apps = temp.path().join("Applications");
        let library = temp.path().join("Library");
        let bundle = write_bundle(&apps, Some("com.example.demo"), Some("Demo App"));

        let options = AppScanOptions {
            running_paths: vec![bundle.join("Contents/MacOS/binary")],
            ..scan_opts(&[apps], &library)
        };
        let report = scan_apps(&options, &AtomicBool::new(false), |_| {});
        assert!(report.apps[0].is_running);
    }

    #[test]
    fn broken_symlinks_in_bin_dirs_are_listed() {
        let temp = tempfile::tempdir().unwrap();
        let bin = temp.path().join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        let target = temp.path().join("gone");
        std::os::unix::fs::symlink(&target, bin.join("dead-link")).unwrap();
        // A working link must NOT be listed.
        let live_target = temp.path().join("alive");
        std::fs::write(&live_target, "x").unwrap();
        std::os::unix::fs::symlink(&live_target, bin.join("live-link")).unwrap();

        let report = scan_apps(
            &AppScanOptions {
                bin_dirs: vec![bin],
                ..scan_opts(
                    &[temp.path().join("Applications")],
                    &temp.path().join("Library"),
                )
            },
            &AtomicBool::new(false),
            |_| {},
        );
        assert_eq!(report.stray_executables.len(), 1);
        assert_eq!(report.stray_executables[0].name, "dead-link");
        assert_eq!(report.stray_executables[0].risk, RiskLevel::Safe);
        assert_eq!(
            report.stray_executables[0].category,
            CleanupCategory::StrayExecutable
        );
    }

    #[test]
    fn binary_scan_item_marks_protected_apps() {
        let temp = tempfile::tempdir().unwrap();
        let apps = temp.path().join("Applications");
        write_bundle(&apps, Some("com.apple.Safari"), Some("Safari"));
        let report = scan_apps(
            &scan_opts(&[apps], &temp.path().join("Library")),
            &AtomicBool::new(false),
            |_| {},
        );
        let safari = &report.apps[0];
        let item = safari.binary_scan_item().unwrap();
        assert_eq!(item.risk, RiskLevel::Protected);
        assert_eq!(item.category, CleanupCategory::AppBinary);
    }

    #[test]
    fn default_selection_includes_only_precise_safe_items() {
        let temp = tempfile::tempdir().unwrap();
        let apps = temp.path().join("Applications");
        let library = temp.path().join("Library");
        write_bundle(&apps, Some("com.example.demo"), Some("Demo Application"));
        std::fs::create_dir_all(library.join("Caches/com.example.demo")).unwrap(); // Safe, precise.
        std::fs::create_dir_all(library.join("Preferences")).unwrap();
        std::fs::write(library.join("Preferences/com.example.demo.plist"), "x").unwrap(); // Caution.

        let report = scan_apps(
            &scan_opts(&[apps], &library),
            &AtomicBool::new(false),
            |_| {},
        );
        let app = &report.apps[0];
        let selected = app.default_selected_related_ids();
        assert_eq!(selected.len(), 1);
        let cache = app
            .related_items
            .iter()
            .find(|r| r.item.category == CleanupCategory::AppCache)
            .unwrap();
        assert!(selected.contains(&cache.item.id));
    }

    #[test]
    fn scan_is_cancellable() {
        let temp = tempfile::tempdir().unwrap();
        let apps = temp.path().join("Applications");
        let library = temp.path().join("Library");
        write_bundle(&apps, Some("com.example.demo"), Some("Demo App"));

        let report = scan_apps(
            &scan_opts(&[apps], &library),
            &AtomicBool::new(true),
            |_| {},
        );
        assert!(report.cancelled);
    }

    #[test]
    fn normalization_and_entry_names() {
        assert_eq!(normalized("Google Chrome 2"), "googlechrome2");
        assert_eq!(
            entry_match_names("com.example.demo.plist"),
            ["com.example.demo.plist", "com.example.demo"]
        );
        assert_eq!(
            entry_match_names("com.example.demo"),
            ["com.example.demo"; 2]
        );
    }

    #[test]
    fn vendor_directory_prefix_matches_fuzzy_not_orphan() {
        let temp = tempfile::tempdir().unwrap();
        let apps_dir = temp.path().join("Applications");
        let library = temp.path().join("Library");
        write_bundle(
            &apps_dir,
            Some("com.microsoft.edgemac"),
            Some("Microsoft Edge"),
        );
        let support = library.join("Application Support");
        std::fs::create_dir_all(support.join("Microsoft")).unwrap();
        std::fs::write(support.join("Microsoft/blob"), "x".repeat(4096)).unwrap();

        let report = scan_apps(
            &scan_opts(&[apps_dir], &library),
            &AtomicBool::new(false),
            |_| {},
        );
        let edge = &report.apps[0];
        assert!(
            edge.related_items
                .iter()
                .any(|r| r.item.name == "Microsoft" && r.quality == MatchQuality::Fuzzy),
            "vendor directory must fuzzy-attribute to the installed vendor app"
        );
        assert!(
            !report
                .apps
                .iter()
                .any(|a| a.source == AppSource::Orphan && a.name == "Microsoft"),
            "live vendor data must never surface as an orphan"
        );
    }

    // ── Orphaned residue (M1.5) ─────────────────────────────────────────

    /// Backdate a path's mtime so it passes the 90-day staleness gate.
    fn set_old_mtime(path: &Path, days_ago: i64) {
        let old = std::time::SystemTime::now()
            - std::time::Duration::from_secs(days_ago as u64 * 24 * 3600);
        let file = std::fs::File::open(path).unwrap();
        file.set_times(std::fs::FileTimes::new().set_modified(old))
            .unwrap();
    }

    fn write_heavy_entry(dir: &Path, name: &str, bytes: usize) -> PathBuf {
        let path = dir.join(name);
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("blob"), vec![0u8; bytes]).unwrap();
        path
    }

    const TEN_MB: usize = 10 * 1024 * 1024;

    #[test]
    fn orphan_residue_requires_size_and_staleness() {
        let temp = tempfile::tempdir().unwrap();
        let apps_dir = temp.path().join("Applications");
        let library = temp.path().join("Library");
        write_bundle(&apps_dir, Some("com.example.demo"), Some("Demo App"));
        let caches = library.join("Caches");
        std::fs::create_dir_all(&caches).unwrap();

        // Big + stale → surfaces as an orphan.
        let gone = write_heavy_entry(&caches, "com.example.gone", TEN_MB + 1024);
        set_old_mtime(&gone, 91);
        // Small + stale → filtered by the size gate.
        let tiny = write_heavy_entry(&caches, "com.example.tiny", 2048);
        set_old_mtime(&tiny, 91);
        // Big + fresh → filtered by the staleness gate.
        write_heavy_entry(&caches, "com.example.fresh", TEN_MB + 1024);

        let report = scan_apps(
            &scan_opts(&[apps_dir], &library),
            &AtomicBool::new(false),
            |_| {},
        );
        let orphans: Vec<&AppEntry> = report
            .apps
            .iter()
            .filter(|a| a.source == AppSource::Orphan)
            .collect();
        assert_eq!(orphans.len(), 1, "expected exactly one orphan");
        let orphan = orphans[0];
        assert_eq!(orphan.name, "com.example.gone");
        assert_eq!(orphan.app_path, None);
        assert!(orphan.binary_scan_item().is_none());
        assert_eq!(orphan.risk, RiskLevel::Caution);
        assert_eq!(orphan.related_items.len(), 1);
        let item = &orphan.related_items[0].item;
        assert_eq!(item.category, CleanupCategory::OrphanResidue);
        assert_eq!(item.risk, RiskLevel::Caution);
        assert!(item.size_bytes >= TEN_MB as u64);
        // Caution risk → not pre-checked.
        assert!(orphan.default_selected_related_ids().is_empty());
    }

    #[test]
    fn apple_owned_entries_never_become_orphans() {
        let temp = tempfile::tempdir().unwrap();
        let apps_dir = temp.path().join("Applications");
        let library = temp.path().join("Library");
        write_bundle(&apps_dir, Some("com.example.demo"), Some("Demo App"));
        let caches = library.join("Caches");
        std::fs::create_dir_all(&caches).unwrap();
        let groups = library.join("Group Containers");
        std::fs::create_dir_all(&groups).unwrap();
        let support = library.join("Application Support");
        std::fs::create_dir_all(&support).unwrap();

        // Every shape of Apple ownership, each big and stale.
        for name in [
            "com.apple.something",
            "group.com.apple.shared",
            "apple.accounts",
            "AppleMediaServices",
            "MobileSync",
        ] {
            let path = write_heavy_entry(&caches, name, TEN_MB + 1024);
            set_old_mtime(&path, 200);
        }
        let backup = write_heavy_entry(&support, "MobileSync", TEN_MB + 1024);
        set_old_mtime(&backup, 200);
        let group = write_heavy_entry(&groups, "group.com.apple.safari", TEN_MB + 1024);
        set_old_mtime(&group, 200);

        let report = scan_apps(
            &scan_opts(&[apps_dir], &library),
            &AtomicBool::new(false),
            |_| {},
        );
        let orphans: Vec<&AppEntry> = report
            .apps
            .iter()
            .filter(|a| a.source == AppSource::Orphan)
            .collect();
        assert!(
            orphans.is_empty(),
            "Apple-owned entries must never surface as orphans: {:?}",
            orphans.iter().map(|o| o.name.clone()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn orphan_entries_aggregate_by_stem() {
        let temp = tempfile::tempdir().unwrap();
        let apps_dir = temp.path().join("Applications");
        let library = temp.path().join("Library");
        write_bundle(&apps_dir, Some("com.example.demo"), Some("Demo App"));
        let caches = library.join("Caches");
        std::fs::create_dir_all(&caches).unwrap();
        let prefs = library.join("Preferences");
        std::fs::create_dir_all(&prefs).unwrap();

        // Same suspected app, two slots: `com.example.gone` (dir) and
        // `com.example.gone.plist` (file) share the stripped stem.
        let cache_dir = write_heavy_entry(&caches, "com.example.gone", TEN_MB + 1024);
        set_old_mtime(&cache_dir, 120);
        let plist = prefs.join("com.example.gone.plist");
        std::fs::write(&plist, vec![0u8; TEN_MB + 1024]).unwrap();
        set_old_mtime(&plist, 120);
        // A different orphan must stay separate.
        let other = write_heavy_entry(&caches, "com.other.tool", TEN_MB + 1024);
        set_old_mtime(&other, 120);

        let report = scan_apps(
            &scan_opts(&[apps_dir], &library),
            &AtomicBool::new(false),
            |_| {},
        );
        let orphans: Vec<&AppEntry> = report
            .apps
            .iter()
            .filter(|a| a.source == AppSource::Orphan)
            .collect();
        assert_eq!(orphans.len(), 2, "two distinct suspected apps");
        let gone = orphans
            .iter()
            .find(|a| a.name == "com.example.gone")
            .expect("aggregated orphan");
        assert_eq!(gone.related_items.len(), 2, "cache dir + plist");
        assert_eq!(
            gone.id, "orphan:comexamplegone",
            "id derives from the normalized stem"
        );
        let other_orphan = orphans
            .iter()
            .find(|a| a.name == "com.other.tool")
            .expect("separate orphan");
        assert_eq!(other_orphan.related_items.len(), 1);
    }

    #[test]
    fn orphan_scan_works_without_any_installed_apps() {
        // A machine with no matching apps still surfaces stale residue.
        let temp = tempfile::tempdir().unwrap();
        let apps_dir = temp.path().join("Applications");
        let library = temp.path().join("Library");
        std::fs::create_dir_all(&apps_dir).unwrap();
        let caches = library.join("Caches");
        std::fs::create_dir_all(&caches).unwrap();
        let gone = write_heavy_entry(&caches, "com.uninstalled.app", TEN_MB + 1024);
        set_old_mtime(&gone, 95);

        let report = scan_apps(
            &scan_opts(&[apps_dir], &library),
            &AtomicBool::new(false),
            |_| {},
        );
        assert_eq!(report.apps.len(), 1);
        assert_eq!(report.apps[0].source, AppSource::Orphan);
        assert_eq!(report.apps[0].name, "com.uninstalled.app");
    }

    // ── Windows registry / shortcut / AppData matching (M2) ─────────────

    fn windows_opts(temp: &Path, windows: WindowsScanInputs) -> AppScanOptions {
        let empty_apps = temp.join("Applications");
        let empty_library = temp.join("Library");
        std::fs::create_dir_all(&empty_apps).unwrap();
        std::fs::create_dir_all(&empty_library).unwrap();
        AppScanOptions {
            app_roots: vec![empty_apps],
            home_dir: None,
            library_dir: Some(empty_library),
            bin_dirs: vec![],
            running_paths: vec![],
            windows,
        }
    }

    fn registry_record(name: &str, install: Option<PathBuf>) -> RegistryAppRecord {
        RegistryAppRecord {
            display_name: name.to_string(),
            display_version: Some("1.2.3".into()),
            publisher: Some("Example Corp".into()),
            install_location: install,
            uninstall_string: Some(format!("MsiExec.exe /X{{{name}}}")),
            estimated_size_bytes: 42 * 1024 * 1024,
            hive: RegistryHive::Machine,
            key_path: format!(r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{name}"),
        }
    }

    #[test]
    fn registry_apps_become_entries_with_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let install = temp.path().join("Program Files/Demo App");
        std::fs::create_dir_all(&install).unwrap();
        let inputs = WindowsScanInputs {
            registry_apps: vec![registry_record("Demo App", Some(install.clone()))],
            ..WindowsScanInputs::default()
        };

        let report = scan_apps(
            &windows_opts(temp.path(), inputs),
            &AtomicBool::new(false),
            |_| {},
        );
        assert_eq!(report.apps.len(), 1);
        let app = &report.apps[0];
        assert_eq!(app.source, AppSource::Registry);
        assert_eq!(app.name, "Demo App");
        assert_eq!(app.version.as_deref(), Some("1.2.3"));
        assert_eq!(app.publisher.as_deref(), Some("Example Corp"));
        assert_eq!(app.app_path.as_deref(), Some(install.as_path()));
        assert_eq!(app.binary_size_bytes, 42 * 1024 * 1024);
        assert_eq!(app.risk, RiskLevel::Caution);
        assert_eq!(
            app.uninstall_string.as_deref(),
            Some("MsiExec.exe /X{Demo App}")
        );
    }

    #[test]
    fn registry_residue_hkcu_is_caution_hklm_is_protected() {
        let temp = tempfile::tempdir().unwrap();
        let inputs = WindowsScanInputs {
            registry_apps: vec![registry_record("Demo App", None)],
            registry_residue: vec![
                RegistryResidueCandidate {
                    name: "DemoApp".into(),
                    key_path: r"HKCU\Software\DemoApp".into(),
                    hive: RegistryHive::User,
                },
                RegistryResidueCandidate {
                    name: "DemoApp".into(),
                    key_path: r"HKLM\SOFTWARE\DemoApp".into(),
                    hive: RegistryHive::Machine,
                },
                // Unrelated vendor key must not attach to any app.
                RegistryResidueCandidate {
                    name: "SomethingElse".into(),
                    key_path: r"HKCU\Software\SomethingElse".into(),
                    hive: RegistryHive::User,
                },
                // Shared vendor container: must NOT fuzzy-attach to
                // "Demo App" via reverse prefix (Windows safety rule).
                RegistryResidueCandidate {
                    name: "Demo".into(),
                    key_path: r"HKCU\Software\Demo".into(),
                    hive: RegistryHive::User,
                },
            ],
            ..WindowsScanInputs::default()
        };

        let report = scan_apps(
            &windows_opts(temp.path(), inputs),
            &AtomicBool::new(false),
            |_| {},
        );
        let app = &report.apps[0];
        let registry: Vec<&RelatedItem> = app
            .related_items
            .iter()
            .filter(|r| r.item.category == CleanupCategory::RegistryLeftover)
            .collect();
        assert_eq!(registry.len(), 2, "one per hive");
        let hkcu = registry
            .iter()
            .find(|r| r.item.path.display().to_string().starts_with("HKCU"))
            .expect("HKCU item");
        assert_eq!(hkcu.item.risk, RiskLevel::Caution);
        assert_eq!(hkcu.item.description, RuleDescription::R153);
        let hklm = registry
            .iter()
            .find(|r| r.item.path.display().to_string().starts_with("HKLM"))
            .expect("HKLM item");
        assert_eq!(hklm.item.risk, RiskLevel::Protected);
        // Caution/Protected → never pre-checked.
        assert!(app.default_selected_related_ids().is_empty());
    }

    #[test]
    fn shortcuts_match_by_target_path_and_filename() {
        let temp = tempfile::tempdir().unwrap();
        let install = temp.path().join("Program Files/Widget");
        std::fs::create_dir_all(&install).unwrap();
        let shortcut_dir = temp.path().join("Start Menu");
        std::fs::create_dir_all(&shortcut_dir).unwrap();
        let by_target = shortcut_dir.join("Widget.lnk");
        std::fs::write(&by_target, "lnk").unwrap();
        let by_name = shortcut_dir.join("Widget Launcher.lnk");
        std::fs::write(&by_name, "lnk").unwrap();

        let inputs = WindowsScanInputs {
            registry_apps: vec![registry_record("Widget", Some(install.clone()))],
            shortcuts: vec![
                ShortcutCandidate {
                    name: "Widget".into(),
                    path: by_target,
                    target: Some(install.join("widget.exe")),
                    size_bytes: 3,
                },
                ShortcutCandidate {
                    name: "Widget Launcher".into(),
                    path: by_name,
                    target: None,
                    size_bytes: 3,
                },
            ],
            ..WindowsScanInputs::default()
        };

        let report = scan_apps(
            &windows_opts(temp.path(), inputs),
            &AtomicBool::new(false),
            |_| {},
        );
        let shortcuts: Vec<&RelatedItem> = report.apps[0]
            .related_items
            .iter()
            .filter(|r| r.item.category == CleanupCategory::AppShortcut)
            .collect();
        assert_eq!(shortcuts.len(), 2);
        assert!(
            shortcuts
                .iter()
                .all(|r| r.item.description == RuleDescription::R154)
        );
        assert_eq!(shortcuts[0].item.risk, RiskLevel::Caution);
    }

    #[test]
    fn appdata_directories_match_installed_apps() {
        let temp = tempfile::tempdir().unwrap();
        let roaming = temp.path().join("Roaming");
        std::fs::create_dir_all(roaming.join("DemoApp")).unwrap();
        std::fs::write(roaming.join("DemoApp/prefs.json"), "{}").unwrap();
        std::fs::create_dir_all(roaming.join("Unrelated")).unwrap();

        let inputs = WindowsScanInputs {
            registry_apps: vec![registry_record("Demo App", None)],
            residue_roots: vec![ResidueRoot {
                path: roaming,
                category: CleanupCategory::AppSupport,
            }],
            ..WindowsScanInputs::default()
        };

        let report = scan_apps(
            &windows_opts(temp.path(), inputs),
            &AtomicBool::new(false),
            |_| {},
        );
        let items: Vec<&RelatedItem> = report.apps[0]
            .related_items
            .iter()
            .filter(|r| r.item.category == CleanupCategory::AppSupport)
            .collect();
        assert_eq!(items.len(), 1, "only the matching vendor directory");
        assert_eq!(items[0].item.name, "DemoApp");
        assert_eq!(items[0].item.risk, RiskLevel::Caution);
    }

    // ── Install-state reconciliation after cleanup ──────────────────────

    #[test]
    fn removed_bundle_demotes_entry_to_residue_only() {
        let temp = tempfile::tempdir().unwrap();
        let apps = temp.path().join("Applications");
        let library = temp.path().join("Library");
        let bundle = write_bundle(&apps, Some("com.example.demo"), Some("Demo App"));
        std::fs::create_dir_all(library.join("Application Support/com.example.demo")).unwrap();

        let mut report = scan_apps(
            &scan_opts(&[apps], &library),
            &AtomicBool::new(false),
            |_| {},
        );
        assert_eq!(report.apps[0].source, AppSource::Applications);
        assert!(report.apps[0].binary_size_bytes > 0);

        // Simulate the executor removing the bundle (residue untouched).
        std::fs::remove_dir_all(&bundle).unwrap();
        report.normalize_install_state();

        let app = &report.apps[0];
        assert!(app.app_path.is_none(), "bundle is gone");
        assert_eq!(app.binary_size_bytes, 0, "no phantom bundle bytes");
        assert_eq!(app.source, AppSource::Orphan, "no longer an installed app");
        assert_eq!(app.size_bytes, app.residue_bytes());
        assert!(!app.related_items.is_empty(), "residue stays reachable");
    }

    #[test]
    fn normalize_drops_entries_with_neither_bundle_nor_residue() {
        let temp = tempfile::tempdir().unwrap();
        let apps = temp.path().join("Applications");
        let library = temp.path().join("Library");
        let bundle = write_bundle(&apps, Some("com.example.demo"), Some("Demo App"));

        let mut report = scan_apps(
            &scan_opts(&[apps], &library),
            &AtomicBool::new(false),
            |_| {},
        );
        assert_eq!(report.apps.len(), 1);

        std::fs::remove_dir_all(&bundle).unwrap();
        report.normalize_install_state();

        assert!(report.apps.is_empty(), "fully removed apps disappear");
    }
}
