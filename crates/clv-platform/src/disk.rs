use std::path::Path;
use sysinfo::Disks;

/// Returns `(total_bytes, used_bytes)` for system storage.
///
/// - **macOS / Linux**: picks the single volume that hosts user data. APFS exposes
///   multiple mount points for one container (`/` and `/System/Volumes/Data`);
///   summing them would double-count capacity.
/// - **Windows**: sums every local fixed disk (all drive letters such as `C:\`,
///   `D:\`, …), excluding removable and network volumes.
pub fn primary_disk_usage() -> Option<(u64, u64)> {
    let disks = Disks::new_with_refreshed_list();
    disk_usage_from_disks(disks.list())
}

#[cfg(target_os = "windows")]
fn disk_usage_from_disks(disks: &[sysinfo::Disk]) -> Option<(u64, u64)> {
    sum_local_fixed_disks(disks.iter().map(|disk| MountStats {
        mount: disk.mount_point(),
        total: disk.total_space(),
        available: disk.available_space(),
        removable: disk.is_removable(),
    }))
}

#[cfg(not(target_os = "windows"))]
fn disk_usage_from_disks(disks: &[sysinfo::Disk]) -> Option<(u64, u64)> {
    disk_usage_for_target(disks, &primary_disk_target())
}

#[cfg(not(target_os = "windows"))]
fn primary_disk_target() -> std::path::PathBuf {
    #[cfg(target_os = "macos")]
    {
        let data = std::path::PathBuf::from("/System/Volumes/Data");
        if data.exists() {
            return data;
        }
    }
    directories::UserDirs::new()
        .map(|u| u.home_dir().to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("/"))
}

#[cfg(target_os = "windows")]
struct MountStats<'a> {
    mount: &'a Path,
    total: u64,
    available: u64,
    removable: bool,
}

#[cfg(target_os = "windows")]
fn sum_local_fixed_disks<'a>(mounts: impl Iterator<Item = MountStats<'a>>) -> Option<(u64, u64)> {
    let mut total = 0u64;
    let mut available = 0u64;

    for mount in mounts {
        if mount.removable || !is_windows_drive_letter(mount.mount) || mount.total == 0 {
            continue;
        }

        total = total.saturating_add(mount.total);
        available = available.saturating_add(mount.available);
    }

    if total == 0 {
        None
    } else {
        Some((total, total.saturating_sub(available)))
    }
}

#[cfg(target_os = "windows")]
fn is_windows_drive_letter(path: &Path) -> bool {
    let bytes = path.as_os_str().as_encoded_bytes();
    bytes.len() == 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
}

#[cfg(not(target_os = "windows"))]
fn disk_usage_for_target(disks: &[sysinfo::Disk], target: &Path) -> Option<(u64, u64)> {
    select_mount_stats(
        disks.iter().map(|disk| {
            (
                disk.mount_point(),
                disk.total_space(),
                disk.available_space(),
            )
        }),
        target,
    )
}

#[cfg(not(target_os = "windows"))]
fn select_mount_stats<'a>(
    mounts: impl Iterator<Item = (&'a Path, u64, u64)>,
    target: &Path,
) -> Option<(u64, u64)> {
    let (_, total, available) = mounts
        .filter(|(mount, _, _)| target.starts_with(mount))
        .max_by_key(|(mount, _, _)| mount_prefix_len(mount))?;

    if total == 0 {
        return None;
    }

    Some((total, total.saturating_sub(available)))
}

#[cfg(not(target_os = "windows"))]
fn mount_prefix_len(path: &Path) -> usize {
    path.as_os_str().len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn picks_longest_mount_prefix_on_macos() {
        let mounts = [
            (PathBuf::from("/"), 1_000_u64, 400_u64),
            (PathBuf::from("/System/Volumes/Data"), 1_000, 300),
        ];
        let target = Path::new("/System/Volumes/Data/Users/test");
        let (total, used) = select_mount_stats(
            mounts.iter().map(|(m, t, a)| (m.as_path(), *t, *a)),
            target,
        )
        .unwrap();
        assert_eq!(total, 1_000);
        assert_eq!(used, 700);
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn picks_root_when_target_is_home_on_legacy_layout() {
        let mounts = [(PathBuf::from("/"), 2_000_u64, 500_u64)];
        let target = Path::new("/Users/test");
        let (total, used) = select_mount_stats(
            mounts.iter().map(|(m, t, a)| (m.as_path(), *t, *a)),
            target,
        )
        .unwrap();
        assert_eq!(total, 2_000);
        assert_eq!(used, 1_500);
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn returns_none_when_no_mount_matches() {
        let mounts = [(PathBuf::from("/Volumes/USB"), 500_u64, 100_u64)];
        let target = Path::new("/Users/test");
        assert!(select_mount_stats(
            mounts.iter().map(|(m, t, a)| (m.as_path(), *t, *a)),
            target,
        )
        .is_none());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn sums_all_local_fixed_drive_letters() {
        let mounts = [
            MountStats {
                mount: Path::new(r"C:\"),
                total: 1_000,
                available: 400,
                removable: false,
            },
            MountStats {
                mount: Path::new(r"D:\"),
                total: 2_000,
                available: 500,
                removable: false,
            },
            MountStats {
                mount: Path::new(r"E:\"),
                total: 500,
                available: 100,
                removable: true,
            },
        ];
        let (total, used) = sum_local_fixed_disks(mounts.into_iter()).unwrap();
        assert_eq!(total, 3_000);
        assert_eq!(used, 2_100);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn ignores_folder_mount_points_to_avoid_double_counting() {
        let mounts = [
            MountStats {
                mount: Path::new(r"D:\"),
                total: 1_000,
                available: 400,
                removable: false,
            },
            MountStats {
                mount: Path::new(r"C:\Mount\Data"),
                total: 1_000,
                available: 400,
                removable: false,
            },
        ];
        let (total, used) = sum_local_fixed_disks(mounts.into_iter()).unwrap();
        assert_eq!(total, 1_000);
        assert_eq!(used, 600);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn recognizes_drive_letter_mount_points() {
        assert!(is_windows_drive_letter(Path::new(r"C:\")));
        assert!(is_windows_drive_letter(Path::new(r"Z:/")));
        assert!(!is_windows_drive_letter(Path::new(r"C:\Users")));
        assert!(!is_windows_drive_letter(Path::new(r"/")));
    }
}
