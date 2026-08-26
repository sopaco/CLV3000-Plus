---
type: agent_context
project: clv3000-plus
title: Agent Architecture Context
source: .
---

## Project Overview

CLV3000 Plus is a native desktop utility that reclaims disk space on developer workstations. It scans configured development directories plus known global caches, identifies cleanable build artifacts across 13+ technology stacks, and safely deletes them via trash/quarantine. Its differentiator is detection of **AI coding agent experiment projects** (Claude/Cursor/Codex/Trae/OpenCode leftovers) — capability no competing cleanup tool offers. Consumers: end users on macOS/Windows/Linux via the GPUI desktop app; coding agents via this doc + `agent/repomix.md`. Key constraints: local-only (no network), destructive operations must be guarded by risk levels and protected-path checks; UI ships trilingual (zh/en/ja).

## Architecture

Three-crate Cargo workspace, layered UI → platform-agnostic logic → OS abstraction:

| Crate | Layer | Responsibility |
|---|---|---|
| `crates/clv-app` | Presentation | GPUI desktop app: window shell, `AppStore` state entity, per-page views, theme, i18n, custom controls |
| `crates/clv-core` | Domain logic (pure) | Scanner rules engine, cleanup engine, agent-project/session detection, models, settings persistence, path safety |
| `crates/clv-platform` | Platform adapter | Disk usage estimation (`primary_disk_usage`), process enumeration (`sysinfo`), OS startup-item helpers |

- **State flow**: views never own scan results; a single `AppStore` entity holds `AppSettings`, current page, last `ScanReport`, cleanup reports, `CleanupHistory`, and `pending_cleanup_notification`. Views subscribe/observe the store.
- **Async jobs in services layer**: `AppStore` delegates background scan/cleanup to `services/scan` and `services/cleanup` (worker thread + mpsc); the GPUI executor polls `ScanEvent` / `CleanupEvent` (including `Progress`) back into the store.
- **Views are lazy singletons**: `ClvApp` creates each page view on first visit and caches the entity (`crates/clv-app/src/app/mod.rs`).
- **Scan runs async** with throttled progress events (`ScanEvent::Progress/Done`) delivered via `services/scan` poll into the store.
- **Cleanup runs async** with per-item `CleanupProgress` events (`CleanupEvent::Progress/Done`) via `services/cleanup` poll; drives shell `cleanup_progress_bar` and CleanupView loading state; on `Done` persists cleanup history and queues a completion notification for `ClvApp`.
- **Cleanup history & notifications**: `CleanupHistory` (90-day JSON in config dir) tracks per-run freed bytes/counts; Dashboard shows 7/30-day trend card; `ClvApp` pushes `gpui-component` success notification on cleanup complete.
- **Typed i18n boundary**: `clv-core` stores `RuleDescription` / `AgentReasonPart` enums (not display strings); user-visible text resolved via `RuleDescription::text(lang)` or `clv-app/i18n` (`rule_description_label`, `format_agent_reason`).
- **Dependency direction**: `clv-app` → `clv-core` + `clv-platform`; no reverse edges.

## Module Map

