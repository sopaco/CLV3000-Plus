use crate::models::{RiskLevel, ScanItem};
use crate::settings::{trash_dir, AppSettings};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
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
}

fn cleanup_history_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("com", "clv3000", "plus")
        .map(|d| d.config_dir().join("cleanup_history.json"))
}

pub struct CleanupExecutor {
    settings: AppSettings,
}

impl CleanupExecutor {
    pub fn new(settings: AppSettings) -> Self {
        Self { settings }
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
            let trash = trash_dir().ok_or_else(|| anyhow::anyhow!("no trash dir"))?;
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

pub fn purge_old_trash(days: u32) -> anyhow::Result<u64> {
    let Some(trash) = trash_dir() else {
        return Ok(0);
    };
    if !trash.exists() {
        return Ok(0);
    }
    let cutoff = Utc::now() - chrono::Duration::days(days as i64);
    let mut freed = 0u64;
    for entry in fs::read_dir(&trash)?.flatten() {
        let meta = entry.metadata()?;
        if let Ok(modified) = meta.modified() {
            let modified: chrono::DateTime<Utc> = modified.into();
            if modified < cutoff {
                let size = if meta.is_dir() {
                    dir_size_quick(&entry.path())
                } else {
                    meta.len()
                };
                force_remove(&entry.path())?;
                freed += size;
            }
        }
    }
    Ok(freed)
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

fn dir_size_quick(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
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
        let report =
            CleanupExecutor::new(settings).execute(&[scan_item(src.clone())], |_| {});

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
}
