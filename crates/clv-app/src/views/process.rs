use crate::app::state::AppStore;
use crate::prelude::*;
use crate::theme::colors;
use clv_core::format_bytes;
use clv_platform::{kill_process, ProcessSort};

pub struct ProcessView {
    store: Entity<AppStore>,
}

impl ProcessView {
    pub fn new(store: Entity<AppStore>, _cx: &mut Context<Self>) -> Self {
        Self { store }
    }
}

impl Render for ProcessView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (sort, top) = {
            let store = self.store.read(cx);
            let sort = store.process_sort;
            let top = store.processes.iter().take(50).cloned().collect::<Vec<_>>();
            (sort, top)
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .min_h_0()
            .child(
                div()
                    .p_6()
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
                                    .child(sort_button(
                                        "sort-mem",
                                        "按内存",
                                        ProcessSort::Memory,
                                        sort,
                                        self.store.clone(),
                                        cx,
                                    ))
                                    .child(sort_button(
                                        "sort-cpu",
                                        "按 CPU",
                                        ProcessSort::Cpu,
                                        sort,
                                        self.store.clone(),
                                        cx,
                                    ))
                                    .child(sort_button(
                                        "sort-name",
                                        "按名称",
                                        ProcessSort::Name,
                                        sort,
                                        self.store.clone(),
                                        cx,
                                    ))
                                    .child(
                                        ui::action_button("proc-refresh", "刷新", None, false, cx)
                                            .on_click({
                                                let store = self.store.clone();
                                                cx.listener(move |_, _, _, cx| {
                                                    store.update(cx, |s, cx| {
                                                        s.refresh_processes(cx);
                                                    });
                                                    cx.notify();
                                                })
                                            }),
                                    ),
                            ),
                    ),
            )
            .child(ui::scroll_y(
                div()
                    .px_6()
                    .pb_6()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        ui::card()
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
                    )
                    .children(top.iter().map(|proc| {
                        let pid = proc.pid;
                        let name = proc.name.clone();
                        let mem = format_bytes(proc.memory_bytes);
                        let cpu = format!("{:.1}%", proc.cpu_percent);
                        let cat = proc.category.label();
                        let kill_id = eid(format!("kill-{pid}"));
                        let store = self.store.clone();

                        ui::card()
                            .px_4()
                            .py_3()
                            .child(
                                h_flex()
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
                                        ui::std_button(
                                            Button::new(kill_id)
                                                .danger()
                                                .label("结束"),
                                        )
                                            .on_click(move |_, window, cx| {
                                                let pid = pid;
                                                let store = store.clone();
                                                window.open_dialog(cx, move |dialog, _window, _cx| {
                                                    dialog
                                                        .title("结束进程")
                                                        .child(format!("确定结束 PID {pid}？"))
                                                        .confirm()
                                                        .on_ok({
                                                            let store = store.clone();
                                                            move |_, window, cx| {
                                                                kill_process(pid).ok();
                                                                store.update(cx, |s, cx| {
                                                                    s.refresh_processes(cx);
                                                                });
                                                                window.close_dialog(cx);
                                                                true
                                                            }
                                                        })
                                                });
                                            }),
                                    ),
                            )
                    })),
            ))
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
        .child(text)
}

fn sort_button(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    sort: ProcessSort,
    current: ProcessSort,
    store: Entity<AppStore>,
    cx: &mut Context<ProcessView>,
) -> Button {
    let active = current == sort;
    let id: SharedString = id.into();
    let label: SharedString = label.into();
    ui::ghost_pill(id, label, active, cx).on_click(cx.listener(move |_, _, _, cx| {
        store.update(cx, |s, cx| {
            s.set_process_sort(sort, cx);
        });
        cx.notify();
    }))
}
