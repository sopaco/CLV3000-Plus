use serde::{Deserialize, Serialize};
use sysinfo::{Pid, ProcessesToUpdate, System};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessSort {
    Memory,
    Cpu,
    Name,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessCategory {
    System,
    User,
    Dev,
    Agent,
}

impl ProcessCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::System => "系统",
            Self::User => "用户",
            Self::Dev => "开发",
            Self::Agent => "Agent",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub category: ProcessCategory,
}

/// Reusable process enumerator — avoids allocating a new `System` on every poll.
pub struct ProcessEnumerator {
    sys: System,
}

impl ProcessEnumerator {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self { sys }
    }

    pub fn list(&mut self, sort: ProcessSort) -> Vec<ProcessInfo> {
        self.sys.refresh_all();
        collect_processes(&self.sys, sort)
    }
}

pub fn list_processes(sort: ProcessSort) -> Vec<ProcessInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();
    collect_processes(&sys, sort)
}

fn collect_processes(sys: &System, sort: ProcessSort) -> Vec<ProcessInfo> {
    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, process)| {
            let name = process.name().to_string_lossy().to_string();
            ProcessInfo {
                pid: pid.as_u32(),
                name: name.clone(),
                cpu_percent: process.cpu_usage(),
                memory_bytes: process.memory(),
                category: categorize_process(&name),
            }
        })
        .collect();

    match sort {
        ProcessSort::Memory => processes.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes)),
        ProcessSort::Cpu => processes.sort_by(|a, b| {
            b.cpu_percent
                .partial_cmp(&a.cpu_percent)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        ProcessSort::Name => processes.sort_by(|a, b| a.name.cmp(&b.name)),
    }

    processes
}

pub fn kill_process(pid: u32) -> anyhow::Result<()> {
    let mut sys = System::new();
    let pid = Pid::from_u32(pid);
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), false);
    if let Some(process) = sys.process(pid) {
        if !process.kill() {
            anyhow::bail!("failed to kill process {pid}");
        }
        Ok(())
    } else {
        anyhow::bail!("process {pid} not found")
    }
}

fn categorize_process(name: &str) -> ProcessCategory {
    let lower = name.to_lowercase();
    let agent_keywords = [
        "cursor", "claude", "codex", "copilot", "windsurf", "aider", "node", "cargo",
        "gradle", "java", "python", "dotnet", "flutter", "dart", "xcodebuild",
    ];
    let system_keywords = ["kernel", "launchd", "systemd", "svchost", "windowserver"];

    if agent_keywords.iter().any(|k| lower.contains(k)) {
        ProcessCategory::Dev
    } else if lower.contains("agent") || lower.contains("workbuddy") {
        ProcessCategory::Agent
    } else if system_keywords.iter().any(|k| lower.contains(k)) {
        ProcessCategory::System
    } else {
        ProcessCategory::User
    }
}
