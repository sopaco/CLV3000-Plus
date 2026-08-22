---
type: agent_context
project: clv3000-plus
title: Agent Architecture Context
source: .
---

## Project Overview

CLV3000 Plus is a native desktop utility that reclaims disk space on developer workstations. It scans configured development directories plus known global caches, identifies cleanable build artifacts across 13+ technology stacks, and safely deletes them via trash/quarantine. Its differentiator is detection of **AI coding agent experiment projects** (Claude/Cursor/Codex leftovers) — capability no competing cleanup tool offers. Consumers: end users on macOS/Windows/Linux via the GPUI desktop app; coding agents via this doc + `agent/repomix.md`. Key constraints: local-only (no network), destructive operations must be guarded by risk levels and protected-path checks; UI ships trilingual (zh/en/ja).

## Architecture

Three-crate Cargo workspace, layered UI → platform-agnostic logic → OS abstraction:

| Crate | Layer | Responsibility |
|---|---|---|
| `crates/clv-app` | Presentation | GPUI desktop app: window shell, `AppStore` state entity, per-page views, theme, i18n, custom controls |
| `crates/clv-core` | Domain logic (pure) | Scanner rules engine, cleanup engine, agent-project/session detection, models, settings persistence, path safety |
| `crates/clv-platform` | Platform adapter | Process enumeration (`sysinfo`), OS-specific helpers |

- **State flow**: views never own scan results; a single `AppStore` entity holds `AppSettings`, current page, last `ScanReport`, cleanup reports. Views subscribe/observe the store.
- **Views are lazy singletons**: `ClvApp` creates each page view on first visit and caches the entity (`crates/clv-app/src/app/mod.rs`).
- **Scan runs async** with throttled progress events (`ScanEvent::Progress/Done`) into the store.
- **Dependency direction**: `clv-app` → `clv-core` + `clv-platform`; no reverse edges.

## Module Map

| Module | Responsibility | Primary paths |
|---|---|---|
| App shell & routing | Root component, page switching, lazy view creation | `crates/clv-app/src/app/mod.rs`, `shell.rs` |
| Global state | `AppStore`, `AppPage`, scan/cleanup events, filters | `crates/clv-app/src/app/state.rs` |
| Scanner | Rule-based discovery of cleanable items + global caches + agent sessions | `crates/clv-core/src/scanner.rs` |
| Cleanup engine | Trash-move deletion, bucket classification, `CleanupReport` | `crates/clv-core/src/cleanup.rs` |
| Agent detection | AI-agent experiment project heuristics + session target discovery | `crates/clv-core/src/agent.rs`, `agent_sessions.rs` |
| Models | `TechStack` (13+ stacks), `RiskLevel`, `ScanReport` types | `crates/clv-core/src/models.rs` |
| Settings & paths | Persistence, default scan dirs, env-var expansion, protected-path guards | `crates/clv-core/src/settings.rs`, `paths.rs` |
| Process manager | Running-process enumeration/search/sort/kill | `crates/clv-platform/src/process.rs` |
| Page views | Dashboard, Cleanup, Agent, Startup, Process, Settings, Onboarding | `crates/clv-app/src/views/*.rs` |
| i18n | Trilingual label catalog (zh/en/ja) | `crates/clv-app/src/i18n/labels.rs` |
| UI kit | Reusable controls, text styles, list widgets, security-styled components | `crates/clv-app/src/ui/` |
| Theme & assets | Colors, icons, embedded assets, Windows icon resource | `crates/clv-app/src/theme.rs`, `assets.rs`, `build.rs` |

## Core Flows

