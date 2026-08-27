use crate::category::CleanupCategory;
use crate::messages::{AgentReasonPart, RuleDescription};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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
    Go,
    Ruby,
    Php,
    Unity,
    Infra,
    Agent,
    System,
    Other,
}

impl TechStack {
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
            Self::Go,
            Self::Ruby,
            Self::Php,
            Self::Unity,
            Self::Infra,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CleanupBucket {
    /// 各项目目录内的编译/构建临时文件，删后重新打开项目通常会再生成。
    ProjectBuildCache,
    /// npm、Cargo 等工具在用户目录下的共用下载缓存，删后需重新下载。
    SharedToolCache,
    /// 虚拟环境、工具链、项目依赖等，删后需重新安装或配置。
    DevEnvironment,
    /// Cursor / Claude 等 AI 工具的会话、缓存与试验项目相关文件。
    AiGenerated,
}

pub fn item_cleanup_bucket(item: &ScanItem) -> CleanupBucket {
    if item.stack == TechStack::Agent {
        return CleanupBucket::AiGenerated;
    }

    let path = item.path.to_string_lossy().to_lowercase();
    for marker in [
        ".claude",
        ".agents",
        ".cursor",
        ".codex",
        ".codebuddy",
        ".aider",
        ".copilot",
        ".windsurf",
        ".trae",
        ".opencode",
    ] {
        if path.contains(marker) {
            return CleanupBucket::AiGenerated;
        }
    }

    item.category.cleanup_bucket()
}

/// Item ids that should be selected by default after a scan (safe-risk only).
pub fn default_selected_item_ids(items: &[ScanItem]) -> HashSet<String> {
    items
        .iter()
        .filter(|i| i.risk == RiskLevel::Safe)
        .map(|i| i.id.clone())
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanItem {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub size_bytes: u64,
    pub stack: TechStack,
    pub risk: RiskLevel,
    pub category: CleanupCategory,
    pub description: RuleDescription,
    pub project_root: Option<PathBuf>,
    pub last_modified: Option<DateTime<Utc>>,
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
    pub reason_parts: Vec<AgentReasonPart>,
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
    pub large_files: Vec<crate::large_files::LargeFileEntry>,
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

    pub fn safe_item_count(&self) -> usize {
        self.items.iter().filter(|i| i.risk == RiskLevel::Safe).count()
    }

    /// Per-bucket reclaimable bytes and item counts (total, safe-only).
    pub fn bucket_summaries(&self) -> Vec<(CleanupBucket, u64, usize, usize)> {
        use std::collections::HashMap;
        let mut map: HashMap<CleanupBucket, (u64, usize, usize)> = HashMap::new();
        for item in &self.items {
            let bucket = item_cleanup_bucket(item);
            let entry = map.entry(bucket).or_insert((0, 0, 0));
            entry.0 += item.size_bytes;
            entry.1 += 1;
            if item.risk == RiskLevel::Safe {
                entry.2 += 1;
            }
        }
        let mut rows: Vec<_> = map
            .into_iter()
            .map(|(bucket, (bytes, total, safe))| (bucket, bytes, total, safe))
            .collect();
        rows.sort_by(|a, b| b.1.cmp(&a.1));
        rows
    }
}

#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub phase: String,
    pub current_path: Option<PathBuf>,
    pub items_found: usize,
    pub bytes_found: u64,
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
