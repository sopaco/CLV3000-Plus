//! Windows app-discovery and cleanup plumbing (M2).
//!
//! The domain matching lives in `clv-core::apps`; this module only touches
//! OS-specific APIs:
//!
//! * registry enumeration of the uninstall keys (`HKLM`, `WOW6432Node`, `HKCU`);
//! * registry-residue candidates under `Software\`;
//! * Start Menu / Desktop `.lnk` collection;
//! * AppData / ProgramData residue roots;
//! * registry deletion with a mandatory `reg export` backup, restore via
//!   `reg import`, and official-uninstaller execution.
//!
//! Every public function has a non-Windows stub so the rest of the workspace
//! compiles and the macOS build is unaffected.

use clv_core::{ScanItem, TrashedEntry, TrashedEntryKind, WindowsScanInputs};
use std::path::PathBuf;

/// Result of removing registry keys. Each success carries the `.reg` backup
/// that can be re-imported from cleanup history.
#[derive(Debug, Clone, Default)]
pub struct RegistryCleanReport {
    pub succeeded: Vec<RegistryBackup>,
    pub failed: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct RegistryBackup {
    /// Full key path that was exported and deleted.
    pub key_path: String,
    /// Display name of the residue entry.
    pub name: String,
    /// `.reg` file created by `reg export`.
    pub backup_path: PathBuf,
}

/// Collects every Windows-specific input for [`clv_core::scan_apps`].
pub fn collect_windows_scan_inputs() -> WindowsScanInputs {
    #[cfg(windows)]
    {
        windows_impl::collect()
    }
    #[cfg(not(windows))]
    {
        WindowsScanInputs::default()
    }
}

/// Exports and deletes `RegistryLeftover` items. Only `HKCU` keys are ever
/// deleted (machine-wide keys are display-only / `Protected`).
pub fn clean_registry_keys(items: &[ScanItem]) -> RegistryCleanReport {
    #[cfg(windows)]
    {
        windows_impl::clean_registry_keys(items)
    }
    #[cfg(not(windows))]
    {
        RegistryCleanReport {
            succeeded: Vec::new(),
            failed: items
                .iter()
                .map(|i| {
                    (
                        i.path.display().to_string(),
                        "registry cleanup is only available on Windows".to_string(),
                    )
                })
                .collect(),
        }
    }
}

/// Re-imports a registry backup created by [`clean_registry_keys`].
pub fn restore_registry_backup(entry: &TrashedEntry) -> anyhow::Result<()> {
    if entry.kind != TrashedEntryKind::RegistryKey {
        anyhow::bail!("not a registry backup entry");
    }
    #[cfg(windows)]
    {
        windows_impl::restore(entry)
    }
    #[cfg(not(windows))]
    {
        let _ = entry;
        anyhow::bail!("registry restore is only available on Windows")
    }
}

/// Runs an app's official uninstaller (`UninstallString`), blocking until it
/// exits. The caller rescans residue afterwards.
pub fn run_uninstaller(uninstall_string: &str) -> anyhow::Result<i32> {
    #[cfg(windows)]
    {
        windows_impl::run_uninstaller(uninstall_string)
    }
    #[cfg(not(windows))]
    {
        let _ = uninstall_string;
        anyhow::bail!("running uninstallers is only available on Windows")
    }
}

#[cfg(windows)]
mod windows_impl {
    use super::*;
    use clv_core::{
        RegistryAppRecord, RegistryHive, RegistryResidueCandidate, ResidueRoot, ShortcutCandidate,
    };
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};
    use winreg::enums::*;
    use winreg::{HKEY, RegKey};

    /// Uninstall-key roots in scan order. Machine-wide first (registry apps
    /// are user-visible either way); `HKCU` follows.
    const UNINSTALL_SOURCES: &[(RegistryHive, &str)] = &[
        (
            RegistryHive::Machine,
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            RegistryHive::Machine,
            r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            RegistryHive::User,
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
    ];

    /// First-level `Software\` roots that may hold per-app residue.
    const RESIDUE_ROOTS: &[(RegistryHive, &str)] = &[
        (RegistryHive::User, "Software"),
        (RegistryHive::Machine, "SOFTWARE"),
    ];

    /// Well-known containers that never represent a single app.
    const RESIDUE_SKIP: &[&str] = &[
        "Classes",
        "Clients",
        "Microsoft",
        "Policies",
        "RegisteredApplications",
        "WOW6432Node",
        "Wow6432Node",
    ];

    fn hkey(hive: RegistryHive) -> HKEY {
        match hive {
            RegistryHive::Machine => HKEY_LOCAL_MACHINE,
            RegistryHive::User => HKEY_CURRENT_USER,
        }
    }

    fn hive_prefix(hive: RegistryHive) -> &'static str {
        hive.token()
    }

