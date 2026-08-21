use crate::models::{format_bytes, RiskLevel, ScanItem};
use crate::settings::{trash_dir, AppSettings};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CleanupReport {
    pub freed_bytes: u64,
    pub success_count: usize,
    pub failed: Vec<(PathBuf, String)>,
    pub trashed: Vec<PathBuf>,
}

impl CleanupReport {
    pub fn summary(&self) -> String {
        format!(
            "已释放 {}，成功 {} 项{}",
            format_bytes(self.freed_bytes),
            self.success_count,
            if self.failed.is_empty() {
                String::new()
            } else {
                format!("，失败 {} 项", self.failed.len())
            }
        )
    }
}

pub struct CleanupExecutor {
    settings: AppSettings,
}

impl CleanupExecutor {
    pub fn new(settings: AppSettings) -> Self {
        Self { settings }
    }

    pub fn execute(&self, items: &[ScanItem]) -> CleanupReport {
        let mut report = CleanupReport {
            freed_bytes: 0,
            success_count: 0,
            failed: Vec::new(),
            trashed: Vec::new(),
        };

        for item in items {
            if !item.selected {
                continue;
            }
            if item.risk == RiskLevel::Protected && !self.settings.expert_mode {
                continue;
            }
            let size = item.size_bytes;
            match self.remove_path(&item.path) {
                Ok(trash_path) => {
                    report.freed_bytes += size;
                    report.success_count += 1;
                    if let Some(p) = trash_path {
                        report.trashed.push(p);
                    }
                }
                Err(e) => {
                    report.failed.push((item.path.clone(), e.to_string()));
                }
            }
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
            if path.is_dir() {
                fs::rename(path, &dest)?;
            } else {
                fs::rename(path, &dest)?;
            }
            Ok(Some(dest))
        } else {
            if path.is_dir() {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_file(path)?;
            }
            Ok(None)
        }
    }
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
                if meta.is_dir() {
                    fs::remove_dir_all(entry.path())?;
                } else {
                    fs::remove_file(entry.path())?;
                }
                freed += size;
            }
        }
    }
    Ok(freed)
}

fn dir_size_quick(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}
