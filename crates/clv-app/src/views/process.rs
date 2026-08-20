use crate::prelude::*;
use crate::theme::colors;
use clv_core::format_bytes;
use clv_platform::{kill_process, list_processes, ProcessInfo, ProcessSort};
use gpui::{size, Pixels, Size};
use gpui_component::v_virtual_list;
use std::ops::Range;
use std::rc::Rc;
use std::time::Duration;

const PROCESS_ROW_H: f32 = 52.;
const MAX_PROCESSES: usize = 50;

pub struct ProcessView {
    processes: Vec<ProcessInfo>,
    sort: ProcessSort,
    row_sizes: Rc<Vec<Size<Pixels>>>,
    poll_generation: u64,
    visible: bool,
}

impl ProcessView {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            processes: Vec::new(),
            sort: ProcessSort::Memory,
            row_sizes: Rc::new(Vec::new()),
            poll_generation: 0,
            visible: false,
        }
    }

    pub fn on_show(&mut self, cx: &mut Context<Self>) {
        if self.visible {
            return;
        }
        self.visible = true;
        self.poll_generation = self.poll_generation.wrapping_add(1);
        self.refresh_now(cx);
        self.spawn_poll_loop(cx);
    }

    pub fn on_hide(&mut self, cx: &mut Context<Self>) {
        if !self.visible {
            return;
        }
        self.visible = false;
        self.poll_generation = self.poll_generation.wrapping_add(1);
        cx.notify();
    }

    fn refresh_now(&mut self, cx: &mut Context<Self>) {
        let poll_gen = self.poll_generation;
        let sort = self.sort;
        cx.spawn(async move |weak, cx| {
            let processes = list_processes(sort);
            weak.update(cx, |view, cx| {
                if view.poll_generation != poll_gen || !view.visible {
                    return;
                }
                view.processes = processes.into_iter().take(MAX_PROCESSES).collect();
                view.sync_row_sizes();
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn spawn_poll_loop(&mut self, cx: &mut Context<Self>) {
        let poll_gen = self.poll_generation;
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
                let processes = list_processes(sort);

                let still_valid = weak
                    .update(cx, |view, cx| {
                        if !view.visible || view.poll_generation != poll_gen {
                            false
                        } else {
                            view.processes = processes.into_iter().take(MAX_PROCESSES).collect();
                            view.sync_row_sizes();
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
        self.refresh_now(cx);
        cx.notify();
    }

    fn sync_row_sizes(&mut self) {
        let count = self.processes.len();
        self.row_sizes = Rc::new(vec![size(px(0.), px(PROCESS_ROW_H)); count]);
    }

    fn render_row(
        &self,
        ix: usize,
        view: Entity<Self>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Div {
        let Some(proc) = self.processes.get(ix) else {
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
                    .justify_between()
                    .items_center()
                    .child(
                        h_flex()
                            .gap_4()
                            .items_center()
                            .child(cell(pid.to_string(), px(60.), colors::text_muted()))
                            .child(cell(name, px(200.), colors::text_primary()))
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
                                                kill_process(pid).ok();
                                                view.update(cx, |v, cx| v.refresh_now(cx));
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
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sort = self.sort;
        let row_sizes = self.row_sizes.clone();
        let view = cx.entity();

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
                                "显示前 50 个进程 · 结束进程前请确认没有未保存工作",
                            ))
                            .child(
                                h_flex()
                                    .gap_2()
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
                                    .gap_4()
                                    .child(header_cell("PID", px(60.)))
                                    .child(header_cell("名称", px(200.)))
                                    .child(header_cell("CPU", px(80.)))
                                    .child(header_cell("内存", px(100.)))
                                    .child(header_cell("类别", px(80.)))
                                    .child(div().flex_grow()),
                            ),
                    ),
            )
            .child(
                div()
                    .id("process-list-pane")
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .px_6()
                    .pb_6()
                    .when(self.processes.is_empty(), |this| {
                        this.flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .text_base()
                                    .text_color(colors::text_muted())
                                    .child("正在加载进程列表…"),
                            )
                    })
                    .when(!self.processes.is_empty(), |this| {
                        this.child(
                            v_virtual_list(
                                view.clone(),
                                "process-list",
                                row_sizes,
                                move |this, visible_range: Range<usize>, window, cx| {
                                    visible_range
                                        .map(|ix| this.render_row(ix, view.clone(), window, cx))
                                        .collect::<Vec<_>>()
                                },
                            )
                            .size_full()
                            .overflow_hidden(),
                        )
                    }),
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
