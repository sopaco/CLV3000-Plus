use crate::models::{RiskLevel, TechStack};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AgentSessionTarget {
    pub path: PathBuf,
    pub stack: TechStack,
    pub risk: RiskLevel,
    pub category: &'static str,
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
            "Agent 会话",
            "Codex 活跃会话记录（JSONL），删除后无法恢复对话历史",
        );
        push_dir(
            &mut targets,
            codex_home.join("archived_sessions"),
            RiskLevel::Caution,
            "Agent 会话",
            "Codex 归档会话记录，删除后无法恢复",
        );
    }

    for claude_root in claude_config_paths(&home) {
        push_dir(
            &mut targets,
            claude_root.join("projects"),
            RiskLevel::Caution,
            "Agent 会话",
            "Claude Code 项目会话目录（JSONL），删除后无法恢复对话历史",
        );
    }

    for cursor_root in cursor_data_roots(&home) {
        push_dir(
            &mut targets,
            cursor_root.join("CachedData"),
            RiskLevel::Safe,
            "Agent 缓存",
            "Cursor Electron 缓存，可安全清理",
        );
        push_dir(
            &mut targets,
            cursor_root.join("GPUCache"),
            RiskLevel::Safe,
            "Agent 缓存",
            "Cursor GPU 缓存，可安全清理",
        );
        push_dir(
            &mut targets,
            cursor_root.join("Code Cache"),
            RiskLevel::Safe,
            "Agent 缓存",
            "Cursor 代码缓存，可安全清理",
        );
    }

    push_dir(
        &mut targets,
        home.join(".cursor/projects"),
        RiskLevel::Caution,
        "Agent 会话",
        "Cursor 项目聊天历史，删除后无法恢复",
    );

    for workbuddy_root in workbuddy_data_roots(&home) {
        push_dir(
            &mut targets,
            workbuddy_root.join("sessions"),
            RiskLevel::Caution,
            "Agent 会话",
            "WorkBuddy / CodeBuddy 活跃会话数据",
        );
        push_dir(
            &mut targets,
            workbuddy_root.join("logs"),
            RiskLevel::Safe,
            "Agent 缓存",
            "WorkBuddy / CodeBuddy 运行日志",
        );
        push_dir(
            &mut targets,
            workbuddy_root.join("traces"),
            RiskLevel::Safe,
            "Agent 缓存",
            "WorkBuddy / CodeBuddy 执行追踪数据",
        );
        push_dir(
            &mut targets,
            workbuddy_root.join("file-history"),
            RiskLevel::Caution,
            "Agent 会话",
            "WorkBuddy / CodeBuddy 文件操作快照（/rewind 依赖）",
        );
        push_dir(
            &mut targets,
            workbuddy_root.join("shell-snapshots"),
            RiskLevel::Safe,
            "Agent 缓存",
            "WorkBuddy / CodeBuddy Bash 沙箱快照缓存",
        );
    }

    targets
}

fn push_dir(
    targets: &mut Vec<AgentSessionTarget>,
    path: PathBuf,
    risk: RiskLevel,
    category: &'static str,
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
        vec![home.join("AppData/Roaming/Cursor")]
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        vec![home.join(".config/Cursor")]
    }
}

fn workbuddy_data_roots(home: &Path) -> Vec<PathBuf> {
    let mut roots = parse_env_paths("CODEBUDDY_HOME");
    if roots.is_empty() {
        roots.push(home.join(".codebuddy"));
    }
    roots
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
}
