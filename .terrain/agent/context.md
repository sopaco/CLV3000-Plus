---
type: agent_context
project: clv3000-plus
title: Agent Architecture Context
source: .
---

## 项目概览
CLV3000 Plus 是面向开发者的原生桌面磁盘清理应用。应用扫描用户配置的开发目录与全局工具缓存，基于预定义规则识别多技术栈构建产物、依赖与 AI 编码工具残留，并通过回收区机制安全清理。目标用户为本地运行的 Rust、Node、Python 等开发者；约束为纯本地文件系统操作、无后端服务、无数据库、仅支持桌面单机运行。

## 架构设计
采用 Rust 工作区三层分工：`clv-core` 为纯领域层，`clv-platform` 为操作系统抽象层，`clv-app` 为表现与编排层。依赖方向为 `clv-app` 依赖 `clv-core` 与 `clv-platform`，核心业务不依赖 GPUI，保证扫描、清理、Agent 识别可独立测试。

| 层 | 容器 | 职责 | 主要依赖 |
|----|------|------|----------|
| 表现层 | `crates/clv-app` | 窗口、托盘、视图、主题、国际化文案、与用户交互 | `clv-core`, `clv-platform`, `gpui-kit 0.6` |
| 领域层 | `crates/clv-core` | 扫描规则、分桶、清理、Agent 会话、消息与分类 | 无 UI 依赖 |
| 平台层 | `crates/clv-platform` | 磁盘、进程、原生对话框 | 系统 API |

应用内运行时分工：`app/` 管理全局状态与窗口外壳，`views/` 渲染各功能页面，`services/` 编排扫描与清理耗时任务，`i18n/` 与 `theme.rs` 提供多语言与主题。`AGENTS.md` 约定的 GPUI 接入、对话框、按钮、主题令牌等 UI 约束均在 `clv-app` 落地。

## 模块地图
| 模块 | 职责 | 主要路径 |
|------|------|----------|
| 应用入口与外壳 | 启动、主窗口、菜单、托盘、窗口生命周期 | `crates/clv-app/src/main.rs`, `crates/clv-app/src/app/shell.rs`, `crates/clv-app/src/tray.rs`, `crates/clv-app/src/platform.rs` |
| 应用状态 | 跨页面状态、选中标识、扫描会话状态 | `crates/clv-app/src/app/state.rs`, `crates/clv-app/src/app/mod.rs`, `crates/clv-app/src/actions.rs` |
| 视图层 | 仪表盘、清理、Agent、大文件、进程、设置、新手引导 | `crates/clv-app/src/views/`, `crates/clv-app/src/ui/` |
| 服务编排 | 扫描与清理后台任务调度 | `crates/clv-app/src/services/scan.rs`, `crates/clv-app/src/services/cleanup.rs` |
| 国际化与主题 | 三语文案、规则描述标签、主题色板与按钮令牌 | `crates/clv-app/src/i18n/labels.rs`, `crates/clv-app/src/theme.rs`, `crates/clv-app/src/prelude.rs` |
| 扫描引擎 | 遍历扫描根、应用规则、生成可清理项 | `crates/clv-core/src/scanner.rs`, `crates/clv-core/src/models.rs`, `crates/clv-core/src/paths.rs` |
| 清理与分类 | 风险分级、分桶、回收区清理 | `crates/clv-core/src/cleanup.rs`, `crates/clv-core/src/category.rs` |
| 清理规则 | 项目级与全局级规则表、规则标记 | `crates/clv-core/src/settings/project_rules.rs`, `crates/clv-core/src/settings/global_rules.rs`, `crates/clv-core/src/settings/rule.rs`, `crates/clv-core/src/settings/markers.rs` |
| Agent 识别 | AI 工具项目与会话缓存识别、原因结构化 | `crates/clv-core/src/agent.rs`, `crates/clv-core/src/agent_sessions.rs`, `crates/clv-core/src/messages/agent_reason.rs` |
| 描述与消息 | 规则描述枚举、三语文案生成 | `crates/clv-core/src/messages/rule_description.rs`, `crates/clv-core/src/messages/mod.rs`, `scripts/rule-description-translations.json`, `scripts/generate-rule-descriptions.py` |
| 平台抽象 | 磁盘用量、进程、原生对话框 | `crates/clv-platform/src/disk.rs`, `crates/clv-platform/src/process.rs`, `crates/clv-platform/src/dialog.rs`, `crates/clv-platform/src/lib.rs` |
| 大文件与语言 | 大文件分析、语言环境 | `crates/clv-core/src/large_files.rs`, `crates/clv-core/src/locale.rs` |

## 核心流程
1. **启动、设置与扫描**
   - 首次启动进入新手引导，配置扫描根目录；主窗口常驻托盘与主窗口双入口。
   

