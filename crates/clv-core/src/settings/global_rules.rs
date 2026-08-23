use crate::category::CleanupCategory;
use crate::models::{RiskLevel, TechStack};
use crate::settings::rule::CleanupRule;
use std::sync::LazyLock;

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
                CleanupCategory::GlobalCache,
                "Cargo 下载缓存（可重建）",
            ),
            CleanupRule::global(
                ".cargo/git",
                TechStack::Rust,
                RiskLevel::Caution,
                CleanupCategory::GlobalCache,
                "Cargo git 依赖 checkout（可重建）",
            ),
            CleanupRule::global(
                ".cargo/registry/index",
                TechStack::Rust,
                RiskLevel::Caution,
                CleanupCategory::GlobalCache,
                "Cargo registry 索引缓存（可重建）",
            ),
            CleanupRule::global(
                ".rustup/toolchains",
                TechStack::Rust,
                RiskLevel::Protected,
                CleanupCategory::Toolchain,
                "Rust 工具链，删除后需 rustup 重新安装",
            ),
            CleanupRule::global(
                ".pyenv/versions",
                TechStack::Python,
                RiskLevel::Protected,
                CleanupCategory::Toolchain,
                "pyenv 已安装 Python 版本，删除后需 pyenv install 恢复",
            ),
            // --- Node / Web ---
            CleanupRule::global(
                "$LOCALAPPDATA/npm-cache/_cacache",
                TechStack::NodeWeb,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "npm 全局缓存",
            ),
            CleanupRule::global(
                ".npm/_cacache",
                TechStack::NodeWeb,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "npm 旧版全局缓存（用户目录）",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/pnpm/store",
                TechStack::NodeWeb,
                RiskLevel::Caution,
                CleanupCategory::GlobalCache,
                "pnpm 内容寻址存储（删除后项目需重新安装依赖）",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/Yarn/Cache",
                TechStack::NodeWeb,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "Yarn 全局缓存",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/bun/install/cache",
                TechStack::NodeWeb,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "Bun 包管理器全局缓存",
            ),
            CleanupRule::global(
                ".bun/install/cache",
                TechStack::NodeWeb,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "Bun 用户目录安装缓存",
            ),
            CleanupRule::global(
                ".cache",
                TechStack::NodeWeb,
                RiskLevel::Caution,
                CleanupCategory::GlobalCache,
                "用户目录通用工具缓存（部分 CLI/前端工具）",
            ),
            // --- Java / Android / .NET / Flutter ---
            CleanupRule::global(
                ".gradle/caches",
                TechStack::Android,
                RiskLevel::Caution,
                CleanupCategory::GlobalCache,
                "Gradle 全局缓存",
            ),
            CleanupRule::global(
                ".gradle/wrapper/dists",
                TechStack::Android,
                RiskLevel::Caution,
                CleanupCategory::GlobalCache,
                "Gradle Wrapper 发行版下载缓存",
            ),
            CleanupRule::global(
                ".m2/repository",
                TechStack::Java,
                RiskLevel::Caution,
                CleanupCategory::GlobalCache,
                "Maven 本地仓库（删除后需重新下载依赖）",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/uv/cache",
                TechStack::Python,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "uv 包管理器缓存",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/pip/Cache",
                TechStack::Python,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "pip 全局缓存",
            ),
            CleanupRule::global(
                ".nuget/packages",
                TechStack::DotNet,
                RiskLevel::Caution,
                CleanupCategory::GlobalCache,
                "NuGet 全局包缓存",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/Pub/Cache",
                TechStack::Flutter,
                RiskLevel::Caution,
                CleanupCategory::GlobalCache,
                "Dart/Flutter pub 全局缓存",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/Microsoft/MSBuild",
                TechStack::DotNet,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "MSBuild 编译缓存",
            ),
            // --- Go / Docker ---
            CleanupRule::global(
                "$LOCALAPPDATA/go-build",
                TechStack::Go,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "Go 编译缓存（GOCACHE 默认位置）",
            ),
            CleanupRule::global(
                "go/pkg/mod",
                TechStack::Go,
                RiskLevel::Caution,
                CleanupCategory::GlobalCache,
                "Go module 下载缓存（删除后需重新 go mod download）",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/Docker",
                TechStack::Other,
                RiskLevel::Caution,
                CleanupCategory::GlobalCache,
                "Docker Desktop 本地数据与缓存",
            ),
            // --- IDE / Editor ---
            CleanupRule::global(
                "$APPDATA/Code/Cache",
                TechStack::Other,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "VS Code Electron 缓存",
            ),
            CleanupRule::global(
                "$APPDATA/Code/CachedData",
                TechStack::Other,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "VS Code 版本缓存",
            ),
            CleanupRule::global(
                "$APPDATA/Code/CachedExtensionVSIXs",
                TechStack::Other,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "VS Code 扩展 VSIX 下载缓存",
            ),
            CleanupRule::global(
                "$APPDATA/Cursor/Cache",
                TechStack::Agent,
                RiskLevel::Safe,
                CleanupCategory::AgentCache,
                "Cursor Electron 缓存",
            ),
            CleanupRule::global(
                "$APPDATA/Cursor/CachedData",
                TechStack::Agent,
                RiskLevel::Safe,
                CleanupCategory::AgentCache,
                "Cursor 版本缓存",
            ),
            CleanupRule::global(
                "$APPDATA/Cursor/logs",
                TechStack::Agent,
                RiskLevel::Safe,
                CleanupCategory::AgentCache,
                "Cursor 运行日志",
            ),
            CleanupRule::global(
                "$APPDATA/Windsurf/Cache",
                TechStack::Agent,
                RiskLevel::Safe,
                CleanupCategory::AgentCache,
                "Windsurf Electron 缓存",
            ),
            CleanupRule::global(
                "$APPDATA/Windsurf/CachedData",
                TechStack::Agent,
                RiskLevel::Safe,
                CleanupCategory::AgentCache,
                "Windsurf 版本缓存",
            ),
            CleanupRule::global(
                "$APPDATA/Trae/Cache",
                TechStack::Agent,
                RiskLevel::Safe,
                CleanupCategory::AgentCache,
                "Trae Electron 缓存",
            ),
            CleanupRule::global(
                "$APPDATA/Trae/CachedData",
                TechStack::Agent,
                RiskLevel::Safe,
                CleanupCategory::AgentCache,
                "Trae 版本缓存",
            ),
            CleanupRule::global(
                "$APPDATA/Trae/logs",
                TechStack::Agent,
                RiskLevel::Safe,
                CleanupCategory::AgentCache,
                "Trae 运行日志",
            ),
            CleanupRule::global(
                "$APPDATA/Trae CN/Cache",
                TechStack::Agent,
                RiskLevel::Safe,
                CleanupCategory::AgentCache,
                "Trae CN Electron 缓存",
            ),
            CleanupRule::global(
                "$APPDATA/Trae CN/CachedData",
                TechStack::Agent,
                RiskLevel::Safe,
                CleanupCategory::AgentCache,
                "Trae CN 版本缓存",
            ),
            CleanupRule::global(
                "$APPDATA/Trae CN/logs",
                TechStack::Agent,
                RiskLevel::Safe,
                CleanupCategory::AgentCache,
                "Trae CN 运行日志",
            ),
            // --- Windows system temp / junk ---
            CleanupRule::global(
                "$TEMP",
                TechStack::System,
                RiskLevel::Safe,
                CleanupCategory::SystemTemp,
                "Windows TEMP/TMP 环境变量指向的临时目录",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/Temp",
                TechStack::System,
                RiskLevel::Safe,
                CleanupCategory::SystemTemp,
                "Windows 用户临时文件目录（安装包/解压残留等）",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/CrashDumps",
                TechStack::System,
                RiskLevel::Safe,
                CleanupCategory::SystemTemp,
                "应用程序崩溃转储文件",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/D3DSCache",
                TechStack::System,
                RiskLevel::Safe,
                CleanupCategory::SystemTemp,
                "DirectX 着色器缓存",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/Microsoft/Windows/INetCache",
                TechStack::System,
                RiskLevel::Safe,
                CleanupCategory::SystemTemp,
                "Windows 浏览器/WebView 缓存",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/Microsoft/Windows/DeliveryOptimization/Cache",
                TechStack::System,
                RiskLevel::Safe,
                CleanupCategory::SystemTemp,
                "Windows 传递优化下载缓存",
            ),
            CleanupRule::global(
                "$LOCALAPPDATA/Microsoft/Windows/Explorer",
                TechStack::System,
                RiskLevel::Caution,
                CleanupCategory::SystemTemp,
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
                CleanupCategory::GlobalCache,
                "Cargo 下载缓存（可重建）",
            ),
            CleanupRule::global(
                ".cargo/git",
                TechStack::Rust,
                RiskLevel::Caution,
                CleanupCategory::GlobalCache,
                "Cargo git 依赖 checkout（可重建）",
            ),
            CleanupRule::global(
                ".rustup/toolchains",
                TechStack::Rust,
                RiskLevel::Protected,
                CleanupCategory::Toolchain,
                "Rust 工具链，删除后需 rustup 重新安装",
            ),
            CleanupRule::global(
                ".pyenv/versions",
                TechStack::Python,
                RiskLevel::Protected,
                CleanupCategory::Toolchain,
                "pyenv 已安装 Python 版本，删除后需 pyenv install 恢复",
            ),
            CleanupRule::global(
                ".npm/_cacache",
                TechStack::NodeWeb,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "npm 全局缓存",
            ),
            CleanupRule::global(
                "Library/Caches/pnpm",
                TechStack::NodeWeb,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "pnpm 全局缓存",
            ),
            CleanupRule::global(
                "Library/Caches/Yarn",
                TechStack::NodeWeb,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "Yarn 全局缓存",
            ),
            CleanupRule::global(
                "Library/Developer/Xcode/DerivedData",
                TechStack::Ios,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "Xcode DerivedData 全局缓存",
            ),
            CleanupRule::global(
                ".gradle/caches",
                TechStack::Android,
                RiskLevel::Caution,
                CleanupCategory::GlobalCache,
                "Gradle 全局缓存",
            ),
            CleanupRule::global(
                "Library/Caches/uv",
                TechStack::Python,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "uv 包管理器缓存",
            ),
            CleanupRule::global(
                ".cache/uv",
                TechStack::Python,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "uv 包管理器缓存",
            ),
            CleanupRule::global(
                "Library/Caches/pip",
                TechStack::Python,
                RiskLevel::Safe,
                CleanupCategory::GlobalCache,
                "pip 全局缓存",
            ),
            CleanupRule::global(
                ".nuget/packages",
                TechStack::DotNet,
                RiskLevel::Caution,
                CleanupCategory::GlobalCache,
                "NuGet 全局包缓存",
            ),
            CleanupRule::global(
                ".pub-cache",
                TechStack::Flutter,
                RiskLevel::Caution,
                CleanupCategory::GlobalCache,
                "Dart/Flutter pub 全局缓存",
            ),
            ]
        });
        RULES.as_slice()
    }
}
