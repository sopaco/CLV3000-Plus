---
type: agent_context
project: clv3000-plus
title: Agent Architecture Context
source: .
---

## 项目概览

CLV3000 Plus 是一款原生桌面工具，用于回收开发者工作站的磁盘空间。它扫描已配置的开发目录及全局缓存，识别 13+ 技术栈的可清理构建产物，并通过回收区安全删除。核心差异化能力是检测 **AI 编码工具实验项目**（Claude/Cursor/Codex/Trae/OpenCode 残留），竞品无此功能。面向终端用户（macOS/Windows/Linux GPUI 桌面应用）和编码代理（本文档 + `agent/repomix.md`）。约束：纯本地无网络，删除操作受风险等级和受保护路径守护；UI 支持三语（zh/en/ja）。

## 架构设计

三层 Cargo 工作区，UI → 平台无关逻辑 → OS 抽象：

| Crate | 层级 | 职责 |
|---|---|---|
| `crates/clv-app` | 展示层 | GPUI 桌面应用：窗口 shell、`ProgressHud`、系统托盘、`AppStore` 状态实体、各页面视图（含大文件）、主题（Defender/Blossom/Neon/Aurora）、i18n、自定义控件 |
| `crates/clv-core` | 领域逻辑（纯） | 扫描规则引擎、清理引擎、大文件发现、AI 代理项目/会话检测、模型、设置持久化、路径安全 |
| `crates/clv-platform` | 平台适配 | 磁盘用量（`primary_disk_usage`、`list_disk_volumes`）、原生文件夹选择器（`pick_folders`）、进程枚举/终止（`sysinfo`）、OS 启动项辅助 |

- **状态流**：视图不持有扫描结果；单一 `AppStore` 实体持有 `AppSettings`、当前页面、最近 `ScanReport`、清理报告、`CleanupHistory`、`pending_cleanup_notification`、`status_message`、磁盘用量和扫描/清理取消句柄。视图订阅/观察 store。
- **异步任务**：`AppStore` 将后台扫描/清理委托给 `services/scan` 和 `services/cleanup`（工作线程 + mpsc）；GPUI executor 轮询 `ScanEvent` / `CleanupEvent`（含 `Progress`）回灌 store。
- **视图延迟单例**：`ClvApp` 首次访问时创建各页面视图并缓存 entity（`crates/clv-app/src/app/mod.rs`）。
- **进度 HUD**：`ProgressHud`（`hud.rs`）实体覆盖页面内容之上，渲染扫描/清理进度条及取消按钮；`AppStore::notify_progress_only` 通知挂载的 HUD。
- **系统托盘**：`TrayController`（`tray.rs`）安装托盘图标（Open/Scan/Quit）；`request_scan` / `take_scan_request` 桥接托盘 Scan 至 `AppStore::start_scan`；双击打开应用；tooltip 随磁盘刷新更新。
- **扫描异步可取消**（`AtomicBool`）：节流进度事件（`ScanEvent::Progress/Done`）经 `services/scan` 轮询进 store；`Scanner::scan_cancellable` 在 `scan_tree` 中内联收集大文件，可能设置 `sizes_truncated`，完成时通过 `save_last_scan` 持久化。
- **清理异步可取消**：逐项 `CleanupProgress` 事件（`CleanupEvent::Progress/Done`）经 `services/cleanup` 轮询；`ProgressHud` 驱动进度 UI；`Done` 时持久化 `CleanupHistory`（含 `TrashedEntry` 记录），按 `soft_delete_days` 清理过期回收区，排队完成通知；支持 `restore_trashed_entry`。
- **回收区与通知**：`CleanupHistory`（配置目录 90 天 JSON）记录每次运行的释放量和可恢复 `TrashedEntry`；Dashboard 显示 7/30 天趋势卡片；`ClvApp` 在清理完成时推送成功通知。
- **类型化 i18n 边界**：`clv-core` 存储 `RuleDescription` / `AgentReasonPart` 枚举（非展示字符串）；用户可见文本经 `RuleDescription::text(lang)` 或 `clv-app/i18n`（`rule_description_label`、`format_agent_reason`）解析。
- **依赖方向**：`clv-app` → `clv-core` + `clv-platform`；无反向边。

## 模块地图

