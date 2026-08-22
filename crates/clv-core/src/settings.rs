use crate::locale::LanguagePreference;
use crate::models::{RiskLevel, TechStack};
use crate::paths::{default_scan_paths, expand_scan_path};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

pub use crate::paths::resolve_global_path;

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

/// Known cleanup targets relative to a project root or global cache.
#[derive(Debug, Clone)]
pub struct CleanupRule {
    pub relative: &'static str,
    pub stack: TechStack,
    pub risk: RiskLevel,
    pub category: &'static str,
    pub description: &'static str,
    pub global: bool,
    /// Require a marker file under the detected project root (supports `*.ext` globs).
    pub requires_marker: Option<&'static str>,
    /// Name prefix (`cmake-build-`) or suffix pattern (`*.egg-info` → ends with `.egg-info`).
    pub relative_prefix: Option<&'static str>,
    /// Parent directory name must match (e.g. `vendor` for Ruby `vendor/bundle`).
    pub requires_parent: Option<&'static str>,
}

impl CleanupRule {
    const fn project(
        relative: &'static str,
        stack: TechStack,
        risk: RiskLevel,
        category: &'static str,
        description: &'static str,
    ) -> Self {
        Self {
            relative,
            stack,
            risk,
            category,
            description,
            global: false,
            requires_marker: None,
            relative_prefix: None,
            requires_parent: None,
        }
    }

    const fn global(
        relative: &'static str,
        stack: TechStack,
        risk: RiskLevel,
        category: &'static str,
        description: &'static str,
    ) -> Self {
        Self {
            relative,
            stack,
            risk,
            category,
            description,
            global: true,
            requires_marker: None,
            relative_prefix: None,
            requires_parent: None,
        }
    }

    const fn marker(self, marker: &'static str) -> Self {
        Self {
            requires_marker: Some(marker),
            ..self
        }
    }

    const fn prefix(self, prefix: &'static str) -> Self {
        Self {
            relative_prefix: Some(prefix),
            ..self
        }
    }

    const fn parent(self, name: &'static str) -> Self {
        Self {
            requires_parent: Some(name),
            ..self
        }
    }
}

