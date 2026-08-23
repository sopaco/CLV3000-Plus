use crate::category::CleanupCategory;
use crate::models::{RiskLevel, TechStack};
use crate::settings::rule::CleanupRule;
use std::sync::LazyLock;

pub fn project_rules() -> &'static [CleanupRule] {
    static RULES: LazyLock<Vec<CleanupRule>> = LazyLock::new(|| {
        vec![
        // Rust
        CleanupRule::project(
            "target",
            TechStack::Rust,
            RiskLevel::Safe,
            CleanupCategory::CompileCache,
            "Rust 编译产物与增量缓存，可重新 cargo build 生成",
        )
        .marker("Cargo.toml"),
        // Node / Web
        CleanupRule::project(
            "node_modules",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::Dependencies,
            "Node.js 依赖目录，可通过 npm/pnpm/yarn install 恢复",
        )
        .marker("package.json"),
        CleanupRule::project(
            ".next",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::BuildCache,
            "Next.js 构建缓存",
        ),
        CleanupRule::project(
            ".nuxt",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::BuildCache,
            "Nuxt 构建缓存",
        ),
        CleanupRule::project(
            ".turbo",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::BuildCache,
            "Turborepo 缓存",
        ),
        CleanupRule::project(
            ".vite",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::BuildCache,
            "Vite 预构建与依赖缓存",
        ),
        CleanupRule::project(
            ".svelte-kit",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::BuildCache,
            "SvelteKit 构建缓存",
        ),
        CleanupRule::project(
            ".astro",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::BuildCache,
            "Astro 构建缓存",
        ),
        CleanupRule::project(
            ".angular",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::BuildCache,
            "Angular CLI 缓存",
        ),
        CleanupRule::project(
            ".output",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::BuildOutput,
            "Nuxt 3 / Nitro 输出目录",
        ),
        CleanupRule::project(
            ".vercel",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::BuildCache,
            "Vercel 本地构建缓存",
        ),
        CleanupRule::project(
            ".netlify",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::BuildCache,
            "Netlify 本地构建缓存",
        ),
        CleanupRule::project(
            ".expo",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::BuildCache,
            "Expo / React Native 缓存",
        ),
        CleanupRule::project(
            ".webpack",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::BuildCache,
            "Webpack 缓存目录",
        ),
        CleanupRule::project(
            "dist",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::BuildOutput,
            "前端构建输出目录",
        )
        .marker("package.json"),
        CleanupRule::project(
            "build",
            TechStack::NodeWeb,
            RiskLevel::Caution,
            CleanupCategory::BuildOutput,
            "前端/通用构建输出（请确认非重要产物）",
        )
        .marker("package.json"),
        CleanupRule::project(
            "build",
            TechStack::Android,
            RiskLevel::Safe,
            CleanupCategory::BuildOutput,
            "Gradle 项目构建输出",
        )
        .marker("build.gradle"),
        CleanupRule::project(
            "build",
            TechStack::Android,
            RiskLevel::Safe,
            CleanupCategory::BuildOutput,
            "Gradle Kotlin DSL 项目构建输出",
        )
        .marker("build.gradle.kts"),
        CleanupRule::project(
            ".cache",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::ToolCache,
            "各类前端工具本地缓存",
        ),
        CleanupRule::project(
            ".parcel-cache",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::BuildCache,
            "Parcel 缓存",
        ),
        CleanupRule::project(
            "cache",
            TechStack::NodeWeb,
            RiskLevel::Caution,
            CleanupCategory::DependencyCache,
            "Yarn Berry 本地包缓存",
        )
        .marker("package.json")
        .parent(".yarn"),
        CleanupRule::project(
            "coverage",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::TestOutput,
            "测试覆盖率报告",
        ),
        CleanupRule::project(
            "htmlcov",
            TechStack::Python,
            RiskLevel::Safe,
            CleanupCategory::TestOutput,
            "Python coverage HTML 报告",
        ),
        CleanupRule::project(
            ".nyc_output",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::TestOutput,
            "NYC 覆盖率原始数据",
        ),
        CleanupRule::project(
            "storybook-static",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::BuildOutput,
            "Storybook 静态导出",
        ),
        // Python
        CleanupRule::project(
            "__pycache__",
            TechStack::Python,
            RiskLevel::Safe,
            CleanupCategory::BytecodeCache,
            "Python 自动生成的缓存，可安全删除",
        ),
        CleanupRule::project(
            ".pytest_cache",
            TechStack::Python,
            RiskLevel::Safe,
            CleanupCategory::TestCache,
            "pytest 缓存",
        ),
        CleanupRule::project(
            ".mypy_cache",
            TechStack::Python,
            RiskLevel::Safe,
            CleanupCategory::TypeCheckCache,
            "mypy 缓存",
        ),
        CleanupRule::project(
            ".ruff_cache",
            TechStack::Python,
            RiskLevel::Safe,
            CleanupCategory::LintCache,
            "Ruff 缓存",
        ),
        CleanupRule::project(
            ".tox",
            TechStack::Python,
            RiskLevel::Caution,
            CleanupCategory::TestEnv,
            "tox 多环境虚拟环境，删除后需重新创建",
        ),
        CleanupRule::project(
            ".nox",
            TechStack::Python,
            RiskLevel::Caution,
            CleanupCategory::TestEnv,
            "nox 多环境虚拟环境，删除后需重新创建",
        ),
        CleanupRule::project(
            ".eggs",
            TechStack::Python,
            RiskLevel::Safe,
            CleanupCategory::BuildOutput,
            "setuptools eggs 目录",
        ),
        CleanupRule::project(
            ".hypothesis",
            TechStack::Python,
            RiskLevel::Safe,
            CleanupCategory::TestCache,
            "Hypothesis 属性测试缓存",
        ),
        CleanupRule::project(
            "",
            TechStack::Python,
            RiskLevel::Safe,
            CleanupCategory::BuildOutput,
            "setuptools egg-info 元数据目录",
        )
        .prefix("*.egg-info"),
        CleanupRule::project(
            ".venv",
            TechStack::Python,
            RiskLevel::Caution,
            CleanupCategory::VirtualEnv,
            "Python 虚拟环境，删除后需重新创建",
        ),
        CleanupRule::project(
            "venv",
            TechStack::Python,
            RiskLevel::Caution,
            CleanupCategory::VirtualEnv,
            "Python 虚拟环境，删除后需重新创建",
        ),
        // Go / Ruby / PHP
        CleanupRule::project(
            "vendor",
            TechStack::Go,
            RiskLevel::Caution,
            CleanupCategory::Dependency,
            "Go vendor 目录，可通过 go mod vendor 恢复",
        )
        .marker("go.mod"),
        CleanupRule::project(
            "vendor",
            TechStack::Php,
            RiskLevel::Caution,
            CleanupCategory::Dependency,
            "Composer vendor 目录，可通过 composer install 恢复",
        )
        .marker("composer.json"),
        CleanupRule::project(
            "bundle",
            TechStack::Ruby,
            RiskLevel::Caution,
            CleanupCategory::Dependency,
            "Bundler vendor/bundle，可通过 bundle install 恢复",
        )
        .marker("Gemfile")
        .parent("vendor"),
        // Java / Android / KMP
        CleanupRule::project(
            ".gradle",
            TechStack::Android,
            RiskLevel::Caution,
            CleanupCategory::GradleCache,
            "项目级 Gradle 缓存",
        ),
        CleanupRule::project(
            "app/build",
            TechStack::Android,
            RiskLevel::Safe,
            CleanupCategory::BuildOutput,
            "Android app 模块构建输出",
        ),
        CleanupRule::project(
            "target",
            TechStack::Java,
            RiskLevel::Safe,
            CleanupCategory::BuildOutput,
            "Maven 构建输出",
        )
        .marker("pom.xml"),
        // iOS
        CleanupRule::project(
            "Pods",
            TechStack::Ios,
            RiskLevel::Caution,
            CleanupCategory::Dependency,
            "CocoaPods 依赖，可通过 pod install 恢复",
        ),
        CleanupRule::project(
            "DerivedData",
            TechStack::Ios,
            RiskLevel::Safe,
            CleanupCategory::XcodeCache,
            "Xcode 本地构建缓存",
        ),
        // Flutter
        CleanupRule::project(
            ".dart_tool",
            TechStack::Flutter,
            RiskLevel::Safe,
            CleanupCategory::ToolCache,
            "Dart/Flutter 工具缓存",
        ),
        CleanupRule::project(
            ".flutter-plugins-dependencies",
            TechStack::Flutter,
            RiskLevel::Safe,
            CleanupCategory::PluginCache,
            "Flutter 插件依赖记录",
        ),
        // .NET
        CleanupRule::project(
            "bin",
            TechStack::DotNet,
            RiskLevel::Safe,
            CleanupCategory::BuildOutput,
            ".NET 编译输出",
        )
        .marker("*.csproj"),
        CleanupRule::project(
            "obj",
            TechStack::DotNet,
            RiskLevel::Safe,
            CleanupCategory::IntermediateOutput,
            ".NET 中间构建文件",
        )
        .marker("*.csproj"),
        // C/C++
        CleanupRule::project(
            "",
            TechStack::Cpp,
            RiskLevel::Safe,
            CleanupCategory::BuildDir,
            "CMake 构建输出",
        )
        .prefix("cmake-build-"),
        CleanupRule::project(
            "out",
            TechStack::Cpp,
            RiskLevel::Safe,
            CleanupCategory::BuildDir,
            "通用 C++ 构建输出",
        )
        .marker("CMakeLists.txt"),
        CleanupRule::project(
            "zig-cache",
            TechStack::Cpp,
            RiskLevel::Safe,
            CleanupCategory::BuildCache,
            "Zig 编译缓存",
        )
        .marker("build.zig"),
        CleanupRule::project(
            "zig-out",
            TechStack::Cpp,
            RiskLevel::Safe,
            CleanupCategory::BuildOutput,
            "Zig 构建输出",
        )
        .marker("build.zig"),
        // Unity
        CleanupRule::project(
            "Library",
            TechStack::Unity,
            RiskLevel::Safe,
            CleanupCategory::BuildCache,
            "Unity 本地 Library 缓存，可重新导入",
        )
        .marker("ProjectSettings/ProjectVersion.txt"),
        CleanupRule::project(
            "Temp",
            TechStack::Unity,
            RiskLevel::Safe,
            CleanupCategory::TempFiles,
            "Unity 临时构建文件",
        )
        .marker("ProjectSettings/ProjectVersion.txt"),
        CleanupRule::project(
            "Logs",
            TechStack::Unity,
            RiskLevel::Safe,
            CleanupCategory::Logs,
            "Unity 编辑器日志",
        )
        .marker("ProjectSettings/ProjectVersion.txt"),
        // Godot
        CleanupRule::project(
            ".godot",
            TechStack::Other,
            RiskLevel::Safe,
            CleanupCategory::EditorCache,
            "Godot 编辑器缓存",
        )
        .marker("project.godot"),
        // Terraform
        CleanupRule::project(
            ".terraform",
            TechStack::Infra,
            RiskLevel::Caution,
            CleanupCategory::ProviderCache,
            "Terraform provider 与模块缓存，terraform init 可恢复",
        ),
        // Elixir
        CleanupRule::project(
            "_build",
            TechStack::Other,
            RiskLevel::Safe,
            CleanupCategory::BuildOutput,
            "Elixir / Mix 构建输出",
        )
        .marker("mix.exs"),
        CleanupRule::project(
            ".cache",
            TechStack::NodeWeb,
            RiskLevel::Safe,
            CleanupCategory::ToolCache,
            "node_modules 内 webpack/babel 等工具缓存",
        )
        .marker("package.json")
        .parent("node_modules"),
        CleanupRule::project(
            "deps",
            TechStack::Other,
            RiskLevel::Caution,
            CleanupCategory::Dependency,
            "Elixir 依赖目录，mix deps.get 可恢复",
        )
        .marker("mix.exs"),
        ]
    });
    RULES.as_slice()
}
