use crate::models::{RiskLevel, ScanItem};
use crate::settings::{trash_dir, AppSettings};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CleanupProgress {
    pub completed: usize,
    pub total: usize,
    pub current_path: PathBuf,
    pub freed_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct CleanupReport {
    pub freed_bytes: u64,
    pub success_count: usize,
    pub successful_paths: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, String)>,
    pub trashed: Vec<PathBuf>,
    pub trashed_entries: Vec<TrashedEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrashedEntry {
    pub original: PathBuf,
    pub trash_path: PathBuf,
    pub size_bytes: u64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupHistoryRecord {
    pub timestamp: DateTime<Utc>,
    pub freed_bytes: u64,
    pub success_count: usize,
    pub failed_count: usize,
    #[serde(default)]
    pub trashed: Vec<TrashedEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupHistory {
    pub records: Vec<CleanupHistoryRecord>,
}

impl CleanupHistory {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn load() -> Self {
        let Some(path) = cleanup_history_path() else {
            return Self::new();
        };
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(Self::new)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let Some(path) = cleanup_history_path() else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn append(&mut self, record: CleanupHistoryRecord) {
        self.records.push(record);
        self.prune_old();
    }

    fn prune_old(&mut self) {
        let cutoff = Utc::now() - Duration::days(90);
        self.records.retain(|r| r.timestamp >= cutoff);
    }

    pub fn freed_in_days(&self, days: i64) -> u64 {
        let cutoff = Utc::now() - Duration::days(days);
        self.records
            .iter()
            .filter(|r| r.timestamp >= cutoff)
            .map(|r| r.freed_bytes)
            .sum()
    }

    pub fn success_count_in_days(&self, days: i64) -> usize {
        let cutoff = Utc::now() - Duration::days(days);
        self.records
            .iter()
            .filter(|r| r.timestamp >= cutoff)
            .map(|r| r.success_count)
            .sum()
    }

    pub fn failed_count_in_days(&self, days: i64) -> usize {
        let cutoff = Utc::now() - Duration::days(days);
        self.records
            .iter()
            .filter(|r| r.timestamp >= cutoff)
            .map(|r| r.failed_count)
            .sum()
    }

    pub fn cleanup_count_in_days(&self, days: i64) -> usize {
        let cutoff = Utc::now() - Duration::days(days);
        self.records.iter().filter(|r| r.timestamp >= cutoff).count()
    }

    pub fn restorable_entries(&self) -> Vec<TrashedEntry> {
        let mut entries: Vec<TrashedEntry> = self
            .records
            .iter()
            .rev()
            .flat_map(|r| r.trashed.iter().cloned())
            .filter(|e| e.trash_path.exists())
            .collect();
        entries.dedup_by(|a, b| a.trash_path == b.trash_path);
        entries
    }

    pub fn remove_trashed(&mut self, trash_path: &Path) {
        for record in &mut self.records {
            record.trashed.retain(|e| e.trash_path != trash_path);
        }
    }

    /// Forget recycle-area records whose files are gone (purged by hand, by
    /// age, or by a failed purge that left a remainder).
    pub fn prune_missing_trash(&mut self) {
        for record in &mut self.records {
            record.trashed.retain(|e| e.trash_path.exists());
        }
    }
}

fn cleanup_history_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("com", "clv3000", "plus")
        .map(|d| d.config_dir().join("cleanup_history.json"))
}

pub struct CleanupExecutor {
    settings: AppSettings,
    /// Recycle-area root override. `None` falls back to [`trash_dir`]; tests
    /// inject a temp directory so they never touch real application data.
    trash_root: Option<PathBuf>,
}

impl CleanupExecutor {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            settings,
            trash_root: None,
        }
    }

    /// Points the recycle area at `root` instead of the per-user one. Tests use
    /// this to keep `cargo test` from writing into the real recycle area.
    pub fn with_trash_root(mut self, root: PathBuf) -> Self {
        self.trash_root = Some(root);
        self
    }

    fn trash_root(&self) -> Option<PathBuf> {
        self.trash_root.clone().or_else(trash_dir)
    }

    pub fn execute<F>(&self, items: &[ScanItem], on_progress: F) -> CleanupReport
    where
        F: FnMut(CleanupProgress),
    {
        self.execute_cancellable(items, &AtomicBool::new(false), on_progress)
    }

    pub fn execute_cancellable<F>(
        &self,
        items: &[ScanItem],
        cancel: &AtomicBool,
        mut on_progress: F,
    ) -> CleanupReport
    where
        F: FnMut(CleanupProgress),
    {
        let mut report = CleanupReport {
            freed_bytes: 0,
            success_count: 0,
            successful_paths: Vec::new(),
            failed: Vec::new(),
            trashed: Vec::new(),
            trashed_entries: Vec::new(),
        };

        let runnable: Vec<&ScanItem> = items
            .iter()
            .filter(|item| {
                !(item.risk == RiskLevel::Protected && !self.settings.expert_mode)
            })
            .collect();
        let total = runnable.len();

        for (index, item) in runnable.iter().enumerate() {
            if cancel.load(Ordering::Relaxed) {
                break;
            }
            on_progress(CleanupProgress {
                completed: index,
                total,
                current_path: item.path.clone(),
                freed_bytes: report.freed_bytes,
            });

            let size = item.size_bytes;
            match self.remove_path(&item.path) {
                Ok(trash_path) => {
                    report.freed_bytes += size;
                    report.success_count += 1;
                    report.successful_paths.push(item.path.clone());
                    if let Some(p) = trash_path {
                        report.trashed.push(p.clone());
                        report.trashed_entries.push(TrashedEntry {
                            original: item.path.clone(),
                            trash_path: p,
                            size_bytes: size,
                            name: item.name.clone(),
                        });
                    }
                }
                Err(e) => {
                    report.failed.push((item.path.clone(), e.to_string()));
                }
            }

            on_progress(CleanupProgress {
                completed: index + 1,
                total,
                current_path: item.path.clone(),
                freed_bytes: report.freed_bytes,
            });
        }

        report
    }

    fn remove_path(&self, path: &Path) -> anyhow::Result<Option<PathBuf>> {
        if !path.exists() {
            return Ok(None);
        }

        if self.settings.soft_delete {
            let trash = self
                .trash_root()
                .ok_or_else(|| anyhow::anyhow!("no trash dir"))?;
            fs::create_dir_all(&trash)?;
            let stamp = Utc::now().format("%Y%m%d-%H%M%S");
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "item".into());
            let dest = trash.join(format!("{stamp}-{name}-{}", Uuid::new_v4()));
            move_entry(path, &dest)?;
            Ok(Some(dest))
        } else {
            force_remove(path)?;
            Ok(None)
        }
    }
}

