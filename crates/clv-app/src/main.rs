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
pub mod ui;
mod views;

use actions::CloseWindow;
use app::{shell::AppShell, ClvApp};
use gpui::*;
use gpui_component::*;
use clv_core::load_settings;
use theme::apply_theme;

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
        #[cfg(target_os = "macos")]
        init_macos_menus(cx);
        let options = window_options(cx);
        cx.spawn(async move |cx| {
            cx.open_window(options, build_root_view)?;
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
    app.open_window(window_options(app), build_root_view)
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
