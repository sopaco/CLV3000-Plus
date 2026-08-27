//! System tray / menu bar integration.

use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    Open,
    Scan,
    Quit,
}

pub type TrayPending = Arc<Mutex<Option<TrayAction>>>;

thread_local! {
    static TRAY: RefCell<Option<&'static TrayController>> = const { RefCell::new(None) };
}

pub struct TrayController {
    _tray: TrayIcon,
}

impl TrayController {
    pub fn install(initial_tooltip: &str, pending: TrayPending) -> bool {
        let (menu_tx, _menu_rx) = std::sync::mpsc::channel::<TrayAction>();

        let Some(menu) = build_menu() else {
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

        let _ = menu_tx;
        let controller = Self { _tray: tray };
        let leaked: &'static TrayController = Box::leak(Box::new(controller));
        TRAY.with(|slot| {
            *slot.borrow_mut() = Some(leaked);
        });
        true
    }

    pub fn set_global_tooltip(text: &str) {
        TRAY.with(|slot| {
            if let Some(tray) = *slot.borrow() {
                tray.set_tooltip(text);
            }
        });
    }

    pub fn set_tooltip(&self, text: &str) {
        let _ = self._tray.set_tooltip(Some(text));
    }
}

fn build_menu() -> Option<Menu> {
    let menu = Menu::new();
    menu.append(&MenuItem::with_id("clv-tray-open", "Open CLV3000 Plus", true, None))
        .ok()?;
    menu.append(&MenuItem::with_id("clv-tray-scan", "Scan Now", true, None))
        .ok()?;
    menu.append(&PredefinedMenuItem::separator()).ok()?;
    menu.append(&MenuItem::with_id("clv-tray-quit", "Quit", true, None))
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
    let size = 32u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let t = (x as f32 / size as f32).clamp(0., 1.);
            let r = (20.0 + t * 40.0) as u8;
            let g = (140.0 + t * 60.0) as u8;
            let b = (220.0 - t * 30.0) as u8;
            let edge = x < 2 || y < 2 || x >= size - 2 || y >= size - 2;
            let alpha = if edge { 0 } else { 255 };
            rgba.extend_from_slice(&[r, g, b, alpha]);
        }
    }
    Icon::from_rgba(rgba, size, size).ok()
}

pub fn new_pending_slot() -> TrayPending {
    Arc::new(Mutex::new(None))
}

pub fn take_pending(pending: &TrayPending) -> Option<TrayAction> {
    pending.lock().ok()?.take()
}
