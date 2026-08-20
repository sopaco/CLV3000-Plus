---
type: agent_context
project: clv3000-plus
title: Agent Architecture Context
source: .
---

## Project Overview

CLV3000 Plus is a desktop utility application for PC maintenance in the Coding Agent era, built with Rust and GPUI (a GPU-accelerated native UI framework). It provides **intelligent cache cleanup** (Rust, Node, Python, Java, Android, iOS, Flutter, KMP, .NET, C/C++), **agent project detection** (Claude/Cursor/Codex/WorkBuddy experiments), **startup item management** (macOS LaunchAgents, Windows registry), and **process management** (CPU/memory monitoring). Targets macOS 12+ and Windows 10+. Offers dual modes: simple (non-technical users) and expert.

## Architecture

Three-layer Rust workspace: **clv-core** (platform-independent scanning rules, agent detection, cleanup logic), **clv-platform** (OS-specific startup/process operations via `sysinfo` and `plist`), and **clv-app** (GPUI-based GUI with view-per-feature layout). Data flows from platform abstractions through core models into GPUI views. No network services — fully local desktop tool.

| Layer | Crate | Role |
|-------|-------|------|
| Core | `clv-core` | Scan rules, agent project detection, cleanup execution, settings, models |
| Platform | `clv-platform` | Startup items (macOS/Windows), process enumeration via `sysinfo` |
| App | `clv-app` | GPUI UI — `main.rs` entry, `app/` (state management), `views/` (feature screens) |

## Module Map

| Module | Responsibility | Primary paths |
|--------|---------------|---------------|
| clv-core | Scan rules, agent detection, cleanup | `crates/clv-core/src/` |
| clv-core::scanner | Multi-language build cache scanning | `crates/clv-core/src/scanner.rs` |
| clv-core::cleanup | File/dir removal logic | `crates/clv-core/src/cleanup.rs` |
| clv-core::agent | Agent project identification | `crates/clv-core/src/agent.rs` |
| clv-core::models | Shared data models | `crates/clv-core/src/models.rs` |
| clv-core::settings | App configuration persistence | `crates/clv-core/src/settings.rs` |
| clv-platform | OS-level startup/process ops | `crates/clv-platform/src/` |
| clv-platform::startup | LaunchAgents, login items, registry | `crates/clv-platform/src/startup.rs` |
| clv-platform::process | CPU/memory process view | `crates/clv-platform/src/process.rs` |
| clv-app | GPUI desktop application | `crates/clv-app/src/` |
| clv-app::views | Feature screens (cleanup, agent, startup, process, dashboard, settings, onboarding) | `crates/clv-app/src/views/` |
| clv-app::app::state | Global app state management | `crates/clv-app/src/app/state.rs` |

## Core Flows

1. **Scan & Clean** — User selects scope (language/agent cache) → `scanner.rs` walks directories via `walkdir` → finds build artifacts (`target/`, `node_modules/`, `.gradle/`, etc.) → presents size summary in cleanup view → user confirms → `cleanup.rs` removes files.

2. **Agent Project Detection** — App scans workspace roots for `.claude/`, `.cursor/`, `.codex/`, `.workbuddy/` marker directories → `agent.rs` classifies project type and metadata → displayed in agent view.

3. **Startup Management** — `startup.rs` enumerates macOS LaunchAgents (`~/Library/LaunchAgents`) and Windows startup registry/folder → user toggles items → `sysinfo` verifies running state → enable/disable/launch operations.

4. **Process Monitoring** — `process.rs` queries `sysinfo` for running processes → dashboard view displays CPU/memory per process → user can sort/filter and terminate processes.

## Tech Stack

- **Language**: Rust 2021 edition
- **UI Framework**: GPUI 0.2.2 + gpui-component 0.5.1 (GPU-accelerated native UI by Longbridge)
- **Serialization**: serde + serde_json
- **Filesystem**: walkdir (recursive scanning), directories (XDG/OS paths)
- **Process**: sysinfo 0.33
- **Platform**: plist 1 (macOS .plist parsing)
- **Logging**: tracing + tracing-subscriber with env-filter
- **Error handling**: anyhow + thiserror
- **Other**: chrono (timestamps), uuid (v4), open (browser/URL launch)

## System Boundaries

| Boundary | Type | Details |
|----------|------|---------|
| macOS LaunchAgents | Local FS | `~/Library/LaunchAgents`, `/Library/LaunchAgents` |
| Windows Registry | OS API | Startup keys under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` |
| Build caches | Local FS | `target/`, `node_modules/`, `.gradle/`, `build/`, `DerivedData/`, etc. |
| Agent markers | Local FS | `.claude/`, `.cursor/`, `.codex/`, `.workbuddy/` directories |
| System processes | OS API | sysinfo cross-platform process enumeration |
| macOS plist | Local FS | `.plist` files parsed via `plist` crate |
| External | None | No network calls, no remote APIs, fully offline tool |

## Code Map Index

| Concept | Location | Notes |
|---------|----------|-------|
| Workspace root | `Cargo.toml` | 3-member workspace |
| Binary entry | `crates/clv-app/src/main.rs` | GPUI app bootstrap |
| App state | `crates/clv-app/src/app/state.rs` | Global state |
| Cleanup view | `crates/clv-app/src/views/cleanup.rs` | Cache scan + removal UI |
| Agent view | `crates/clv-app/src/views/agent.rs` | Agent project display |
| Startup view | `crates/clv-app/src/views/startup.rs` | Startup item management UI |
| Process view | `crates/clv-app/src/views/process.rs` | Process monitor UI |
| Dashboard | `crates/clv-app/src/views/dashboard.rs` | Overview/summary screen |
| Settings view | `crates/clv-app/src/views/settings.rs` | Configuration UI |
| Onboarding | `crates/clv-app/src/views/onboarding.rs` | First-run flow |
| Scanner | `crates/clv-core/src/scanner.rs` | Language-specific scan rules |
| Cleanup engine | `crates/clv-core/src/cleanup.rs` | File removal logic |
| Agent detection | `crates/clv-core/src/agent.rs` | Agent project classifier |
| Data models | `crates/clv-core/src/models.rs` | Shared types |
| Settings persistence | `crates/clv-core/src/settings.rs` | Config read/write |
| Platform startup | `crates/clv-platform/src/startup.rs` | OS startup items |
| Platform process | `crates/clv-platform/src/process.rs` | Process enumeration |