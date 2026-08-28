use crate::locale::{localized_text_matches_query, Language, tr};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentReasonPart {
    NameContainsPattern(String),
    HasAgentMarker(String),
    LongUnusedProject,
    InactiveOver30Days,
}

pub(crate) fn agent_reason_part_text(lang: Language, part: &AgentReasonPart) -> String {
    match part {
        AgentReasonPart::NameContainsPattern(pattern) => match lang {
            Language::Zh => format!("目录名包含「{pattern}」"),
            Language::En => format!("Directory name contains \"{pattern}\""),
            Language::Ja => format!("ディレクトリ名に「{pattern}」を含む"),
        },
        AgentReasonPart::HasAgentMarker(marker) => match lang {
            Language::Zh => format!("存在 Agent 标记 {marker}"),
            Language::En => format!("Contains agent marker {marker}"),
            Language::Ja => format!("Agent マーカー {marker} が存在"),
        },
        AgentReasonPart::LongUnusedProject => tr(
            lang,
            "长期未使用的项目",
            "Long-unused project",
            "長期未使用のプロジェクト",
        )
        .to_string(),
        AgentReasonPart::InactiveOver30Days => tr(
            lang,
            "超过 30 天未修改",
            "Not modified for over 30 days",
            "30 日以上更新なし",
        )
        .to_string(),
    }
}

pub fn format_agent_reason(lang: Language, parts: &[AgentReasonPart]) -> String {
    if parts.is_empty() {
        return String::new();
    }
    let sep = match lang {
        Language::Zh | Language::Ja => "；",
        Language::En => "; ",
    };
    parts
        .iter()
        .map(|part| agent_reason_part_text(lang, part))
        .collect::<Vec<_>>()
        .join(sep)
}

pub fn agent_reason_matches_query(parts: &[AgentReasonPart], query: &str) -> bool {
    localized_text_matches_query(query, |lang| format_agent_reason(lang, parts))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_combined_reason_in_english() {
        let parts = vec![
            AgentReasonPart::HasAgentMarker(".cursor".into()),
            AgentReasonPart::InactiveOver30Days,
        ];
        let text = format_agent_reason(Language::En, &parts);
        assert!(text.contains("agent marker"));
        assert!(text.contains("30 days"));
    }
}