/// Total size of `items` that live on a different volume than the recycle area.
///
/// Moving those entries into the trash needs a copy+delete fallback, which
/// temporarily consumes space on the trash volume. Callers use this to refuse a
/// cleanup that would fill the system drive before deleting anything.
pub fn cross_volume_bytes<F>(items: &[ScanItem], trash: &Path, volume_of: F) -> u64
where
    F: Fn(&Path) -> String,
{
    let trash_volume = volume_of(trash);
    items
        .iter()
        .filter(|item| volume_of(&item.path) != trash_volume)
        .map(|item| item.size_bytes)
        .sum()
}

/// Move or rename `src` to `dest`, falling back to copy+delete on cross-volume moves (Windows).
fn move_entry(src: &Path, dest: &Path) -> io::Result<()> {
    match fs::rename(src, dest) {
        Ok(()) => Ok(()),
        Err(e) if is_cross_device_error(&e) => {
            if src.is_dir() {
                copy_dir_all(src, dest)?;
                force_remove(src)?;
            } else {
                fs::copy(src, dest)?;
                force_remove(src)?;
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn force_remove(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        clear_readonly_tree(path)?;
        fs::remove_dir_all(path)
    } else {
        clear_readonly(path)?;
        fs::remove_file(path)
    }
}

fn clear_readonly_tree(root: &Path) -> io::Result<()> {
    if root.is_dir() {
        for entry in walkdir::WalkDir::new(root).contents_first(true) {
            clear_readonly(&entry?.path())?;
        }
    }
    clear_readonly(root)
}

#[cfg(windows)]
fn clear_readonly(path: &Path) -> io::Result<()> {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_READONLY: u32 = 0x1;
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e),
    };
    if meta.file_attributes() & FILE_ATTRIBUTE_READONLY != 0 {
        let mut perms = meta.permissions();
        perms.set_readonly(false);
        fs::set_permissions(path, perms)?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn clear_readonly(_path: &Path) -> io::Result<()> {
    Ok(())
}

fn is_cross_device_error(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::CrossesDevices || error.raw_os_error() == Some(17)
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Actual bytes and entry count currently in the recycle area.
///
/// Walks the trash directory instead of summing history records: recorded
/// `size_bytes` are scan-time snapshots and can differ from what is on disk,
/// while orphaned entries have no record at all. The UI must report what a
/// permanent delete will really free.
pub fn trash_usage() -> (u64, usize) {
    let Some(trash) = trash_dir() else {
        return (0, 0);
    };
    trash_usage_at(&trash)
}

fn trash_usage_at(trash: &Path) -> (u64, usize) {
    let Ok(entries) = fs::read_dir(trash) else {
        return (0, 0);
    };
    let mut bytes = 0u64;
    let mut count = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        let size = match entry.metadata() {
            Ok(meta) if meta.is_dir() => dir_size_on_disk(&path),
            Ok(meta) => on_disk_size(&meta),
            Err(_) => continue,
        };
        bytes += size;
        count += 1;
    }
    (bytes, count)
}

pub fn purge_old_trash(days: u32) -> anyhow::Result<u64> {
    purge_old_trash_at(days, Utc::now(), &CleanupHistory::load())
}

/// Age-based purge.
///
/// A trashed entry's age comes from the history record that moved it, not from
/// the file's mtime: copy+delete moves and restores can refresh mtime, which
/// would otherwise make old entries immortal. Only orphan entries (no record,
/// e.g. after history pruning) fall back to mtime.
fn purge_old_trash_at(
    days: u32,
    now: DateTime<Utc>,
    history: &CleanupHistory,
) -> anyhow::Result<u64> {
    let Some(trash) = trash_dir() else {
        return Ok(0);
    };
    purge_old_trash_in(&trash, days, now, history)
}

fn purge_old_trash_in(
    trash: &Path,
    days: u32,
    now: DateTime<Utc>,
    history: &CleanupHistory,
) -> anyhow::Result<u64> {
    if !trash.exists() {
        return Ok(0);
    }
    let cutoff = now - Duration::days(days as i64);
    let mut recorded: HashMap<&Path, DateTime<Utc>> = HashMap::new();
    for record in &history.records {
        for entry in &record.trashed {
            recorded
                .entry(entry.trash_path.as_path())
                .and_modify(|timestamp| {
                    if record.timestamp > *timestamp {
                        *timestamp = record.timestamp;
                    }
                })
                .or_insert(record.timestamp);
        }
    }

    let mut freed = 0u64;
    for entry in fs::read_dir(trash)?.flatten() {
        let path = entry.path();
        let meta = match entry.metadata() {
            Ok(meta) => meta,
            Err(e) => {
                tracing::warn!("purge old trash stat {}: {e}", path.display());
                continue;
            }
        };
        let created_at = recorded.get(path.as_path()).copied().or_else(|| {
            meta.modified()
                .ok()
                .map(|modified| DateTime::<Utc>::from(modified))
        });
        let Some(created_at) = created_at else {
            continue;
        };
        if created_at < cutoff {
            let size = if meta.is_dir() {
                dir_size_on_disk(&path)
            } else {
                on_disk_size(&meta)
            };
            // One unreadable entry must not abort the whole purge — the same
            // policy as `purge_trash_dir`. Skipped entries stay in the recycle
            // area and remain counted/purgeable.
            match force_remove(&path) {
                Ok(()) => freed += size,
                Err(e) => tracing::warn!("purge old trash remove {}: {e}", path.display()),
            }
        }
    }
    Ok(freed)
}

/// Permanently delete everything in the recycle area.
///
/// Returns `(freed_bytes, removed_count, remaining_count)`. Entries that cannot
/// be removed (locked, or owned by another user) are skipped with a warning so
/// one bad file does not abort the whole purge; `remaining_count` is how many
/// top-level entries are still there afterwards, so the UI can avoid claiming
/// "emptied" when it is not. The caller re-reads [`trash_usage`] to refresh the
/// pending figure.
pub fn purge_all_trash() -> anyhow::Result<(u64, usize, usize)> {
    let Some(trash) = trash_dir() else {
        return Ok((0, 0, 0));
    };
    purge_trash_dir(&trash)
}

fn purge_trash_dir(trash: &Path) -> anyhow::Result<(u64, usize, usize)> {
    if !trash.exists() {
        return Ok((0, 0, 0));
    }
    let mut freed = 0u64;
    let mut removed = 0usize;
    let mut total = 0usize;
    for entry in fs::read_dir(trash)?.flatten() {
        total += 1;
        let path = entry.path();
        let size = match entry.metadata() {
            Ok(meta) if meta.is_dir() => dir_size_on_disk(&path),
            Ok(meta) => on_disk_size(&meta),
            Err(e) => {
                tracing::warn!("purge trash stat {}: {e}", path.display());
                continue;
            }
        };
        match force_remove(&path) {
            Ok(()) => {
                freed += size;
                removed += 1;
            }
            Err(e) => tracing::warn!("purge trash remove {}: {e}", path.display()),
        }
    }
    Ok((freed, removed, total - removed))
}

pub fn restore_trashed(entry: &TrashedEntry) -> anyhow::Result<()> {
    if !entry.trash_path.exists() {
        anyhow::bail!("trash item is gone");
    }
    if entry.original.exists() {
        anyhow::bail!("original path already exists");
    }
    if let Some(parent) = entry.original.parent() {
        fs::create_dir_all(parent)?;
    }
    move_entry(&entry.trash_path, &entry.original)?;
    Ok(())
}

/// Disk space a single file occupies. Allocated blocks where the platform
/// exposes them, so reported numbers match what `du` shows and what a delete
/// actually returns to `df`; logical size elsewhere.
fn on_disk_size(meta: &fs::Metadata) -> u64 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        meta.blocks() * 512
    }
    #[cfg(not(unix))]
    {
        meta.len()
    }
}

/// Sum of [`on_disk_size`] over everything under `path`.
fn dir_size_on_disk(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .map(|m| on_disk_size(&m))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::CleanupCategory;
    use crate::messages::RuleDescription;
    use crate::models::{RiskLevel, ScanItem, TechStack};
    use crate::settings::AppSettings;

    fn scan_item(path: PathBuf) -> ScanItem {
        ScanItem {
            id: "test".into(),
            path,
            name: "item".into(),
            size_bytes: 1024,
            stack: TechStack::Rust,
            risk: RiskLevel::Safe,
            category: CleanupCategory::CompileCache,
            description: RuleDescription::R001,
            project_root: None,
            last_modified: None,
        }
    }

    #[test]
    fn soft_delete_moves_directory_into_trash() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("target");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("artifact"), "x".repeat(1024)).unwrap();

        let mut settings = AppSettings::default();
        settings.soft_delete = true;
        // Hermetic recycle area: without this the executor would drop test
        // artifacts into the developer's real recycle area.
        let report = CleanupExecutor::new(settings)
            .with_trash_root(temp.path().join("trash"))
            .execute(&[scan_item(src.clone())], |_| {});

        assert_eq!(report.success_count, 1);
        assert!(!src.exists());
        assert_eq!(report.successful_paths, vec![src]);
        assert_eq!(report.trashed.len(), 1);
        assert!(report.trashed[0].exists());
    }

    #[test]
    fn hard_delete_removes_directory() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("node_modules");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("pkg.js"), "x".repeat(1024)).unwrap();

        let mut settings = AppSettings::default();
        settings.soft_delete = false;
        let report =
            CleanupExecutor::new(settings).execute(&[scan_item(src.clone())], |_| {});

        assert_eq!(report.success_count, 1);
        assert!(!src.exists());
        assert!(report.trashed.is_empty());
    }

    #[test]
    fn protected_items_are_skipped_without_expert_mode() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("toolchains");
        fs::create_dir_all(&src).unwrap();

        let mut item = scan_item(src.clone());
        item.risk = RiskLevel::Protected;

        let settings = AppSettings::default();
        let report = CleanupExecutor::new(settings).execute(&[item], |_| {});

        assert_eq!(report.success_count, 0);
        assert!(src.exists());
        assert!(report.successful_paths.is_empty());
    }

    #[test]
    fn missing_paths_count_as_success_without_failure() {
        let missing = PathBuf::from("/definitely/missing/path/for/clv-test");
        let mut settings = AppSettings::default();
        settings.soft_delete = false;

        let report =
            CleanupExecutor::new(settings).execute(&[scan_item(missing.clone())], |_| {});

        assert_eq!(report.success_count, 1);
        assert_eq!(report.successful_paths, vec![missing]);
        assert!(report.failed.is_empty());
    }

    #[test]
    fn move_entry_falls_back_when_rename_crosses_devices() {
        if is_cross_device_error(&io::Error::from_raw_os_error(17)) {
            // Windows / platforms that surface ERROR_NOT_SAME_DEVICE.
        }
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src.txt");
        let dest = temp.path().join("nested").join("dest.txt");
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::write(&src, "payload").unwrap();

        move_entry(&src, &dest).unwrap();
        assert!(!src.exists());
        assert_eq!(fs::read_to_string(dest).unwrap(), "payload");
    }

    #[test]
    fn cross_volume_bytes_sums_only_foreign_volumes() {
        let mut a = scan_item(PathBuf::from("/data/a"));
        a.size_bytes = 100;
        let mut b = scan_item(PathBuf::from("/data/b"));
        b.size_bytes = 200;
        let mut c = scan_item(PathBuf::from("/mnt/ext/c"));
        c.size_bytes = 400;

        let volume_of = |path: &Path| {
            if path.starts_with("/mnt/ext") {
                "/mnt/ext".to_string()
            } else {
                "/".to_string()
            }
        };

        let total = cross_volume_bytes(&[a, b, c], Path::new("/data/trash"), volume_of);
        assert_eq!(total, 400);
    }

    #[test]
    fn cross_volume_bytes_is_zero_when_everything_is_local() {
        let a = scan_item(PathBuf::from("/data/a"));
        let total = cross_volume_bytes(&[a], Path::new("/data/trash"), |_| "/".to_string());
        assert_eq!(total, 0);
    }

    fn history_with_trashed_at(path: &Path, timestamp: DateTime<Utc>) -> CleanupHistory {
        let mut history = CleanupHistory::new();
        history.append(CleanupHistoryRecord {
            timestamp,
            freed_bytes: 1024,
            success_count: 1,
            failed_count: 0,
            trashed: vec![TrashedEntry {
                original: PathBuf::from("/original"),
                trash_path: path.to_path_buf(),
                size_bytes: 1024,
                name: "item".into(),
            }],
        });
        history
    }

    #[test]
    fn trash_usage_matches_actual_contents_including_orphans() {
        let temp = tempfile::tempdir().unwrap();
        let trash = temp.path().join("trash");
        fs::create_dir_all(&trash).unwrap();
        // Orphan directory: present on disk but absent from history.
        let orphan = trash.join("20260827-orphan-target");
        fs::create_dir_all(&orphan).unwrap();
        fs::write(orphan.join("pkg.js"), "x".repeat(2048)).unwrap();
        // Loose orphan file.
        fs::write(trash.join("loose.log"), "x".repeat(512)).unwrap();

        let expected = dir_size_on_disk(&orphan)
            + on_disk_size(&fs::metadata(trash.join("loose.log")).unwrap());
        assert_eq!(trash_usage_at(&trash), (expected, 2));

        // What the UI reports must equal what a permanent delete frees.
        let (freed, removed, remaining) = purge_trash_dir(&trash).unwrap();
        assert_eq!((freed, removed, remaining), (expected, 2, 0));
        assert_eq!(trash_usage_at(&trash), (0, 0));
    }

    #[test]
    fn old_records_expire_even_when_mtime_is_fresh() {
        let temp = tempfile::tempdir().unwrap();
        let trash = temp.path().join("trash");
        fs::create_dir_all(&trash).unwrap();
        let path = trash.join("20260827-old-target");
        fs::write(&path, "x".repeat(1024)).unwrap();
        // The file was just written, so its mtime is "now"; only the history
        // record knows it is eight days old.
        let now = Utc::now();
        let history = history_with_trashed_at(&path, now - Duration::days(8));

        let freed = purge_old_trash_in(&trash, 7, now, &history).unwrap();
        assert!(freed > 0);
        assert!(!path.exists());
    }

    #[test]
    fn recent_records_are_kept() {
        let temp = tempfile::tempdir().unwrap();
        let trash = temp.path().join("trash");
        fs::create_dir_all(&trash).unwrap();
        let path = trash.join("recent-target");
        fs::write(&path, "x".repeat(1024)).unwrap();
        let now = Utc::now();
        let history = history_with_trashed_at(&path, now - Duration::days(3));

        let freed = purge_old_trash_in(&trash, 7, now, &history).unwrap();
        assert_eq!(freed, 0);
        assert!(path.exists());
    }

    #[test]
    fn orphan_entries_fall_back_to_mtime() {
        let temp = tempfile::tempdir().unwrap();
        let trash = temp.path().join("trash");
        fs::create_dir_all(&trash).unwrap();
        let path = trash.join("orphan");
        fs::write(&path, "x".repeat(512)).unwrap();
        // No record and a fresh mtime: keep it rather than guessing from the
        // name.
        let now = Utc::now();
        assert_eq!(
            purge_old_trash_in(&trash, 7, now, &CleanupHistory::new()).unwrap(),
            0
        );
        assert!(path.exists());
    }

    #[test]
    fn purge_trash_dir_removes_everything_and_reports_size() {
        let temp = tempfile::tempdir().unwrap();
        let trash = temp.path().join("trash");
        let dir = trash.join("stamp-node_modules-uuid");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("pkg.js"), "x".repeat(1024)).unwrap();
        fs::write(trash.join("loose.log"), "x".repeat(512)).unwrap();

        let expected = dir_size_on_disk(&dir)
            + on_disk_size(&fs::metadata(trash.join("loose.log")).unwrap());
        let (freed, removed, remaining) = purge_trash_dir(&trash).unwrap();

        assert_eq!(removed, 2);
        assert_eq!(freed, expected);
        assert_eq!(remaining, 0);
        assert!(fs::read_dir(&trash).unwrap().next().is_none());
    }

    #[test]
    fn purge_trash_dir_tolerates_missing_directory() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("never-created");
        assert_eq!(purge_trash_dir(&missing).unwrap(), (0, 0, 0));
    }

    #[test]
    #[cfg(unix)]
    fn purge_trash_dir_reports_undeletable_entries_as_remaining() {
        let temp = tempfile::tempdir().unwrap();
        let trash = temp.path().join("trash");
        let locked = trash.join("locked-dir");
        fs::create_dir_all(locked.join("inner")).unwrap();
        fs::write(trash.join("removable.log"), "x".repeat(512)).unwrap();
        // A write-protected directory cannot be emptied by this process, the
        // same way root-owned npm cache dirs behave in real recycle areas.
        use std::os::unix::fs::PermissionsExt;
        for dir in [locked.join("inner"), locked.clone()] {
            let mut perms = fs::metadata(&dir).unwrap().permissions();
            perms.set_mode(0o555);
            fs::set_permissions(&dir, perms).unwrap();
        }

        let (freed, removed, remaining) = purge_trash_dir(&trash).unwrap();

        assert_eq!(removed, 1);
        assert_eq!(remaining, 1);
        assert!(freed >= 512);
        assert!(locked.exists());

        // Restore permissions so the temp dir can be cleaned up.
        let mut perms = fs::metadata(&locked).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&locked, perms).unwrap();
    }
}
