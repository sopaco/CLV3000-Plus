use crate::app::state::{AppPage, AppStore};
use crate::prelude::*;
use crate::theme::colors;
use clv_core::format_bytes;
use clv_platform::{kill_process, ProcessEnumerator, ProcessInfo, ProcessSort};
use gpui::{ScrollStrategy, Subscription, UniformListScrollHandle};
use gpui_component::input::{Input, InputEvent, InputState};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const PROCESS_ROW_H: f32 = 52.;
const DEFAULT_MAX_PROCESSES: usize = 50;
const SEARCH_MAX_PROCESSES: usize = 200;

pub struct ProcessView {
    store: Entity<AppStore>,
    #[allow(dead_code)]
    _store_subscription: Subscription,
    all_processes: Vec<ProcessInfo>,
    search_query: String,
    search_input: Option<Entity<InputState>>,
    _search_subscription: Option<Subscription>,
    sort: ProcessSort,
    scroll_handle: UniformListScrollHandle,
    poll_generation: u64,
    visible: bool,
    enumerator: Option<Arc<Mutex<ProcessEnumerator>>>,
}

impl ProcessView {
    pub fn new(store: Entity<AppStore>, cx: &mut Context<Self>) -> Self {
        let store_subscription = cx.observe(&store, |view, store, cx| {
            let active = store.read(cx).page == AppPage::Process;
            if active && !view.visible {
                view.on_show(cx);
            } else if !active && view.visible {
                view.on_hide(cx);
            }
        });

        Self {
            store,
            _store_subscription: store_subscription,
            all_processes: Vec::new(),
            search_query: String::new(),
            search_input: None,
            _search_subscription: None,
            sort: ProcessSort::Memory,
            scroll_handle: UniformListScrollHandle::new(),
            poll_generation: 0,
            visible: false,
            enumerator: None,
        }
    }

    fn processes_from_enumerator(
        enumerator: &Arc<Mutex<ProcessEnumerator>>,
        sort: ProcessSort,
    ) -> Vec<ProcessInfo> {
        enumerator
            .lock()
            .expect("process enumerator lock")
            .list(sort)
    }

    fn filtered_processes(&self) -> Vec<ProcessInfo> {
        let query = self.search_query.trim().to_lowercase();
        if query.is_empty() {
            return self
                .all_processes
                .iter()
                .take(DEFAULT_MAX_PROCESSES)
                .cloned()
                .collect();
        }

        self.all_processes
            .iter()
            .filter(|proc| {
                proc.name.to_lowercase().contains(&query)
                    || proc.pid.to_string().contains(&query)
                    || proc.category.label().contains(&query)
            })
            .take(SEARCH_MAX_PROCESSES)
            .cloned()
            .collect()
    }

