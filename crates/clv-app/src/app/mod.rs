pub mod shell;
pub mod state;

use crate::i18n::I18n;
use crate::prelude::*;
use crate::theme::colors;
use crate::views::{
    agent::AgentView, cleanup::CleanupView, dashboard::DashboardView, onboarding::OnboardingView,
    process::ProcessView, settings::SettingsView, startup::StartupView,
};
use clv_core::{load_settings, save_settings};
use state::{AppPage, AppStore};

pub struct ClvApp {
    store: Entity<AppStore>,
    _store_subscription: gpui::Subscription,
    dashboard: Option<Entity<DashboardView>>,
    cleanup: Option<Entity<CleanupView>>,
    agent: Option<Entity<AgentView>>,
    startup: Option<Entity<StartupView>>,
    process: Option<Entity<ProcessView>>,
    settings: Option<Entity<SettingsView>>,
    onboarding: Option<Entity<OnboardingView>>,
}

impl ClvApp {
    pub fn new(window: &Window, cx: &mut Context<Self>) -> Self {
        let settings = load_settings();
        let store = cx.new(|cx| AppStore::new(settings, cx));
        let store_subscription = cx.observe(&store, |_this, _store, cx| {
            cx.notify();
        });
        store.update(cx, |s, cx| s.refresh_disk_usage_async(cx));

        if !store.read(cx).settings.onboarding_done {
            store.update(cx, |s, cx| {
                s.page = AppPage::Onboarding;
                cx.notify();
            });
        }

        let _ = window;
        Self {
            store,
            _store_subscription: store_subscription,
            dashboard: None,
            cleanup: None,
            agent: None,
            startup: None,
            process: None,
            settings: None,
            onboarding: None,
        }
    }

    fn dashboard(&mut self, cx: &mut Context<Self>) -> Entity<DashboardView> {
        if let Some(view) = &self.dashboard {
            return view.clone();
        }
        let view = cx.new(|cx| DashboardView::new(self.store.clone(), cx));
        self.dashboard = Some(view.clone());
        view
    }

    fn cleanup(&mut self, cx: &mut Context<Self>) -> Entity<CleanupView> {
        if let Some(view) = &self.cleanup {
            return view.clone();
        }
        let view = cx.new(|cx| CleanupView::new(self.store.clone(), cx));
        self.cleanup = Some(view.clone());
        view
    }

    fn agent(&mut self, cx: &mut Context<Self>) -> Entity<AgentView> {
        if let Some(view) = &self.agent {
            return view.clone();
        }
        let view = cx.new(|cx| AgentView::new(self.store.clone(), cx));
        self.agent = Some(view.clone());
        view
    }

    fn startup(&mut self, cx: &mut Context<Self>) -> Entity<StartupView> {
        if let Some(view) = &self.startup {
            return view.clone();
        }
        let view = cx.new(|cx| StartupView::new(self.store.clone(), cx));
        self.startup = Some(view.clone());
        view
    }

    fn process(&mut self, cx: &mut Context<Self>) -> Entity<ProcessView> {
        if let Some(view) = &self.process {
            return view.clone();
        }
        let view = cx.new(|cx| ProcessView::new(self.store.clone(), cx));
        self.process = Some(view.clone());
        view
    }

    fn settings_view(&mut self, cx: &mut Context<Self>) -> Entity<SettingsView> {
        if let Some(view) = &self.settings {
            return view.clone();
        }
        let view = cx.new(|cx| SettingsView::new(self.store.clone(), cx));
        self.settings = Some(view.clone());
        view
    }

    fn onboarding(&mut self, cx: &mut Context<Self>) -> Entity<OnboardingView> {
        if let Some(view) = &self.onboarding {
            return view.clone();
        }
        let view = cx.new(|cx| OnboardingView::new(self.store.clone(), cx));
        self.onboarding = Some(view.clone());
        view
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

    pub fn i18n(&self, cx: &App) -> I18n {
        self.store.read(cx).i18n()
    }

    fn render_page(&mut self, page: AppPage, cx: &mut Context<Self>) -> impl IntoElement {
        match page {
            AppPage::Dashboard => self.dashboard(cx).into_any_element(),
            AppPage::Cleanup => self.cleanup(cx).into_any_element(),
            AppPage::Agent => self.agent(cx).into_any_element(),
            AppPage::Startup => self.startup(cx).into_any_element(),
            AppPage::Process => self.process(cx).into_any_element(),
            AppPage::Settings => self.settings_view(cx).into_any_element(),
            AppPage::Onboarding => self.onboarding(cx).into_any_element(),
        }
    }
}

impl Render for ClvApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = self.store.read(cx).page;
        let i18n = self.store.read(cx).i18n();

        if page == AppPage::Onboarding {
            return div()
                .size_full()
                .bg(ui::hero_gradient_alt())
                .child(self.onboarding(cx));
        }

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
                                i18n.nav_home(),
                                AppPage::Dashboard,
                                window,
                                cx,
                            ))
                            .child(self.nav_icon(
                                "nav-clean",
                                ui::nav_cleanup_icon(),
                                i18n.nav_cleanup(),
                                AppPage::Cleanup,
                                window,
                                cx,
                            ))
                            .child(self.nav_icon(
                                "nav-agent",
                                ui::NAV_AGENT,
                                i18n.nav_agent(),
                                AppPage::Agent,
                                window,
                                cx,
                            ))
                            .child(self.nav_icon(
                                "nav-startup",
                                ui::NAV_STARTUP,
                                i18n.nav_startup(),
                                AppPage::Startup,
                                window,
                                cx,
                            ))
                            .child(self.nav_icon(
                                "nav-process",
                                ui::NAV_PROCESS,
                                i18n.nav_process(),
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
                                i18n.nav_settings(),
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
                                    &i18n,
                                    &scan_phase,
                                    scan_items_found,
                                    scan_bytes_found,
                                    scan_current_path.as_deref(),
                                ))
                            })
                            .child(ui::page_transition(
                                page.transition_key(),
                                self.render_page(page, cx),
                            )),
                    )
                    .child(self.render_status_bar(cx)),
            )
    }
}

impl ClvApp {
    fn render_status_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let store = self.store.read(cx);
        let i18n = store.i18n();
        let scanning = store.scanning;
        let cleaning = store.cleaning;
        let status = if cleaning {
            i18n.status_cleaning().to_string()
        } else if scanning {
            i18n.status_scanning().to_string()
        } else if let Some(msg) = &store.status_message {
            msg.clone()
        } else if let Some(report) = &store.last_report {
            i18n.status_protected(&report.total_reclaimable_human(), report.items.len())
        } else {
            i18n.status_idle().to_string()
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
            .bg(colors::status_bar_bg().opacity(0.6))
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
