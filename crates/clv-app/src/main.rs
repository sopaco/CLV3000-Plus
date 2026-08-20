mod app;
mod assets;
mod platform;
mod prelude;
pub mod theme;
pub mod ui;
mod views;

use app::ClvApp;
use gpui::*;
use gpui_component::*;
use theme::apply_clv_theme;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("clv_app=info".parse().unwrap()))
        .init();

    Application::new()
        .with_assets(assets::Assets)
        .run(|cx| {
            gpui_component::init(cx);
            apply_clv_theme(cx);
            platform::apply_app_icon();
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::centered(size(px(1280.), px(720.)), cx)),
                ..WindowOptions::default()
            };
            cx.spawn(async move |cx| {
                cx.open_window(options, |window, cx| {
                    let view = cx.new(|cx| ClvApp::new(window, cx));
                    cx.new(|cx| Root::new(view, window, cx))
                })?;
                Ok::<_, anyhow::Error>(())
            })
            .detach();
        });
}
