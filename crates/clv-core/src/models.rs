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
            Self::Go => "Go",
            Self::Ruby => "Ruby",
            Self::Php => "PHP",
            Self::Unity => "Unity",
            Self::Infra => "基础设施",
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

impl RiskLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Safe => "安全",
            Self::Caution => "建议确认",
            Self::Protected => "受保护",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl CleanupBucket {
    pub fn label(self) -> &'static str {
        match self {
            Self::ProjectBuildCache => "项目临时产物",
            Self::SharedToolCache => "构建下载缓存",
            Self::DevEnvironment => "环境与依赖",
            Self::AiGenerated => "AI 工具数据",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::ProjectBuildCache => "项目里的编译/测试临时文件，多数可安全清理",
            Self::SharedToolCache => "各工具共用的下载缓存，删后首次使用会重新下载",
            Self::DevEnvironment => "依赖包、虚拟环境、语言工具链，删后需重新安装",
            Self::AiGenerated => "AI 助手会话、缓存与相关试验项目",
        }
    }
}

pub fn item_cleanup_bucket(item: &ScanItem) -> CleanupBucket {
    if item.stack == TechStack::Agent {
        return CleanupBucket::AiGenerated;
    }

    if item.category == "Agent 会话" || item.category == "Agent 缓存" {
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
    ] {
        if path.contains(marker) {
            return CleanupBucket::AiGenerated;
        }
    }

    if item.category == "全局缓存" || item.category == "系统临时" {
        return CleanupBucket::SharedToolCache;
    }

    match item.category.as_str() {
        "工具链" | "虚拟环境" | "测试环境" | "依赖包" | "依赖" | "依赖缓存" | "Provider 缓存" => {
            CleanupBucket::DevEnvironment
        }
        "编译缓存" | "构建缓存" | "构建产物" | "字节码缓存" | "中间产物" | "Xcode 缓存"
        | "构建目录" | "测试缓存" | "工具缓存" | "测试产物" | "Lint 缓存" | "类型检查缓存"
        | "Gradle 缓存" | "插件缓存" | "编辑器缓存" | "临时文件" | "日志" => {
            CleanupBucket::ProjectBuildCache
        }
        _ => {
            if item.risk == RiskLevel::Protected || item.risk == RiskLevel::Caution {
                CleanupBucket::DevEnvironment
            } else {
                CleanupBucket::ProjectBuildCache
            }
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
