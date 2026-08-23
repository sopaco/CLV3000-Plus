mod global_rules;
mod markers;
mod project_rules;
mod rule;

use crate::locale::{LanguagePreference, ThemePreference};
use crate::paths::{default_scan_paths, expand_scan_path};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub use crate::paths::resolve_global_path;
pub use global_rules::global_cache_rules;
pub use markers::{agent_marker_files, agent_name_patterns, project_marker_files};
pub use project_rules::project_rules;
pub use rule::CleanupRule;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub scan_paths: Vec<PathBuf>,
    pub expert_mode: bool,
    pub soft_delete: bool,
    pub soft_delete_days: u32,
    pub include_agent_heuristics: bool,
    pub auto_scan_weekly: bool,
    pub onboarding_done: bool,
    #[serde(default)]
    pub language: LanguagePreference,
    #[serde(default)]
    pub theme: ThemePreference,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            scan_paths: default_scan_paths(),
            expert_mode: false,
            soft_delete: true,
            soft_delete_days: 7,
            include_agent_heuristics: true,
            auto_scan_weekly: false,
            onboarding_done: false,
            language: LanguagePreference::default(),
            theme: ThemePreference::default(),
        }
    }
}

pub fn settings_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("com", "clv3000", "plus")
        .map(|d| d.config_dir().join("settings.json"))
}

pub fn load_settings() -> AppSettings {
    let Some(path) = settings_path() else {
        return AppSettings::default();
    };
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn format_scan_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn parse_scan_paths(text: &str) -> Vec<PathBuf> {
    text.lines()
        .map(expand_scan_path)
        .filter(|path| !path.as_os_str().is_empty())
        .collect()
}

pub fn save_settings(settings: &AppSettings) -> anyhow::Result<()> {
    let Some(path) = settings_path() else {
        anyhow::bail!("cannot resolve settings path");
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn trash_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("com", "clv3000", "plus")
        .map(|d| d.data_local_dir().join("trash"))
}

/// System paths that must never be scanned or deleted.
pub fn is_protected_system_path(path: &Path) -> bool {
    crate::paths::is_protected_system_path(path)
}
