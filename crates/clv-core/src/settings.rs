use crate::models::{RiskLevel, TechStack};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub scan_paths: Vec<PathBuf>,
    pub expert_mode: bool,
    pub soft_delete: bool,
    pub soft_delete_days: u32,
    pub include_agent_heuristics: bool,
    pub auto_scan_weekly: bool,
    pub onboarding_done: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        let home = directories::UserDirs::new()
            .map(|d| d.home_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            scan_paths: vec![
                home.join("Projects"),
                home.join("projects"),
                home.join("Documents"),
                home.join("Desktop"),
                home.join("Developer"),
                home.join("dev"),
                home.join("code"),
                home.join("Code"),
            ],
            expert_mode: false,
            soft_delete: true,
            soft_delete_days: 7,
            include_agent_heuristics: true,
            auto_scan_weekly: false,
            onboarding_done: false,
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
        "c:\\windows",
        "c:\\program files",
        "c:\\program files (x86)",
        "c:\\programdata",
    ];
    blocked.iter().any(|b| s.starts_with(b))
}

/// Known cleanup targets relative to a project root or global cache.
#[derive(Debug, Clone)]
pub struct CleanupRule {
    pub relative: &'static str,
    pub stack: TechStack,
    pub risk: RiskLevel,
    pub category: &'static str,
    pub description: &'static str,
    pub global: bool,
}

pub fn project_rules() -> &'static [CleanupRule] {
    &[
        // Rust
        CleanupRule {
            relative: "target",
            stack: TechStack::Rust,
            risk: RiskLevel::Safe,
            category: "编译缓存",
            description: "Rust 编译产物与增量缓存，可重新 cargo build 生成",
            global: false,
        },
        // Node / Web
        CleanupRule {
            relative: "node_modules",
            stack: TechStack::NodeWeb,
            risk: RiskLevel::Safe,
            category: "依赖包",
            description: "Node.js 依赖目录，可通过 npm/pnpm/yarn install 恢复",
            global: false,
        },
        CleanupRule {
            relative: ".next",
            stack: TechStack::NodeWeb,
            risk: RiskLevel::Safe,
            category: "构建缓存",
            description: "Next.js 构建缓存",
            global: false,
        },
        CleanupRule {
            relative: ".nuxt",
            stack: TechStack::NodeWeb,
            risk: RiskLevel::Safe,
            category: "构建缓存",
            description: "Nuxt 构建缓存",
            global: false,
        },
        CleanupRule {
            relative: ".turbo",
            stack: TechStack::NodeWeb,
            risk: RiskLevel::Safe,
            category: "构建缓存",
            description: "Turborepo 缓存",
            global: false,
        },
        CleanupRule {
            relative: "dist",
            stack: TechStack::NodeWeb,
            risk: RiskLevel::Safe,
            category: "构建产物",
            description: "前端构建输出目录",
            global: false,
        },
        CleanupRule {
            relative: "build",
            stack: TechStack::NodeWeb,
            risk: RiskLevel::Caution,
            category: "构建产物",
            description: "通用构建输出（也可能是 Android/Flutter 产物，请确认）",
            global: false,
        },
        CleanupRule {
            relative: ".cache",
            stack: TechStack::NodeWeb,
            risk: RiskLevel::Safe,
            category: "工具缓存",
            description: "各类前端工具本地缓存",
            global: false,
        },
        CleanupRule {
            relative: ".parcel-cache",
            stack: TechStack::NodeWeb,
            risk: RiskLevel::Safe,
            category: "构建缓存",
            description: "Parcel 缓存",
            global: false,
        },
        // Python
        CleanupRule {
            relative: "__pycache__",
            stack: TechStack::Python,
            risk: RiskLevel::Safe,
            category: "字节码缓存",
            description: "Python 自动生成的缓存，可安全删除",
            global: false,
        },
        CleanupRule {
            relative: ".pytest_cache",
            stack: TechStack::Python,
            risk: RiskLevel::Safe,
            category: "测试缓存",
            description: "pytest 缓存",
            global: false,
        },
        CleanupRule {
            relative: ".mypy_cache",
            stack: TechStack::Python,
            risk: RiskLevel::Safe,
            category: "类型检查缓存",
            description: "mypy 缓存",
            global: false,
        },
        CleanupRule {
            relative: ".ruff_cache",
            stack: TechStack::Python,
            risk: RiskLevel::Safe,
            category: "Lint 缓存",
            description: "Ruff 缓存",
            global: false,
        },
        CleanupRule {
            relative: ".venv",
            stack: TechStack::Python,
            risk: RiskLevel::Caution,
            category: "虚拟环境",
            description: "Python 虚拟环境，删除后需重新创建",
            global: false,
        },
        CleanupRule {
            relative: "venv",
            stack: TechStack::Python,
            risk: RiskLevel::Caution,
            category: "虚拟环境",
            description: "Python 虚拟环境，删除后需重新创建",
            global: false,
        },
        // Java / Android / KMP
        CleanupRule {
            relative: ".gradle",
            stack: TechStack::Android,
            risk: RiskLevel::Caution,
            category: "Gradle 缓存",
            description: "项目级 Gradle 缓存",
            global: false,
        },
        CleanupRule {
            relative: "app/build",
            stack: TechStack::Android,
            risk: RiskLevel::Safe,
            category: "构建产物",
            description: "Android app 模块构建输出",
            global: false,
        },
        CleanupRule {
            relative: "target",
            stack: TechStack::Java,
            risk: RiskLevel::Safe,
            category: "构建产物",
            description: "Maven 构建输出",
            global: false,
        },
        // iOS
        CleanupRule {
            relative: "Pods",
            stack: TechStack::Ios,
            risk: RiskLevel::Caution,
            category: "依赖",
            description: "CocoaPods 依赖，可通过 pod install 恢复",
            global: false,
        },
        CleanupRule {
            relative: "DerivedData",
            stack: TechStack::Ios,
            risk: RiskLevel::Safe,
            category: "Xcode 缓存",
            description: "Xcode 本地构建缓存",
            global: false,
        },
        // Flutter
        CleanupRule {
            relative: ".dart_tool",
            stack: TechStack::Flutter,
            risk: RiskLevel::Safe,
            category: "工具缓存",
            description: "Dart/Flutter 工具缓存",
            global: false,
        },
        CleanupRule {
            relative: ".flutter-plugins-dependencies",
            stack: TechStack::Flutter,
            risk: RiskLevel::Safe,
            category: "插件缓存",
            description: "Flutter 插件依赖记录",
            global: false,
        },
        // .NET
        CleanupRule {
            relative: "bin",
            stack: TechStack::DotNet,
            risk: RiskLevel::Safe,
            category: "构建产物",
            description: ".NET 编译输出",
            global: false,
        },
        CleanupRule {
            relative: "obj",
            stack: TechStack::DotNet,
            risk: RiskLevel::Safe,
            category: "中间产物",
            description: ".NET 中间构建文件",
            global: false,
        },
        // C/C++
        CleanupRule {
            relative: "cmake-build-debug",
            stack: TechStack::Cpp,
            risk: RiskLevel::Safe,
            category: "构建目录",
            description: "CMake Debug 构建输出",
            global: false,
        },
        CleanupRule {
            relative: "cmake-build-release",
            stack: TechStack::Cpp,
            risk: RiskLevel::Safe,
            category: "构建目录",
            description: "CMake Release 构建输出",
            global: false,
        },
        CleanupRule {
            relative: "out",
            stack: TechStack::Cpp,
            risk: RiskLevel::Safe,
            category: "构建目录",
            description: "通用 C++ 构建输出",
            global: false,
        },
    ]
}

