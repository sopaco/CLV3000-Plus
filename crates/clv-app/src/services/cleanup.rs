use clv_core::{
    cleanup::{CleanupExecutor, CleanupReport},
    AppSettings, ScanItem,
};
use std::path::PathBuf;
use std::sync::mpsc;

pub enum CleanupEvent {
    Done(CleanupReport, Vec<PathBuf>),
}

pub struct CleanupSpawn {
    pub rx: mpsc::Receiver<CleanupEvent>,
}

pub enum CleanupPoll {
    Done(CleanupReport, Vec<PathBuf>),
    Disconnected,
    Idle,
}

pub fn spawn_cleanup(settings: AppSettings, items: Vec<ScanItem>) -> CleanupSpawn {
    let selected_paths: Vec<_> = items.iter().map(|i| i.path.clone()).collect();
    let (tx, rx) = mpsc::channel::<CleanupEvent>();

    std::thread::spawn(move || {
        let executor = CleanupExecutor::new(settings);
        let result = executor.execute(&items);
        let _ = tx.send(CleanupEvent::Done(result, selected_paths));
    });

    CleanupSpawn { rx }
}

pub fn poll_cleanup(rx: &mpsc::Receiver<CleanupEvent>) -> CleanupPoll {
    match rx.try_recv() {
        Ok(CleanupEvent::Done(report, paths)) => CleanupPoll::Done(report, paths),
        Err(mpsc::TryRecvError::Empty) => CleanupPoll::Idle,
        Err(mpsc::TryRecvError::Disconnected) => CleanupPoll::Disconnected,
    }
}
