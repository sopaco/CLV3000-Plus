# GPUI Component Examples

Copy-adapt patterns for agent-generated code. Verify imports against current docs.

## 1. Minimal app with theme background

```rust
use gpui::*;
use gpui_component::{button::*, *};

struct AppView;

impl Render for AppView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .child(Button::new("go").primary().label("Go"))
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(gpui_component_assets::Assets)
        .run(|cx| {
            gpui_component::init(cx);
            cx.spawn(async move |cx| {
                cx.open_window(WindowOptions::default(), |window, cx| {
                    let view = cx.new(|_| AppView);
                    cx.new(|cx| Root::new(view, window, cx).bg(cx.theme().background))
                })
                .unwrap();
            })
            .detach();
        });
}
```

## 2. View with local state + button click

```rust
struct Counter {
    count: i32,
}

impl Render for Counter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2()
            .child(format!("Count: {}", self.count))
            .child(
                Button::new("inc")
                    .label("+1")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.count += 1;
                        cx.notify();
                    })),
            )
    }
}
```

## 3. Text input (Entity pattern)

```rust
use gpui_component::input::{Input, InputState};

struct FormView {
    name: Entity<InputState>,
}

impl FormView {
    fn new(window: &Window, cx: &mut Context<Self>) -> Self {
        let name = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Your name")
        });
        Self { name }
    }
}

impl Render for FormView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2()
            .child("Name")
            .child(Input::new(&self.name))
    }
}
```

## 4. Select with search

```rust
use gpui_component::select::{Select, SelectState};

struct PickerView {
    select: Entity<SelectState<Vec<String>>>,
}

impl PickerView {
    fn new(window: &Window, cx: &mut Context<Self>) -> Self {
        let items = vec!["Rust".into(), "TypeScript".into(), "Go".into()];
        let select = cx.new(|cx| SelectState::new(items, window, cx));
        Self { select }
    }
}

impl Render for PickerView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Select::new(&self.select).searchable(true)
    }
}
```

## 5. DataTable with delegate

```rust
use gpui_component::table::{Column, DataTable, TableDelegate, TableState};

struct Row { id: u32, name: SharedString }

struct MyDelegate {
    rows: Vec<Row>,
    cols: Vec<Column>,
}

impl MyDelegate {
    fn new() -> Self {
        Self {
            rows: vec![
                Row { id: 1, name: "Alice".into() },
                Row { id: 2, name: "Bob".into() },
            ],
            cols: vec![
                Column::new("id", "ID").width(60.),
                Column::new("name", "Name").width(200.).sortable(),
            ],
        }
    }
}

impl TableDelegate for MyDelegate {
    fn columns_count(&self, _: &App) -> usize { self.cols.len() }
    fn rows_count(&self, _: &App) -> usize { self.rows.len() }
    fn column(&self, ix: usize, _: &App) -> Column { self.cols[ix].clone() }
    fn render_td(
        &mut self,
        row: usize,
        col: usize,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        match self.cols[col].key.as_ref() {
            "id" => self.rows[row].id.to_string(),
            "name" => self.rows[row].name.clone(),
            _ => String::new(),
        }
    }
}

// In view::new:
// let state = cx.new(|cx| TableState::new(MyDelegate::new(), window, cx));
// In render: DataTable::new(&state)
```

## 6. Dialog on button click

```rust
.child(
    Button::new("open")
        .label("Open Dialog")
        .on_click(|_, window, cx| {
            window.open_dialog(cx, |cx| {
                Dialog::new(cx)
                    .title("Hello")
                    .child("Dialog content")
                    .footer(|cx| {
                        Button::new("close")
                            .label("Close")
                            .on_click(|_, window, cx| window.close_dialog(cx))
                    })
            });
        }),
)
```

## 7. Toast notification

```rust
Button::new("save")
    .primary()
    .label("Save")
    .on_click(|_, window, cx| {
        window.push_notification(
            Notification::new()
                .title("Saved")
                .message("Changes saved successfully")
                .success(),
            cx,
        );
    })
```

## 8. Subscribe to entity events

```rust
struct ListenerView {
    table: Entity<TableState<MyDelegate>>,
    _subscriptions: Vec<Subscription>,
}

impl ListenerView {
    fn new(window: &Window, cx: &mut Context<Self>) -> Self {
        let table = cx.new(|cx| TableState::new(MyDelegate::new(), window, cx));
        let sub = cx.subscribe(&table, |_, _, event: &TableEvent, cx| {
            // handle selection, sort, etc.
            cx.notify();
        });
        Self {
            table,
            _subscriptions: vec![sub],
        }
    }
}
```

## 9. Actions and keyboard shortcuts

```rust
actions!(my_app, [SaveAction, UndoAction]);

impl Render for EditorView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context("Editor")
            .track_focus(&self.focus_handle(cx))
            .on_action(cx.listener(|this, _: &SaveAction, _, cx| {
                this.save(cx);
            }))
    }
}
```

## 10. Async work (background → UI update)

```rust
Button::new("load")
    .label("Load")
    .on_click(cx.listener(|this, _, window, cx| {
        cx.background_spawn(async move {
            let data = fetch_data().await;
            this.update(cx, |this, cx| {
                this.rows = data;
                cx.notify();
            }).log_err();
        })
        .detach();
    }))
```

## 11. gpui-base unstyled button (custom design system)

```rust
use gpui_base::button::Button as BaseButton;

// BaseButton::new("save") — no default padding/colors
// Caller supplies all visual children via div() styling
BaseButton::new("save")
    .on_click(|_, _, _| { /* ... */ })
    .child(
        div()
            .px_3()
            .py_1()
            .rounded_md()
            .bg(cx.theme().primary) // your tokens
            .child("Save")
    )
```

## 12. Layout helpers

```rust
// Vertical / horizontal flex shorthand
v_flex().gap_4().children(items)
h_flex().items_center().justify_between().child(left).child(right)

// Full-size centered content
div().size_full().flex().items_center().justify_center().child(content)

// Scrollable panel
div().size_full().overflow_y_scroll().child(long_content)
```

## 13. Code editor (feature flag required)

```toml
gpui-component = { git = "https://github.com/longbridge/gpui-component", features = ["tree-sitter-languages"] }
```

```rust
use gpui_component::input::{Input, InputState, InputMode};

let editor = cx.new(|cx| {
    InputState::new(window, cx)
        .mode(InputMode::code_editor("rust"))
        .default_value("fn main() {}\n")
});
```

## 14. Custom RenderOnce component

```rust
#[derive(IntoElement)]
struct StatusBadge {
    label: SharedString,
    variant: SharedString,
}

impl RenderOnce for StatusBadge {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .px_2()
            .py_0p5()
            .rounded_md()
            .bg(cx.theme().secondary)
            .child(self.label)
    }
}
```

## Debugging checklist

```text
[ ] gpui_component::init(cx) called
[ ] Root wraps top-level view
[ ] Entity created in new(), not render()
[ ] cx.spawn Task detached
[ ] Icons: with_assets(Assets) or custom SVGs
[ ] Feature flags for CodeEditor / webview
[ ] cargo check passes
```