| Module | Responsibility | Primary paths |
|---|---|---|
| App shell & routing | Root component, page switching, lazy view creation, cleanup completion notifications | `crates/clv-app/src/app/mod.rs`, `shell.rs` |
| Global state | `AppStore` (scan + cleanup progress, `CleanupHistory`, `pending_cleanup_notification`), page state, scan/cleanup orchestration via `services/` | `crates/clv-app/src/app/state.rs`, `services/scan.rs`, `services/cleanup.rs` |
| Scanner | Rule-based discovery of cleanable items + global caches (incl. Bun/Homebrew/Docker on non-Windows) + agent sessions; locale-aware scan phases; consumes `project_rules`/`global_rules` | `crates/clv-core/src/scanner.rs`, `settings/global_rules.rs`, `settings/project_rules.rs`, `locale.rs` |
| Cleanup engine | Trash-move via robust `move_entry` (readonly clear, cross-device copy fallback), per-item `CleanupProgress` callbacks, enriched `CleanupReport`, `CleanupHistory` JSON persistence (90-day prune) | `crates/clv-core/src/cleanup.rs` |
| Agent detection | AI-agent experiment project heuristics + session target discovery; structured `reason_parts` | `crates/clv-core/src/agent.rs`, `agent_sessions.rs`, `messages/agent_reason.rs` |
| Localized messages | Typed rule descriptions (`RuleDescription` R001–R135) + agent reason parts; translation table + codegen | `crates/clv-core/src/messages/rule_description.rs`, `messages/agent_reason.rs`, `scripts/generate-rule-descriptions.py`, `scripts/rule-description-translations.json` |
| Models | `TechStack` (13+ stacks), `RiskLevel`, `CleanupCategory`, `ScanReport` types; `ScanItem.description` is `RuleDescription` | `crates/clv-core/src/models.rs`, `category.rs` |
| Settings & paths | Persistence, rule tables (typed descriptions), marker definitions, env-var expansion, protected-path guards | `crates/clv-core/src/settings/mod.rs`, `global_rules.rs`, `project_rules.rs`, `rule.rs`, `markers.rs`, `paths.rs` |
| Platform adapters | Disk usage (mount-aware macOS; Windows sums local fixed drives, avoids folder-mount double-count), process enum/kill, startup items | `crates/clv-platform/src/disk.rs`, `process.rs` |
| Page views | Dashboard (cleanup trend card), Cleanup, Agent, Startup, Process, Settings, Onboarding | `crates/clv-app/src/views/*.rs` |
| i18n | Trilingual label catalog; resolves `RuleDescription` + `AgentReasonPart`; cleanup progress/status/summary/history/notification strings | `crates/clv-app/src/i18n/labels.rs`, `mod.rs` |
| UI kit & assets | Reusable controls, scan/cleanup progress bars (`ui/security.rs`), list widgets, theme, icons, embedded assets | `crates/clv-app/src/ui/`, `theme.rs`, `assets.rs`, `build.rs` |

## Core Flows

1. **Startup**: `main.rs` → load persisted settings → create `AppStore` → if `onboarding_done` is false show Onboarding page, else Dashboard → kick off async disk-usage refresh via `clv-platform::primary_disk_usage`.
2. **Scan**: user triggers scan → `spawn_scan` (`services/scan`) runs `Scanner::scan` over configured roots (pruning nested matches), resolves global cache rules (cargo/npm/Bun/Homebrew/Docker/Xcode/Trae caches…), discovers agent session targets (Claude/Cursor/Codex/Trae/OpenCode/TraeX…) → emits throttled localized `ScanProgress` (via `locale::scan_phase_*`) → `poll_scan` delivers `ScanReport` (items by tech stack, risk level, size; descriptions as `RuleDescription` IDs) into `AppStore`.
3. **Cleanup**: CleanupView lists report items grouped by bucket/risk filter → user selects → `run_cleanup` → `spawn_cleanup` (`services/cleanup`) runs cleanup engine with per-item `CleanupProgress` callbacks (completed/total/freed_bytes/current_path), using robust `move_entry` (readonly clear, cross-device fallback) to trash (skipping protected paths) → `poll_cleanup` delivers `CleanupPoll::Progress` into `AppStore` (shell `cleanup_progress_bar` + CleanupView loading state) then `CleanupPoll::Done(CleanupReport, removed_paths)`; report tracks `freed_bytes`, `success_count`, `failed` (path + error), `trashed`; store removes cleaned paths, re-detects agent projects, refreshes disk usage; summary via `cleanup_summary`; appends `CleanupHistoryRecord` to persisted `cleanup_history.json` (90-day retention), sets `pending_cleanup_notification`; `ClvApp` shows success toast; Dashboard `history_card` displays 7/30-day freed totals.
4. **Agent projects**: AgentView reads `report.agent_projects`, applies search filter (name/`reason_parts`/path/stack); sessions from Claude/Cursor/Codex/Trae/OpenCode etc. surfaced with structured `AgentReasonPart` reasons formatted per language before cleanup.
5. **Process management**: ProcessView polls `clv-platform` enumerator on page-show/refresh trigger → search/sort in-memory list → kill selected PID.

## Tech Stack

- Language: Rust (edition 2024), workspace resolver 2, release profile with thin LTO + strip
- UI: `gpui` 0.2 + `gpui-component` 0.5 (+ assets crate) — native GPU-rendered desktop UI
- Filesystem traversal: `walkdir`; process/system info: `sysinfo`
- Persistence/config: `serde`/`serde_json`, `directories` (XDG/home layout)
- Utilities: `anyhow`, `thiserror`, `chrono`, `uuid`, `open` (reveal in Finder/Explorer), `sys-locale`
- i18n codegen: `scripts/generate-rule-descriptions.py` generates `RuleDescription` enum from `scripts/rule-description-translations.json`
- Logging: `tracing` + `tracing-subscriber` (warn-level in release)
- Packaging: `scripts/bundle-macos.sh`; icons under `assets/icons/`

