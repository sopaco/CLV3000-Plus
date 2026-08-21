use serde::{Deserialize, Serialize};
use sysinfo::System;

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
    #[cfg(target_os = "windows")]
    {
        return kill_process_windows(pid);
    }

    #[cfg(unix)]
    {
        return kill_process_unix(pid);
    }
}

#[cfg(unix)]
fn kill_process_unix(pid: u32) -> anyhow::Result<()> {
    use std::time::Duration;

    unsafe {
        if libc::kill(pid as i32, 0) != 0 {
            let err = std::io::Error::last_os_error();
            anyhow::bail!("进程 {pid} 不存在或无法访问：{err}");
        }

        if libc::kill(pid as i32, libc::SIGTERM) == 0 {
            for _ in 0..10 {
                std::thread::sleep(Duration::from_millis(50));
                if libc::kill(pid as i32, 0) != 0 {
                    return Ok(());
                }
            }
        }

        if libc::kill(pid as i32, libc::SIGKILL) != 0 {
            let err = std::io::Error::last_os_error();
            anyhow::bail!("无法结束进程 {pid}：{err}");
        }

        std::thread::sleep(Duration::from_millis(100));
        if libc::kill(pid as i32, 0) == 0 {
            anyhow::bail!("进程 {pid} 仍在运行，可能需要管理员权限");
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn kill_process_windows(pid: u32) -> anyhow::Result<()> {
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, FALSE};
    use windows_sys::Win32::System::Threading::{
        OpenProcess, TerminateProcess, PROCESS_TERMINATE,
    };

    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, FALSE, pid);
        if handle == 0 {
            anyhow::bail!(
                "无法打开进程 {pid}（错误码 {}），可能需要管理员权限",
                GetLastError()
            );
        }

        let terminated = TerminateProcess(handle, 1);
        CloseHandle(handle);

        if terminated == FALSE {
            anyhow::bail!(
                "无法结束进程 {pid}（错误码 {}），可能需要管理员权限",
                GetLastError()
            );
        }
    }

    // Give the kernel a moment to reap the process, then verify.
    std::thread::sleep(std::time::Duration::from_millis(100));
    let mut sys = System::new();
    let pid_obj = Pid::from_u32(pid);
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid_obj]), false);
    if sys.process(pid_obj).is_some() {
        anyhow::bail!("进程 {pid} 仍在运行，可能需要管理员权限");
    }

    Ok(())
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
