use clv_core::{AppScanOptions, AppScanProgress, AppScanReport, scan_apps};
use std::sync::{Arc, atomic::AtomicBool, mpsc};

pub enum AppScanEvent {
    Progress(AppScanProgress),
    Done(AppScanReport),
}

pub struct AppScanSpawn {
    pub rx: mpsc::Receiver<AppScanEvent>,
}

pub enum AppScanPoll {
    Progress(AppScanProgress),
    Done(AppScanReport),
    Disconnected,
    Idle,
}

/// Runs the app-uninstaller scan on a background thread. Running executable
/// paths are collected here (platform concern) and injected into the domain
/// scanner.
pub fn spawn_app_scan(cancel: Arc<AtomicBool>) -> AppScanSpawn {
    let (tx, rx) = mpsc::sync_channel::<AppScanEvent>(64);

    std::thread::spawn(move || {
        let running_paths = clv_platform::running_executable_paths();
        let windows = clv_platform::collect_windows_scan_inputs();
        let options = AppScanOptions {
            running_paths,
            windows,
            ..AppScanOptions::default()
        };
        let report = scan_apps(&options, &cancel, |progress| {
            let _ = tx.try_send(AppScanEvent::Progress(progress));
        });
        let _ = tx.send(AppScanEvent::Done(report));
    });

    AppScanSpawn { rx }
}

pub fn poll_app_scan(rx: &mpsc::Receiver<AppScanEvent>) -> AppScanPoll {
    let mut latest_progress = None;
    loop {
        match rx.try_recv() {
            Ok(AppScanEvent::Progress(progress)) => latest_progress = Some(progress),
            Ok(AppScanEvent::Done(report)) => return AppScanPoll::Done(report),
            Err(mpsc::TryRecvError::Empty) => {
                return latest_progress
                    .map(AppScanPoll::Progress)
                    .unwrap_or(AppScanPoll::Idle);
            }
            Err(mpsc::TryRecvError::Disconnected) => return AppScanPoll::Disconnected,
        }
    }
}