| 模块 | 职责 | 主要路径 |
|---|---|---|
| 应用 shell 与路由 | 根组件、页面切换、延迟视图创建、`ProgressHud` + 状态栏、托盘动作/扫描分发、清理完成通知 | `crates/clv-app/src/app/mod.rs`、`hud.rs`、`shell.rs`、`tray.rs` |
| 全局状态 | `AppStore`（扫描/清理进度 + 取消、`CleanupHistory`/`TrashedEntry` 恢复、`pending_cleanup_notification`、`status_message`、磁盘总量）、页面状态、扫描/清理编排、文件夹选择器 | `crates/clv-app/src/app/state.rs`、`services/scan.rs`、`services/cleanup.rs` |
| 扫描器 | 基于规则的可清理项发现 + 全局缓存（含 Bun/Homebrew/Docker/浏览器缓存，非 Windows）+ 代理会话；locale 感知扫描阶段；可取消扫描（`should_skip_dir`）；内联大文件通道；消费 `project_rules` / `global_rules` | `crates/clv-core/src/scanner.rs`、`large_files.rs`、`settings/global_rules.rs`、`settings/project_rules.rs`、`locale.rs` |
| 清理引擎 | 通过健壮 `move_entry` 移入回收区（readonly 清除、跨设备复制回退），逐项 `CleanupProgress` 回调，`TrashedEntry` 追踪，`restore_trashed` / `purge_old_trash`，增强的 `CleanupReport`，`CleanupHistory` JSON 持久化（90 天修剪） | `crates/clv-core/src/cleanup.rs` |
| 代理检测 | AI 代理实验项目启发式 + 会话目标发现；结构化 `reason_parts`；跳过仅含活跃标记的仓库 | `crates/clv-core/src/agent.rs`、`agent_sessions.rs`、`messages/agent_reason.rs` |
| 本地化消息 | 类型化规则描述（`RuleDescription` R001–R140）+ 代理原因部分；翻译表 + 代码生成 | `crates/clv-core/src/messages/rule_description.rs`、`messages/agent_reason.rs`、`scripts/generate-rule-descriptions.py`、`scripts/rule-description-translations.json` |
| 模型 | `TechStack`（13+ 技术栈）、`RiskLevel`、`CleanupCategory`、`ScanReport` 类型（含 `large_files`、`cancelled`、`sizes_truncated`）；`ScanItem.description` 为 `RuleDescription` | `crates/clv-core/src/models.rs`、`category.rs` |
| 设置与路径 | 持久化、规则表（类型化描述）、标记定义、环境变量展开、受保护路径守卫、`soft_delete_days`、上次扫描 JSON | `crates/clv-core/src/settings/mod.rs`、`global_rules.rs`、`project_rules.rs`、`rule.rs`、`markers.rs`、`paths.rs` |
| 主题与 locale | `ThemePreference`（Defender/Blossom/Neon/Aurora）、`LanguagePreference`、`resolve_language`、扫描阶段文案 | `crates/clv-core/src/locale.rs` |
| 平台适配 | 磁盘用量 + 卷列表、原生文件夹选择器、进程枚举/终止、启动项 | `crates/clv-platform/src/disk.rs`、`dialog.rs`、`process.rs` |
| 页面视图 | Dashboard（健康评分、按需磁盘卷对话框、大文件磁贴、历史/恢复）、Cleanup、Agent、Large Files、Startup、Process（可见性感知轮询）、Settings、Onboarding | `crates/clv-app/src/views/*.rs` |
| i18n | 三语标签目录；解析 `RuleDescription` + `AgentReasonPart`；清理进度/状态/摘要/历史/通知/恢复、托盘、大文件、文件夹选择器文案 | `crates/clv-app/src/i18n/labels.rs`、`mod.rs` |
| UI 工具与资产 | 可复用控件、健康评分 `hero_banner` / `compute_health`、扫描/清理进度条（`ui/security.rs`）、列表组件、主题主题、图标、嵌入资产 | `crates/clv-app/src/ui/`、`theme.rs`、`assets.rs`、`build.rs` |

## 核心流程

