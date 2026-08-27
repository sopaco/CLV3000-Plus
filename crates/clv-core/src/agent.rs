use crate::messages::AgentReasonPart;
use crate::models::{AgentProject, ScanItem, TechStack};
use crate::scanner::{agent_path_signals, detect_project_stacks};
use chrono::Utc;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const MARKER_INACTIVE_DAYS: i64 = 14;

/// Group cleanup items and known agent roots. Marker-only projects surface
/// when inactive; name-pattern hits always surface. Does not walk the disk.
pub fn detect_agent_projects(items: &[ScanItem], known_agent_roots: &[PathBuf]) -> Vec<AgentProject> {
    let mut by_root: HashMap<PathBuf, Vec<ScanItem>> = HashMap::new();

    for item in items {
        if let Some(root) = &item.project_root {
            by_root.entry(root.clone()).or_default().push(item.clone());
        }
    }

    for root in known_agent_roots {
        by_root.entry(root.clone()).or_default();
    }

    let mut projects = Vec::new();

    for (root, root_items) in by_root {
        let (name_hit, marker_hit, mut reason_parts) = agent_path_signals(&root);
        let stacks = detect_project_stacks(&root);
        let is_zombie = !root_items.is_empty()
            && root_items.iter().all(|i| {
                i.last_modified
                    .map(|m| (Utc::now() - m).num_days() > 30)
                    .unwrap_or(true)
            });

        let total_bytes: u64 = root_items.iter().map(|i| i.size_bytes).sum();
        let last_modified = root_items
            .iter()
            .filter_map(|i| i.last_modified)
            .max()
            .or_else(|| last_modified_dir(&root));

        let days_inactive = last_modified.map(|m| (Utc::now() - m).num_days());
        let long_inactive = days_inactive.unwrap_or(i64::MAX) >= MARKER_INACTIVE_DAYS;

        // Name-pattern experiment folders always count. Marker files
        // (AGENTS.md, .cursor, …) only count after inactivity so real
        // repos that adopted agent tooling are not listed as leftovers.
        if !name_hit && !(marker_hit && (long_inactive || is_zombie)) {
            continue;
        }

        if !name_hit && !marker_hit {
            reason_parts = vec![AgentReasonPart::LongUnusedProject];
        }
        if is_zombie || long_inactive {
            if !reason_parts
                .iter()
                .any(|p| matches!(p, AgentReasonPart::InactiveOver30Days | AgentReasonPart::LongUnusedProject))
            {
                reason_parts.push(if is_zombie {
                    AgentReasonPart::InactiveOver30Days
                } else {
                    AgentReasonPart::LongUnusedProject
                });
            }
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
            reason_parts,
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
