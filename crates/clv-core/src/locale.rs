use serde::{Deserialize, Serialize};
use std::path::Path;

/// Application color theme / visual skin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
    /// Track the OS light/dark setting; resolves to a concrete skin via
    /// [`resolve_theme`] before reaching the UI layer.
    #[default]
    System,
    /// Default security-software dark blue aesthetic.
    Defender,
    /// Soft rose & lavender — warm, feminine palette.
    Blossom,
    /// Electric cyan & purple — vibrant youth style.
    Neon,
    /// Fresh mint & sky — light, energetic feel.
    Aurora,
    /// Neutral grays with the Windows system blue — light office style.
    Fluent,
}

/// Resolve a [`ThemePreference`] into a concrete skin.
///
/// `System` maps to the dark skin when the OS appearance is dark and to the
/// light skin otherwise; explicit preferences pass through untouched. Pure
/// domain logic — the caller supplies the OS appearance so the core layer
/// stays free of UI dependencies.
pub fn resolve_theme(preference: ThemePreference, system_dark: bool) -> ThemePreference {
    match preference {
        ThemePreference::System => {
            if system_dark {
                ThemePreference::Defender
            } else {
                ThemePreference::Fluent
            }
        }
        other => other,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LanguagePreference {
    #[default]
    System,
    Zh,
    En,
    Ja,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Zh,
    En,
    Ja,
}

pub fn resolve_language(preference: LanguagePreference) -> Language {
    match preference {
        LanguagePreference::System => detect_system_language(),
        LanguagePreference::Zh => Language::Zh,
        LanguagePreference::En => Language::En,
        LanguagePreference::Ja => Language::Ja,
    }
}

pub(crate) fn detect_system_language() -> Language {
    let locale = sys_locale::get_locale()
        .unwrap_or_else(|| "en-US".to_string())
        .to_lowercase();
    if locale.starts_with("zh") {
        Language::Zh
    } else if locale.starts_with("ja") {
        Language::Ja
    } else {
        Language::En
    }
}

pub fn scan_phase_preparing(lang: Language) -> String {
    tr(lang, "准备扫描…", "Preparing scan…", "スキャン準備中…").to_string()
}

pub(crate) fn scan_phase_scanning_path(lang: Language, path: &Path) -> String {
    match lang {
        Language::Zh => format!("扫描 {}", path.display()),
        Language::En => format!("Scanning {}", path.display()),
        Language::Ja => format!("{} をスキャン中", path.display()),
    }
}

pub(crate) fn scan_phase_scanning_projects(lang: Language) -> String {
    tr(
        lang,
        "扫描项目目录",
        "Scanning project directories",
        "プロジェクトをスキャン中",
    )
    .to_string()
}

pub(crate) fn scan_phase_agent_sessions(lang: Language) -> String {
    tr(
        lang,
        "发现 Agent 会话",
        "Discovering agent sessions",
        "Agent セッションを検出中",
    )
    .to_string()
}

pub(crate) fn scan_phase_discovering(lang: Language) -> String {
    tr(
        lang,
        "发现可清理项",
        "Discovering cleanable items",
        "削除可能な項目を検出中",
    )
    .to_string()
}

pub fn tr<'a>(lang: Language, zh: &'a str, en: &'a str, ja: &'a str) -> &'a str {
    match lang {
        Language::Zh => zh,
        Language::En => en,
        Language::Ja => ja,
    }
}

pub const ALL_LANGUAGES: [Language; 3] = [Language::Zh, Language::En, Language::Ja];

pub fn localized_text_matches_query<S: AsRef<str>>(
    query: &str,
    text: impl Fn(Language) -> S,
) -> bool {
    ALL_LANGUAGES
        .iter()
        .any(|&lang| text(lang).as_ref().to_lowercase().contains(query))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_explicit_preferences() {
        assert_eq!(resolve_language(LanguagePreference::Zh), Language::Zh);
        assert_eq!(resolve_language(LanguagePreference::En), Language::En);
        assert_eq!(resolve_language(LanguagePreference::Ja), Language::Ja);
    }

    #[test]
    fn resolve_theme_maps_system_to_concrete_skins() {
        assert_eq!(
            resolve_theme(ThemePreference::System, true),
            ThemePreference::Defender,
            "系统深色应映射到 Defender（暗色皮肤）"
        );
        assert_eq!(
            resolve_theme(ThemePreference::System, false),
            ThemePreference::Fluent,
            "系统浅色应映射到 Fluent（亮色皮肤）"
        );
    }

    #[test]
    fn resolve_theme_passes_explicit_preferences_through() {
        for pref in [
            ThemePreference::Defender,
            ThemePreference::Blossom,
            ThemePreference::Neon,
            ThemePreference::Aurora,
            ThemePreference::Fluent,
        ] {
            assert_eq!(resolve_theme(pref, true), pref);
            assert_eq!(resolve_theme(pref, false), pref);
        }
    }

    #[test]
    fn system_theme_is_the_default() {
        assert_eq!(ThemePreference::default(), ThemePreference::System);
    }

    /// 老版本配置文件里存的是具体皮肤名；新增 `System` 变体后这些值必须照常
    /// 反序列化，否则升级用户的主题选择会被静默重置。
    #[test]
    fn theme_preference_deserializes_legacy_values() {
        assert_eq!(
            serde_json::from_str::<ThemePreference>("\"defender\"").unwrap(),
            ThemePreference::Defender
        );
        assert_eq!(
            serde_json::from_str::<ThemePreference>("\"blossom\"").unwrap(),
            ThemePreference::Blossom
        );
        assert_eq!(
            serde_json::from_str::<ThemePreference>("\"neon\"").unwrap(),
            ThemePreference::Neon
        );
        assert_eq!(
            serde_json::from_str::<ThemePreference>("\"aurora\"").unwrap(),
            ThemePreference::Aurora
        );
        assert_eq!(
            serde_json::from_str::<ThemePreference>("\"system\"").unwrap(),
            ThemePreference::System
        );
    }

    /// `theme` 字段带 `#[serde(default)]`：全新配置或旧版缺省字段落到默认值。
    #[test]
    fn settings_deserialize_missing_theme_field_uses_default() {
        let legacy = serde_json::json!({
            "scan_paths": [],
            "expert_mode": false,
            "soft_delete": true,
            "soft_delete_days": 7,
            "include_agent_heuristics": true,
            "auto_scan_weekly": false,
            "onboarding_done": true
        });
        let settings: crate::AppSettings = serde_json::from_value(legacy).unwrap();
        assert_eq!(settings.theme, ThemePreference::System);
        assert_eq!(settings.language, LanguagePreference::System);
    }
}
