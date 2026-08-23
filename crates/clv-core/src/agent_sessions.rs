use crate::category::CleanupCategory;
use crate::models::{RiskLevel, TechStack};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AgentSessionTarget {
    pub path: PathBuf,
    pub stack: TechStack,
    pub risk: RiskLevel,
    pub category: CleanupCategory,
    pub description: &'static str,
}

/// Known coding-agent session / cache locations (inspired by ccusage path resolution).
pub fn discover_agent_session_targets() -> Vec<AgentSessionTarget> {
    let Some(home) = directories::UserDirs::new().map(|d| d.home_dir().to_path_buf()) else {
        return Vec::new();
    };

    let mut targets = Vec::new();

    for codex_home in codex_home_paths(&home) {
        push_dir(
            &mut targets,
            codex_home.join("sessions"),
            RiskLevel::Caution,
            CleanupCategory::AgentSession,
            "Codex 活跃会话记录（JSONL），删除后无法恢复对话历史",
        );
        push_dir(
            &mut targets,
            codex_home.join("archived_sessions"),
            RiskLevel::Caution,
            CleanupCategory::AgentSession,
            "Codex 归档会话记录，删除后无法恢复",
        );
    }

    for claude_root in claude_config_paths(&home) {
        push_dir(
            &mut targets,
            claude_root.join("projects"),
            RiskLevel::Caution,
            CleanupCategory::AgentSession,
            "Claude Code 项目会话目录（JSONL），删除后无法恢复对话历史",
        );
    }

    for cursor_root in cursor_data_roots(&home) {
        push_dir(
            &mut targets,
            cursor_root.join("CachedData"),
            RiskLevel::Safe,
            CleanupCategory::AgentCache,
            "Cursor Electron 缓存，可安全清理",
        );
        push_dir(
            &mut targets,
            cursor_root.join("GPUCache"),
            RiskLevel::Safe,
            CleanupCategory::AgentCache,
            "Cursor GPU 缓存，可安全清理",
        );
        push_dir(
            &mut targets,
            cursor_root.join("Code Cache"),
            RiskLevel::Safe,
            CleanupCategory::AgentCache,
            "Cursor 代码缓存，可安全清理",
        );
        push_dir(
            &mut targets,
            cursor_root.join("CachedExtensionVSIXs"),
            RiskLevel::Safe,
            CleanupCategory::AgentCache,
            "Cursor 扩展 VSIX 下载缓存",
        );
        push_dir(
            &mut targets,
            cursor_root.join("logs"),
            RiskLevel::Safe,
            CleanupCategory::AgentCache,
            "Cursor 运行日志",
        );
    }

    for windsurf_root in windsurf_data_roots(&home) {
        push_dir(
            &mut targets,
            windsurf_root.join("CachedData"),
            RiskLevel::Safe,
            CleanupCategory::AgentCache,
            "Windsurf Electron 缓存，可安全清理",
        );
        push_dir(
            &mut targets,
            windsurf_root.join("GPUCache"),
            RiskLevel::Safe,
            CleanupCategory::AgentCache,
            "Windsurf GPU 缓存，可安全清理",
        );
        push_dir(
            &mut targets,
            windsurf_root.join("Code Cache"),
            RiskLevel::Safe,
            CleanupCategory::AgentCache,
            "Windsurf 代码缓存，可安全清理",
        );
        push_dir(
            &mut targets,
            windsurf_root.join("logs"),
            RiskLevel::Safe,
            CleanupCategory::AgentCache,
            "Windsurf 运行日志",
        );
    }

    push_dir(
        &mut targets,
        home.join(".cursor/projects"),
        RiskLevel::Caution,
        CleanupCategory::AgentSession,
        "Cursor 项目聊天历史，删除后无法恢复",
    );

    for workbuddy_root in workbuddy_data_roots(&home) {
        push_dir(
            &mut targets,
            workbuddy_root.join("sessions"),
            RiskLevel::Caution,
            CleanupCategory::AgentSession,
            "WorkBuddy / CodeBuddy 活跃会话数据",
        );
        push_dir(
            &mut targets,
            workbuddy_root.join("logs"),
            RiskLevel::Safe,
            CleanupCategory::AgentCache,
            "WorkBuddy / CodeBuddy 运行日志",
        );
        push_dir(
            &mut targets,
            workbuddy_root.join("traces"),
            RiskLevel::Safe,
            CleanupCategory::AgentCache,
            "WorkBuddy / CodeBuddy 执行追踪数据",
        );
        push_dir(
            &mut targets,
            workbuddy_root.join("file-history"),
            RiskLevel::Caution,
            CleanupCategory::AgentSession,
            "WorkBuddy / CodeBuddy 文件操作快照（/rewind 依赖）",
        );
        push_dir(
            &mut targets,
            workbuddy_root.join("shell-snapshots"),
            RiskLevel::Safe,
            CleanupCategory::AgentCache,
            "WorkBuddy / CodeBuddy Bash 沙箱快照缓存",
        );
    }

    for trae_root in trae_app_data_roots(&home) {
        push_dir(
            &mut targets,
            trae_root.join("CachedData"),
            RiskLevel::Safe,
            CleanupCategory::AgentCache,
            "Trae Electron 缓存，可安全清理",
        );
        push_dir(
            &mut targets,
            trae_root.join("GPUCache"),
            RiskLevel::Safe,
            CleanupCategory::AgentCache,
            "Trae GPU 缓存，可安全清理",
        );
        push_dir(
            &mut targets,
            trae_root.join("Code Cache"),
            RiskLevel::Safe,
            CleanupCategory::AgentCache,
            "Trae 代码缓存，可安全清理",
        );
        push_dir(
            &mut targets,
            trae_root.join("Cache"),
            RiskLevel::Safe,
            CleanupCategory::AgentCache,
            "Trae Electron 缓存，可安全清理",
        );
        push_dir(
            &mut targets,
            trae_root.join("logs"),
            RiskLevel::Safe,
            CleanupCategory::AgentCache,
            "Trae 运行日志",
        );
        push_dir(
            &mut targets,
            trae_root.join("ModularData/ai-agent/snapshot"),
            RiskLevel::Caution,
            CleanupCategory::AgentSession,
            "Trae AI 对话快照，删除后无法回滚到对话前状态",
        );
        push_dir(
            &mut targets,
            trae_root.join("User/workspaceStorage"),
            RiskLevel::Caution,
            CleanupCategory::AgentSession,
            "Trae 项目聊天历史，删除后无法恢复",
        );
        push_dir(
            &mut targets,
            trae_root.join("User/globalStorage"),
            RiskLevel::Caution,
            CleanupCategory::AgentSession,
            "Trae 全局聊天历史，删除后无法恢复",
        );
    }

    for traex_root in traex_session_roots(&home) {
        push_dir(
            &mut targets,
            traex_root.join("sessions"),
            RiskLevel::Caution,
            CleanupCategory::AgentSession,
            "TraeX CLI 活跃会话记录（JSONL），删除后无法恢复对话历史",
        );
        push_dir(
            &mut targets,
            traex_root.join("archived_sessions"),
            RiskLevel::Caution,
            CleanupCategory::AgentSession,
            "TraeX CLI 归档会话记录，删除后无法恢复",
        );
    }

    for opencode_root in opencode_data_roots(&home) {
        push_dir(
            &mut targets,
            opencode_root.join("log"),
            RiskLevel::Safe,
            CleanupCategory::AgentCache,
            "OpenCode 运行日志，可安全清理",
        );
        push_dir(
            &mut targets,
            opencode_root.join("project"),
            RiskLevel::Caution,
            CleanupCategory::AgentSession,
            "OpenCode 项目会话与消息数据，删除后无法恢复对话历史",
        );
        push_dir(
            &mut targets,
            opencode_root.join("storage"),
            RiskLevel::Caution,
            CleanupCategory::AgentSession,
            "OpenCode 会话存储（JSON/SQLite），删除后无法恢复对话历史",
        );
    }

    targets
}

