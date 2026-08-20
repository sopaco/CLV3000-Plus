pub mod shell;
pub mod state;

use crate::prelude::*;
use crate::theme::colors;
use crate::views::{
    agent::AgentView, cleanup::CleanupView, dashboard::DashboardView, onboarding::OnboardingView,
    process::ProcessView, settings::SettingsView, startup::StartupView,
};
use clv_core::{format_bytes, load_settings, save_settings};
use state::{AppPage, AppStore};

pub struct ClvApp {
    store: Entity<AppStore>,
    dashboard: Entity<DashboardView>,
    cleanup: Entity<CleanupView>,
    agent: Entity<AgentView>,
    startup: Entity<StartupView>,
    process: Entity<ProcessView>,
    settings: Entity<SettingsView>,
    onboarding: Entity<OnboardingView>,
}

impl ClvApp {
    pub fn new(window: &Window, cx: &mut Context<Self>) -> Self {
        let settings = load_settings();
        let store = cx.new(|cx| AppStore::new(settings, cx));
        let dashboard = cx.new(|cx| DashboardView::new(store.clone(), cx));
        let cleanup = cx.new(|cx| CleanupView::new(store.clone(), cx));
        let agent = cx.new(|cx| AgentView::new(store.clone(), cx));
        let startup = cx.new(|cx| StartupView::new(store.clone(), cx));
        let process = cx.new(|cx| ProcessView::new(store.clone(), cx));
        let settings_view = cx.new(|cx| SettingsView::new(store.clone(), cx));
        let onboarding = cx.new(|cx| OnboardingView::new(store.clone(), cx));

        if !store.read(cx).settings.onboarding_done {
            store.update(cx, |s, cx| {
                s.page = AppPage::Onboarding;
                cx.notify();
            });
        }

        let _ = window;
        Self {
            store,
            dashboard,
            cleanup,
            agent,
            startup,
            process,
            settings: settings_view,
            onboarding,
        }
    }

    fn nav_icon(
        &self,
        id: &'static str,
        icon: impl Into<Icon>,
        label: &'static str,
        page: AppPage,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let active = self.store.read(cx).page == page;
        let store = self.store.clone();
        ui::nav_icon(id, icon, label, active, window, cx, move |_, _, cx| {
            store.update(cx, |s, cx| {
                s.set_page(page, cx);
            });
        })
    }

    pub fn current_page(&self, cx: &App) -> AppPage {
        self.store.read(cx).page
    }

    fn render_page(&self, page: AppPage) -> impl IntoElement {
        match page {
            AppPage::Dashboard => self.dashboard.clone().into_any_element(),
            AppPage::Cleanup => self.cleanup.clone().into_any_element(),
            AppPage::Agent => self.agent.clone().into_any_element(),
            AppPage::Startup => self.startup.clone().into_any_element(),
            AppPage::Process => self.process.clone().into_any_element(),
            AppPage::Settings => self.settings.clone().into_any_element(),
            AppPage::Onboarding => self.onboarding.clone().into_any_element(),
        }
    }
}

impl Render for ClvApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = self.store.read(cx).page;

        if page == AppPage::Onboarding {
            return div()
                .size_full()
                .bg(ui::hero_gradient_alt())
                .child(self.onboarding.clone());
        }

        let page_content = self.render_page(page);
        let show_scan_bar = self.store.read(cx).scanning;
        let scan_phase = self.store.read(cx).scan_phase.clone();
        let scan_items_found = self.store.read(cx).scan_items_found;
        let scan_bytes_found = self.store.read(cx).scan_bytes_found;
        let scan_current_path = self.store.read(cx).scan_current_path.clone();

        div()
            .size_full()
            .flex()
            .bg(colors::bg_app())
            .text_color(colors::text_primary())
            .child(
                // Vertical icon sidebar — antivirus style
                div()
                    .w(px(88.))
                    .h_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .bg(ui::sidebar_gradient())
                    .border_r_1()
                    .border_color(colors::panel_divider())
                    .child(ui::sidebar_logo())
                    .child(
                        div()
                            .flex_1()
                            .w_full()
                            .min_h_0()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_1()
                            .pt_1()
                            .child(self.nav_icon(
                                "nav-home",
                                ui::NAV_HOME,
                                "首页",
                                AppPage::Dashboard,
                                window,
                                cx,
                            ))
                            .child(self.nav_icon(
                                "nav-clean",
                                ui::nav_cleanup_icon(),
                                "清理",
                                AppPage::Cleanup,
                                window,
                                cx,
                            ))
                            .child(self.nav_icon(
                                "nav-agent",
                                ui::NAV_AGENT,
                                "Agent",
                                AppPage::Agent,
                                window,
                                cx,
                            ))
                            .child(self.nav_icon(
                                "nav-startup",
                                ui::NAV_STARTUP,
                                "启动",
                                AppPage::Startup,
                                window,
                                cx,
                            ))
                            .child(self.nav_icon(
                                "nav-process",
                                ui::NAV_PROCESS,
                                "进程",
                                AppPage::Process,
                                window,
                                cx,
                            )),
                    )
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .items_center()
                            .pb_3()
                            .child(self.nav_icon(
                                "nav-settings",
                                ui::NAV_SETTINGS,
                                "设置",
                                AppPage::Settings,
                                window,
                                cx,
                            )),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .bg(ui::content_gradient())
                    .child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .flex()
                            .flex_col()
                            .p_6()
                            .when(show_scan_bar, |this| {
                                this.child(ui::scan_progress_bar(
                                    &scan_phase,
                                    scan_items_found,
                                    scan_bytes_found,
                                    scan_current_path.as_deref(),
                                ))
                            })
                            .child(page_content),
                    )
                    .child(self.render_status_bar(cx)),
            )
    }
}

impl ClvApp {
    fn render_status_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let store = self.store.read(cx);
        let scanning = store.scanning;
        let cleaning = store.cleaning;
        let status = if cleaning {
            "正在清理选中项，请稍候…".into()
        } else if scanning {
            format!(
                "扫描中：{} · 已发现 {} 项（{}）",
                store.scan_phase,
                store.scan_items_found,
                format_bytes(store.scan_bytes_found)
            )
        } else if let Some(msg) = &store.status_message {
            msg.clone()
        } else if let Some(report) = &store.last_report {
            format!(
                "保护中 · 可释放 {} · {} 项待清理",
                report.total_reclaimable_human(),
                report.items.len()
            )
        } else {
            "实时防护已开启 — 点击首页「立即体检」".into()
        };

        h_flex()
            .w_full()
            .min_h(px(40.))
            .px_5()
            .py_2()
            .items_center()
            .gap_2()
            .border_t_1()
            .border_color(colors::panel_divider())
            .bg(colors::from_hex(0x0a1628).opacity(0.6))
            .when(scanning, |el| {
                el.child(
                    div()
                        .w(px(8.))
                        .h(px(8.))
                        .bg(colors::accent_cyan()),
                )
            })
            .child(
                div()
                    .text_sm()
                    .text_color(colors::text_muted())
                    .child(status),
            )
    }
}

#[allow(dead_code)]
pub fn save_settings_from_store(store: &AppStore) {
    if let Err(e) = save_settings(&store.settings) {
        tracing::error!("save settings: {e}");
    }
}
