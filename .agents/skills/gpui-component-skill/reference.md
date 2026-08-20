# GPUI Component Reference

Authoritative external sources override this file when they diverge.

## Workspace crates

| Crate | Path | Purpose |
|-------|------|---------|
| `gpui-base` | `crates/base` | Unstyled behavior, state engines, virtualization |
| `gpui-component` | `crates/ui` | Styled components (main dependency) |
| `gpui-component-macros` | `crates/macros` | `icon_named` and proc macros |
| `gpui-component-assets` | `crates/assets` | Bundled SVG icons |
| `gpui-component-story` | `crates/story` | Interactive gallery + dev harness |
| `story-web` | `crates/story-web` | WASM gallery build |

## gpui-base module families

| Family | Examples | Interface |
|--------|----------|-----------|
| Semantic elements | Button, Checkbox, Switch, Tabs | `IntoElement` + `Styled` + `InteractiveElement` |
| Compound roots | Dialog, Popover, Select, Sheet | Multi-part; open state, focus trap, placement |
| Stateful systems | `InputState`, `TreeState`, `ToastManager` | `Entity` or keyed state |
| Infrastructure | `VirtualList`, `FocusTrapElement`, `Positioner` | Shared algorithms |

Init: `gpui_base::init(cx)` only when **not** using `gpui_component::init`.

## Component module catalog

### Input & controls

| Component | Import path |
|-----------|-------------|
| Button | `gpui_component::button::{Button, ButtonGroup}` |
| Input | `gpui_component::input::{Input, InputState, InputMode}` |
| Textarea | `gpui_component::input` |
| Select | `gpui_component::select::{Select, SelectState, SelectItem, SelectDelegate}` |
| Combobox | `gpui_component::combobox::{Combobox, ComboboxState}` |
| Checkbox / Radio / Switch | `gpui_component::checkbox`, `radio`, `switch` |
| Slider | `gpui_component::slider` |
| Number input | `gpui_component::number_input` |
| OTP | `gpui_component::otp_input` |
| Date picker | `gpui_component::date_picker` |
| Color picker | `gpui_component::color_picker` |
| Form layout | `gpui_component::form` |

### Data display

| Component | Import path |
|-----------|-------------|
| List (virtualized) | `gpui_component::list` |
| Table (static) | `gpui_component::table::{Table, TableRow, TableCell, ...}` |
| DataTable | `gpui_component::table::{DataTable, TableState, TableDelegate, Column}` |
| Tree | `gpui_component::tree` |
| Description list | `gpui_component::description_list` |
| Tag / Badge | `gpui_component::tag` |
| Avatar | `gpui_component::avatar` |

### Layout & navigation

| Component | Import path |
|-----------|-------------|
| Root (required) | `gpui_component::Root` |
| Dock | `gpui_component::dock::{DockArea, DockItem, Panel, PanelRegistry}` |
| Resizable | `gpui_component::resizable` |
| Sidebar | `gpui_component::sidebar` |
| Tabs | `gpui_component::tab` |
| Breadcrumb | `gpui_component::breadcrumb` |
| Pagination | `gpui_component::pagination` |
| Accordion / Collapsible | `gpui_component::accordion`, `collapsible` |
| Title bar / menu bar | `gpui_component::title_bar`, `app_menu_bar` |

### Overlay & feedback

| Component | Import path |
|-----------|-------------|
| Dialog | `gpui_component::dialog::{Dialog, DialogHeader, DialogTitle, ...}` |
| Sheet | `gpui_component::sheet` |
| Popover | `gpui_component::popover` |
| Popup menu | `gpui_component::popup_menu` |
| Context menu | via window APIs + popup_menu |
| Notification | `gpui_component::notification::Notification` |
| Alert | `gpui_component::alert` |
| Tooltip / HoverCard | `gpui_component::tooltip`, `hover_card` |
| Progress / Spinner / Skeleton | `gpui_component::progress`, `spinner`, `skeleton` |

### Content & visualization

| Component | Import path |
|-----------|-------------|
| TextView | `gpui_component::text_view` |
| Markdown | `gpui_component::markdown` |
| Chart | `gpui_component::chart` |
| Plot primitives | `gpui_component::plot` |
| Icon | `gpui_component::{Icon, IconName}` |

### Settings & debug

| Component | Import path |
|-----------|-------------|
| Settings page | `gpui_component::setting` |
| Inspector | `gpui_component::inspector` (feature `inspector`) |
| Kbd display | `gpui_component::kbd` |

