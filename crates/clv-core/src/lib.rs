pub mod agent;
pub mod agent_sessions;
pub mod cleanup;
pub mod models;
pub mod paths;
pub mod scanner;
pub mod settings;

pub use agent::detect_agent_projects;
pub use cleanup::{CleanupExecutor, CleanupReport};
pub use models::*;
pub use scanner::Scanner;
pub use settings::{format_scan_paths, load_settings, parse_scan_paths, save_settings, AppSettings};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::AppSettings;
    use std::path::PathBuf;

    #[test]
    fn scanner_finds_node_modules_pattern() {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("my-app");
        std::fs::create_dir_all(project.join("node_modules/nested")).unwrap();
        std::fs::write(project.join("package.json"), "{}").unwrap();
        std::fs::write(
            project.join("node_modules/nested/file"),
            "x".repeat(1024 * 1024),
        )
        .unwrap();

        let mut settings = AppSettings::default();
        settings.scan_paths = vec![project.clone()];
        let report = Scanner::new(settings).scan(|_| {});
        assert!(
            report.items.iter().any(|i| i.name == "node_modules"),
            "items: {:?}",
            report.items.iter().map(|i| &i.path).collect::<Vec<_>>()
        );
    }

    #[test]
    fn scanner_prunes_nested_cleanup_inside_matched_parent() {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("web-app");
        std::fs::create_dir_all(project.join("node_modules/.cache")).unwrap();
        std::fs::write(project.join("package.json"), "{}").unwrap();
        std::fs::write(
            project.join("node_modules/.cache/webpack.bin"),
            "x".repeat(2 * 1024 * 1024),
        )
        .unwrap();

        let mut settings = AppSettings::default();
        settings.scan_paths = vec![project.clone()];
        let report = Scanner::new(settings).scan(|_| {});

        let paths: Vec<_> = report
            .items
            .iter()
            .filter(|i| i.path.starts_with(&project))
            .map(|i| i.path.clone())
            .collect();
        assert!(
            paths.iter().any(|p| p.ends_with("node_modules")),
            "expected node_modules, got: {paths:?}"
        );
        assert!(
            !paths.iter().any(|p| p.ends_with(".cache")),
            "nested .cache should be pruned, got: {paths:?}"
        );
        let project_items: Vec<_> = report
            .items
            .iter()
            .filter(|i| i.path.starts_with(&project))
            .collect();
        assert_eq!(project_items.len(), 1, "expected single item, got: {project_items:?}");
        assert_eq!(project_items[0].name, "node_modules");
    }

    #[test]
    fn scanner_keeps_sibling_build_directories() {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("android-app");
        std::fs::create_dir_all(project.join("build")).unwrap();
        std::fs::create_dir_all(project.join("app/build")).unwrap();
        std::fs::write(project.join("build.gradle"), "plugins {}").unwrap();
        std::fs::write(
            project.join("build/output.bin"),
            "x".repeat(2 * 1024 * 1024),
        )
        .unwrap();
        std::fs::write(
            project.join("app/build/output.bin"),
            "x".repeat(2 * 1024 * 1024),
        )
        .unwrap();

        let mut settings = AppSettings::default();
        settings.scan_paths = vec![project.clone()];
        let report = Scanner::new(settings).scan(|_| {});

        let names: Vec<_> = report.items.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"build"), "expected root build, got: {names:?}");
        assert!(
            report.items.iter().any(|i| i.path.ends_with("app/build")),
            "expected app/build, got: {:?}",
            report.items.iter().map(|i| &i.path).collect::<Vec<_>>()
        );
    }

    #[test]
    fn rule_prefix_and_marker_matching() {
        use crate::scanner::{rule_matches_dir_name, rule_matches_marker};
        use crate::settings::CleanupRule;

        let cmake = CleanupRule {
            relative: "",
            stack: TechStack::Cpp,
            risk: RiskLevel::Safe,
            category: "构建目录",
            description: "CMake 构建输出",
            global: false,
            requires_marker: None,
            relative_prefix: Some("cmake-build-"),
            requires_parent: None,
        };
        assert!(rule_matches_dir_name("cmake-build-debug", &cmake));
        assert!(!rule_matches_dir_name("build", &cmake));

        let egg = CleanupRule {
            relative: "",
            stack: TechStack::Python,
            risk: RiskLevel::Safe,
            category: "构建产物",
            description: "egg-info",
            global: false,
            requires_marker: None,
            relative_prefix: Some("*.egg-info"),
            requires_parent: None,
        };
        assert!(rule_matches_dir_name("my_pkg.egg-info", &egg));

        let dist = CleanupRule {
            relative: "dist",
            stack: TechStack::NodeWeb,
            risk: RiskLevel::Safe,
            category: "构建产物",
            description: "dist",
            global: false,
            requires_marker: Some("package.json"),
            relative_prefix: None,
            requires_parent: None,
        };
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("web-app");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("package.json"), "{}").unwrap();
        assert!(rule_matches_marker(Some(&root), &dist));
    }

    #[test]
    fn cleanup_bucket_classification() {
        use crate::item_cleanup_bucket;
        use std::path::PathBuf;

        let cargo_cache = ScanItem {
            id: "1".into(),
            path: PathBuf::from("/Users/me/.cargo/registry/cache"),
            name: "cache".into(),
            size_bytes: 2 * 1024 * 1024,
            stack: TechStack::Rust,
            risk: RiskLevel::Caution,
            category: "全局缓存".into(),
            description: "Cargo 下载缓存".into(),
            project_root: None,
            last_modified: None,
            selected: false,
        };
        assert_eq!(
            item_cleanup_bucket(&cargo_cache),
            CleanupBucket::SharedToolCache
        );

        let rustup = ScanItem {
            id: "2".into(),
            path: PathBuf::from("/Users/me/.rustup/toolchains"),
            name: "toolchains".into(),
            size_bytes: 5 * 1024 * 1024,
            stack: TechStack::Rust,
            risk: RiskLevel::Protected,
            category: "工具链".into(),
            description: "Rust 工具链".into(),
            project_root: None,
            last_modified: None,
            selected: false,
        };
        assert_eq!(
            item_cleanup_bucket(&rustup),
            CleanupBucket::DevEnvironment
        );

        let target = ScanItem {
            id: "3".into(),
            path: PathBuf::from("/Users/me/project/target"),
            name: "target".into(),
            size_bytes: 3 * 1024 * 1024,
            stack: TechStack::Rust,
            risk: RiskLevel::Safe,
            category: "编译缓存".into(),
            description: "Rust 编译产物".into(),
            project_root: Some(PathBuf::from("/Users/me/project")),
            last_modified: None,
            selected: true,
        };
        assert_eq!(
            item_cleanup_bucket(&target),
            CleanupBucket::ProjectBuildCache
        );
    }

    #[test]
    fn parse_scan_paths_skips_blank_lines() {
        let parsed = parse_scan_paths(" /tmp/a \n\n /tmp/b ");
        assert_eq!(parsed, vec![PathBuf::from("/tmp/a"), PathBuf::from("/tmp/b")]);
    }

    #[test]
    fn agent_marker_directory_detection() {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("my-project");
        std::fs::create_dir_all(project.join(".agents")).unwrap();
        let (is_agent, _) = crate::scanner::is_agent_project_path(&project);
        assert!(is_agent);
    }
}
