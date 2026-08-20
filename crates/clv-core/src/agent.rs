use crate::models::{AgentProject, ScanItem, TechStack};
use crate::scanner::{detect_project_stacks, is_agent_project_path};
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn detect_agent_projects(items: &[ScanItem]) -> Vec<AgentProject> {
    let mut by_root: HashMap<PathBuf, Vec<ScanItem>> = HashMap::new();

    for item in items {
        if let Some(root) = &item.project_root {
            by_root.entry(root.clone()).or_default().push(item.clone());
        }
    }

    // Also scan for agent-named directories even without cleanup items yet
    let mut projects = Vec::new();

    for (root, root_items) in by_root {
        let (is_agent, reason) = is_agent_project_path(&root);
        let stacks = detect_project_stacks(&root);
        let is_zombie = root_items.iter().all(|i| {
            i.last_modified
                .map(|m| (Utc::now() - m).num_days() > 30)
                .unwrap_or(true)
        });

        if !is_agent && !is_zombie {
            continue;
        }

        let total_bytes: u64 = root_items.iter().map(|i| i.size_bytes).sum();
        let last_modified = root_items
            .iter()
            .filter_map(|i| i.last_modified)
            .max();

        let days_inactive = last_modified.map(|m| (Utc::now() - m).num_days());

        let mut reason = if is_agent {
            reason
        } else {
            "长期未使用的项目".to_string()
        };

        if is_zombie {
            reason.push_str("；超过 30 天未修改");
        }

        let name = root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| root.display().to_string());

        let mut stack_list = stacks;
        if stack_list.is_empty() {
            stack_list = root_items
                .iter()
                .map(|i| i.stack)
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
        }
        if stack_list.is_empty() {
            stack_list.push(TechStack::Other);
        }

        projects.push(AgentProject {
            path: root,
            name,
            total_bytes,
            stacks: stack_list,
            last_modified,
            days_inactive,
            reason,
            items: root_items,
        });
    }

    projects.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
    projects
}
