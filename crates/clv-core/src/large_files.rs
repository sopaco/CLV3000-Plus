use crate::models::format_bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Minimum file size to appear in the large-files view.
pub const LARGE_FILE_THRESHOLD_BYTES: u64 = 500 * 1024 * 1024;

/// Maximum large files kept in a scan report.
pub const MAX_LARGE_FILES: usize = 80;

const LARGE_FILE_MAX_DEPTH: usize = 6;
const SKIP_DIR_NAMES: &[&str] = &[".git", ".svn", ".hg", ".terrain", "node_modules", "target"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeFileEntry {
    pub path: PathBuf,
    pub name: String,
    pub size_bytes: u64,
    pub last_modified: Option<DateTime<Utc>>,
}

impl LargeFileEntry {
    pub fn size_human(&self) -> String {
        format_bytes(self.size_bytes)
    }
}

/// Walks scan roots and collects individual files above [`LARGE_FILE_THRESHOLD_BYTES`].
pub fn scan_large_files(roots: &[PathBuf]) -> Vec<LargeFileEntry> {
    let mut entries = Vec::new();

    for root in roots {
        if !root.exists() {
            continue;
        }
        collect_large_files_in_root(root, &mut entries);
        if entries.len() >= MAX_LARGE_FILES {
            break;
        }
    }

    entries.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    entries.truncate(MAX_LARGE_FILES);
    entries
}

fn collect_large_files_in_root(root: &Path, out: &mut Vec<LargeFileEntry>) {
    for entry in WalkDir::new(root)
        .follow_links(false)
        .max_depth(LARGE_FILE_MAX_DEPTH)
        .into_iter()
        .filter_entry(|e| {
            e.file_type().is_dir()
                && !should_skip_dir(e.path())
        })
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let Ok(meta) = path.metadata() else {
            continue;
        };
        let size = meta.len();
        if size < LARGE_FILE_THRESHOLD_BYTES {
            continue;
        }
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());
        let last_modified = meta.modified().ok().map(DateTime::<Utc>::from);
        out.push(LargeFileEntry {
            path: path.to_path_buf(),
            name,
            size_bytes: size,
            last_modified,
        });
        if out.len() >= MAX_LARGE_FILES {
            return;
        }
    }
}

fn should_skip_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| SKIP_DIR_NAMES.contains(&name))
}