1. **启动**：`main.rs` → 安装系统托盘（`TrayController`）→ 加载持久化设置 → 创建 `AppStore`（通过 `load_last_scan` 加载 `last_report`，后台 `purge_old_trash`）→ 若 `onboarding_done` 为 false 显示 Onboarding 页，否则显示 Dashboard → 启动异步磁盘刷新（`primary_disk_usage`）→ 更新托盘 tooltip；`ClvApp` 轮询 `take_scan_request` 处理托盘触发的扫描。
2. **扫描**：用户触发扫描 → `spawn_scan`（带取消标志）运行 `Scanner::scan_cancellable` 遍历配置根目录（剪枝嵌套匹配），解析全局缓存规则，发现代理会话目标 → 在 `scan_tree` 中收集大文件 → 发射节流的本地化 `ScanProgress` → `poll_scan` 将 `ScanReport`（items、`large_files`、`cancelled`、`sizes_truncated`）交付 `AppStore`；取消设置 `scan_restart_pending` 以便立即重新扫描；完成时 `save_last_scan` 持久化。
3. **清理**：CleanupView 按 bucket/risk 筛选分组列出报告项 → 用户选择 → `run_cleanup` → `spawn_cleanup`（带取消标志）运行清理引擎，逐项 `CleanupProgress` 回调，通过健壮 `move_entry` 移入回收区（跳过受保护路径）→ `poll_cleanup` 将进度交付 `ProgressHud` 然后 `CleanupPoll::Done`；报告追踪 `freed_bytes`、`success_count`、`failed`、`trashed_entries`；store 移除已清理路径，重新检测代理项目，刷新磁盘用量；追加 `CleanupHistoryRecord`（含 `TrashedEntry` 列表），设置 `pending_cleanup_notification`；Dashboard `history_card` 展示趋势并通过 `restore_trashed_entry` 支持恢复。
4. **代理项目**：AgentView 读取 `report.agent_projects`，应用搜索筛选（name/`reason_parts`/path/stack）；来自 Claude/Cursor/Codex/Trae/OpenCode 等的会话以结构化 `AgentReasonPart` 原因呈现，清理前按语言格式化。
5. **进程管理**：ProcessView 在页面显示/刷新触发时轮询 `clv-platform` 枚举器（可见性感知）→ 内存中搜索/排序进程列表 → 终止选中 PID。

## 技术选型

- 语言：Rust（edition 2024），工作区 resolver 2，release profile 含 thin LTO + strip
- UI：`gpui-kit` 0.6（Longbridge GPUI 工具箱）— 统一入口，含样式化组件、行为层、默认图标资源；所有 GPUI 基础类型（`div`/`px`/`Window`/`App`…）经 `gpui_kit::{…}` 访问
- 文件系统遍历：`walkdir`；进程/系统信息：`sysinfo`
- 持久化/配置：`serde`/`serde_json`、`directories`（XDG/home 布局）
- 工具库：`anyhow`、`thiserror`、`chrono`、`uuid`、`open`（Finder/Explorer 中显示）、`sys-locale`
- 原生对话框与托盘：`rfd`（文件夹选择器）、`tray-icon` + `muda`（系统托盘菜单）
- i18n 代码生成：`scripts/generate-rule-descriptions.py` 从 `scripts/rule-description-translations.json` 生成 `RuleDescription` 枚举
- 日志：`tracing` + `tracing-subscriber`（release 级别 warn）
- 打包：`scripts/bundle-macos.sh`；图标位于 `assets/icons/`

## 系统边界

- **纯本地文件系统** — 无网络调用、无遥测、无外部 API。
- **读侧**：主目录、配置扫描根目录（默认 `~/Projects`、Documents、Desktop…）、全局工具缓存（`~/.cargo`、npm/pip/Bun/Homebrew/Docker/Xcode derived data、浏览器缓存、Trae/OpenCode 应用缓存等）、代理 CLI 会话目录（`~/.trae/cli`、OpenCode data dirs；可通过 `TRAE_DIR`/`TRAEX_SESSIONS_DIR`/`OPENCODE_DIR` 覆盖）。
- **写侧**：应用管理的回收区目录（清理引擎）；配置目录中的 settings JSON、`cleanup_history.json` 和上次扫描 JSON。其他均为只读。
- **信任边界 — 受保护路径**：`paths.rs` 硬阻系统位置（Unix root/system 目录、Windows %SystemRoot% 变体）禁止任何删除操作；风险等级（`Safe`/`Caution`/`Protected`）门控 UI 操作；活跃项目可将 `Safe` 提升为 `Caution`。
- **OS 集成**：系统托盘图标（Open/Scan/Quit）；原生文件夹选择器用于扫描路径配置；通过 `sysinfo` 枚举/终止进程；通过 `open` crate 打开文件夹；StartupView 中展示登录/启动项。
- **第三方风险面**：`scanner.rs` 中的规则表决定可删除内容；嵌套匹配剪枝防止在匹配的父目录内删除项目源码。

## 代码映射索引

