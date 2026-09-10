# Agent — AI 工具项目识别

## 这个模块在做什么

Agent 检测模块是 CLV3000 Plus 的"侦探"。在 AI 编程工具爆发式增长的今天，开发者经常在不同时期使用 Cursor、Claude Code、Codex、Windsurf、Trae 等工具，每个工具都会在用户目录下留下自己的"足迹"——配置目录、会话历史、缓存文件、实验性项目。时间一长，这些足迹混杂在一起，用户自己都分不清哪些是真正需要的、哪些是早就废弃的。

这个侦探的工作方式分为两步。第一步是"指纹识别"——`agent_path_signals()` 检查一个目录是否包含 AI 工具的特征：目录名中是否含有 `claude`、`cursor`、`codex` 等 16 个关键词模式（`agent_name_patterns()`），或者目录下是否存在 `.cursor`、`.claude`、`AGENTS.md` 等 10 个标记文件（`agent_marker_files()`）。第二步是"行为分析"——`detect_agent_projects()` 将 Scanner 收集的候选项按 `project_root` 分组，对每组计算不活跃天数（`days_inactive`），然后用两条启发式规则判定：

1. **名称匹配的实验文件夹**（如名字含 `claude` 的临时目录）始终被视为 AI 项目；
2. **仅靠标记文件匹配的项目**（如一个真实仓库 adopted 了 `.cursor`），只有当不活跃超过 14 天（`MARKER_INACTIVE_DAYS`）或所有候选项都超过 30 天未修改（zombie 项目）时，才会被归类为 AI 项目遗留物。

这种"宽松入口、严格判定"的策略，就像侦探办案时宁可多查也不轻易下结论——真正活跃使用 AI 工具的仓库不会被误判为"垃圾"。

此外，`agent_sessions.rs` 是另一个维度的发现器：它不依赖 Scanner 的目录遍历，而是直接根据已知路径模式（环境变量 + 默认位置）定位 AI 工具的会话存储目录，生成独立的 `AgentSessionTarget` 列表供 Scanner 量化。

## 核心功能点

1. **名称模式匹配** — `agent_path_signals()` 对目录名做小写化后，与 `agent_name_patterns()` 中的 16 个模式（`claude`、`cursor`、`codex`、`workbuddy`、`work-buddy`、`agent`、`generated`、`aider`、`copilot`、`windsurf`、`trae`、`opencode`、`devin`、`bolt`、`v0`、`replit`）逐一比对，命中即返回 `AgentReasonPart::NameContainsPattern`。
2. **标记文件检测** — 检查目录下是否存在 `.agents`、`.cursor`、`.claude`、`.aider`、`.copilot`、`.windsurf`、`.trae`、`.opencode`、`AGENTS.md`、`CLAUDE.md` 等 10 个标记文件，命中返回 `AgentReasonPart::HasAgentMarker`。
3. **不活跃判定** — 14 天阈值（`MARKER_INACTIVE_DAYS`）用于标记文件匹配的项目；30 天阈值用于 zombie 检测（所有候选项均超过 30 天未修改）。两条规则分别产生 `AgentReasonPart::LongUnusedProject` 和 `AgentReasonPart::InactiveOver30Days`。
4. **AI 会话目录发现** — `discover_agent_session_targets()` 硬编码了 10+ 种 AI 工具的会话/缓存路径模式（Codex sessions、Claude projects、Cursor CachedData/GPUCache/Code Cache 等），支持环境变量覆盖（`CODEX_HOME`、`CLAUDE_CONFIG_DIR`、`TRAE_DIR`、`OPENCODE_DIR` 等）。
5. **多平台路径解析** — `cursor_data_roots()`、`windsurf_data_roots()`、`trae_app_data_roots()` 等函数通过 `#[cfg]` 属性根据编译目标平台（macOS/Windows/Linux）返回不同的默认路径。
6. **项目技术栈推断** — `detect_agent_projects()` 调用 `detect_project_stacks()` 探测 AI 项目根目录下的技术栈标记文件，如果推断不出则从候选项的 `stack` 字段聚合，最终默认归为 `TechStack::Agent`。
7. **环境变量路径覆盖** — 所有工具路径都支持环境变量覆盖（如 `CODEX_HOME`、`CLAUDE_CONFIG_DIR`），便于非标准安装的用户使用。

## 关键组件

| 组件/类型 | 文件路径 | 一句话职责 |
|-----------|----------|-----------|
| `AgentProject` | `crates/clv-core/src/models.rs:104` | AI 项目聚合实体，含路径、总大小、技术栈、不活跃天数和识别原因 |
| `AgentReasonPart` | `crates/clv-core/src/messages/agent_reason.rs` | 结构化识别原因枚举（NameContainsPattern、HasAgentMarker、LongUnusedProject、InactiveOver30Days） |
| `AgentSessionTarget` | `crates/clv-core/src/agent_sessions.rs:7` | AI 工具会话目录描述，含路径、技术栈、风险等级和清理类别 |
| `detect_agent_projects` | `crates/clv-core/src/agent.rs:12` | 核心分组函数，将候选项按 root 分组后判定哪些属于 AI 项目 |
| `agent_path_signals` | `crates/clv-core/src/scanner.rs:611` | 路径信号分析函数，返回名称匹配和标记文件匹配结果 |
| `is_agent_project_path` | `crates/clv-core/src/scanner.rs:639` | 快捷判定函数，组合名称和标记文件两种信号 |
| `discover_agent_session_targets` | `crates/clv-core/src/agent_sessions.rs:17` | 会话目录发现函数，返回所有已知 AI 工具的会话/缓存路径 |
| `agent_name_patterns` | `crates/clv-core/src/settings/markers.rs:3` | 16 个 AI 工具名称关键词的静态列表 |
| `agent_marker_files` | `crates/clv-core/src/settings/markers.rs:24` | 10 个 AI 工具标记文件名的静态列表 |

