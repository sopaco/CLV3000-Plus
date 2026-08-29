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
| `crates/clv-app` | Presentation | GPUI desktop app: window shell, `ProgressHud`, system tray icon, `AppStore` state entity, per-page views (incl. Large Files), theme, i18n, custom controls |
| `crates/clv-core` | Domain logic (pure) | Scanner rules engine, cleanup engine, large-file discovery, agent-project/session detection, models, settings persistence, path safety |
| `crates/clv-platform` | Platform adapter | Disk usage (`primary_disk_usage`, `list_disk_volumes`), native folder picker (`pick_folders`), process enumeration/kill (`sysinfo`), OS startup-item helpers |

- **State flow**: views never own scan results; a single `AppStore` entity holds `AppSettings`, current page, last `ScanReport`, cleanup reports, `CleanupHistory`, `pending_cleanup_notification`, `status_message`, disk usage totals, and scan/cleanup cancel handles. Views subscribe/observe the store.
- **Async jobs in services layer**: `AppStore` delegates background scan/cleanup to `services/scan` and `services/cleanup` (worker thread + mpsc); the GPUI executor polls `ScanEvent` / `CleanupEvent` (including `Progress`) back into the store.
- **Views are lazy singletons**: `ClvApp` creates each page view on first visit and caches the entity (`crates/clv-app/src/app/mod.rs`).
- **Progress HUD**: `ProgressHud` (`hud.rs`) entity above page content renders scan/cleanup progress bars with cancel buttons; `AppStore::notify_progress_only` notifies the attached HUD.
- **System tray**: `TrayController` (`tray.rs`) installs tray icon (Open/Scan/Quit); `request_scan`/`take_scan_request` bridge tray Scan to `AppStore::start_scan` (polled in `ClvApp::new`); double-click opens app; tooltip updated on disk refresh.
- **Scan runs async** (cancellable via `AtomicBool`) with throttled progress events (`ScanEvent::Progress/Done`) delivered via `services/scan` poll into the store; `Scanner::scan_cancellable` collects large files inline during `scan_tree`, may set `sizes_truncated`, and persists report via `save_last_scan`.
- **Cleanup runs async** (cancellable) with per-item `CleanupProgress` events (`CleanupEvent::Progress/Done`) via `services/cleanup` poll; `ProgressHud` drives progress UI; on `Done` persists cleanup history (incl. `TrashedEntry` records), purges aged trash per `soft_delete_days`, queues completion notification; supports `restore_trashed_entry`.
- **Cleanup history & notifications**: `CleanupHistory` (90-day JSON in config dir) tracks per-run freed bytes/counts and restorable `TrashedEntry` items; Dashboard shows 7/30-day trend card; `ClvApp` pushes `gpui-component` success notification on cleanup complete.
- **Typed i18n boundary**: `clv-core` stores `RuleDescription` / `AgentReasonPart` enums (not display strings); user-visible text resolved via `RuleDescription::text(lang)` or `clv-app/i18n` (`rule_description_label`, `format_agent_reason`).
- **Dependency direction**: `clv-app` → `clv-core` + `clv-platform`; no reverse edges.

## Module Map

| Module | Responsibility | Primary paths |
|---|---|---|
| App shell & routing | Root component, page switching, lazy view creation, `ProgressHud` + status bar, tray action/scan dispatch, cleanup completion notifications | `crates/clv-app/src/app/mod.rs`, `hud.rs`, `shell.rs`, `tray.rs` |
| Global state | `AppStore` (scan/cleanup progress + cancel, `CleanupHistory`/`TrashedEntry` restore, `pending_cleanup_notification`, `status_message`, disk totals), page state, scan/cleanup orchestration, folder picker | `crates/clv-app/src/app/state.rs`, `services/scan.rs`, `services/cleanup.rs` |
| Scanner | Rule-based discovery of cleanable items + global caches (incl. Bun/Homebrew/Docker/browser caches on non-Windows) + agent sessions; locale-aware scan phases; cancellable scan with `should_skip_dir`; inline large-file pass; consumes `project_rules`/`global_rules` | `crates/clv-core/src/scanner.rs`, `large_files.rs`, `settings/global_rules.rs`, `settings/project_rules.rs`, `locale.rs` |
| Cleanup engine | Trash-move via robust `move_entry` (readonly clear, cross-device copy fallback), per-item `CleanupProgress` callbacks, `TrashedEntry` tracking, `restore_trashed`/`purge_old_trash`, enriched `CleanupReport`, `CleanupHistory` JSON persistence (90-day prune) | `crates/clv-core/src/cleanup.rs` |
| Agent detection | AI-agent experiment project heuristics + session target discovery; structured `reason_parts`; skips active marker-only repos | `crates/clv-core/src/agent.rs`, `agent_sessions.rs`, `messages/agent_reason.rs` |
| Localized messages | Typed rule descriptions (`RuleDescription` R001–R140) + agent reason parts; translation table + codegen | `crates/clv-core/src/messages/rule_description.rs`, `messages/agent_reason.rs`, `scripts/generate-rule-descriptions.py`, `scripts/rule-description-translations.json` |
| Models | `TechStack` (13+ stacks), `RiskLevel`, `CleanupCategory`, `ScanReport` types (incl. `large_files`, `cancelled`, `sizes_truncated`); `ScanItem.description` is `RuleDescription` | `crates/clv-core/src/models.rs`, `category.rs` |
| Settings & paths | Persistence, rule tables (typed descriptions), marker definitions, env-var expansion, protected-path guards, `soft_delete_days`, last-scan JSON | `crates/clv-core/src/settings/mod.rs`, `global_rules.rs`, `project_rules.rs`, `rule.rs`, `markers.rs`, `paths.rs` |
| Platform adapters | Disk usage + volume listing, native folder picker, process enum/kill, startup items | `crates/clv-platform/src/disk.rs`, `dialog.rs`, `process.rs` |
| Page views | Dashboard (health score, on-demand disk-volumes dialog, large-files tile, history/restore), Cleanup, Agent, Large Files, Startup, Process (visibility-aware poll), Settings, Onboarding | `crates/clv-app/src/views/*.rs` |
| i18n | Trilingual label catalog; resolves `RuleDescription` + `AgentReasonPart`; cleanup progress/status/summary/history/notification/restore, tray, large-files, folder-picker strings | `crates/clv-app/src/i18n/labels.rs`, `mod.rs` |
| UI kit & assets | Reusable controls, health-score `hero_banner` / `compute_health`, scan/cleanup progress bars (`ui/security.rs`), list widgets, theme, icons, embedded assets | `crates/clv-app/src/ui/`, `theme.rs`, `assets.rs`, `build.rs` |