pub fn project_rules() -> &'static [CleanupRule] {
    static RULES: LazyLock<Vec<CleanupRule>> = LazyLock::new(|| {
        vec![
        // Rust
        CleanupRule::project(
            "target",
            TechStack::Rust,
            RiskLevel::Safe,
            "编译缓存",
            "Rust 编译产物与增量缓存，可重新 cargo build 生成",
        )
        .marker("Cargo.toml"),
        // Node / Web
        CleanupRule::project(
            "node_modules",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "依赖包",
            "Node.js 依赖目录，可通过 npm/pnpm/yarn install 恢复",
        )
        .marker("package.json"),
        CleanupRule::project(
            ".next",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "构建缓存",
            "Next.js 构建缓存",
        ),
        CleanupRule::project(
            ".nuxt",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "构建缓存",
            "Nuxt 构建缓存",
        ),
        CleanupRule::project(
            ".turbo",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "构建缓存",
            "Turborepo 缓存",
        ),
        CleanupRule::project(
            ".vite",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "构建缓存",
            "Vite 预构建与依赖缓存",
        ),
        CleanupRule::project(
            ".svelte-kit",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "构建缓存",
            "SvelteKit 构建缓存",
        ),
        CleanupRule::project(
            ".astro",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "构建缓存",
            "Astro 构建缓存",
        ),
        CleanupRule::project(
            ".angular",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "构建缓存",
            "Angular CLI 缓存",
        ),
        CleanupRule::project(
            ".output",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "构建产物",
            "Nuxt 3 / Nitro 输出目录",
        ),
        CleanupRule::project(
            ".vercel",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "构建缓存",
            "Vercel 本地构建缓存",
        ),
        CleanupRule::project(
            ".netlify",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "构建缓存",
            "Netlify 本地构建缓存",
        ),
        CleanupRule::project(
            ".expo",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "构建缓存",
            "Expo / React Native 缓存",
        ),
        CleanupRule::project(
            ".webpack",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "构建缓存",
            "Webpack 缓存目录",
        ),
        CleanupRule::project(
            "dist",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "构建产物",
            "前端构建输出目录",
        )
        .marker("package.json"),
        CleanupRule::project(
            "build",
            TechStack::NodeWeb,
            RiskLevel::Caution,
            "构建产物",
            "前端/通用构建输出（请确认非重要产物）",
        )
        .marker("package.json"),
        CleanupRule::project(
            "build",
            TechStack::Android,
            RiskLevel::Safe,
            "构建产物",
            "Gradle 项目构建输出",
        )
        .marker("build.gradle"),
        CleanupRule::project(
            "build",
            TechStack::Android,
            RiskLevel::Safe,
            "构建产物",
            "Gradle Kotlin DSL 项目构建输出",
        )
        .marker("build.gradle.kts"),
        CleanupRule::project(
            ".cache",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "工具缓存",
            "各类前端工具本地缓存",
        ),
        CleanupRule::project(
            ".parcel-cache",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "构建缓存",
            "Parcel 缓存",
        ),
        CleanupRule::project(
            "cache",
            TechStack::NodeWeb,
            RiskLevel::Caution,
            "依赖缓存",
            "Yarn Berry 本地包缓存",
        )
        .marker("package.json")
        .parent(".yarn"),
        CleanupRule::project(
            "coverage",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "测试产物",
            "测试覆盖率报告",
        ),
        CleanupRule::project(
            "htmlcov",
            TechStack::Python,
            RiskLevel::Safe,
            "测试产物",
            "Python coverage HTML 报告",
        ),
        CleanupRule::project(
            ".nyc_output",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "测试产物",
            "NYC 覆盖率原始数据",
        ),
        CleanupRule::project(
            "storybook-static",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "构建产物",
            "Storybook 静态导出",
        ),
        // Python
        CleanupRule::project(
            "__pycache__",
            TechStack::Python,
            RiskLevel::Safe,
            "字节码缓存",
            "Python 自动生成的缓存，可安全删除",
        ),
        CleanupRule::project(
            ".pytest_cache",
            TechStack::Python,
            RiskLevel::Safe,
            "测试缓存",
            "pytest 缓存",
        ),
        CleanupRule::project(
            ".mypy_cache",
            TechStack::Python,
            RiskLevel::Safe,
            "类型检查缓存",
            "mypy 缓存",
        ),
        CleanupRule::project(
            ".ruff_cache",
            TechStack::Python,
            RiskLevel::Safe,
            "Lint 缓存",
            "Ruff 缓存",
        ),
        CleanupRule::project(
            ".tox",
            TechStack::Python,
            RiskLevel::Caution,
            "测试环境",
            "tox 多环境虚拟环境，删除后需重新创建",
        ),
        CleanupRule::project(
            ".nox",
            TechStack::Python,
            RiskLevel::Caution,
            "测试环境",
            "nox 多环境虚拟环境，删除后需重新创建",
        ),
        CleanupRule::project(
            ".eggs",
            TechStack::Python,
            RiskLevel::Safe,
            "构建产物",
            "setuptools eggs 目录",
        ),
        CleanupRule::project(
            ".hypothesis",
            TechStack::Python,
            RiskLevel::Safe,
            "测试缓存",
            "Hypothesis 属性测试缓存",
        ),
        CleanupRule::project(
            "",
            TechStack::Python,
            RiskLevel::Safe,
            "构建产物",
            "setuptools egg-info 元数据目录",
        )
        .prefix("*.egg-info"),
        CleanupRule::project(
            ".venv",
            TechStack::Python,
            RiskLevel::Caution,
            "虚拟环境",
            "Python 虚拟环境，删除后需重新创建",
        ),
        CleanupRule::project(
            "venv",
            TechStack::Python,
            RiskLevel::Caution,
            "虚拟环境",
            "Python 虚拟环境，删除后需重新创建",
        ),
        // Go / Ruby / PHP
        CleanupRule::project(
            "vendor",
            TechStack::Go,
            RiskLevel::Caution,
            "依赖",
            "Go vendor 目录，可通过 go mod vendor 恢复",
        )
        .marker("go.mod"),
        CleanupRule::project(
            "vendor",
            TechStack::Php,
            RiskLevel::Caution,
            "依赖",
            "Composer vendor 目录，可通过 composer install 恢复",
        )
        .marker("composer.json"),
        CleanupRule::project(
            "bundle",
            TechStack::Ruby,
            RiskLevel::Caution,
            "依赖",
            "Bundler vendor/bundle，可通过 bundle install 恢复",
        )
        .marker("Gemfile")
        .parent("vendor"),
        // Java / Android / KMP
        CleanupRule::project(
            ".gradle",
            TechStack::Android,
            RiskLevel::Caution,
            "Gradle 缓存",
            "项目级 Gradle 缓存",
        ),
        CleanupRule::project(
            "app/build",
            TechStack::Android,
            RiskLevel::Safe,
            "构建产物",
            "Android app 模块构建输出",
        ),
        CleanupRule::project(
            "target",
            TechStack::Java,
            RiskLevel::Safe,
            "构建产物",
            "Maven 构建输出",
        )
        .marker("pom.xml"),
        // iOS
        CleanupRule::project(
            "Pods",
            TechStack::Ios,
            RiskLevel::Caution,
            "依赖",
            "CocoaPods 依赖，可通过 pod install 恢复",
        ),
        CleanupRule::project(
            "DerivedData",
            TechStack::Ios,
            RiskLevel::Safe,
            "Xcode 缓存",
            "Xcode 本地构建缓存",
        ),
        // Flutter
        CleanupRule::project(
            ".dart_tool",
            TechStack::Flutter,
            RiskLevel::Safe,
            "工具缓存",
            "Dart/Flutter 工具缓存",
        ),
        CleanupRule::project(
            ".flutter-plugins-dependencies",
            TechStack::Flutter,
            RiskLevel::Safe,
            "插件缓存",
            "Flutter 插件依赖记录",
        ),
        // .NET
        CleanupRule::project(
            "bin",
            TechStack::DotNet,
            RiskLevel::Safe,
            "构建产物",
            ".NET 编译输出",
        )
        .marker("*.csproj"),
        CleanupRule::project(
            "obj",
            TechStack::DotNet,
            RiskLevel::Safe,
            "中间产物",
            ".NET 中间构建文件",
        )
        .marker("*.csproj"),
        // C/C++
        CleanupRule::project(
            "",
            TechStack::Cpp,
            RiskLevel::Safe,
            "构建目录",
            "CMake 构建输出",
        )
        .prefix("cmake-build-"),
        CleanupRule::project(
            "out",
            TechStack::Cpp,
            RiskLevel::Safe,
            "构建目录",
            "通用 C++ 构建输出",
        )
        .marker("CMakeLists.txt"),
        CleanupRule::project(
            "zig-cache",
            TechStack::Cpp,
            RiskLevel::Safe,
            "构建缓存",
            "Zig 编译缓存",
        )
        .marker("build.zig"),
        CleanupRule::project(
            "zig-out",
            TechStack::Cpp,
            RiskLevel::Safe,
            "构建产物",
            "Zig 构建输出",
        )
        .marker("build.zig"),
        // Unity
        CleanupRule::project(
            "Library",
            TechStack::Unity,
            RiskLevel::Safe,
            "构建缓存",
            "Unity 本地 Library 缓存，可重新导入",
        )
        .marker("ProjectSettings/ProjectVersion.txt"),
        CleanupRule::project(
            "Temp",
            TechStack::Unity,
            RiskLevel::Safe,
            "临时文件",
            "Unity 临时构建文件",
        )
        .marker("ProjectSettings/ProjectVersion.txt"),
        CleanupRule::project(
            "Logs",
            TechStack::Unity,
            RiskLevel::Safe,
            "日志",
            "Unity 编辑器日志",
        )
        .marker("ProjectSettings/ProjectVersion.txt"),
        // Godot
        CleanupRule::project(
            ".godot",
            TechStack::Other,
            RiskLevel::Safe,
            "编辑器缓存",
            "Godot 编辑器缓存",
        )
        .marker("project.godot"),
        // Terraform
        CleanupRule::project(
            ".terraform",
            TechStack::Infra,
            RiskLevel::Caution,
            "Provider 缓存",
            "Terraform provider 与模块缓存，terraform init 可恢复",
        ),
        // Elixir
        CleanupRule::project(
            "_build",
            TechStack::Other,
            RiskLevel::Safe,
            "构建产物",
            "Elixir / Mix 构建输出",
        )
        .marker("mix.exs"),
        CleanupRule::project(
            ".cache",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            "工具缓存",
            "node_modules 内 webpack/babel 等工具缓存",
        )
        .marker("package.json")
        .parent("node_modules"),
        CleanupRule::project(
            "deps",
            TechStack::Other,
            RiskLevel::Caution,
            "依赖",
            "Elixir 依赖目录，mix deps.get 可恢复",
        )
        .marker("mix.exs"),
        ]
    });
    RULES.as_slice()
}