## Table vs DataTable

| | `Table` | `DataTable<D>` |
|---|---------|----------------|
| State | Stateless `RenderOnce` | `Entity<TableState<D>>` |
| Data size | Small/static | Large (virtual scroll) |
| Sorting/selection | Manual | Built-in |
| Integration | Compose rows/cells | Implement `TableDelegate` |

`TableDelegate` required methods: `columns_count`, `rows_count`, `column`, `render_td`. Optional: sorting, context menus, infinite scroll hooks.

## Dock and panels

Key types:

- `DockArea` — root dock container (`Entity<DockAreaState>`)
- `DockItem` — `Split`, `Tabs`, `Panel`, `Tiles`
- `Panel` trait — `panel_id`, `title`, `render`, drag/drop hooks
- `PanelRegistry` — register panel types for layout restore/serialization
- `TabPanel`, `StackPanel` — tabbed/stacked panel groups

Typical flow:

1. Create `PanelRegistry`, register panel factories.
2. Create `Entity<DockAreaState>` with initial layout.
3. Render `DockArea::new(&state)` in main view.
4. Persist layout via serialization APIs on dock state.

See `examples/` and story `dock` stories for working layouts.

## Dialog and notification APIs

```rust
// Imperative dialog (on Window)
window.open_dialog(cx, |cx| {
    Dialog::new(cx)
        .title("Confirm")
        .child("Are you sure?")
        .footer(|cx| Button::new("ok").primary().label("OK"))
});

// Notification
window.push_notification(
    Notification::new().message("Saved").success(),
    cx,
);
window.remove_notification::<MyType>(cx);
```

Declarative parts: `DialogHeader`, `DialogTitle`, `DialogDescription`, `DialogFooter`.

## Select / Combobox

```rust
// Select — single value
let state = cx.new(|cx| SelectState::new(items, window, cx));
Select::new(&state).searchable(true)

// Combobox — multi-select, custom trigger
let state = cx.new(|cx| ComboboxState::new(searchable_vec, window, cx));
Combobox::new(&state).multiple(true)
```

`String`, `SharedString`, `&'static str` implement `SelectItem` by default.

## Input modes

| Mode | Use |
|------|-----|
| `InputMode::SingleLine` | Single-line text |
| `InputMode::MultiLine` | Textarea |
| `InputMode::AutoGrow` | Growing textarea |
| `InputMode::CodeEditor` | Syntax highlight, line numbers, LSP |

Code editor uses `ropey` internally; enable `tree-sitter-languages` for grammars.

## Theming traits

| Trait / type | Purpose |
|--------------|---------|
| `Theme` | Global theme singleton |
| `ActiveTheme` | `cx.theme()` accessor |
| `ThemeColor` | Color tokens |
| `SemanticThemeTokens` | gpui-base semantic colors |

## Feature flags (complete)

| Feature | Description |
|---------|-------------|
| `decimal` | Decimal number support |
| `inspector` | UI inspector |
| `tree-sitter` | Core tree-sitter |
| `tree-sitter-languages` | Meta-feature: all language grammars |
| `tree-sitter-rust`, `tree-sitter-json`, ... | Individual grammars |
| `webview` | Embedded WebView (Wry) |

## Useful example binaries

| Example | Demonstrates |
|---------|--------------|
| `hello_world` | Minimal app + Root |
| `input` | InputState patterns |
| `dialog_overlay` | Dialog layering |
| `window_title` | Custom title bar |
| `app_assets` | Custom asset loading |
| `focus_trap` | Focus management |
| `webview` | WebView integration |

## Comparison anchors (framework choice)

| | GPUI Component | Iced | egui | Qt 6 |
|---|----------------|------|------|------|
| Renderer | GPUI (GPU) | wgpu | wgpu | Qt |
| Large table | DataTable virtual | Limited | List virtual | Yes |
| Dock layout | Yes | Yes | Yes | Yes |
| Code editor + LSP | Yes | Basic | Basic | Basic |
| Built themes | Yes | No | No | No |
| Web target | WASM gallery only | Yes | Yes | Yes |

## Links

- Docs: https://longbridge.github.io/gpui-component/docs/
- gpui-base: https://longbridge.github.io/gpui-component/base/
- GitHub: https://github.com/longbridge/gpui-component
- GPUI: https://gpui.rs
- Zed GPUI crate: https://github.com/zed-industries/zed/tree/main/crates/gpui