    fn ensure_search_input(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<InputState> {
        if let Some(input) = &self.search_input {
            return input.clone();
        }

        let input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("搜索名称、PID 或类别…")
        });
        let subscription = cx.subscribe(&input, |view, input, event, cx| {
            if matches!(event, InputEvent::Change) {
                view.search_query = input.read(cx).value().to_string();
                view.scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
                cx.notify();
            }
        });
        self._search_subscription = Some(subscription);
        self.search_input = Some(input.clone());
        input
    }

    pub fn on_show(&mut self, cx: &mut Context<Self>) {
        if self.visible {
            return;
        }
        self.visible = true;
        self.poll_generation = self.poll_generation.wrapping_add(1);
        let enumerator = Arc::new(Mutex::new(ProcessEnumerator::new()));
        self.all_processes = Self::processes_from_enumerator(&enumerator, self.sort);
        self.enumerator = Some(enumerator);
        cx.notify();
        self.spawn_poll_loop(cx);
    }

    pub fn on_hide(&mut self, cx: &mut Context<Self>) {
        if !self.visible {
            return;
        }
        self.visible = false;
        self.poll_generation = self.poll_generation.wrapping_add(1);
        self.enumerator = None;
        cx.notify();
    }

    fn refresh_now(&mut self, cx: &mut Context<Self>) {
        let Some(enumerator) = self.enumerator.clone() else {
            return;
        };
        let poll_gen = self.poll_generation;
        let sort = self.sort;
        cx.spawn(async move |weak, cx| {
            let processes = std::thread::spawn(move || {
                Self::processes_from_enumerator(&enumerator, sort)
            })
            .join()
            .unwrap_or_default();
            weak.update(cx, |view, cx| {
                if view.poll_generation != poll_gen || !view.visible {
                    return;
                }
                view.all_processes = processes;
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn spawn_poll_loop(&mut self, cx: &mut Context<Self>) {
        let poll_gen = self.poll_generation;
        let Some(enumerator) = self.enumerator.clone() else {
            return;
        };
        cx.spawn(async move |weak, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_secs(1))
                    .await;

                let should_continue = weak
                    .read_with(cx, |view, _| view.visible && view.poll_generation == poll_gen)
                    .unwrap_or(false);
                if !should_continue {
                    break;
                }

                let sort = weak
                    .read_with(cx, |view, _| view.sort)
                    .unwrap_or(ProcessSort::Memory);
                let enumerator = enumerator.clone();
                let processes = std::thread::spawn(move || {
                    Self::processes_from_enumerator(&enumerator, sort)
                })
                .join()
                .unwrap_or_default();

                let still_valid = weak
                    .update(cx, |view, cx| {
                        if !view.visible || view.poll_generation != poll_gen {
                            false
                        } else {
                            view.all_processes = processes;
                            cx.notify();
                            true
                        }
                    })
                    .unwrap_or(false);
                if !still_valid {
                    break;
                }
            }
        })
        .detach();
    }

    fn set_sort(&mut self, sort: ProcessSort, cx: &mut Context<Self>) {
        if self.sort == sort {
            return;
        }
        self.sort = sort;
        self.scroll_handle.scroll_to_item(0, ScrollStrategy::Top);
        self.refresh_now(cx);
        cx.notify();
    }

    fn render_row(
        &self,
        ix: usize,
        processes: &[ProcessInfo],
        view: Entity<Self>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Div {
        let Some(proc) = processes.get(ix) else {
            return div().h(px(PROCESS_ROW_H));
        };

        let pid = proc.pid;
        let name = proc.name.clone();
        let mem = format_bytes(proc.memory_bytes);
        let cpu = format!("{:.1}%", proc.cpu_percent);
        let cat = proc.category.label();
        let kill_id = eid(format!("kill-{pid}"));

        ui::soft_card()
            .h(px(PROCESS_ROW_H))
            .px_4()
            .child(
                h_flex()
                    .size_full()
                    .items_center()
                    .gap_4()
                    .child(
                        h_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_4()
                            .items_center()
                            .child(cell(pid.to_string(), px(60.), colors::text_muted()))
                            .child(name_cell(name, colors::text_primary()))
                            .child(cell(cpu, px(80.), colors::accent_blue()))
                            .child(cell(mem, px(100.), colors::text_secondary()))
                            .child(cell(cat.to_string(), px(80.), colors::text_muted())),
                    )
                    .child(
                        ui::std_button(Button::new(kill_id).danger().label("结束")).on_click(
                            move |_, window, cx| {
                                let view = view.clone();
                                window.open_dialog(cx, move |dialog, _window, _cx| {
                                    dialog
                                        .title("结束进程")
                                        .child(format!("确定结束 PID {pid}？"))
                                        .confirm()
                                        .on_ok({
                                            let view = view.clone();
                                            move |_, window, cx| {
                                                if let Err(e) = kill_process(pid) {
                                                    window.open_dialog(cx, move |dialog, _, _| {
                                                        dialog
                                                            .title("结束进程失败")
                                                            .child(e.to_string())
                                                            .confirm()
                                                    });
                                                } else {
                                                    view.update(cx, |v, cx| v.refresh_now(cx));
                                                }
                                                window.close_dialog(cx);
                                                true
                                            }
                                        })
                                });
                            },
                        ),
                    ),
            )
    }
}

impl Render for ProcessView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.store.read(cx).page == AppPage::Process && !self.visible {
            self.on_show(cx);
        }

        let search_input = self.ensure_search_input(window, cx);
        let sort = self.sort;
        let processes = self.filtered_processes();
        let count = processes.len();
        let total = self.all_processes.len();
        let scroll_handle = self.scroll_handle.clone();
        let searching = !self.search_query.trim().is_empty();
        let _ = &self.store;

        div()
            .id("process-view")
            .size_full()
            .flex()
            .flex_col()
            .min_h_0()
            .overflow_hidden()
            .child(
                div()
                    .flex_shrink_0()
                    .px_6()
                    .pt_6()
                    .pb_4()
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(ui::page_header(
                                "进程管理",
                                if searching {
                                    format!("搜索到 {count} / {total} 个进程")
                                } else {
                                    format!("显示前 {DEFAULT_MAX_PROCESSES} 个进程 · 共 {total} 个")
                                },
                            ))
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        div()
                                            .w(px(240.))
                                            .child(Input::new(&search_input)),
                                    )
                                    .child(sort_button("sort-mem", "按内存", ProcessSort::Memory, sort, cx))
                                    .child(sort_button("sort-cpu", "按 CPU", ProcessSort::Cpu, sort, cx))
                                    .child(sort_button(
                                        "sort-name",
                                        "按名称",
                                        ProcessSort::Name,
                                        sort,
                                        cx,
                                    ))
                                    .child(
                                        ui::action_button("proc-refresh", "刷新", None, false, cx)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.refresh_now(cx);
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    ),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .px_6()
                    .pb_2()
                    .child(
                        ui::soft_card()
                            .px_4()
                            .py_2()
                            .child(
                                h_flex()
                                    .w_full()
                                    .gap_4()
                                    .child(header_cell("PID", px(60.)))
                                    .child(header_name_cell("名称"))
                                    .child(header_cell("CPU", px(80.)))
                                    .child(header_cell("内存", px(100.)))
                                    .child(header_cell("类别", px(80.)))
                                    .child(div().w(px(96.))),
                            ),
                    ),
            )
            .child(
                ui::list_body(
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .px_6()
                        .pb_6()
                        .when(count == 0, |this| {
                            this.flex().items_center().justify_center().child({
                                let message: SharedString = if searching {
                                    "没有匹配的进程".into()
                                } else if total == 0 {
                                    "正在加载进程列表…".into()
                                } else {
                                    "没有可显示的进程".into()
                                };
                                div()
                                    .text_base()
                                    .text_color(colors::text_muted())
                                    .child(message)
                            })
                        })
                        .when(count > 0, |this| {
                            this.child(ui::uniform_list_pane(
                                "process-rows",
                                count,
                                scroll_handle,
                                cx,
                                move |this, visible_range, window, cx| {
                                    let view = cx.entity();
                                    let processes = this.filtered_processes();
                                    visible_range
                                        .map(|ix| {
                                            this.render_row(ix, &processes, view.clone(), window, cx)
                                        })
                                        .collect()
                                },
                            ))
                        }),
                ),
            )
    }
}

fn header_cell(text: &str, width: gpui::Pixels) -> Div {
    div()
        .w(width)
        .text_sm()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(colors::text_muted())
        .child(text.to_string())
}

fn header_name_cell(text: &str) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .text_sm()
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(colors::text_muted())
        .child(text.to_string())
}

fn name_cell(text: String, color: gpui::Hsla) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .text_base()
        .text_color(color)
        .truncate()
        .child(text)
}

fn cell(text: String, width: gpui::Pixels, color: gpui::Hsla) -> Div {
    div()
        .w(width)
        .text_base()
        .text_color(color)
        .truncate()
        .child(text)
}

fn sort_button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    sort: ProcessSort,
    current: ProcessSort,
    cx: &mut Context<ProcessView>,
) -> Button {
    let active = current == sort;
    let id: SharedString = id.into();
    let label: SharedString = label.into();
    ui::ghost_pill(id, label, active, cx).on_click(cx.listener(move |this, _, _, cx| {
        this.set_sort(sort, cx);
    }))
}