fn push_dir(
    targets: &mut Vec<AgentSessionTarget>,
    path: PathBuf,
    risk: RiskLevel,
    category: CleanupCategory,
    description: &'static str,
) {
    if !path.exists() {
        return;
    }
    targets.push(AgentSessionTarget {
        path,
        stack: TechStack::Agent,
        risk,
        category,
        description,
    });
}

fn parse_env_paths(var: &str) -> Vec<PathBuf> {
    env::var(var)
        .ok()
        .map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
                .collect()
        })
        .unwrap_or_default()
}

fn codex_home_paths(home: &Path) -> Vec<PathBuf> {
    let mut paths = parse_env_paths("CODEX_HOME");
    if paths.is_empty() {
        paths.push(home.join(".codex"));
    }
    paths
}

fn claude_config_paths(home: &Path) -> Vec<PathBuf> {
    let mut paths = parse_env_paths("CLAUDE_CONFIG_DIR");
    if paths.is_empty() {
        paths.push(home.join(".claude"));
        #[cfg(not(target_os = "windows"))]
        {
            paths.push(home.join(".config/claude"));
        }
        #[cfg(target_os = "windows")]
        {
            paths.push(home.join("AppData/Roaming/Claude"));
        }
    }
    paths
}

fn cursor_data_roots(home: &Path) -> Vec<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        vec![home.join("Library/Application Support/Cursor")]
    }
    #[cfg(target_os = "windows")]
    {
        vec![
            home.join("AppData/Roaming/Cursor"),
            home.join("AppData/Local/Programs/cursor"),
        ]
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        vec![home.join(".config/Cursor")]
    }
}

