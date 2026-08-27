use crate::models::format_bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Minimum file size to appear in the large-files view.
pub const LARGE_FILE_THRESHOLD_BYTES: u64 = 100 * 1024 * 1024;

/// Maximum large files kept in a scan report.
pub const MAX_LARGE_FILES: usize = 80;

pub const LARGE_FILE_MAX_DEPTH: usize = 6;
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

    finalize_large_files(entries)
}

pub fn finalize_large_files(mut entries: Vec<LargeFileEntry>) -> Vec<LargeFileEntry> {
    entries.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    entries.truncate(MAX_LARGE_FILES);
    entries
}

/// Record a file during a unified scan walk. Returns true if `out` is full.
pub fn consider_large_file(
    path: &Path,
    meta: &Metadata,
    depth: usize,
    out: &mut Vec<LargeFileEntry>,
) -> bool {
    if out.len() >= MAX_LARGE_FILES {
        return true;
    }
    if depth > LARGE_FILE_MAX_DEPTH || !meta.is_file() {
        return false;
    }
    let size = meta.len();
    if size < LARGE_FILE_THRESHOLD_BYTES {
        return false;
    }
    if path.ancestors().any(should_skip_dir) {
        return false;
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
    out.len() >= MAX_LARGE_FILES
}

fn collect_large_files_in_root(root: &Path, out: &mut Vec<LargeFileEntry>) {
    for entry in WalkDir::new(root)
        .follow_links(false)
        .max_depth(LARGE_FILE_MAX_DEPTH)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                !should_skip_dir(e.path())
            } else {
                true
            }
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
        if consider_large_file(path, &meta, entry.depth(), out) {
            return;
        }
    }
}

pub fn should_skip_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| SKIP_DIR_NAMES.contains(&name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn scan_large_files_includes_files_not_only_dirs() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let big = root.join("movie.bin");
        fs::File::create(&big)
            .unwrap()
            .set_len(LARGE_FILE_THRESHOLD_BYTES + 8)
            .unwrap();
        fs::create_dir_all(root.join("node_modules")).unwrap();
        fs::File::create(root.join("node_modules/skipped.bin"))
            .unwrap()
            .set_len(LARGE_FILE_THRESHOLD_BYTES + 8)
            .unwrap();

        let found = scan_large_files(&[root.to_path_buf()]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].path, big);
    }
}