    fn read_string(key: &RegKey, name: &str) -> Option<String> {
        key.get_value::<String, _>(name)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn read_u32(key: &RegKey, name: &str) -> Option<u32> {
        if let Ok(value) = key.get_value::<u32, _>(name) {
            return Some(value);
        }
        read_string(key, name)?.parse().ok()
    }

    pub fn collect() -> WindowsScanInputs {
        WindowsScanInputs {
            registry_apps: collect_registry_apps(),
            registry_residue: collect_registry_residue(),
            shortcuts: collect_shortcuts(),
            residue_roots: collect_residue_roots(),
        }
    }

    fn collect_registry_apps() -> Vec<RegistryAppRecord> {
        let mut apps = Vec::new();
        for (hive, path) in UNINSTALL_SOURCES {
            let Ok(root) = RegKey::predef(hkey(*hive)).open_subkey_with_flags(path, KEY_READ)
            else {
                continue;
            };
            for subkey_name in root.enum_keys().flatten() {
                let Ok(key) = root.open_subkey_with_flags(&subkey_name, KEY_READ) else {
                    continue;
                };
                // System components and updates are not user-facing apps.
                if read_u32(&key, "SystemComponent") == Some(1) {
                    continue;
                }
                if read_string(&key, "ParentKeyName").is_some() {
                    continue;
                }
                let Some(display_name) = read_string(&key, "DisplayName") else {
                    continue;
                };
                let install_location = read_string(&key, "InstallLocation")
                    .map(|s| PathBuf::from(s.trim_matches('"')))
                    .filter(|p| !p.as_os_str().is_empty());
                let estimated_size_bytes = read_u32(&key, "EstimatedSize")
                    .map(|kb| u64::from(kb) * 1024)
                    .unwrap_or(0);
                apps.push(RegistryAppRecord {
                    display_name,
                    display_version: read_string(&key, "DisplayVersion"),
                    publisher: read_string(&key, "Publisher"),
                    install_location,
                    uninstall_string: read_string(&key, "UninstallString"),
                    estimated_size_bytes,
                    hive: *hive,
                    key_path: format!(r"{}\{}\{}", hive_prefix(*hive), path, subkey_name),
                });
            }
        }
        apps
    }

    fn collect_registry_residue() -> Vec<RegistryResidueCandidate> {
        let mut candidates = Vec::new();
        for (hive, path) in RESIDUE_ROOTS {
            let Ok(root) = RegKey::predef(hkey(*hive)).open_subkey_with_flags(path, KEY_READ)
            else {
                continue;
            };
            for name in root.enum_keys().flatten() {
                if RESIDUE_SKIP
                    .iter()
                    .any(|skip| skip.eq_ignore_ascii_case(&name))
                {
                    continue;
                }
                candidates.push(RegistryResidueCandidate {
                    key_path: format!(r"{}\{}\{}", hive_prefix(*hive), path, name),
                    name,
                    hive: *hive,
                });
            }
        }
        candidates
    }

    fn collect_shortcuts() -> Vec<ShortcutCandidate> {
        let mut folders = Vec::new();
        if let Ok(appdata) = std::env::var("APPDATA") {
            folders.push(PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs"));
        }
        if let Ok(programdata) = std::env::var("PROGRAMDATA") {
            folders.push(PathBuf::from(programdata).join(r"Microsoft\Windows\Start Menu\Programs"));
        }
        if let Ok(profile) = std::env::var("USERPROFILE") {
            folders.push(PathBuf::from(profile).join("Desktop"));
        }
        if let Ok(public) = std::env::var("PUBLIC") {
            folders.push(PathBuf::from(public).join("Desktop"));
        }

        let mut shortcuts = Vec::new();
        for folder in folders {
            for entry in walkdir::WalkDir::new(&folder)
                .max_depth(4)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if !path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("lnk"))
                {
                    continue;
                }
                // Target resolution needs COM `IShellLink`, which is not part
                // of `windows-sys`; the domain falls back to filename matching
                // (documented in PRD §7.2).
                shortcuts.push(ShortcutCandidate {
                    name: path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Shortcut".to_string()),
                    path: path.to_path_buf(),
                    target: None,
                    size_bytes: entry.metadata().map(|m| m.len()).unwrap_or(0),
                });
            }
        }
        shortcuts
    }