pub fn global_cache_rules() -> &'static [CleanupRule] {
    &[
        CleanupRule {
            relative: ".cargo/registry/cache",
            stack: TechStack::Rust,
            risk: RiskLevel::Caution,
            category: "全局缓存",
            description: "Cargo 下载缓存（可重建）",
            global: true,
        },
        CleanupRule {
            relative: ".npm/_cacache",
            stack: TechStack::NodeWeb,
            risk: RiskLevel::Safe,
            category: "全局缓存",
            description: "npm 全局缓存",
            global: true,
        },
        CleanupRule {
            relative: "Library/Caches/pnpm",
            stack: TechStack::NodeWeb,
            risk: RiskLevel::Safe,
            category: "全局缓存",
            description: "pnpm 全局缓存",
            global: true,
        },
        CleanupRule {
            relative: "Library/Caches/Yarn",
            stack: TechStack::NodeWeb,
            risk: RiskLevel::Safe,
            category: "全局缓存",
            description: "Yarn 全局缓存",
            global: true,
        },
        CleanupRule {
            relative: "Library/Developer/Xcode/DerivedData",
            stack: TechStack::Ios,
            risk: RiskLevel::Safe,
            category: "全局缓存",
            description: "Xcode DerivedData 全局缓存",
            global: true,
        },
        CleanupRule {
            relative: ".gradle/caches",
            stack: TechStack::Android,
            risk: RiskLevel::Caution,
            category: "全局缓存",
            description: "Gradle 全局缓存",
            global: true,
        },
        CleanupRule {
            relative: ".cache/uv",
            stack: TechStack::Python,
            risk: RiskLevel::Safe,
            category: "全局缓存",
            description: "uv 包管理器缓存",
            global: true,
        },
        CleanupRule {
            relative: "Library/Caches/pip",
            stack: TechStack::Python,
            risk: RiskLevel::Safe,
            category: "全局缓存",
            description: "pip 全局缓存",
            global: true,
        },
        CleanupRule {
            relative: ".nuget/packages",
            stack: TechStack::DotNet,
            risk: RiskLevel::Caution,
            category: "全局缓存",
            description: "NuGet 全局包缓存",
            global: true,
        },
        CleanupRule {
            relative: ".pub-cache",
            stack: TechStack::Flutter,
            risk: RiskLevel::Caution,
            category: "全局缓存",
            description: "Dart/Flutter pub 全局缓存",
            global: true,
        },
    ]
}

pub fn agent_name_patterns() -> &'static [&'static str] {
    &[
        "claude",
        "cursor",
        "codex",
        "workbuddy",
        "work-buddy",
        "agent",
        "generated",
        "aider",
        "copilot",
        "windsurf",
        "devin",
        "bolt",
        "v0",
        "replit",
    ]
}

pub fn agent_marker_files() -> &'static [&'static str] {
    &[
        ".cursor",
        ".claude",
        ".aider",
        ".copilot",
        ".windsurf",
        "AGENTS.md",
        "CLAUDE.md",
    ]
}

pub fn project_marker_files() -> &'static [(&'static str, TechStack)] {
    &[
        ("Cargo.toml", TechStack::Rust),
        ("package.json", TechStack::NodeWeb),
        ("pnpm-lock.yaml", TechStack::NodeWeb),
        ("yarn.lock", TechStack::NodeWeb),
        ("build.gradle", TechStack::Android),
        ("build.gradle.kts", TechStack::Android),
        ("settings.gradle.kts", TechStack::Kmp),
        ("Podfile", TechStack::Ios),
        ("pubspec.yaml", TechStack::Flutter),
        ("pom.xml", TechStack::Java),
        ("pyproject.toml", TechStack::Python),
        ("requirements.txt", TechStack::Python),
        ("*.csproj", TechStack::DotNet),
        ("CMakeLists.txt", TechStack::Cpp),
    ]
}