## System Boundaries

- **Local filesystem only** — no network calls, no telemetry, no external APIs.
- **Read side**: home directory, configured scan roots (default `~/Projects`, Documents, Desktop…), global tool caches (`~/.cargo`, npm/pip/Bun/Homebrew/Docker/Xcode derived data, Trae/OpenCode app caches, etc.), agent CLI session directories (`~/.trae/cli`, OpenCode data dirs; overridable via `TRAE_DIR`/`TRAEX_SESSIONS_DIR`/`OPENCODE_DIR`).
- **Write side**: trash/quarantine directory managed by the cleanup engine; settings JSON and `cleanup_history.json` in the app config dir. Everything else is read-only.
- **Trust boundary — protected paths**: `paths.rs` hard-blocks system locations (Unix root/system dirs, Windows %SystemRoot% variants) from any delete operation; risk levels (`Safe`/`Caution`/`Protected`) gate UI actions.
- **OS integration**: process enumeration/kill via `sysinfo`; "open folder" via `open` crate; login/startup items surfaced in StartupView.
- **Third-party risk surface**: rule tables in `scanner.rs` decide what is deletable; nested-match pruning prevents deleting project sources inside matched parents.

## Code Map Index

| Concept | Location | Notes |
|---|---|---|
| Entry point / window bootstrapping | `crates/clv-app/src/main.rs` | GPUI app launch |
| Root component & page routing | `crates/clv-app/src/app/mod.rs` | Lazy view entities; shell progress bars; cleanup completion notifications |
| Central state store | `crates/clv-app/src/app/state.rs` | `AppStore`, cleanup progress/history/notification fields, `run_cleanup`, filters |
| Async scan orchestration | `crates/clv-app/src/services/scan.rs` | `spawn_scan` / `poll_scan` worker + mpsc |
| Async cleanup orchestration | `crates/clv-app/src/services/cleanup.rs` | `CleanupEvent`/`CleanupPoll` with `Progress`; worker + mpsc |
| Scan orchestration & rules | `crates/clv-core/src/scanner.rs` | Progress throttling, pruning; rules in `settings/` |
| Deletion engine | `crates/clv-core/src/cleanup.rs` | `CleanupProgress`, robust `move_entry`, trash move + enriched report + `CleanupHistory` |
| Agent project/session detection | `crates/clv-core/src/agent.rs`, `agent_sessions.rs` | Trae/Trae CN/SOLO, TraeX, OpenCode session targets |
| Typed i18n messages | `crates/clv-core/src/messages/`, `scripts/generate-rule-descriptions.py` | `RuleDescription` R001–R135, `AgentReasonPart`; JSON → codegen |
| Domain models | `crates/clv-core/src/models.rs`, `category.rs` | `TechStack`, `RiskLevel`, `CleanupCategory`, reports |
| Locale & scan phases | `crates/clv-core/src/locale.rs` | `Language`, `scan_phase_*` helpers |
| Path safety & defaults | `crates/clv-core/src/paths.rs` | Protected-path guards |
| Settings persistence & rules | `crates/clv-core/src/settings/mod.rs` | `AppSettings`, `global_rules`, `project_rules`, `rule`, `markers` |
| Platform disk usage | `crates/clv-platform/src/disk.rs` | `primary_disk_usage`; mount-aware macOS; Windows multi-drive sum |
| Process enumeration | `crates/clv-platform/src/process.rs` | sysinfo wrapper |
| Feature pages | `crates/clv-app/src/views/` | dashboard (cleanup trend card)/cleanup/agent/process/startup/settings/onboarding |
| Label translations | `crates/clv-app/src/i18n/labels.rs`, `mod.rs` | zh/en/ja; cleanup progress/status/summary/history/notification helpers |
| Styling & controls | `crates/clv-app/src/theme.rs`, `crates/clv-app/src/ui/` | Reusable widgets; `scan_progress_bar` / `cleanup_progress_bar` in `security.rs` |
| macOS packaging | `scripts/bundle-macos.sh` | App bundle build |
| Unit tests (rules, safety) | `crates/clv-core/src/lib.rs` | Scanner/cleanup/path/i18n test suite; cleanup `move_entry` cross-device tests |

*Implementation detail lives in `.terrain/agent/repomix.md` — grep/read it for signatures and source.*