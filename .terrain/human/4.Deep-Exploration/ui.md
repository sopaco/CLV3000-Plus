# UI Kit Domain

**Module path:** `crates/clv-app/src/ui/`, `crates/clv-app/src/theme.rs`, `crates/clv-app/src/assets.rs`  
**Generated:** 2026-08-28

---

## What This Module Does

The UI kit provides reusable visual building blocks—progress bars, list rows, health score widgets, icons—so feature views stay focused on layout and user flows rather than reimplementing the same GPUI styling everywhere. Theme helpers translate disk usage ratios into color-coded health scores that make the Dashboard immediately scannable.

---

## Core Capabilities

1. **Progress widgets** — `scan_progress_bar` and `cleanup_progress_bar` in `ui/security.rs` with cancel button integration.

2. **Health score** — `compute_health` and `hero_banner` in `theme.rs` drive Dashboard visual summary.

3. **List components** — Reusable list widgets in `ui/list.rs` for Cleanup and Agent pages.

4. **Icons** — Embedded SVG/PNG icons via `ui/icons.rs` and `assets.rs` + `build.rs` compilation.

5. **Theme colors** — Centralized palette in `theme.rs` with multiple theme presets (Default, Cherry, Aurora, Neon).

---

## Key Components

| Component | File path | Responsibility |
|-----------|-----------|----------------|
| `scan_progress_bar` | `crates/clv-app/src/ui/security.rs` | Scan progress + cancel UI |
| `cleanup_progress_bar` | `crates/clv-app/src/ui/security.rs` | Cleanup progress + cancel UI |
| `compute_health` | `crates/clv-app/src/theme.rs` | Disk health score calculation |
| `hero_banner` | `crates/clv-app/src/theme.rs` | Dashboard hero section |
| List widgets | `crates/clv-app/src/ui/list.rs` | Shared list rendering |
| `controls.rs` | `crates/clv-app/src/ui/controls.rs` | Buttons, toggles, inputs |
| `Assets` | `crates/clv-app/src/assets.rs` | GPUI asset bundle |
| `build.rs` | `crates/clv-app/build.rs` | Compile-time asset embedding |

---

## Cross-Module Interactions

| Module | Direction | Interface | Notes |
|--------|-----------|-----------|-------|
| ProgressHud | Uses | progress bar widgets | Overlay during scan/cleanup |
| DashboardView | Uses | hero_banner, compute_health | Landing page visuals |
| CleanupView / AgentView | Uses | list widgets | Item/project lists |
| AppStore | Observed by | progress bar bindings | Live progress fields |

---

## Implementation Highlights

- Risk-level color coding in cleanup lists aligns with `RiskLevel` enum semantics.
- Theme applied at window creation via `apply_theme` from settings preference.
- Icons include app icon and tray icon (`assets/icons/tray.png`).
