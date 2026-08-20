use crate::models::{AgentProject, ScanItem, TechStack};
use crate::scanner::{detect_project_stacks, is_agent_project_path};
use crate::settings::{agent_marker_files, is_protected_system_path};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn discover_agent_roots(scan_paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut roots = HashSet::new();
    let marker_dirs: HashSet<&str> = agent_marker_files()
        .iter()
        .filter(|m| m.starts_with('.'))
        .copied()
        .collect();

    for root in scan_paths {
        if !root.exists() || is_protected_system_path(root) {
            continue;
        }
        for entry in WalkDir::new(root)
            .follow_links(false)
            .max_depth(8)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if is_protected_system_path(path) {
                continue;
            }
            let name = entry.file_name().to_string_lossy();
            if marker_dirs.contains(name.as_ref()) {
                if let Some(parent) = path.parent() {
                    roots.insert(parent.to_path_buf());
                }
            }
            if name == "AGENTS.md" || name == "CLAUDE.md" {
                if let Some(parent) = path.parent() {
                    roots.insert(parent.to_path_buf());
                }
            }
        }
    }

    let mut list: Vec<PathBuf> = roots.into_iter().collect();
    list.sort();
    list
}

pub fn detect_agent_projects(items: &[ScanItem], scan_paths: &[PathBuf]) -> Vec<AgentProject> {
    let mut by_root: HashMap<PathBuf, Vec<ScanItem>> = HashMap::new();

    for item in items {
        if let Some(root) = &item.project_root {
            by_root.entry(root.clone()).or_default().push(item.clone());
        }
    }

    for root in discover_agent_roots(scan_paths) {
        by_root.entry(root).or_default();
    }

    let mut projects = Vec::new();

    for (root, root_items) in by_root {
        let (is_agent, reason) = is_agent_project_path(&root);
        let stacks = detect_project_stacks(&root);
        let is_zombie = !root_items.is_empty()
            && root_items.iter().all(|i| {
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
            .max()
            .or_else(|| last_modified_dir(&root));

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
            stack_list.push(TechStack::Agent);
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

fn last_modified_dir(path: &Path) -> Option<chrono::DateTime<Utc>> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    Some(chrono::DateTime::<Utc>::from(modified))
}
