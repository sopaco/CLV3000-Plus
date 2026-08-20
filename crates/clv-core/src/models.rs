use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TechStack {
    Rust,
    NodeWeb,
    Android,
    Ios,
    Flutter,
    Kmp,
    Java,
    Python,
    DotNet,
    Cpp,
    Agent,
    System,
    Other,
}

impl TechStack {
    pub fn label(self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::NodeWeb => "Node.js / Web",
            Self::Android => "Android",
            Self::Ios => "iOS",
            Self::Flutter => "Flutter",
            Self::Kmp => "KMP",
            Self::Java => "Java",
            Self::Python => "Python",
            Self::DotNet => ".NET",
            Self::Cpp => "C/C++",
            Self::Agent => "Agent 项目",
            Self::System => "系统缓存",
            Self::Other => "其他",
        }
    }

    pub fn all() -> &'static [TechStack] {
        &[
            Self::Rust,
            Self::NodeWeb,
            Self::Android,
            Self::Ios,
            Self::Flutter,
            Self::Kmp,
            Self::Java,
            Self::Python,
            Self::DotNet,
            Self::Cpp,
            Self::Agent,
            Self::System,
            Self::Other,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe = 0,
    Caution = 1,
    Protected = 2,
}

impl RiskLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Safe => "安全",
            Self::Caution => "建议确认",
            Self::Protected => "受保护",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanItem {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub size_bytes: u64,
    pub stack: TechStack,
    pub risk: RiskLevel,
    pub category: String,
    pub description: String,
    pub project_root: Option<PathBuf>,
    pub last_modified: Option<DateTime<Utc>>,
    pub selected: bool,
}

impl ScanItem {
    pub fn size_human(&self) -> String {
        format_bytes(self.size_bytes)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProject {
    pub path: PathBuf,
    pub name: String,
    pub total_bytes: u64,
    pub stacks: Vec<TechStack>,
    pub last_modified: Option<DateTime<Utc>>,
    pub days_inactive: Option<i64>,
    pub reason: String,
    pub items: Vec<ScanItem>,
}

impl AgentProject {
    pub fn size_human(&self) -> String {
        format_bytes(self.total_bytes)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanReport {
    pub items: Vec<ScanItem>,
    pub agent_projects: Vec<AgentProject>,
    pub scanned_at: Option<DateTime<Utc>>,
    pub scan_duration_ms: u64,
    pub roots_scanned: Vec<PathBuf>,
}

impl ScanReport {
    pub fn total_reclaimable(&self) -> u64 {
        self.items
            .iter()
            .filter(|i| i.risk != RiskLevel::Protected)
            .map(|i| i.size_bytes)
            .sum()
    }

    pub fn total_reclaimable_human(&self) -> String {
        format_bytes(self.total_reclaimable())
    }

    pub fn safe_reclaimable(&self) -> u64 {
        self.items
            .iter()
            .filter(|i| i.risk == RiskLevel::Safe)
            .map(|i| i.size_bytes)
            .sum()
    }

    pub fn by_stack(&self, stack: TechStack) -> Vec<&ScanItem> {
        self.items.iter().filter(|i| i.stack == stack).collect()
    }

    pub fn stack_total(&self, stack: TechStack) -> u64 {
        self.by_stack(stack).iter().map(|i| i.size_bytes).sum()
    }
}

#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub phase: String,
    pub current_path: Option<PathBuf>,
    pub items_found: usize,
    pub bytes_found: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserMode {
    Simple,
    Expert,
}

impl UserMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Simple => "简单模式",
            Self::Expert => "专家模式",
        }
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.1} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}
