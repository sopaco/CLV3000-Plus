pub mod agent;
pub mod agent_sessions;
pub mod category;
pub mod cleanup;
pub mod large_files;
pub mod locale;
pub mod messages;
pub mod models;
pub mod paths;
pub mod scanner;
pub mod settings;

pub use agent::detect_agent_projects;
pub use category::CleanupCategory;
pub use cleanup::{
    restore_trashed, purge_old_trash, CleanupExecutor, CleanupHistory, CleanupHistoryRecord,
    CleanupProgress, CleanupReport, TrashedEntry,
};
pub use messages::{
    agent_reason_matches_query, format_agent_reason, rule_description_matches_query,
    AgentReasonPart, RuleDescription,
};
pub use large_files::{LargeFileEntry, LARGE_FILE_THRESHOLD_BYTES};
pub use models::*;
pub use scanner::Scanner;
pub use locale::{
    localized_text_matches_query, resolve_language, scan_phase_preparing, tr, Language,
    LanguagePreference, ThemePreference,
};
pub use settings::{
    format_scan_paths, load_last_scan, load_settings, parse_scan_paths, save_last_scan,
    save_settings, AppSettings,
};

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
        use crate::category::CleanupCategory;
        use crate::messages::RuleDescription;
        use crate::scanner::{rule_matches_dir_name, rule_matches_marker};
        use crate::settings::CleanupRule;

        let cmake = CleanupRule {
            relative: "",
            stack: TechStack::Cpp,
            risk: RiskLevel::Safe,
            category: CleanupCategory::BuildDir,
            description: RuleDescription::R001,
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
            category: CleanupCategory::BuildOutput,
            description: RuleDescription::R034,
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
            category: CleanupCategory::BuildOutput,
            description: RuleDescription::R015,
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
        use crate::category::CleanupCategory;
        use crate::item_cleanup_bucket;
        use std::path::PathBuf;

        let cargo_cache = ScanItem {
            id: "1".into(),
            path: PathBuf::from("/Users/me/.cargo/registry/cache"),
            name: "cache".into(),
            size_bytes: 2 * 1024 * 1024,
            stack: TechStack::Rust,
            risk: RiskLevel::Caution,
            category: CleanupCategory::GlobalCache,
            description: RuleDescription::R060,
            project_root: None,
            last_modified: None,
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
            category: CleanupCategory::Toolchain,
            description: RuleDescription::R063,
            project_root: None,
            last_modified: None,
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
            category: CleanupCategory::CompileCache,
            description: RuleDescription::R001,
            project_root: Some(PathBuf::from("/Users/me/project")),
            last_modified: None,
        };
        assert_eq!(
            item_cleanup_bucket(&target),
            CleanupBucket::ProjectBuildCache
        );

        let agent_session = ScanItem {
            id: "4".into(),
            path: PathBuf::from("/Users/me/.claude/projects"),
            name: "projects".into(),
            size_bytes: 1024,
            stack: TechStack::Agent,
            risk: RiskLevel::Caution,
            category: CleanupCategory::AgentSession,
            description: RuleDescription::R108,
            project_root: None,
            last_modified: None,
        };
        assert_eq!(
            item_cleanup_bucket(&agent_session),
            CleanupBucket::AiGenerated
        );
    }

    #[test]
    fn default_selected_item_ids_only_safe() {
        use crate::category::CleanupCategory;
        use std::path::PathBuf;

        let items = vec![
            ScanItem {
                id: "safe".into(),
                path: PathBuf::from("/tmp/safe"),
                name: "safe".into(),
                size_bytes: 1,
                stack: TechStack::Rust,
                risk: RiskLevel::Safe,
                category: CleanupCategory::CompileCache,
                description: RuleDescription::R001,
                project_root: None,
                last_modified: None,
            },
            ScanItem {
                id: "caution".into(),
                path: PathBuf::from("/tmp/caution"),
                name: "caution".into(),
                size_bytes: 1,
                stack: TechStack::Rust,
                risk: RiskLevel::Caution,
                category: CleanupCategory::GlobalCache,
                description: RuleDescription::R001,
                project_root: None,
                last_modified: None,
            },
        ];
        let selected = default_selected_item_ids(&items);
        assert_eq!(selected.len(), 1);
        assert!(selected.contains("safe"));
    }

    #[test]
    fn cleanup_category_bucket_mapping() {
        use crate::category::CleanupCategory;

        assert_eq!(
            CleanupCategory::GlobalCache.cleanup_bucket(),
            CleanupBucket::SharedToolCache
        );
        assert_eq!(
            CleanupCategory::Dependencies.cleanup_bucket(),
            CleanupBucket::DevEnvironment
        );
        assert_eq!(
            CleanupCategory::BuildCache.cleanup_bucket(),
            CleanupBucket::ProjectBuildCache
        );
        assert_eq!(
            CleanupCategory::AgentCache.cleanup_bucket(),
            CleanupBucket::AiGenerated
        );
    }

    #[test]
    fn parse_scan_paths_skips_blank_lines() {
        let parsed = parse_scan_paths(" /tmp/a \n\n /tmp/b ");
        assert_eq!(parsed, vec![PathBuf::from("/tmp/a"), PathBuf::from("/tmp/b")]);
    }

    #[test]
    fn rule_description_translations_avoid_mixed_language() {
        use crate::locale::Language;
        use crate::messages::RuleDescription;

        fn has_cjk(s: &str) -> bool {
            s.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
        }

        for desc in RuleDescription::ALL {
            let en = desc.text(Language::En);
            let ja = desc.text(Language::Ja);
            assert!(!en.is_empty() && !ja.is_empty(), "empty translation for {desc:?}");
            assert!(
                !has_cjk(en),
                "English text must not contain CJK characters: {en}"
            );
        }
    }

    #[test]
    fn agent_marker_directory_detection() {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("my-project");
        std::fs::create_dir_all(project.join(".agents")).unwrap();
        let (is_agent, parts) = crate::scanner::is_agent_project_path(&project);
        assert!(is_agent);
        assert!(!parts.is_empty());
    }

    #[test]
    fn detect_agent_projects_skips_active_marker_only_repos() {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("real-app");
        std::fs::create_dir_all(project.join(".cursor")).unwrap();
        std::fs::write(project.join("AGENTS.md"), "# agents").unwrap();
        let projects = detect_agent_projects(&[], &[project.clone()]);
        assert!(
            projects.is_empty(),
            "active repos with AGENTS.md/.cursor must not be listed: {projects:?}"
        );
    }

    #[test]
    fn detect_agent_projects_keeps_name_pattern_hits() {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("claude-experiment");
        std::fs::create_dir_all(&project).unwrap();
        let projects = detect_agent_projects(&[], &[project.clone()]);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].path, project);
    }

    #[test]
    fn scan_cancellable_stops_when_flag_set() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let cancel = AtomicBool::new(true);
        let mut settings = AppSettings::default();
        settings.scan_paths = vec![PathBuf::from("/tmp")];
        let report = Scanner::new(settings).scan_cancellable(|_| {}, &cancel);
        assert!(report.cancelled || cancel.load(Ordering::Relaxed));
    }
}