## Core Flows

1. **Startup**: `main.rs` → install system tray (`TrayController`) → load persisted settings → create `AppStore` (loads `last_report` via `load_last_scan`, background `purge_old_trash`) → if `onboarding_done` is false show Onboarding page, else Dashboard → kick off async disk refresh via `primary_disk_usage`; update tray tooltip; `ClvApp` polls `take_scan_request` for tray-triggered scans.
2. **Scan**: user triggers scan → `spawn_scan` (with cancel flag) runs `Scanner::scan_cancellable` over configured roots (pruning nested matches), resolves global cache rules, discovers agent session targets → collects large files during `scan_tree` → emits throttled localized `ScanProgress` → `poll_scan` delivers `ScanReport` (items, `large_files`, `cancelled`, `sizes_truncated`) into `AppStore`; cancel sets `scan_restart_pending` for immediate re-scan; `save_last_scan` persists on completion.
3. **Cleanup**: CleanupView lists report items grouped by bucket/risk filter → user selects → `run_cleanup` → `spawn_cleanup` (with cancel flag) runs cleanup engine with per-item `CleanupProgress` callbacks, using robust `move_entry` to trash (skipping protected paths) → `poll_cleanup` delivers progress to `ProgressHud` then `CleanupPoll::Done`; report tracks `freed_bytes`, `success_count`, `failed`, `trashed_entries`; store removes cleaned paths, re-detects agent projects, refreshes disk usage; appends `CleanupHistoryRecord` (with `TrashedEntry` list), sets `pending_cleanup_notification`; Dashboard `history_card` shows trends and restore actions via `restore_trashed_entry`.
4. **Agent projects**: AgentView reads `report.agent_projects`, applies search filter (name/`reason_parts`/path/stack); sessions from Claude/Cursor/Codex/Trae/OpenCode etc. surfaced with structured `AgentReasonPart` reasons formatted per language before cleanup.
5. **Process management**: ProcessView polls `clv-platform` enumerator on page-show/refresh trigger (visibility-aware) → search/sort in-memory list → kill selected PID.

## Tech Stack

- Language: Rust (edition 2024), workspace resolver 2, release profile with thin LTO + strip
- UI: `gpui` 0.2 + `gpui-component` 0.5 (+ assets crate) — native GPU-rendered desktop UI
- Filesystem traversal: `walkdir`; process/system info: `sysinfo`
- Persistence/config: `serde`/`serde_json`, `directories` (XDG/home layout)
- Utilities: `anyhow`, `thiserror`, `chrono`, `uuid`, `open` (reveal in Finder/Explorer), `sys-locale`
- Native dialogs & tray: `rfd` (folder picker), `tray-icon` + `muda` (system tray menu)
- i18n codegen: `scripts/generate-rule-descriptions.py` generates `RuleDescription` enum from `scripts/rule-description-translations.json`
- Logging: `tracing` + `tracing-subscriber` (warn-level in release)
- Packaging: `scripts/bundle-macos.sh`; icons under `assets/icons/`

## System Boundaries

