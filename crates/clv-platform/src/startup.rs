use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartupImpact {
    Low,
    Medium,
    High,
}

impl StartupImpact {
    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "低",
            Self::Medium => "中",
            Self::High => "高",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StartupKind {
    LoginItem,
    LaunchAgent,
    LaunchDaemon,
    ScheduledTask,
    RegistryRun,
    StartupFolder,
    Service,
}

impl StartupKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::LoginItem => "登录项",
            Self::LaunchAgent => "LaunchAgent",
            Self::LaunchDaemon => "后台服务",
            Self::ScheduledTask => "计划任务",
            Self::RegistryRun => "注册表启动",
            Self::StartupFolder => "启动文件夹",
            Self::Service => "系统服务",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupItem {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub impact: StartupImpact,
    pub kind: StartupKind,
    pub path: Option<PathBuf>,
    pub description: String,
}

pub fn list_startup_items() -> Vec<StartupItem> {
    #[cfg(target_os = "macos")]
    {
        return macos::list_startup_items();
    }
    #[cfg(target_os = "windows")]
    {
        return windows::list_startup_items();
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Vec::new()
    }
}

pub fn set_startup_enabled(id: &str, enabled: bool) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    {
        return macos::set_startup_enabled(id, enabled);
    }
    #[cfg(target_os = "windows")]
    {
        return windows::set_startup_enabled(id, enabled);
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = (id, enabled);
        anyhow::bail!("unsupported platform")
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::fs;

    pub fn list_startup_items() -> Vec<StartupItem> {
        let mut items = Vec::new();
        scan_launch_agents(&mut items, true);
        scan_launch_agents(&mut items, false);
        scan_login_items(&mut items);
        items.sort_by(|a, b| a.name.cmp(&b.name));
        items
    }

    fn scan_launch_agents(items: &mut Vec<StartupItem>, user: bool) {
        let dirs = if user {
            directories::UserDirs::new()
                .map(|u| vec![u.home_dir().join("Library/LaunchAgents")])
                .unwrap_or_default()
        } else {
            vec![PathBuf::from("/Library/LaunchAgents")]
        };

        for dir in dirs {
            if !dir.exists() {
                continue;
            }
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("plist") {
                    continue;
                }
                let name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".into());
                let disabled = path.with_extension("plist.disabled").exists();
                let display = path.display().to_string();
                items.push(StartupItem {
                    id: format!("launchagent:{}", path.display()),
                    name: name.clone(),
                    enabled: !disabled,
                    impact: guess_impact(&name),
                    kind: if user {
                        StartupKind::LaunchAgent
                    } else {
                        StartupKind::LaunchDaemon
                    },
                    path: Some(path),
                    description: format!("LaunchAgent: {display}"),
                });
            }
        }
    }

    fn scan_login_items(items: &mut Vec<StartupItem>) {
        // Parse `osascript -e 'tell application "System Events" to get the name of every login item'`
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to get the name of every login item")
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                for name in text.split(',').map(|s| s.trim()) {
                    if name.is_empty() {
                        continue;
                    }
                    items.push(StartupItem {
                        id: format!("loginitem:{name}"),
                        name: name.to_string(),
                        enabled: true,
                        impact: guess_impact(name),
                        kind: StartupKind::LoginItem,
                        path: None,
                        description: "macOS 登录时自动启动".into(),
                    });
                }
            }
        }
    }

    pub fn set_startup_enabled(id: &str, enabled: bool) -> anyhow::Result<()> {
        if let Some(path) = id.strip_prefix("launchagent:") {
            let path = PathBuf::from(path);
            let disabled = path.with_extension("plist.disabled");
            if enabled {
                if disabled.exists() {
                    fs::rename(&disabled, &path)?;
                }
            } else if path.exists() {
                fs::rename(&path, &disabled)?;
            }
            return Ok(());
        }

        if id.starts_with("loginitem:") {
            anyhow::bail!("登录项请在系统设置中管理（暂不支持自动禁用）");
        }

        anyhow::bail!("unknown startup item id")
    }

    fn guess_impact(name: &str) -> StartupImpact {
        let lower = name.to_lowercase();
        if lower.contains("docker")
            || lower.contains("spotify")
            || lower.contains("steam")
            || lower.contains("dropbox")
            || lower.contains("onedrive")
        {
            StartupImpact::High
        } else if lower.contains("update") || lower.contains("helper") {
            StartupImpact::Medium
        } else {
            StartupImpact::Low
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::process::Command;

    pub fn list_startup_items() -> Vec<StartupItem> {
        let mut items = Vec::new();
        scan_registry_run(&mut items, "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        scan_registry_run(&mut items, "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        scan_startup_folder(&mut items);
        items
    }

    fn scan_registry_run(items: &mut Vec<StartupItem>, key: &str) {
        let output = Command::new("reg")
            .args(["query", key])
            .output();
        if let Ok(out) = output {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines().skip(2) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let name = parts[0].to_string();
                        items.push(StartupItem {
                            id: format!("reg:{key}:{name}"),
                            name: name.clone(),
                            enabled: true,
                            impact: StartupImpact::Medium,
                            kind: StartupKind::RegistryRun,
                            path: None,
                            description: format!("注册表启动项: {key}"),
                        });
                    }
                }
            }
        }
    }

    fn scan_startup_folder(items: &mut Vec<StartupItem>) {
        if let Some(home) = directories::UserDirs::new() {
            let folder = home
                .home_dir()
                .join("AppData/Roaming/Microsoft/Windows/Start Menu/Programs/Startup");
            if folder.exists() {
                if let Ok(entries) = std::fs::read_dir(&folder) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        items.push(StartupItem {
                            id: format!("startupfolder:{}", entry.path().display()),
                            name: name.clone(),
                            enabled: true,
                            impact: StartupImpact::Medium,
                            kind: StartupKind::StartupFolder,
                            path: Some(entry.path()),
                            description: "启动文件夹".into(),
                        });
                    }
                }
            }
        }
    }

    pub fn set_startup_enabled(id: &str, enabled: bool) -> anyhow::Result<()> {
        if id.starts_with("reg:") {
            anyhow::bail!("注册表启动项请使用 reg delete 手动管理（v0.1 仅展示）");
        }
        if id.starts_with("startupfolder:") {
            let path = id.trim_start_matches("startupfolder:");
            if !enabled {
                let p = PathBuf::from(path);
                if p.exists() {
                    std::fs::remove_file(&p)?;
                }
            }
            return Ok(());
        }
        let _ = enabled;
        anyhow::bail!("unsupported startup item")
    }
}
