// Windows 下以 GUI 子系统链接：双击启动不再弹出黑色终端窗口。
// 从终端（cargo run）启动时 stdout 句柄仍会被继承，debug 日志输出不受影响。
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod actions;
mod app;
mod assets;
mod i18n;
mod platform;
mod prelude;
mod services;
pub mod theme;
pub mod tray;
pub mod ui;
mod views;

use actions::CloseWindow;
use app::{shell::AppShell, ClvApp};
use clv_core::{load_settings, resolve_language};
use gpui::*;
use gpui_component::*;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use theme::apply_theme;
use tray::{
    new_pending_slot, take_pending, TrayAction, TrayController, TrayPending,
};

static TRAY_PENDING: OnceLock<TrayPending> = OnceLock::new();
static MAIN_WINDOW: Mutex<Option<WindowHandle<Root>>> = Mutex::new(None);

fn tray_pending() -> TrayPending {
    TRAY_PENDING
        .get_or_init(|| {
            let pending = new_pending_slot();
            let lang = resolve_language(load_settings().language);
            if !TrayController::install("CLV3000 Plus", pending.clone(), lang) {
                tracing::warn!("system tray unavailable on this platform");
            }
            pending
        })
        .clone()
}

fn remember_window(handle: WindowHandle<Root>) {
    if let Ok(mut slot) = MAIN_WINDOW.lock() {
        *slot = Some(handle);
    }
}

fn ensure_main_window(cx: &mut App) {
    cx.activate(true);
    let handle = MAIN_WINDOW.lock().ok().and_then(|g| g.clone());
    if let Some(handle) = handle {
        if handle
            .update(cx, |_, window, _| {
                window.activate_window();
            })
            .is_ok()
        {
            return;
        }
    }
    match open_main_window(cx) {
        Ok(handle) => remember_window(handle),
        Err(e) => tracing::error!("open window: {e}"),
    }
}

fn main() {
    init_tracing();

    let settings = load_settings();
    let theme = settings.theme;
    let application = Application::new().with_assets(assets::Assets);
    application.on_reopen(|app| {
        app.activate(true);
        if let Err(e) = open_main_window(app) {
            tracing::error!("reopen window: {e}");
        }
    });
    application.run(move |cx| {
        gpui_component::init(cx);
        apply_theme(theme, cx);
        platform::apply_app_icon();
        init_window_close_shortcuts(cx);
        let _ = tray_pending();
        let pending = tray_pending();
        cx.spawn(async move |cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(250))
                    .await;
                let action = take_pending(&pending);
                if action.is_none() {
                    continue;
                }
                let _ = cx.update(|cx| match action {
                    Some(TrayAction::Open) => ensure_main_window(cx),
                    Some(TrayAction::Scan) => ensure_main_window(cx),
                    Some(TrayAction::Quit) => cx.quit(),
                    None => {}
                });
            }
        })
        .detach();
        #[cfg(target_os = "macos")]
        init_macos_menus(cx);
        let options = window_options(cx);
        cx.spawn(async move |cx| {
            let handle = cx.open_window(options, build_root_view)?;
            remember_window(handle);
            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}

fn window_options(cx: &App) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::centered(size(px(1280.), px(720.)), cx)),
        titlebar: Some(TitleBar::title_bar_options()),
        ..WindowOptions::default()
    }
}

fn build_root_view(window: &mut Window, cx: &mut App) -> Entity<Root> {
    let app = cx.new(|cx| ClvApp::new(window, cx));
    let shell = cx.new(|cx| AppShell::new(app, window, cx));
    cx.new(|cx| Root::new(shell, window, cx))
}

fn open_main_window(app: &mut App) -> anyhow::Result<WindowHandle<Root>> {
    let handle = app.open_window(window_options(app), build_root_view)?;
    remember_window(handle.clone());
    Ok(handle)
}

fn close_active_window(_: &CloseWindow, cx: &mut App) {
    if let Some(handle) = cx.active_window() {
        let _ = handle.update(cx, |_, window, _| window.remove_window());
    }
}

fn init_window_close_shortcuts(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-w", CloseWindow, None),
        KeyBinding::new("secondary-w", CloseWindow, None),
    ]);
    cx.on_action(close_active_window);
    let _ = cx.intercept_keystrokes(|event, _window, cx| {
        if event.action.is_some() {
            return;
        }
        if is_cmd_close_window(&event.keystroke) {
            close_active_window(&CloseWindow, cx);
            cx.stop_propagation();
        }
    });
}

fn is_cmd_close_window(keystroke: &Keystroke) -> bool {
    keystroke.modifiers.platform
        && !keystroke.modifiers.control
        && !keystroke.modifiers.alt
        && !keystroke.modifiers.function
        && keystroke.key.eq_ignore_ascii_case("w")
}

#[cfg(target_os = "macos")]
fn init_macos_menus(cx: &mut App) {
    cx.set_menus(vec![Menu {
        name: "Window".into(),
        items: vec![MenuItem::action("Close Window", CloseWindow)],
    }]);
}

#[cfg(debug_assertions)]
fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("clv_app=info".parse().unwrap()))
        .init();
}

#[cfg(not(debug_assertions))]
fn init_tracing() {}
