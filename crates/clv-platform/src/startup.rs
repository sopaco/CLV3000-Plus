use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartupImpact {
    Low,
    Medium,
    High,
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

/// 按名称/命令行启发式估计启动项的开机影响度
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
    use std::path::Path;

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
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();

                let (plist_path, enabled) = if file_name.ends_with(".plist.disabled") {
                    let stem = file_name.trim_end_matches(".disabled");
                    let original = path.parent().unwrap().join(stem);
                    (original, false)
                } else if path.extension().and_then(|e| e.to_str()) == Some("plist") {
                    (path.clone(), true)
                } else {
                    continue;
                };

                let name = plist_path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unknown".into());
                let display = plist_path.display().to_string();
                items.push(StartupItem {
                    id: format!("launchagent:{}", plist_path.display()),
                    name: name.clone(),
                    enabled,
                    impact: guess_impact(&name),
                    kind: if user {
                        StartupKind::LaunchAgent
                    } else {
                        StartupKind::LaunchDaemon
                    },
                    path: Some(plist_path),
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
            return set_launchagent_enabled(&path, enabled);
        }

        if let Some(name) = id.strip_prefix("loginitem:") {
            return set_login_item_enabled(name, enabled);
        }

        anyhow::bail!("unknown startup item id")
    }

    fn disabled_agent_path(path: &Path) -> PathBuf {
        PathBuf::from(format!("{}.disabled", path.display()))
    }

    fn gui_domain() -> String {
        format!("gui/{}", unsafe { libc::getuid() })
    }

    fn plist_label(path: &Path) -> anyhow::Result<String> {
        let bytes = fs::read(path)?;
        let value: plist::Value = plist::from_bytes(&bytes)?;
        match value {
            plist::Value::Dictionary(dict) => dict
                .get("Label")
                .and_then(|v| v.as_string())
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow::anyhow!("plist 缺少 Label 字段：{}", path.display())),
            _ => anyhow::bail!("无效的 plist：{}", path.display()),
        }
    }

    fn launchctl(args: &[&str]) -> anyhow::Result<std::process::Output> {
        std::process::Command::new("/bin/launchctl")
            .args(args)
            .output()
            .map_err(|e| anyhow::anyhow!("无法执行 launchctl：{e}"))
    }

    fn launchctl_stderr_ok(output: &std::process::Output) -> bool {
        if output.status.success() {
            return true;
        }
        let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
        stderr.contains("already loaded")
            || stderr.contains("already bootstrapped")
            || stderr.contains("no such process")
            || stderr.contains("could not find service")
            || stderr.contains("not found")
    }

    fn set_launchagent_enabled(path: &Path, enabled: bool) -> anyhow::Result<()> {
        let disabled = disabled_agent_path(path);
        let domain = gui_domain();

        if enabled {
            if disabled.exists() {
                fs::rename(&disabled, path)?;
            }
            if !path.exists() {
                anyhow::bail!("找不到 LaunchAgent 配置文件：{}", path.display());
            }

            let label = plist_label(path)?;
            let service = format!("{domain}/{label}");
            let enable = launchctl(&["enable", &service])?;
            if !enable.status.success() {
                let stderr = String::from_utf8_lossy(&enable.stderr);
                anyhow::bail!("无法启用 LaunchAgent：{stderr}");
            }
            // 仅恢复“登录时启动”配置，不 bootstrap —— bootstrap 会立刻加载并运行该 Agent。
            return Ok(());
        }

        if path.exists() {
            if let Ok(label) = plist_label(path) {
                let service = format!("{domain}/{label}");
                let _ = launchctl(&["disable", &service]);
            }
            let bootout = launchctl(&["bootout", &domain, &path.to_string_lossy()]);
            if let Ok(out) = bootout {
                if !launchctl_stderr_ok(&out) {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    tracing::warn!("launchctl bootout: {stderr}");
                }
            }
            fs::rename(path, &disabled)?;
            return Ok(());
        }

        if disabled.exists() {
            return Ok(());
        }

        anyhow::bail!("找不到 LaunchAgent 配置文件：{}", path.display())
    }

    fn set_login_item_enabled(name: &str, enabled: bool) -> anyhow::Result<()> {
        if enabled {
            anyhow::bail!("登录项请在系统设置 → 通用 → 登录项中重新添加");
        }

        let escaped = name.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!(
            "tell application \"System Events\" to delete login item \"{escaped}\""
        );
        let output = std::process::Command::new("/usr/bin/osascript")
            .arg("-e")
            .arg(&script)
            .output()?;
        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "无法禁用登录项「{name}」：{stderr}。请在系统设置 → 隐私与安全性 → 自动化 中授权本应用控制「System Events」，或在系统设置 → 通用 → 登录项中手动管理"
        )
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};
    use winreg::enums::*;
    use winreg::types::FromRegValue;
    use winreg::{RegKey, RegValue, HKEY};

    /// 任务管理器禁用/启用启动项所使用的“启动批准”注册表分支。
    /// 写入 HKCU 即对当前用户生效，无需管理员权限。
    const APPROVED_BASE: &str =
        "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved";

    struct RegRunSource {
        /// id 中的来源标记，用于写回时定位批准子键
        token: &'static str,
        hive: HKEY,
        path: &'static str,
        /// 对应的 StartupApproved 子键名
        approved: &'static str,
        label: &'static str,
    }

    const RUN_SOURCES: &[RegRunSource] = &[
        RegRunSource {
            token: "hkcu-run",
            hive: HKEY_CURRENT_USER,
            path: "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            approved: "Run",
            label: "HKCU Run（当前用户）",
        },
        RegRunSource {
            token: "hkcu-runonce",
            hive: HKEY_CURRENT_USER,
            path: "Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
            approved: "Run",
            label: "HKCU RunOnce（当前用户，一次性）",
        },
        RegRunSource {
            token: "hklm-run",
            hive: HKEY_LOCAL_MACHINE,
            path: "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            approved: "Run",
            label: "HKLM Run（所有用户）",
        },
        RegRunSource {
            token: "hklm-runonce",
            hive: HKEY_LOCAL_MACHINE,
            path: "Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
            approved: "Run",
            label: "HKLM RunOnce（所有用户，一次性）",
        },
        RegRunSource {
            token: "hklm-run32",
            hive: HKEY_LOCAL_MACHINE,
            path: "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Run",
            approved: "Run32",
            label: "HKLM Run 32 位（所有用户）",
        },
    ];

    pub fn list_startup_items() -> Vec<StartupItem> {
        let mut items = Vec::new();
        for source in RUN_SOURCES {
            scan_registry_run(&mut items, source);
        }
        scan_startup_folder(&mut items, user_startup_folder(), "启动文件夹（当前用户）");
        if let Some(common) = common_startup_folder() {
            scan_startup_folder(&mut items, common, "启动文件夹（所有用户）");
        }
        items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        items
    }

    fn scan_registry_run(items: &mut Vec<StartupItem>, source: &RegRunSource) {
        let Ok(key) = RegKey::predef(source.hive).open_subkey_with_flags(source.path, KEY_READ)
        else {
            return;
        };
        for entry in key.enum_values().flatten() {
            let (name, value) = entry;
            let command = reg_value_to_string(&value);
            items.push(StartupItem {
                id: format!("reg:{}:{name}", source.token),
                name: name.clone(),
                enabled: !approval_disabled(source.approved, &name),
                impact: guess_impact(&format!("{name} {command}")),
                kind: StartupKind::RegistryRun,
                path: None,
                description: format!("{}：{command}", source.label),
            });
        }
    }

    fn scan_startup_folder(items: &mut Vec<StartupItem>, folder: PathBuf, label: &str) {
        let Ok(entries) = std::fs::read_dir(&folder) else {
            return;
        };
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let lower = file_name.to_lowercase();
            if lower == "desktop.ini" || lower == "thumbs.db" {
                continue;
            }
            let path = entry.path();
            let display_path = path.display().to_string();
            items.push(StartupItem {
                id: format!("startupfolder:{display_path}"),
                name: file_name.clone(),
                // 任务管理器以“文件名（含扩展名）”记录启动文件夹项的批准状态
                enabled: !approval_disabled("StartupFolder", &file_name),
                impact: guess_impact(&file_name),
                kind: StartupKind::StartupFolder,
                path: Some(path),
                description: format!("{label}：{display_path}"),
            });
        }
    }

    fn user_startup_folder() -> PathBuf {
        directories::UserDirs::new()
            .map(|u| {
                u.home_dir()
                    .join("AppData/Roaming/Microsoft/Windows/Start Menu/Programs/Startup")
            })
            .unwrap_or_else(|| PathBuf::from("."))
    }

    fn common_startup_folder() -> Option<PathBuf> {
        std::env::var_os("ProgramData").map(|dir| {
            PathBuf::from(dir).join("Microsoft/Windows/Start Menu/Programs/Startup")
        })
    }

    pub fn set_startup_enabled(id: &str, enabled: bool) -> anyhow::Result<()> {
        if let Some(rest) = id.strip_prefix("reg:") {
            let (token, name) = rest
                .split_once(':')
                .ok_or_else(|| anyhow::anyhow!("无效的注册表启动项标识"))?;
            let Some(source) = RUN_SOURCES.iter().find(|s| s.token == token) else {
                anyhow::bail!("未知的注册表启动项来源 {token}");
            };
            return set_approval(source.approved, name, enabled);
        }

        if let Some(path) = id.strip_prefix("startupfolder:") {
            let file_name = Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .ok_or_else(|| anyhow::anyhow!("无效的启动文件夹路径"))?;
            return set_approval("StartupFolder", &file_name, enabled);
        }

        anyhow::bail!("不支持的启动项标识")
    }

    /// 读取启动批准状态：首字节低位置 1（0x01/0x03）为禁用，0x02/0x06 为启用。
    /// 优先取当前用户（HKCU）的覆盖值，其次回退机器级（HKLM）默认状态。
    fn approval_disabled(subkey: &str, name: &str) -> bool {
        read_approval_flag(HKEY_CURRENT_USER, subkey, name)
            .or_else(|| read_approval_flag(HKEY_LOCAL_MACHINE, subkey, name))
            .unwrap_or(false)
    }

    fn read_approval_flag(hive: HKEY, subkey: &str, name: &str) -> Option<bool> {
        let path = format!("{APPROVED_BASE}\\{subkey}");
        let key = RegKey::predef(hive)
            .open_subkey_with_flags(&path, KEY_READ)
            .ok()?;
        let value = key.get_raw_value(name).ok()?;
        Some(!value.bytes.is_empty() && (value.bytes[0] & 1) == 1)
    }

    /// 写入启动批准状态（写入 HKCU，当前用户生效）。
    /// 与任务管理器行为一致：禁用 = 0x03 + FILETIME，启用 = 0x02 + FILETIME。
    fn set_approval(subkey: &str, name: &str, enabled: bool) -> anyhow::Result<()> {
        let path = format!("{APPROVED_BASE}\\{subkey}");
        let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey_with_flags(&path, KEY_SET_VALUE)
            .map_err(|e| anyhow::anyhow!("无法写入启动批准键 {path}：{e}"))?;
        let mut bytes = vec![if enabled { 0x02 } else { 0x03 }, 0, 0, 0];
        bytes.extend_from_slice(&filetime_now_bytes());
        key.set_raw_value(name, &RegValue { bytes, vtype: RegType::REG_BINARY })
            .map_err(|e| anyhow::anyhow!("无法更新启动项状态：{e}"))?;
        Ok(())
    }

    fn reg_value_to_string(value: &RegValue) -> String {
        if matches!(value.vtype, RegType::REG_SZ | RegType::REG_EXPAND_SZ) {
            String::from_reg_value(value).unwrap_or_default()
        } else {
            format!("（{:?} 值数据）", value.vtype)
        }
    }

    /// 当前时间对应的 FILETIME（1601-01-01 起的 100ns 计数），小端字节序
    fn filetime_now_bytes() -> [u8; 8] {
        const UNIX_TO_FILETIME_SECS: u64 = 11_644_473_600;
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default();
        secs.saturating_add(UNIX_TO_FILETIME_SECS)
            .saturating_mul(10_000_000)
            .to_le_bytes()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn list_startup_items_smoke() {
            let items = list_startup_items();
            let mut ids: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
            let total = ids.len();
            ids.sort_unstable();
            ids.dedup();
            assert_eq!(ids.len(), total, "启动项 id 不应重复");
        }

        #[test]
        fn approval_roundtrip() {
            const NAME: &str = "clv3000-plus-selftest";
            set_approval("Run", NAME, false).expect("写入禁用状态");
            assert!(approval_disabled("Run", NAME), "禁用状态应可读回");
            set_approval("Run", NAME, true).expect("写入启用状态");
            assert!(!approval_disabled("Run", NAME), "启用状态应可读回");
            // 清理测试残留
            if let Ok(key) = RegKey::predef(HKEY_CURRENT_USER)
                .open_subkey_with_flags(format!("{APPROVED_BASE}\\Run"), KEY_SET_VALUE)
            {
                let _ = key.delete_value(NAME);
            }
        }
    }
}
