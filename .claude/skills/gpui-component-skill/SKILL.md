---
name: gpui-component-skill
description: >-
  Guides Rust desktop UI development with GPUI and longbridge/gpui-component.
  Use when building or modifying GPUI apps, gpui-component widgets, gpui-base
  primitives, docking layouts, DataTable, dialogs, theming, or migrating from
  web UI patterns to native GPUI.
version: 1.0.0
---

# GPUI Component Skill

Build cross-platform desktop UIs in Rust using **GPUI** (Zed's GPU renderer) and **[gpui-component](https://github.com/longbridge/gpui-component)** (60+ styled components). This skill encodes ecosystem knowledge so agents avoid common GPUI lifecycle and state mistakes.

## When to load

Load this skill when the task involves any of:

- `gpui`, `gpui_platform`, `gpui-component`, `gpui-base`, `gpui-component-assets`
- GPUI `Render` / `RenderOnce` / `Entity` / `Context` patterns
- Desktop UI: buttons, forms, tables, docking, charts, code editor, notifications
- Choosing between **styled components** vs **unstyled primitives**

## Ecosystem map (read first)

```text
Application
    ├─ gpui-component   → styled, ship-ready (Button, DataTable, DockArea, Theme)
    └─ gpui-base        → unstyled behavior (focus, popups, VirtualList, InputBase)
           └─ GPUI (zed-industries/zed) → Entity system, div(), layout, actions
```

| Layer | Crate | Choose when |
|-------|-------|-------------|
| Framework | `gpui` + `gpui_platform` | Always (window, entity, layout) |
| Styled UI | `gpui-component` | Default — product UI with themes |
| Behavior only | `gpui-base` | Custom design system; own all visuals |
| Icons/assets | `gpui-component-assets` | Optional bundled Lucide SVGs |

**Thesis**: behavior belongs to foundation (`gpui-base`); presentation belongs to app or `gpui-component`.

Official docs: https://longbridge.github.io/gpui-component/docs/  
GPUI framework: https://gpui.rs  
Deep architecture: https://github.com/longbridge/gpui-component/blob/main/docs/ARCHITECTURE.md

## Mandatory application skeleton

Every GPUI Component app **must** follow this sequence:

```rust
use gpui::*;
use gpui_component::{button::*, *};

fn main() {
    let app = gpui_platform::application()
        .with_assets(gpui_component_assets::Assets); // optional but typical

    app.run(move |cx| {
        gpui_component::init(cx); // FIRST line — themes, dock, input, etc.

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|_| MyView::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx)) // window root MUST be Root
            })
            .expect("open window");
        })
        .detach(); // Task must be detached or stored
    });
}
```

**Non-negotiable rules**

1. Call `gpui_component::init(cx)` once inside `app.run` before any component API.
2. First-level window child **must** be `Root::new(view, window, cx)` — enables dialogs, sheets, notifications, focus layers.
3. Open windows inside `cx.spawn` + `.detach()` (or store the `Task`).
4. Do **not** call `gpui_base::init` if you already call `gpui_component::init` (base init is included).

### Cargo.toml baseline

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed" }
gpui_platform = { git = "https://github.com/zed-industries/zed", features = ["font-kit"] }
gpui-component = { git = "https://github.com/longbridge/gpui-component" }
gpui-component-assets = { git = "https://github.com/longbridge/gpui-component" } # optional
anyhow = "1"
```

Pin to a release tag when stability matters: `gpui-component = { git = "...", tag = "v0.5.1" }`.

## Agent decision tree

```
Need UI?
├─ Ship product fast, accept built-in theme → gpui-component
├─ Own every pixel / design system → gpui-base (+ raw gpui div styling)
└─ Mix: gpui-component for chrome, gpui-base primitives for custom parts

Component type?
├─ Stateless (Button, Tag, Dialog declarative parts) → RenderOnce, builder chain
└─ Stateful (Input, Select, DataTable, DockArea) → Entity<State> in view struct

Overlay?
├─ Modal → window.open_dialog(...) or Dialog builder
├─ Toast → window.push_notification(Notification::...)
└─ Popover/menu → Popover / PopupMenu / window.open_context_menu
```

## GPUI core concepts (agent essentials)

| Concept | Role | Agent note |
|---------|------|------------|
| `App` / `Context<T>` | Global + per-entity context | Use closure's inner `cx`, not outer |
| `Entity<T>` | Strong handle to state | Create with `cx.new`, hold in view, `.clone()` in render |
| `Render` | Stateful view (`&mut self`) | Main app views implement this |
| `RenderOnce` | Stateless element (`self` by value) | Most gpui-component widgets |
| `IntoElement` | Converts to render tree | `render` return type |
| `div()` | Layout + Tailwind-like styling | `.v_flex()`, `.gap_2()`, `.child()` |
| `Action` | Keyboard/command dispatch | `.on_action(cx.listener(...))` |
| `cx.subscribe` / `cx.observe` | React to entity events | Store in `Vec<Subscription>` or `.detach()` |
| `cx.spawn` | Async on UI thread | Always `.detach()` or store `Task` |
| `cx.background_spawn` | CPU/IO work off UI thread | Return to UI via `cx.update` |

### GPUI pitfalls (avoid these)

- **Nested entity update** while already in `update` → panic.
- **Dropped `Task`** from `cx.spawn` → work cancelled silently.
- **Wrong `cx` in closures** → borrow errors or stale state.
- **Stateful component without `Entity`** → state resets every frame.
- **Missing `Root`** → dialogs/notifications/sheets break.
- **Icons without assets** → empty icon slots; wire `gpui-component-assets` or custom SVG paths.
- **Silent error discard** — propagate with `?` or log; Zed/GPUI style discourages `let _ =`.

## Component patterns in gpui-component

### Pattern A — Stateless (`RenderOnce`)

View owns all state; components are pure builders.

```rust
impl Render for MyView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2()
            .child(Button::new("save").primary().label("Save"))
            .child(Tag::secondary().child(format!("Count: {}", self.count)))
    }
}
```

### Pattern B — Stateful (`Entity<State>`)

Input, Select, DataTable, DockArea, List use internal state engines.

```rust
struct MyView {
    input: Entity<InputState>,
}