fn windsurf_data_roots(home: &Path) -> Vec<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        vec![home.join("Library/Application Support/Windsurf")]
    }
    #[cfg(target_os = "windows")]
    {
        vec![home.join("AppData/Roaming/Windsurf")]
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        vec![home.join(".config/Windsurf")]
    }
}

fn workbuddy_data_roots(home: &Path) -> Vec<PathBuf> {
    let mut roots = parse_env_paths("CODEBUDDY_HOME");
    if roots.is_empty() {
        roots.push(home.join(".codebuddy"));
    }
    roots
}

const TRAE_APP_NAMES: &[&str] = &["Trae", "Trae CN", "TRAE SOLO CN"];

fn trae_app_data_roots(home: &Path) -> Vec<PathBuf> {
    let roots = parse_env_paths("TRAE_DIR");
    if !roots.is_empty() {
        return roots
            .into_iter()
            .map(|user_dir| {
                user_dir
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or(user_dir)
            })
            .collect();
    }

    TRAE_APP_NAMES
        .iter()
        .map(|name| trae_app_data_root(home, name))
        .collect()
}

fn trae_app_data_root(home: &Path, name: &str) -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        home.join("Library/Application Support").join(name)
    }
    #[cfg(target_os = "windows")]
    {
        home.join("AppData/Roaming").join(name)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        home.join(".config").join(name)
    }
}

fn traex_session_roots(home: &Path) -> Vec<PathBuf> {
    let mut roots = parse_env_paths("TRAEX_SESSIONS_DIR");
    if roots.is_empty() {
        roots.push(home.join(".trae/cli"));
    } else {
        roots = roots
            .into_iter()
            .map(|path| {
                if path.ends_with("sessions") || path.ends_with("archived_sessions") {
                    path.parent()
                        .map(Path::to_path_buf)
                        .unwrap_or(path)
                } else {
                    path
                }
            })
            .collect();
    }
    roots
}

fn opencode_data_roots(home: &Path) -> Vec<PathBuf> {
    let mut paths = parse_env_paths("OPENCODE_DIR");
    if paths.is_empty() {
        paths.push(home.join(".local/share/opencode"));
        #[cfg(not(target_os = "windows"))]
        if let Ok(xdg) = env::var("XDG_DATA_HOME") {
            paths.push(PathBuf::from(xdg).join("opencode"));
        }
        #[cfg(target_os = "windows")]
        {
            if let Ok(local) = env::var("LOCALAPPDATA") {
                paths.push(PathBuf::from(local).join("opencode"));
            }
            if let Ok(appdata) = env::var("APPDATA") {
                paths.push(PathBuf::from(appdata).join("opencode"));
            }
        }
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_home_respects_env() {
        let temp = tempfile::tempdir().unwrap();
        let custom = temp.path().join("custom-codex");
        std::fs::create_dir_all(&custom).unwrap();
        unsafe {
            env::set_var("CODEX_HOME", &custom);
        }
        let paths = codex_home_paths(temp.path());
        assert_eq!(paths, vec![custom]);
        unsafe {
            env::remove_var("CODEX_HOME");
        }
    }

    #[test]
    fn claude_defaults_include_home_dot_claude() {
        let temp = tempfile::tempdir().unwrap();
        let paths = claude_config_paths(temp.path());
        assert!(paths.iter().any(|p| p.ends_with(".claude")));
    }

    #[test]
    fn trae_dir_respects_env() {
        let temp = tempfile::tempdir().unwrap();
        let custom_user = temp.path().join("custom-trae/User");
        std::fs::create_dir_all(&custom_user).unwrap();
        unsafe {
            env::set_var("TRAE_DIR", &custom_user);
        }
        let paths = trae_app_data_roots(temp.path());
        assert_eq!(paths, vec![custom_user.parent().unwrap().to_path_buf()]);
        unsafe {
            env::remove_var("TRAE_DIR");
        }
    }

    #[test]
    fn opencode_dir_respects_env() {
        let temp = tempfile::tempdir().unwrap();
        let custom = temp.path().join("custom-opencode");
        std::fs::create_dir_all(&custom).unwrap();
        unsafe {
            env::set_var("OPENCODE_DIR", &custom);
        }
        let paths = opencode_data_roots(temp.path());
        assert_eq!(paths, vec![custom]);
        unsafe {
            env::remove_var("OPENCODE_DIR");
        }
    }
}
