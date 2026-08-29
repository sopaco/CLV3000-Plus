//! System tray / menu bar integration.

use crate::i18n::I18n;
use clv_core::Language;
use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    Open,
    Scan,
    Quit,
}

pub type TrayPending = Arc<Mutex<Option<TrayAction>>>;

static SCAN_REQUESTED: AtomicBool = AtomicBool::new(false);

thread_local! {
    static TRAY: RefCell<Option<&'static Mutex<TrayIcon>>> = const { RefCell::new(None) };
}

pub fn take_scan_request() -> bool {
    SCAN_REQUESTED.swap(false, Ordering::Relaxed)
}

pub fn request_scan() {
    SCAN_REQUESTED.store(true, Ordering::Relaxed);
}

pub struct TrayController;

impl TrayController {
    pub fn install(initial_tooltip: &str, pending: TrayPending, lang: Language) -> bool {
        let Some(menu) = build_menu(lang) else {
            return false;
        };
        let Some(icon) = tray_icon() else {
            return false;
        };

        let Ok(tray) = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip(initial_tooltip)
            .with_icon(icon)
            .build()
        else {
            return false;
        };

        let pending_menu = pending.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            if let Some(action) = menu_action_for_id(event.id().as_ref()) {
                if matches!(action, TrayAction::Scan) {
                    request_scan();
                }
                if let Ok(mut slot) = pending_menu.lock() {
                    *slot = Some(action);
                }
            }
        }));

        let pending_tray = pending;
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
            if let TrayIconEvent::DoubleClick { .. } = event {
                if let Ok(mut slot) = pending_tray.lock() {
                    *slot = Some(TrayAction::Open);
                }
            }
        }));

        let leaked: &'static Mutex<TrayIcon> = Box::leak(Box::new(Mutex::new(tray)));
        TRAY.with(|slot| {
            *slot.borrow_mut() = Some(leaked);
        });
        true
    }

    pub fn set_global_tooltip(text: &str) {
        TRAY.with(|slot| {
            if let Some(tray) = *slot.borrow() {
                if let Ok(icon) = tray.lock() {
                    let _ = icon.set_tooltip(Some(text));
                }
            }
        });
    }

    pub fn set_global_menu(lang: Language) {
        TRAY.with(|slot| {
            if let Some(tray) = *slot.borrow() {
                if let (Ok(icon), Some(menu)) = (tray.lock(), build_menu(lang)) {
                    icon.set_menu(Some(Box::new(menu)));
                }
            }
        });
    }
}

fn build_menu(lang: Language) -> Option<Menu> {
    let i18n = I18n { lang };
    let menu = Menu::new();
    menu.append(&MenuItem::with_id(
        "clv-tray-open",
        i18n.tray_open(),
        true,
        None,
    ))
    .ok()?;
    menu.append(&MenuItem::with_id(
        "clv-tray-scan",
        i18n.tray_scan(),
        true,
        None,
    ))
    .ok()?;
    menu.append(&PredefinedMenuItem::separator()).ok()?;
    menu.append(&MenuItem::with_id("clv-tray-quit", i18n.tray_quit(), true, None))
        .ok()?;
    Some(menu)
}

fn menu_action_for_id(id: &str) -> Option<TrayAction> {
    match id {
        "clv-tray-open" => Some(TrayAction::Open),
        "clv-tray-scan" => Some(TrayAction::Scan),
        "clv-tray-quit" => Some(TrayAction::Quit),
        _ => None,
    }
}

fn tray_icon() -> Option<Icon> {
    const PNG: &[u8] = include_bytes!("../assets/icons/tray.png");
    let img = image::load_from_memory(PNG).ok()?.into_rgba8();
    // Status-item backends expect a small bitmap; the source asset is 512².
    let img = if img.width() > 64 || img.height() > 64 {
        image::imageops::resize(&img, 64, 64, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    let (width, height) = img.dimensions();
    Icon::from_rgba(img.into_raw(), width, height).ok()
}

pub fn new_pending_slot() -> TrayPending {
    Arc::new(Mutex::new(None))
}

pub fn take_pending(pending: &TrayPending) -> Option<TrayAction> {
    pending.lock().ok()?.take()
}
