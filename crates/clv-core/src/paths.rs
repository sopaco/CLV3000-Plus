use std::env;
use std::path::{Path, PathBuf};

pub fn user_home_dir() -> Option<PathBuf> {
    directories::UserDirs::new().map(|d| d.home_dir().to_path_buf())
}

pub fn default_scan_paths() -> Vec<PathBuf> {
    let home = user_home_dir().unwrap_or_else(|| PathBuf::from("."));

    #[cfg(target_os = "windows")]
    {
        vec![
            home.join("Projects"),
            home.join("projects"),
            home.join("source").join("repos"),
            home.join("Documents").join("GitHub"),
            home.join("Documents").join("Source"),
            home.join("Documents"),
            home.join("Desktop"),
            home.join("dev"),
            home.join("Dev"),
            home.join("code"),
            home.join("Code"),
            home.join("workspace"),
            home.join("Workspace"),
            home.join("repos"),
            home.join("Repos"),
            home.join("work"),
            home.join("Work"),
            home.join("Downloads"),
        ]
    }

    #[cfg(not(target_os = "windows"))]
    {
        vec![
            home.join("Projects"),
            home.join("projects"),
            home.join("Documents"),
            home.join("Desktop"),
            home.join("Developer"),
            home.join("dev"),
            home.join("code"),
            home.join("Code"),
        ]
    }
}

/// Resolve a global cleanup rule path. Supports `$LOCALAPPDATA/`, `$APPDATA/`,
/// `$TEMP/`, `$USERPROFILE/` prefixes; otherwise treated as relative to home.
pub fn resolve_global_path(relative: &str) -> Option<PathBuf> {
    if let Some(rest) = strip_env_prefix(relative, "$LOCALAPPDATA") {
        let base = env::var("LOCALAPPDATA").ok().map(PathBuf::from)?;
        return Some(base.join(rest));
    }
    if let Some(rest) = strip_env_prefix(relative, "$APPDATA") {
        let base = env::var("APPDATA").ok().map(PathBuf::from)?;
        return Some(base.join(rest));
    }
    if let Some(rest) = strip_env_prefix(relative, "$TEMP") {
        let base = env_temp_dir()?;
        return Some(base.join(rest));
    }
    if let Some(rest) = strip_env_prefix(relative, "$USERPROFILE") {
        let base = user_home_dir()?;
        return Some(base.join(rest));
    }

    let home = user_home_dir()?;
    Some(home.join(relative))
}

fn strip_env_prefix<'a>(relative: &'a str, prefix: &str) -> Option<&'a str> {
    if relative == prefix {
        return Some("");
    }
    let expected = format!("{prefix}/");
    relative
        .strip_prefix(&expected)
        .or_else(|| relative.strip_prefix(prefix))
        .map(|rest| rest.trim_start_matches(['/', '\\']))
}

pub fn env_temp_dir() -> Option<PathBuf> {
    env::var("TEMP")
        .or_else(|_| env::var("TMP"))
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            user_home_dir().map(|home| home.join("AppData").join("Local").join("Temp"))
        })
}

pub fn expand_scan_path(line: &str) -> PathBuf {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return PathBuf::new();
    }

    #[cfg(target_os = "windows")]
    {
        if trimmed.contains('%') {
            if let Some(expanded) = expand_windows_percent_vars(trimmed) {
                return PathBuf::from(expanded);
            }
        }
    }

    if let Some(rest) = trimmed.strip_prefix("~/") {
        return user_home_dir()
            .map(|home| home.join(rest))
            .unwrap_or_else(|| PathBuf::from(trimmed));
    }
    if trimmed == "~" {
        return user_home_dir().unwrap_or_else(|| PathBuf::from("."));
    }

    PathBuf::from(trimmed)
}

#[cfg(target_os = "windows")]
fn expand_windows_percent_vars(input: &str) -> Option<String> {
    let mut result = input.to_string();
    let mut guard = 0;
    while result.contains('%') {
        guard += 1;
        if guard > 32 {
            return None;
        }
        let start = result.find('%')?;
        let rest = &result[start + 1..];
        let end = rest.find('%')?;
        let var = &rest[..end];
        if var.is_empty() {
            return None;
        }
        let value = env::var(var).ok()?;
        let token = format!("%{var}%");
        result = result.replacen(&token, &value, 1);
    }
    Some(result)
}

/// System paths that must never be scanned or deleted.
pub fn is_protected_system_path(path: &Path) -> bool {
    #[cfg(target_os = "windows")]
    {
        return is_protected_windows_path(path);
    }

    let s = path.to_string_lossy().to_lowercase();
    // macOS temp lives under /var/folders — do not treat as system
    if s.contains("/var/folders/") {
        return false;
    }
    let blocked = [
        "/system",
        "/library",
        "/usr",
        "/bin",
        "/sbin",
        "/etc",
        "/private/var",
        "/applications",
    ];
    blocked.iter().any(|b| s.starts_with(b))
}

#[cfg(target_os = "windows")]
fn is_protected_windows_path(path: &Path) -> bool {
    if is_windows_user_temp(path) {
        return false;
    }

    let lowered = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
        .collect::<Vec<_>>();

    if lowered
        .iter()
        .any(|part| part == "$recycle.bin" || part == "system volume information")
    {
        return true;
    }

    // Block OS install roots on any drive: X:\Windows, X:\Program Files, X:\ProgramData
    if lowered.len() >= 2 {
        if let Some(root) = lowered.first() {
            if root.len() == 2 && root.ends_with(':') {
                let second = lowered[1].as_str();
                if matches!(
                    second,
                    "windows" | "program files" | "program files (x86)" | "programdata"
                ) {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(target_os = "windows")]
fn is_windows_user_temp(path: &Path) -> bool {
    let Some(temp) = env_temp_dir() else {
        return false;
    };
    let Some(local) = env::var("LOCALAPPDATA")
        .ok()
        .map(PathBuf::from)
        .map(|p| p.join("Temp"))
    else {
        return path.starts_with(&temp);
    };

    path.starts_with(&temp) || path.starts_with(&local)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scan_paths_include_common_windows_dev_dirs() {
        let paths = default_scan_paths();
        let joined = paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        #[cfg(target_os = "windows")]
        {
            assert!(joined.iter().any(|p| p.ends_with("source\\repos") || p.ends_with("source/repos")));
            assert!(joined.iter().any(|p| p.contains("GitHub")));
        }

        #[cfg(not(target_os = "windows"))]
        {
            assert!(joined.iter().any(|p| p.ends_with("Developer") || p.ends_with("developer")));
        }
    }

    #[test]
    fn resolve_global_path_supports_env_prefix() {
        let resolved = resolve_global_path("$LOCALAPPDATA/npm-cache/_cacache");
        if let Ok(local) = env::var("LOCALAPPDATA") {
            assert_eq!(
                resolved,
                Some(PathBuf::from(local).join("npm-cache/_cacache"))
            );
        }
    }

    #[test]
    fn windows_temp_is_not_protected() {
        if let Some(temp) = env_temp_dir() {
            assert!(!is_protected_system_path(&temp));
        }
    }
}