pub fn global_cache_rules() -> &'static [CleanupRule] {
    #[cfg(target_os = "windows")]
    {
        static RULES: LazyLock<Vec<CleanupRule>> = LazyLock::new(|| {
            vec![
            // --- Rust / Python toolchains ---
            CleanupRule::global(
                ".cargo/registry/cache",
                TechStack::Rust,
                RiskLevel::Caution,
                "全局缓存",
                "Cargo 下载缓存（可重建）",
            ),
            CleanupRule::global(
                ".cargo/git",
                TechStack::Rust,
                RiskLevel::Caution,
                "全局缓存",
                "Cargo git 依赖 checkout（可重建）",
            ),
            CleanupRule::global(
                ".cargo/registry/index",
                TechStack::Rust,
                RiskLevel::Caution,
                "全局缓存",
                "Cargo registry 索引缓存（可重建）",
            ),
            CleanupRule::global(
                ".rustup/toolchains",
                TechStack::Rust,
                RiskLevel::Protected,
                "工具链",
                "Rust 工具链，删除后需 rustup 重新安装",
            ),
            CleanupRule::global(
                ".pyenv/versions",
                TechStack::Python,
                RiskLevel::Protected,
                "工具链",
                "pyenv 已安装 Python 版本，删除后需 pyenv install 恢复",
            ),
            // --- Node / Web ---
            CleanupRule::global(
                "$LOCALAPPDATA/npm-cache/_cacache",
                TechStack::NodeWeb,
                RiskLevel::Safe,
                "全局缓存",
                "npm 全局缓存",
            ),
            CleanupRule::global(
                ".npm/_cacache",
                TechStack::NodeWeb,
                RiskLevel::Safe,
                "全局缓存",
                "npm 旧版全局缓存（用户目录）",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/pnpm/store",
                TechStack::NodeWeb,
                RiskLevel::Caution,
                "全局缓存",
                "pnpm 内容寻址存储（删除后项目需重新安装依赖）",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/Yarn/Cache",
                TechStack::NodeWeb,
                RiskLevel::Safe,
                "全局缓存",
                "Yarn 全局缓存",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/bun/install/cache",
                TechStack::NodeWeb,
                RiskLevel::Safe,
                "全局缓存",
                "Bun 包管理器全局缓存",
            ),
            CleanupRule::global(
                ".bun/install/cache",
                TechStack::NodeWeb,
                RiskLevel::Safe,
                "全局缓存",
                "Bun 用户目录安装缓存",
            ),
            CleanupRule::global(
                ".cache",
                TechStack::NodeWeb,
                RiskLevel::Caution,
                "全局缓存",
                "用户目录通用工具缓存（部分 CLI/前端工具）",
            ),
            // --- Java / Android / .NET / Flutter ---
            CleanupRule::global(
                ".gradle/caches",
                TechStack::Android,
                RiskLevel::Caution,
                "全局缓存",
                "Gradle 全局缓存",
            ),
            CleanupRule::global(
                ".gradle/wrapper/dists",
                TechStack::Android,
                RiskLevel::Caution,
                "全局缓存",
                "Gradle Wrapper 发行版下载缓存",
            ),
            CleanupRule::global(
                ".m2/repository",
                TechStack::Java,
                RiskLevel::Caution,
                "全局缓存",
                "Maven 本地仓库（删除后需重新下载依赖）",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/uv/cache",
                TechStack::Python,
                RiskLevel::Safe,
                "全局缓存",
                "uv 包管理器缓存",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/pip/Cache",
                TechStack::Python,
                RiskLevel::Safe,
                "全局缓存",
                "pip 全局缓存",
            ),
            CleanupRule::global(
                ".nuget/packages",
                TechStack::DotNet,
                RiskLevel::Caution,
                "全局缓存",
                "NuGet 全局包缓存",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/Pub/Cache",
                TechStack::Flutter,
                RiskLevel::Caution,
                "全局缓存",
                "Dart/Flutter pub 全局缓存",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/Microsoft/MSBuild",
                TechStack::DotNet,
                RiskLevel::Safe,
                "全局缓存",
                "MSBuild 编译缓存",
            ),
            // --- Go / Docker ---
            CleanupRule::global(
                "$LOCALAPPDATA/go-build",
                TechStack::Go,
                RiskLevel::Safe,
                "全局缓存",
                "Go 编译缓存（GOCACHE 默认位置）",
            ),
            CleanupRule::global(
                "go/pkg/mod",
                TechStack::Go,
                RiskLevel::Caution,
                "全局缓存",
                "Go module 下载缓存（删除后需重新 go mod download）",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/Docker",
                TechStack::Other,
                RiskLevel::Caution,
                "全局缓存",
                "Docker Desktop 本地数据与缓存",
            ),
            // --- IDE / Editor ---
            CleanupRule::global(
                "$APPDATA/Code/Cache",
                TechStack::Other,
                RiskLevel::Safe,
                "全局缓存",
                "VS Code Electron 缓存",
            ),
            CleanupRule::global(
                "$APPDATA/Code/CachedData",
                TechStack::Other,
                RiskLevel::Safe,
                "全局缓存",
                "VS Code 版本缓存",
            ),
            CleanupRule::global(
                "$APPDATA/Code/CachedExtensionVSIXs",
                TechStack::Other,
                RiskLevel::Safe,
                "全局缓存",
                "VS Code 扩展 VSIX 下载缓存",
            ),
            CleanupRule::global(
                "$APPDATA/Cursor/Cache",
                TechStack::Agent,
                RiskLevel::Safe,
                "Agent 缓存",
                "Cursor Electron 缓存",
            ),
            CleanupRule::global(
                "$APPDATA/Cursor/CachedData",
                TechStack::Agent,
                RiskLevel::Safe,
                "Agent 缓存",
                "Cursor 版本缓存",
            ),
            CleanupRule::global(
                "$APPDATA/Cursor/logs",
                TechStack::Agent,
                RiskLevel::Safe,
                "Agent 缓存",
                "Cursor 运行日志",
            ),
            CleanupRule::global(
                "$APPDATA/Windsurf/Cache",
                TechStack::Agent,
                RiskLevel::Safe,
                "Agent 缓存",
                "Windsurf Electron 缓存",
            ),
            CleanupRule::global(
                "$APPDATA/Windsurf/CachedData",
                TechStack::Agent,
                RiskLevel::Safe,
                "Agent 缓存",
                "Windsurf 版本缓存",
            ),
            // --- Windows system temp / junk ---
            CleanupRule::global(
                "$TEMP",
                TechStack::System,
                RiskLevel::Safe,
                "系统临时",
                "Windows TEMP/TMP 环境变量指向的临时目录",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/Temp",
                TechStack::System,
                RiskLevel::Safe,
                "系统临时",
                "Windows 用户临时文件目录（安装包/解压残留等）",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/CrashDumps",
                TechStack::System,
                RiskLevel::Safe,
                "系统临时",
                "应用程序崩溃转储文件",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/D3DSCache",
                TechStack::System,
                RiskLevel::Safe,
                "系统临时",
                "DirectX 着色器缓存",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/Microsoft/Windows/INetCache",
                TechStack::System,
                RiskLevel::Safe,
                "系统临时",
                "Windows 浏览器/WebView 缓存",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/Microsoft/Windows/DeliveryOptimization/Cache",
                TechStack::System,
                RiskLevel::Safe,
                "系统临时",
                "Windows 传递优化下载缓存",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/Microsoft/Windows/Explorer",
                TechStack::System,
                RiskLevel::Caution,
                "系统临时",
                "Windows 资源管理器缩略图缓存",
            ),
            ]
        });
        RULES.as_slice()
    }
    #[cfg(not(target_os = "windows"))]
    {
        static RULES: LazyLock<Vec<CleanupRule>> = LazyLock::new(|| {
            vec![
            CleanupRule::global(
                ".cargo/registry/cache",
                TechStack::Rust,
                RiskLevel::Caution,
                "全局缓存",
                "Cargo 下载缓存（可重建）",
            ),
            CleanupRule::global(
                ".cargo/git",
                TechStack::Rust,
                RiskLevel::Caution,
                "全局缓存",
                "Cargo git 依赖 checkout（可重建）",
            ),
            CleanupRule::global(
                ".rustup/toolchains",
                TechStack::Rust,
                RiskLevel::Protected,
                "工具链",
                "Rust 工具链，删除后需 rustup 重新安装",
            ),
            CleanupRule::global(
                ".pyenv/versions",
                TechStack::Python,
                RiskLevel::Protected,
                "工具链",
                "pyenv 已安装 Python 版本，删除后需 pyenv install 恢复",
            ),
            CleanupRule::global(
                ".npm/_cacache",
                TechStack::NodeWeb,
                RiskLevel::Safe,
                "全局缓存",
                "npm 全局缓存",
            ),
            CleanupRule::global(
                "Library/Caches/pnpm",
                TechStack::NodeWeb,
                RiskLevel::Safe,
                "全局缓存",
                "pnpm 全局缓存",
            ),
            CleanupRule::global(
                "Library/Caches/Yarn",
                TechStack::NodeWeb,
                RiskLevel::Safe,
                "全局缓存",
                "Yarn 全局缓存",
            ),
            CleanupRule::global(
                "Library/Developer/Xcode/DerivedData",
                TechStack::Ios,
                RiskLevel::Safe,
                "全局缓存",
                "Xcode DerivedData 全局缓存",
            ),
            CleanupRule::global(
                ".gradle/caches",
                TechStack::Android,
                RiskLevel::Caution,
                "全局缓存",
                "Gradle 全局缓存",
            ),
            CleanupRule::global(
                "Library/Caches/uv",
                TechStack::Python,
                RiskLevel::Safe,
                "全局缓存",
                "uv 包管理器缓存",
            ),
            CleanupRule::global(
                ".cache/uv",
                TechStack::Python,
                RiskLevel::Safe,
                "全局缓存",
                "uv 包管理器缓存",
            ),
            CleanupRule::global(
                "Library/Caches/pip",
                TechStack::Python,
                RiskLevel::Safe,
                "全局缓存",
                "pip 全局缓存",
            ),
            CleanupRule::global(
                ".nuget/packages",
                TechStack::DotNet,
                RiskLevel::Caution,
                "全局缓存",
                "NuGet 全局包缓存",
            ),
            CleanupRule::global(
                ".pub-cache",
                TechStack::Flutter,
                RiskLevel::Caution,
                "全局缓存",
                "Dart/Flutter pub 全局缓存",
            ),
            ]
        });
        RULES.as_slice()
    }
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
        ".agents",
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
        ("go.mod", TechStack::Go),
        ("Gemfile", TechStack::Ruby),
        ("composer.json", TechStack::Php),
        ("project.godot", TechStack::Unity),
        ("mix.exs", TechStack::Ruby),
        ("build.zig", TechStack::Cpp),
        ("main.tf", TechStack::Infra),
        ("versions.tf", TechStack::Infra),
    ]
}
