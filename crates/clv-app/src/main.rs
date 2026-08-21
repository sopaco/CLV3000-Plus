// Windows 下以 GUI 子系统链接：双击启动不再弹出黑色终端窗口。
// 从终端（cargo run）启动时 stdout 句柄仍会被继承，debug 日志输出不受影响。
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod assets;
mod platform;
mod prelude;
pub mod theme;
pub mod ui;
mod views;

use app::{shell::AppShell, ClvApp};
use gpui::*;
use gpui_component::*;
use theme::apply_clv_theme;

fn main() {
    init_tracing();

    let application = Application::new().with_assets(assets::Assets);
    application.on_reopen(|app| {
        app.activate(true);
        if let Err(e) = open_main_window(app) {
            tracing::error!("reopen window: {e}");
        }
    });
    application.run(|cx| {
        gpui_component::init(cx);
        apply_clv_theme(cx);
        platform::apply_app_icon();
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

#[cfg(debug_assertions)]
fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("clv_app=info".parse().unwrap()))
        .init();
}

#[cfg(not(debug_assertions))]
fn init_tracing() {}
