pub mod agent;
pub mod cleanup;
pub mod models;
pub mod scanner;
pub mod settings;

pub use agent::detect_agent_projects;
pub use cleanup::{CleanupExecutor, CleanupReport};
pub use models::*;
pub use scanner::Scanner;
pub use settings::{load_settings, save_settings, AppSettings};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::AppSettings;

    #[test]
    fn scanner_finds_node_modules_pattern() {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("my-app");
        std::fs::create_dir_all(project.join("node_modules/nested")).unwrap();
        std::fs::write(project.join("package.json"), "{}").unwrap();
        std::fs::write(project.join("node_modules/nested/file"), "x".repeat(1024)).unwrap();

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
    fn agent_name_detection() {
        let temp = tempfile::tempdir().unwrap();
        let agent_dir = temp.path().join("claude-todo-app");
        std::fs::create_dir_all(&agent_dir).unwrap();
        let (is_agent, _) = crate::scanner::is_agent_project_path(&agent_dir);
        assert!(is_agent);
    }
}