## 内部数据流

```mermaid
flowchart TD
    A["Scanner::scan_tree()"] --> B["maybe_record_agent_root()<br/>检测标记文件"]
    B --> C["agent_roots: HashSet"]

    A --> D["Scanner 生成 ScanItem 列表"]
    D --> E["Scanner::scan_cancellable()<br/>最终阶段"]

    C --> E
    E --> F["detect_agent_projects()<br/>(items, known_agent_roots)"]

    F --> G["by_root 分组<br/>HashMap<PathBuf, Vec<ScanItem>>"]
    G --> H{"对每个 root"}
    H --> I["agent_path_signals()<br/>名称 + 标记文件检测"]
    H --> J["detect_project_stacks()<br/>技术栈推断"]
    H --> K{"不活跃判定"}

    K -->|名称匹配| L["始终纳入"]
    K -->|标记匹配 + 14天| L
    K -->|zombie(30天)| L
    K -->|其他| M["排除"]

    L --> N["AgentProject<br/>(路径 + 原因 + items)"]
    N --> O["按 total_bytes 降序排序"]
    O --> P["ScanReport.agent_projects"]

    Q["agent_sessions::<br/>discover_agent_session_targets()"] --> R["AgentSessionTarget 列表"]
    R --> S["Scanner::try_add_agent_session()<br/>量化并生成 ScanItem"]
    S --> D
```

## 关键接口与扩展点

- **`detect_agent_projects(items, known_agent_roots)`** — 核心 API，接受扫描候选项和已知 AI 根目录列表，返回排序后的 `Vec<AgentProject>`。新的判定逻辑只需在函数内部扩展 `if` 条件。
- **`agent_path_signals(path)`** — 路径分析 API，返回 `(name_hit, marker_hit, Vec<AgentReasonPart>)`。新增识别维度（如检查文件内容）只需在此函数中添加逻辑。
- **`discover_agent_session_targets()`** — 会话发现 API，返回 `Vec<AgentSessionTarget>`。新增 AI 工具支持只需添加路径解析函数和 `push_dir` 调用。
- **`agent_name_patterns()` / `agent_marker_files()`** — 静态配置列表，位于 `settings/markers.rs`。新增工具支持只需追加条目。
- **`AgentReasonPart`** — 结构化原因枚举，UI 层通过 `crates/clv-app/src/i18n/labels.rs` 中的翻译函数将其格式化为用户可见文案。

## 与其他模块的交互

| 交互模块 | 交互内容 | 说明 |
|----------|----------|------|
| `scanner` | `agent_path_signals()`, `detect_project_stacks()`, `maybe_record_agent_root()` | Scanner 在遍历过程中收集 AI 信号，Agent 模块提供信号检测函数 |
| `models` | `AgentProject`, `ScanItem`, `TechStack` | Agent 模块的输入和输出数据结构 |
| `messages` | `AgentReasonPart` | 结构化识别原因，由 Agent 模块产生，UI 层消费 |
| `settings` | `agent_name_patterns()`, `agent_marker_files()` | 名称模式和标记文件的静态配置列表 |
| `app-state` | `AppStore` 中的 `last_report.agent_projects` | UI 层读取 AI 项目列表用于展示和操作 |

## 跨模块协作场景

**场景一：AI 项目自动识别**
Scanner 在遍历用户配置的扫描目录时，通过 `maybe_record_agent_root()` 发现 `~/projects/claude-test/.cursor` 文件，将 `~/projects/claude-test` 加入 `agent_roots`。扫描完成后，`detect_agent_projects()` 将该项目分组，发现名称含 `claude`（命中 `agent_name_patterns`），直接归类为 `AgentProject` 并标记 `AgentReasonPart::NameContainsPattern("claude")`。

**场景二：活跃仓库不被误判**
一个真实项目仓库 `~/dev/my-app` 因团队规范引入了 `.cursor` 配置。Scanner 在遍历时记录该目录为 `agent_root`。`detect_agent_projects()` 检查发现项目最近 3 天有修改（`is_likely_active_project` 返回 true），不满足 14 天不活跃阈值，因此不会被归类为 AI 项目。用户不会看到这个活跃仓库出现在"AI 工具遗留物"列表中。

**场景三：会话目录独立发现**
`discover_agent_session_targets()` 直接根据已知路径模式（如 `~/.claude/projects`、`~/Library/Application Support/Cursor/CachedData`）生成 `AgentSessionTarget` 列表。这些目标不经过 Scanner 的目录遍历，而是在 `try_add_agent_session()` 中独立量化后生成 `ScanItem`，补充了项目级规则无法覆盖的全局会话缓存。

## 性能考量

- **不走磁盘遍历**：`detect_agent_projects()` 完全基于 Scanner 已收集的候选项工作，不会发起额外的文件系统遍历。这是一个纯内存操作。
- **阈值常量化**：`MARKER_INACTIVE_DAYS = 14` 和 zombie 检测的 30 天阈值定义为编译期常量，运行时零开销。
- **HashMap 分组**：使用 `HashMap<PathBuf, Vec<ScanItem>>` 按 `project_root` 分组，O(n) 时间复杂度完成分组，避免嵌套循环。
- **环境变量懒解析**：`codex_home_paths()`、`claude_config_paths()` 等函数在 `discover_agent_session_targets()` 调用时才解析环境变量，不占用模块加载时间。
- **排序策略**：`detect_agent_projects()` 按 `total_bytes` 降序排序返回结果，让 UI 层默认展示最大的项目在前，用户无需滚动即可发现主要清理目标。
