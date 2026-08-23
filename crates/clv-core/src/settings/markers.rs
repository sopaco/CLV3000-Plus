use crate::models::TechStack;

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
        "trae",
        "opencode",
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
        ".trae",
        ".opencode",
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
