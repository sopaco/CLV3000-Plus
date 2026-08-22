use serde::{Deserialize, Serialize};
use std::path::Path;

/// Application color theme / visual skin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreference {
    /// Default security-software dark blue aesthetic.
    #[default]
    Defender,
    /// Soft rose & lavender — warm, feminine palette.
    Blossom,
    /// Electric cyan & purple — vibrant youth style.
    Neon,
    /// Fresh mint & sky — light, energetic feel.
    Aurora,
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

pub fn detect_system_language() -> Language {
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
    tr(lang, "准备扫描", "Preparing scan", "スキャン準備中").to_string()
}

pub fn scan_phase_scanning_path(lang: Language, path: &Path) -> String {
    match lang {
        Language::Zh => format!("扫描 {}", path.display()),
        Language::En => format!("Scanning {}", path.display()),
        Language::Ja => format!("{} をスキャン中", path.display()),
    }
}

pub fn scan_phase_scanning_projects(lang: Language) -> String {
    tr(lang, "扫描项目目录", "Scanning project directories", "プロジェクトをスキャン中").to_string()
}

pub fn scan_phase_agent_sessions(lang: Language) -> String {
    tr(
        lang,
        "发现 Agent 会话",
        "Discovering agent sessions",
        "Agent セッションを検出中",
    )
    .to_string()
}

pub fn scan_phase_discovering(lang: Language) -> String {
    tr(lang, "发现可清理项", "Discovering cleanable items", "削除可能な項目を検出中")
        .to_string()
}

pub fn tr<'a>(lang: Language, zh: &'a str, en: &'a str, ja: &'a str) -> &'a str {
    match lang {
        Language::Zh => zh,
        Language::En => en,
        Language::Ja => ja,
    }
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
}
