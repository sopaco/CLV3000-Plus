use clv_core::{
    cleanup::{CleanupExecutor, CleanupProgress, CleanupReport},
    AppSettings, ScanItem,
};
use std::path::PathBuf;
use std::sync::{atomic::AtomicBool, mpsc, Arc};

pub enum CleanupEvent {
    Progress(CleanupProgress),
    Done(CleanupReport, Vec<PathBuf>),
}

pub struct CleanupSpawn {
    pub rx: mpsc::Receiver<CleanupEvent>,
}

pub enum CleanupPoll {
    Progress(CleanupProgress),
    Done(CleanupReport, Vec<PathBuf>),
    Disconnected,
    Idle,
}

pub fn spawn_cleanup(
    settings: AppSettings,
    items: Vec<ScanItem>,
    cancel: Arc<AtomicBool>,
) -> CleanupSpawn {
    let (tx, rx) = mpsc::sync_channel::<CleanupEvent>(64);

    std::thread::spawn(move || {
        let executor = CleanupExecutor::new(settings);
        let result = executor.execute_cancellable(&items, &cancel, |progress| {
            let _ = tx.try_send(CleanupEvent::Progress(progress));
        });
        let cleaned_paths = result.successful_paths.clone();
        let _ = tx.send(CleanupEvent::Done(result, cleaned_paths));
    });

    CleanupSpawn { rx }
}

pub fn poll_cleanup(rx: &mpsc::Receiver<CleanupEvent>) -> CleanupPoll {
    let mut latest_progress = None;
    loop {
        match rx.try_recv() {
            Ok(CleanupEvent::Progress(progress)) => latest_progress = Some(progress),
            Ok(CleanupEvent::Done(report, paths)) => {
                return CleanupPoll::Done(report, paths);
            }
            Err(mpsc::TryRecvError::Empty) => {
                return latest_progress
                    .map(CleanupPoll::Progress)
                    .unwrap_or(CleanupPoll::Idle);
            }
            Err(mpsc::TryRecvError::Disconnected) => return CleanupPoll::Disconnected,
        }
    }
}