| 概念 | 位置 | 备注 |
|---|---|---|
| 入口 / 窗口引导 | `crates/clv-app/src/main.rs` | GPUI 应用启动；托盘安装；`TrayPending` 插槽 |
| 根组件与页面路由 | `crates/clv-app/src/app/mod.rs` | 延迟视图 entity；`ProgressHud` + 状态栏；托盘动作/扫描轮询；清理完成通知 |
| 进度 HUD | `crates/clv-app/src/app/hud.rs` | 扫描/清理进度条 + 取消；观察 `AppStore` |
| 中央状态 store | `crates/clv-app/src/app/state.rs` | `AppStore`、取消标志、清理进度/历史/恢复、`status_message`、`run_cleanup_safe`、`pick_scan_folders` |
| 系统托盘 | `crates/clv-app/src/tray.rs` | `TrayController`、`TrayAction`（Open/Scan/Quit）、`request_scan`/`take_scan_request`、tooltip 更新 |
| 异步扫描编排 | `crates/clv-app/src/services/scan.rs` | `spawn_scan` / `poll_scan`（取消 `AtomicBool`） |
| 异步清理编排 | `crates/clv-app/src/services/cleanup.rs` | `CleanupEvent`/`CleanupPoll`（含 `Progress`）+ 取消 |
| 扫描编排与规则 | `crates/clv-core/src/scanner.rs` | `scan_cancellable`、`ProgressThrottle`、`should_skip_dir`；内联大文件；规则在 `settings/` |
| 大文件发现 | `crates/clv-core/src/large_files.rs` | `finalize_large_files`、`LargeFileEntry`；在 `scan_tree` 中收集 |
| 删除引擎 | `crates/clv-core/src/cleanup.rs` | `CleanupProgress`、`TrashedEntry`、`restore_trashed`、`purge_old_trash`、健壮 `move_entry`、`CleanupHistory` |
| 代理项目/会话检测 | `crates/clv-core/src/agent.rs`、`agent_sessions.rs` | Trae/Trae CN/SOLO、TraeX、OpenCode 会话目标 |
| 类型化 i18n 消息 | `crates/clv-core/src/messages/`、`scripts/generate-rule-descriptions.py` | `RuleDescription` R001–R140、`AgentReasonPart`；JSON → codegen |
| 领域模型 | `crates/clv-core/src/models.rs`、`category.rs` | `TechStack`、`RiskLevel`、`CleanupCategory`、`ScanReport`（+ `large_files`、`cancelled`、`sizes_truncated`） |
| Locale 与扫描阶段 | `crates/clv-core/src/locale.rs` | `Language`、`LanguagePreference`、`ThemePreference`、`scan_phase_*` 辅助 |
| 路径安全与默认值 | `crates/clv-core/src/paths.rs` | 受保护路径守卫 |
| 设置持久化与规则 | `crates/clv-core/src/settings/mod.rs` | `AppSettings`、`soft_delete_days`、`load_last_scan`/`save_last_scan`、规则 |
| 主题系统 | `crates/clv-app/src/theme.rs` | `AppColors`、`ThemePreference` → 4 主题（Defender/Blossom/Neon/Aurora）、`apply_theme` |
| 平台磁盘用量 | `crates/clv-platform/src/disk.rs` | `primary_disk_usage`、`list_disk_volumes`、`DiskVolume`；macOS 挂载感知；Windows 多驱动器求和 |
| 平台文件夹选择器 | `crates/clv-platform/src/dialog.rs` | `pick_folders`（`rfd`） |
| 进程枚举 | `crates/clv-platform/src/process.rs` | sysinfo 封装；`ProcessCategory`、`ProcessEnumerator` |
| 功能页面 | `crates/clv-app/src/views/` | dashboard（健康评分、磁盘卷对话框、大文件磁贴、历史/恢复）/ cleanup / agent / large_files / process / startup / settings / onboarding |
| 标签翻译 | `crates/clv-app/src/i18n/labels.rs`、`mod.rs` | zh/en/ja；托盘、大文件、恢复/取消、文件夹选择器、健康评分辅助 |
| 样式与控件 | `crates/clv-app/src/ui/`、`theme.rs` | `compute_health`、`hero_banner`；`scan_progress_bar` / `cleanup_progress_bar` 在 `security.rs` |
| macOS 打包 | `scripts/bundle-macos.sh` | App bundle 构建 |
| 单元测试（规则、安全） | `crates/clv-core/src/lib.rs` | 扫描器/清理/路径/i18n 测试套件；`scan_cancellable` 和清理 `move_entry` 跨设备测试 |

*实现细节见 `.terrain/agent/repomix.md` — grep/read 获取签名和源码。*