- **Local filesystem only** — no network calls, no telemetry, no external APIs.
- **Read side**: home directory, configured scan roots (default `~/Projects`, Documents, Desktop…), global tool caches (`~/.cargo`, npm/pip/Bun/Homebrew/Docker/Xcode derived data, browser caches, Trae/OpenCode app caches, etc.), agent CLI session directories (`~/.trae/cli`, OpenCode data dirs; overridable via `TRAE_DIR`/`TRAEX_SESSIONS_DIR`/`OPENCODE_DIR`).
- **Write side**: app-managed trash/quarantine directory (cleanup engine); settings JSON, `cleanup_history.json`, and last-scan JSON in the app config dir. Everything else is read-only.
- **Trust boundary — protected paths**: `paths.rs` hard-blocks system locations (Unix root/system dirs, Windows %SystemRoot% variants) from any delete operation; risk levels (`Safe`/`Caution`/`Protected`) gate UI actions; active projects may elevate `Safe` → `Caution`.
- **OS integration**: system tray icon (Open/Scan/Quit); native folder picker for scan-path configuration; process enumeration/kill via `sysinfo`; "open folder" via `open` crate; login/startup items surfaced in StartupView.
- **Third-party risk surface**: rule tables in `scanner.rs` decide what is deletable; nested-match pruning prevents deleting project sources inside matched parents.

## Code Map Index

| Concept | Location | Notes |
|---|---|---|
| Entry point / window bootstrapping | `crates/clv-app/src/main.rs` | GPUI app launch; tray install; `TrayPending` slot |
| Root component & page routing | `crates/clv-app/src/app/mod.rs` | Lazy view entities; `ProgressHud` + status bar; tray action/scan polling; cleanup completion notifications |
| Progress HUD | `crates/clv-app/src/app/hud.rs` | Scan/cleanup progress bars with cancel; observes `AppStore` |
| Central state store | `crates/clv-app/src/app/state.rs` | `AppStore`, cancel flags, cleanup progress/history/restore, `status_message`, `run_cleanup_safe`, `pick_scan_folders` |
| System tray | `crates/clv-app/src/tray.rs` | `TrayController`, `TrayAction` (Open/Scan/Quit), `request_scan`/`take_scan_request`, tooltip updates |
| Async scan orchestration | `crates/clv-app/src/services/scan.rs` | `spawn_scan` / `poll_scan` with cancel `AtomicBool` |
| Async cleanup orchestration | `crates/clv-app/src/services/cleanup.rs` | `CleanupEvent`/`CleanupPoll` with `Progress` and cancel |
| Scan orchestration & rules | `crates/clv-core/src/scanner.rs` | `scan_cancellable`, `ProgressThrottle`, `should_skip_dir`; inline large files; rules in `settings/` |
| Large file discovery | `crates/clv-core/src/large_files.rs` | `finalize_large_files`, `LargeFileEntry`; collected during `scan_tree` |
| Deletion engine | `crates/clv-core/src/cleanup.rs` | `CleanupProgress`, `TrashedEntry`, `restore_trashed`, `purge_old_trash`, robust `move_entry`, `CleanupHistory` |
| Agent project/session detection | `crates/clv-core/src/agent.rs`, `agent_sessions.rs` | Trae/Trae CN/SOLO, TraeX, OpenCode session targets |
| Typed i18n messages | `crates/clv-core/src/messages/`, `scripts/generate-rule-descriptions.py` | `RuleDescription` R001–R140, `AgentReasonPart`; JSON → codegen |
| Domain models | `crates/clv-core/src/models.rs`, `category.rs` | `TechStack`, `RiskLevel`, `CleanupCategory`, `ScanReport` (+ `large_files`, `cancelled`, `sizes_truncated`) |
| Locale & scan phases | `crates/clv-core/src/locale.rs` | `Language`, `scan_phase_*` helpers |
| Path safety & defaults | `crates/clv-core/src/paths.rs` | Protected-path guards |
| Settings persistence & rules | `crates/clv-core/src/settings/mod.rs` | `AppSettings`, `soft_delete_days`, `load_last_scan`/`save_last_scan`, rules |
| Platform disk usage | `crates/clv-platform/src/disk.rs` | `primary_disk_usage`, `list_disk_volumes`, `DiskVolume`; mount-aware macOS; Windows multi-drive sum |
| Platform folder picker | `crates/clv-platform/src/dialog.rs` | `pick_folders` via `rfd` |
| Process enumeration | `crates/clv-platform/src/process.rs` | sysinfo wrapper; `ProcessCategory`, `ProcessEnumerator` |
| Feature pages | `crates/clv-app/src/views/` | dashboard (health score, disk-volumes dialog, large-files tile, history/restore)/cleanup/agent/large_files/process/startup/settings/onboarding |
| Label translations | `crates/clv-app/src/i18n/labels.rs`, `mod.rs` | zh/en/ja; tray, large-files, restore/cancel, folder-picker, health-score helpers |
| Styling & controls | `crates/clv-app/src/theme.rs`, `crates/clv-app/src/ui/` | `compute_health`, `hero_banner`; `scan_progress_bar` / `cleanup_progress_bar` in `security.rs` |
| macOS packaging | `scripts/bundle-macos.sh` | App bundle build |
| Unit tests (rules, safety) | `crates/clv-core/src/lib.rs` | Scanner/cleanup/path/i18n test suite; `scan_cancellable` and cleanup `move_entry` cross-device tests |

*Implementation detail lives in `.terrain/agent/repomix.md` — grep/read it for signatures and source.*