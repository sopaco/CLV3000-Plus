# Settings and Rules Domain

**Module path:** `crates/clv-core/src/settings/`, `crates/clv-core/src/paths.rs`  
**Generated:** 2026-08-28

---

## What This Module Does

Settings and rules are the app's rulebook—it defines *what* to scan, *where* to scan, and *how dangerous* each finding is. User preferences (`AppSettings`) persist as JSON, while cleanup rules are compile-time tables mapping directory patterns to typed descriptions, risk levels, and technology stacks.

Changing what the app cleans usually means editing rule tables here, not scanner logic.

---

## Core Capabilities

1. **Settings persistence** — `load_settings` / `save_settings` to XDG config path (`settings/mod.rs:53-87`).

2. **Project cleanup rules** — `project_rules()` returns hundreds of patterns for 13+ stacks (`project_rules.rs`).

3. **Global cache rules** — Home-relative paths for Cargo, npm, Docker, Homebrew, browser caches (`global_rules.rs`).

4. **Marker definitions** — Project markers (`package.json`, `Cargo.toml`) and agent markers (`.cursor`, `AGENTS.md`) in `markers.rs`.

5. **Path safety** — `is_protected_system_path`, `default_scan_paths`, env expansion in `paths.rs`.

---

## Key Components

| Component | File path | Responsibility |
|-----------|-----------|----------------|
| `AppSettings` | `crates/clv-core/src/settings/mod.rs:18` | User preference struct |
| `CleanupRule` | `crates/clv-core/src/settings/rule.rs` | Rule matcher definition |
| `project_rules` | `crates/clv-core/src/settings/project_rules.rs` | Project artifact patterns |
| `global_cache_rules` | `crates/clv-core/src/settings/global_rules.rs` | Global cache locations |
| `resolve_global_path` | `crates/clv-core/src/paths.rs:52` | Home/env path resolution |
| `is_protected_system_path` | `crates/clv-core/src/paths.rs:165` | Deletion blocklist |
| `default_scan_paths` | `crates/clv-core/src/paths.rs:8` | Default project roots |
| `load_last_scan` / `save_last_scan` | `crates/clv-core/src/settings/mod.rs:100` | Scan snapshot persistence |

---

## CleanupRule Structure

Each rule specifies:
- `relative` path or name pattern
- `stack: TechStack` and `risk: RiskLevel`
- `category: CleanupCategory` for bucket mapping
- `description: RuleDescription` typed enum ID
- Optional: `requires_marker`, `relative_prefix`, `requires_parent`, `global` flag

---

## Cross-Module Interactions

| Module | Direction | Interface | Notes |
|--------|-----------|-----------|-------|
| Scanner | Consumed by | `project_rules()`, `global_cache_rules()` | Read-only at scan time |
| Cleanup | Consumed by | `trash_dir()`, `AppSettings` flags | Soft delete behavior |
| AppStore | Loads/saves | `load_settings`, `save_settings` | Settings page edits |
| Messages | References | `RuleDescription` per rule | i18n lookup |

---

## Implementation Highlights

- `parse_scan_paths` / `format_scan_paths` for Settings textarea editing (`settings/mod.rs:63-76`).
- `soft_delete_days` controls both trash purge and user expectation messaging.
- Rule tests in `lib.rs:134-183` validate prefix and marker matching logic.