1. **Startup**: `main.rs` → load persisted settings → create `AppStore` → if `onboarding_done` is false show Onboarding page, else Dashboard → kick off async disk-usage refresh.
2. **Scan**: user triggers scan → `Scanner::scan` walks configured roots (pruning nested matches), resolves global cache rules (cargo/npm/Xcode caches…), discovers agent session targets → emits throttled `ScanProgress` → produces `ScanReport` (items by tech stack, risk level, size) stored in `AppStore`.
3. **Cleanup**: CleanupView lists report items grouped by bucket/risk filter → user selects → cleanup engine moves targets to trash dir (skipping protected paths) → `CleanupReport { success, trashed, failed }` recorded; store refreshes usage.
4. **Agent projects**: AgentView reads `report.agent_projects`, applies search filter (name/reason/path/stack); sessions from Claude/Cursor/Codex etc. are surfaced with reasons for safe review before cleanup.
5. **Process management**: ProcessView polls `clv-platform` enumerator on page-show/refresh trigger → search/sort in-memory list → kill selected PID.

## Tech Stack

- Language: Rust (edition 2024), workspace resolver 2, release profile with thin LTO + strip
- UI: `gpui` 0.2 + `gpui-component` 0.5 (+ assets crate) — native GPU-rendered desktop UI
- Filesystem traversal: `walkdir`; process/system info: `sysinfo`
- Persistence/config: `serde`/`serde_json`, `directories` (XDG/home layout)
- Utilities: `anyhow`, `thiserror`, `chrono`, `uuid`, `open` (reveal in Finder/Explorer), `sys-locale`
- Logging: `tracing` + `tracing-subscriber` (warn-level in release)
- Packaging: `scripts/bundle-macos.sh`; icons under `assets/icons/`

## System Boundaries

- **Local filesystem only** — no network calls, no telemetry, no external APIs.
- **Read side**: home directory, configured scan roots (default `~/Projects`, Documents, Desktop…), global tool caches (`~/.cargo`, npm/pip/Xcode derived data, etc.), agent CLI session directories.
- **Write side**: trash/quarantine directory managed by the cleanup engine; settings JSON in the app config dir. Everything else is read-only.
- **Trust boundary — protected paths**: `paths.rs` hard-blocks system locations (Unix root/system dirs, Windows %SystemRoot% variants) from any delete operation; risk levels (`Safe`/`Caution`/`Protected`) gate UI actions.
- **OS integration**: process enumeration/kill via `sysinfo`; "open folder" via `open` crate; login/startup items surfaced in StartupView.
- **Third-party risk surface**: rule tables in `scanner.rs` decide what is deletable; nested-match pruning prevents deleting project sources inside matched parents.

## Code Map Index

| Concept | Location | Notes |
|---|---|---|
| Entry point / window bootstrapping | `crates/clv-app/src/main.rs` | GPUI app launch |
| Root component & page routing | `crates/clv-app/src/app/mod.rs` | Lazy view entities |
| Central state store | `crates/clv-app/src/app/state.rs` | `AppStore`, events, filters |
| Scan orchestration & rules | `crates/clv-core/src/scanner.rs` | Progress throttling, pruning |
| Deletion engine | `crates/clv-core/src/cleanup.rs` | Trash move + report |
| Agent project/session detection | `crates/clv-core/src/agent.rs`, `agent_sessions.rs` | Differentiating feature |
| Domain models | `crates/clv-core/src/models.rs` | `TechStack`, `RiskLevel`, reports |
| Path safety & defaults | `crates/clv-core/src/paths.rs` | Protected-path guards |
| Settings persistence | `crates/clv-core/src/settings.rs` | serde JSON |
| Process enumeration | `crates/clv-platform/src/process.rs` | sysinfo wrapper |
| Feature pages | `crates/clv-app/src/views/` | dashboard/cleanup/agent/process/startup/settings/onboarding |
| Label translations | `crates/clv-app/src/i18n/labels.rs` | zh/en/ja catalog |
| Styling & controls | `crates/clv-app/src/theme.rs`, `crates/clv-app/src/ui/` | Reusable widgets |
| macOS packaging | `scripts/bundle-macos.sh` | App bundle build |
| Unit tests (rules, safety) | `crates/clv-core/src/lib.rs` | Scanner/cleanup/path test suite |

*Implementation detail lives in `.terrain/agent/repomix.md` — grep/read it for signatures and source.*