- `services/scan.rs` 启动后台扫描，调用 `clv-core::Scanner` 遍历各根目录并命中 `settings/*_rules` 规则。
   - 结果按 `CleanupCategory` 与 `RiskLevel` 分桶送入 `AppStore`，视图层展示仪表盘与清理列表。

2. **安全清理**
   - 用户在清理视图勾选条目，选中标识维护于 `AppStore.selected_item_ids`。
   

- `services/cleanup.rs` 调用 `cleanup.rs` 校验风险分级后移入回收区，不做永久删除。
   - 清理结果回写状态，磁盘用量经 `clv-platform::disk` 刷新；需二次确认时经 `open_alert_dialog` 呈现确定与取消按钮。

3. **AI 代理残留检测**
   - 扫描过程同步调用 `agent.rs` 识别编码工具项目，命中 `agent_sessions.rs` 预定义 Agent 路径。
   - 识别原因以 `AgentReasonPart` 结构化保存，`agent_reason.rs` 按当前语言格式化展示来源。
   - 残留项目作为独立分桶进入清理视图，支持按 Agent 来源筛选与批量清理。

4. **大文件与进程辅助分析**
   

- `large_files.rs` 对扫描根做大文件聚合，视图层呈现大文件榜单，供用户定位非规则占用。
   

- `clv-platform::process` 辅助展示占用进程上下文，用于排查构建产物被占用的场景。

## 技术选型
- **语言与工作区**：Rust Workspace，`Cargo.toml` 统一管理 `clv-app`、`clv-core`、`clv-platform` 三个 crate。
- **桌面框架**：`gpui-kit 0.6` 提供 GPUI 本体、样式组件与行为层，自定义 action 经 `gpui_kit::actions`，图标经 `gpui_kit::assets::Assets`。
- **规则与国际化**：`RuleDescription` 枚举加三语翻译表 `scripts/rule-description-translations.json`，经 `scripts/generate-rule-descriptions.py` 生成；中文为源语言，英文与日文为人工维护翻译。
- **跨平台**：macOS 与 Windows 双目标，全局规则通过条件编译分平台；构建打包经 `scripts/bundle-macos.sh`。
- **安全删除**：移入系统回收区而非永久删除；风险分级控制清理入口。

## 系统边界
- **本地文件系统**：读开发目录与全局缓存，写回收区；唯一外部副作用。无网络、无服务端、无数据库、无遥测。
- **操作系统集成**：主窗口、系统菜单、托盘菜单与重开行为、原生文件对话框、进程与磁盘信息查询。
- **信任边界**：扫描根与已选清理路径均为用户配置；清理范围限定在规则命中目录，不跨根递归删除。无第三方云端 API 依赖，离线可用。
- **维护边界**：规则描述枚举为自动生成，禁止手改；新增清理规则须先改规则源与翻译表，再运行生成脚本。

## 代码映射索引
| 概念 | 位置 | 备注 |
|------|------|------|
| 扫描引擎与扫描项 | `crates/clv-core/src/scanner.rs`, `crates/clv-core/src/models.rs` | `ScanItem.description` 为枚举而非字符串 |
| 清理分桶与类别 | `crates/clv-core/src/cleanup.rs`, `crates/clv-core/src/category.rs` | `CleanupBucket` 按类别驱动 |
| 项目级清理规则 | `crates/clv-core/src/settings/project_rules.rs` | 项目内构建产物与依赖 |
| 全局清理规则 | `crates/clv-core/src/settings/global_rules.rs` | 工具缓存，分平台条件编译 |
| Agent 会话路径 | `crates/clv-core/src/agent_sessions.rs` | Agent 工具会话与缓存目录 |
| Agent 识别与原因 | `crates/clv-core/src/agent.rs`, `crates/clv-core/src/messages/agent_reason.rs` | 结构化原因，UI 层按语言格式化 |
| 规则描述枚举 | `crates/clv-core/src/messages/rule_description.rs` | 自动生成，禁止手改 |
| 翻译表与生成脚本 | `scripts/rule-description-translations.json`, `scripts/generate-rule-descriptions.py` | 键顺序决定枚举编号，新增条目追加末尾 |
| UI 文案 | `crates/clv-app/src/i18n/labels.rs` | 分类与描述标签解析 |
| 扫描与清理服务 | `crates/clv-app/src/services/scan.rs`, `crates/clv-app/src/services/cleanup.rs` | 后台任务编排 |
| 主题与按钮令牌 | `crates/clv-app/src/theme.rs` | 令牌须跟随调色板回算 |
| 平台磁盘与进程 | `crates/clv-platform/src/disk.rs`, `crates/clv-platform/src/process.rs` | 仅供用量与进程上下文 |
| 原生对话框 | `crates/clv-platform/src/dialog.rs` | 确认框须用告警对话框形态 |
| 入口与托盘 | `crates/clv-app/src/main.rs`, `crates/clv-app/src/tray.rs` | 主窗口句柄与生命周期 |