    fn collect_residue_roots() -> Vec<ResidueRoot> {
        use clv_core::CleanupCategory;
        let mut roots = Vec::new();
        if let Ok(appdata) = std::env::var("APPDATA") {
            roots.push(ResidueRoot {
                path: PathBuf::from(appdata),
                category: CleanupCategory::AppSupport,
            });
        }
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            roots.push(ResidueRoot {
                path: PathBuf::from(local),
                category: CleanupCategory::AppSupport,
            });
        }
        if let Ok(profile) = std::env::var("USERPROFILE") {
            roots.push(ResidueRoot {
                path: PathBuf::from(profile).join(r"AppData\LocalLow"),
                category: CleanupCategory::AppSupport,
            });
        }
        if let Ok(programdata) = std::env::var("PROGRAMDATA") {
            roots.push(ResidueRoot {
                path: PathBuf::from(programdata),
                category: CleanupCategory::AppSupport,
            });
        }
        roots
    }

    /// Splits `HKCU\Software\Vendor` into hive + remaining key path.
    fn parse_key_path(raw: &str) -> Option<(RegistryHive, String)> {
        let (hive, rest) = raw.split_once('\\')?;
        let hive = match hive {
            "HKCU" => RegistryHive::User,
            "HKLM" => RegistryHive::Machine,
            _ => return None,
        };
        Some((hive, rest.to_string()))
    }

    fn backup_dir() -> anyhow::Result<PathBuf> {
        let dir = directories::ProjectDirs::from("com", "clv3000", "plus")
            .map(|d| d.data_dir().join("registry_backups"))
            .ok_or_else(|| anyhow::anyhow!("no data directory"))?;
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    fn backup_file(key_path: &str, dir: &Path) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let sanitized: String = key_path
            .chars()
            .map(|c| if c == '\\' || c == '/' { '_' } else { c })
            .collect();
        dir.join(format!("{stamp}-{sanitized}.reg"))
    }

    pub fn clean_registry_keys(items: &[ScanItem]) -> RegistryCleanReport {
        let mut report = RegistryCleanReport::default();
        let Ok(dir) = backup_dir() else {
            report.failed.extend(
                items
                    .iter()
                    .map(|i| (i.path.display().to_string(), "no backup directory".into())),
            );
            return report;
        };
        for item in items {
            let raw = item.path.display().to_string();
            let Some((hive, subkey)) = parse_key_path(&raw) else {
                report
                    .failed
                    .push((raw, "unrecognized registry path".into()));
                continue;
            };
            if hive != RegistryHive::User {
                report
                    .failed
                    .push((raw, "machine-wide keys are display-only".into()));
                continue;
            }
            let backup = backup_file(&raw, &dir);
            let exported = std::process::Command::new("reg")
                .args(["export", &raw, &backup.to_string_lossy(), "/y"])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if !exported {
                report
                    .failed
                    .push((raw.clone(), "reg export failed".into()));
                continue;
            }
            let deleted = std::process::Command::new("reg")
                .args(["delete", &raw, "/f"])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if !deleted {
                // Keep the backup — it is harmless and may help recovery.
                report
                    .failed
                    .push((raw.clone(), "reg delete failed (backup kept)".into()));
                continue;
            }
            let _ = subkey;
            report.succeeded.push(RegistryBackup {
                key_path: raw,
                name: item.name.clone(),
                backup_path: backup,
            });
        }
        report
    }

    pub fn restore(entry: &TrashedEntry) -> anyhow::Result<()> {
        if !entry.trash_path.exists() {
            anyhow::bail!("registry backup is gone");
        }
        let status = std::process::Command::new("reg")
            .args(["import", &entry.trash_path.to_string_lossy()])
            .status()?;
        if !status.success() {
            anyhow::bail!("reg import failed");
        }
        Ok(())
    }

    pub fn run_uninstaller(uninstall_string: &str) -> anyhow::Result<i32> {
        let status = std::process::Command::new("cmd")
            .args(["/C", uninstall_string])
            .status()?;
        Ok(status.code().unwrap_or(-1))
    }
}
