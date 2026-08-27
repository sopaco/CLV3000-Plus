use clv_core::{ScanProgress, ScanReport, Scanner, AppSettings};
use std::sync::{atomic::AtomicBool, mpsc, Arc};

pub enum ScanEvent {
    Progress(ScanProgress),
    Done(ScanReport),
}

pub struct ScanSpawn {
    pub rx: mpsc::Receiver<ScanEvent>,
}

pub enum ScanPoll {
    Progress(ScanProgress),
    Done(ScanReport),
    Disconnected,
    Idle,
}

pub fn spawn_scan(settings: AppSettings, cancel: Arc<AtomicBool>) -> ScanSpawn {
    let (tx, rx) = mpsc::sync_channel::<ScanEvent>(64);

    std::thread::spawn(move || {
        let scanner = Scanner::new(settings);
        let report = scanner.scan_cancellable(
            |progress| {
                let _ = tx.try_send(ScanEvent::Progress(progress));
            },
            &cancel,
        );
        let _ = tx.send(ScanEvent::Done(report));
    });

    ScanSpawn { rx }
}

pub fn poll_scan(rx: &mpsc::Receiver<ScanEvent>) -> ScanPoll {
    let mut latest_progress = None;
    loop {
        match rx.try_recv() {
            Ok(ScanEvent::Progress(progress)) => latest_progress = Some(progress),
            Ok(ScanEvent::Done(report)) => return ScanPoll::Done(report),
            Err(mpsc::TryRecvError::Empty) => {
                return latest_progress
                    .map(ScanPoll::Progress)
                    .unwrap_or(ScanPoll::Idle);
            }
            Err(mpsc::TryRecvError::Disconnected) => return ScanPoll::Disconnected,
        }
    }
}