impl MyView {
    fn new(window: &Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).default_value("Hello"));
        Self { input }
    }
}

impl Render for MyView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Input::new(&self.input) // or self.input.clone() depending on API
    }
}
```

**Rule**: create `Entity` in `new` / `cx.new` callback, not inside `render`.

### Pattern C — Delegate-driven data (DataTable, List)

Implement a `*Delegate` trait; hold `Entity<TableState<D>>` in the view. See [examples.md](examples.md).

### Pattern D — Dock / IDE layout

`DockArea` + panels implementing `Panel` trait + `PanelRegistry` for serialization. See [reference.md](reference.md#dock-and-panels).

## Theming and sizing

```rust
use gpui_component::ActiveTheme;

// In render:
cx.theme().background
cx.theme().primary
cx.theme().foreground

// Sizes (Sizable trait): .xsmall() .small() .medium() .large()
// Variants: .primary() .danger() .ghost() .outline()
```

Switch light/dark via theme APIs on `Theme` global (initialized by `gpui_component::init`).

## Icons and assets

Icons are **not** bundled in the core crate.

```rust
// App entry:
gpui_platform::application().with_assets(gpui_component_assets::Assets)

// Usage:
Icon::new(IconName::Search).small()
```

Custom icons: name SVG files to match `IconName` enum variants.

## Feature flags

Enable on `gpui-component` dependency:

| Feature | Use |
|---------|-----|
| `tree-sitter-languages` | Syntax highlighting in CodeEditor / Markdown (30+ langs) |
| `tree-sitter` | Core tree-sitter only |
| `inspector` | Runtime UI inspector (debug) |
| `decimal` | `rust_decimal` in numeric inputs |
| `webview` | Embedded web view (Wry) |

Example: `gpui-component = { git = "...", features = ["tree-sitter-languages"] }`

## Agent workflow

When implementing a GPUI feature:

1. **Classify** — styled component vs custom primitive vs raw GPUI.
2. **Scaffold** — verify `init`, `Root`, `Cargo.toml` deps.
3. **Find reference** — check official docs component page OR repo `examples/` OR `crates/story`.
4. **Match pattern** — stateless builder vs `Entity` vs delegate.
5. **Wire events** — `on_click`, `cx.subscribe`, `TableEvent`, `SelectEvent`, actions.
6. **Validate** — `cargo check`; run `cargo run --example <name>` if available.

### Local exploration (clone repo)

```bash
git clone https://github.com/longbridge/gpui-component.git
cd gpui-component
cargo run              # component gallery (story)
cargo run --example hello_world
cargo run --example input
```

Use story crate (`crates/story`) as ground truth for component usage.

## Subsystem quick reference

| Need | Module | Key types |
|------|--------|-----------|
| Button | `gpui_component::button` | `Button`, `ButtonGroup` |
| Text input | `gpui_component::input` | `Input`, `InputState`, `InputMode` |
| Code editor | `input` + `InputMode::CodeEditor` | LSP, tree-sitter highlighting |
| Select | `gpui_component::select` | `Select`, `SelectState`, `SelectItem` |
| Combobox | `gpui_component::combobox` | `Combobox`, `ComboboxState` |
| Simple table | `gpui_component::table` | `Table`, `TableRow`, `TableCell` |
| Large table | `gpui_component::table` | `DataTable`, `TableState`, `TableDelegate` |
| List | `gpui_component::list` | virtualized list + `ListState` |
| Dialog | `gpui_component::dialog` | `Dialog`, `window.open_dialog` |
| Notification | `gpui_component::notification` | `Notification`, `push_notification` |
| Dock IDE layout | `gpui_component::dock` | `DockArea`, `Panel`, `PanelRegistry` |
| Chart | `gpui_component::chart` | line/bar/area charts |
| Markdown | `gpui_component::markdown` | `Markdown`, `MarkdownElement` |
| Settings UI | `gpui_component::setting` | settings page scaffolding |
| Unstyled primitive | `gpui_base::...` | see https://longbridge.github.io/gpui-component/base/ |

Full import table: [reference.md](reference.md)

## gpui-base vs gpui-component

Use **gpui-base** when:

- Building a custom design system (shadcn/Base UI analogy)
- Need behavior (focus trap, popup positioning, virtualization) without default styles
- Want controlled-value pattern with your own `div()` styling

Use **gpui-component** when:

- Shipping a desktop app with consistent modern theme
- Need DataTable, DockArea, CodeEditor, charts out of the box

`gpui-component` re-exports much of `gpui-base`; prefer `gpui_component::` paths unless intentionally bypassing styles.

## Documentation priority

When answering API questions, consult in order:

1. Official component docs — https://longbridge.github.io/gpui-component/docs/components/
2. Repo `examples/` and `crates/story/src/stories/`
3. `docs/ARCHITECTURE.md` and gpui-base docs — https://longbridge.github.io/gpui-component/base/
4. docs.rs — https://docs.rs/gpui-component
5. GPUI glossary in zed repo — `crates/gpui/README.md`

Do not invent component APIs; verify against docs or story examples.

## Additional resources

- [reference.md](reference.md) — crate layout, module catalog, dock/table/dialog APIs
- [examples.md](examples.md) — copy-paste patterns for common tasks
