# Agents Guide

This file guides AI coding agents working in this repository.

## 清理规则与国际化（必读）

本项目的扫描/清理规则使用**类型化 ID + 三语翻译表**，不要在领域层硬编码展示文案。

### 架构要点

| 概念 | 位置 | 说明 |
|------|------|------|
| `CleanupCategory` | `crates/clv-core/src/category.rs` | 规则分类枚举，驱动 `CleanupBucket` 分桶 |
| `RuleDescription` | `crates/clv-core/src/messages/rule_description.rs` | 规则/会话描述 ID（`R001`…`R133`），**自动生成** |
| 三语翻译表 | `scripts/rule-description-translations.json` | 中文 → `[English, 日本語]`，**人工维护** |
| 生成脚本 | `scripts/generate-rule-descriptions.py` | 由翻译表生成 `RuleDescription` 枚举 |
| 规则表 | `crates/clv-core/src/settings/project_rules.rs`、`global_rules.rs` | 项目级清理规则 |
| Agent 会话路径 | `crates/clv-core/src/agent_sessions.rs` | Agent 工具会话/缓存目录 |
| Agent 识别原因 | `crates/clv-core/src/messages/agent_reason.rs` | `AgentReasonPart` 结构化原因，UI 层按语言格式化 |
| UI 文案 | `crates/clv-app/src/i18n/labels.rs` | `scan_category_label`、`rule_description_label` 等 |

**依赖方向**：`clv-core` 只存 `RuleDescription` / `CleanupCategory` 等枚举；用户可见字符串在 `RuleDescription::text(lang)` 或 `clv-app/i18n` 中解析。`ScanItem.description` 是 `RuleDescription`，不是 `String`。

### 新增或修改清理规则

1. **在规则源文件中添加条目**（三处之一）：
   - `settings/project_rules.rs` — 项目内构建产物/依赖
   - `settings/global_rules.rs` — 全局工具缓存（注意 `#[cfg(target_os = "windows")]` 分平台）
   - `agent_sessions.rs` — Agent 会话/缓存目录

   规则写法示例（描述先用**中文占位**，下一步会替换为枚举）：

   ```rust
   CleanupRule::project(
       "target",
       TechStack::Rust,
       RiskLevel::Safe,
       CleanupCategory::CompileCache,
       "Rust 编译产物与增量缓存，可重新 cargo build 生成",
   )
   ```

2. **在翻译表中追加三语**（`scripts/rule-description-translations.json`）：

   ```json
   "Rust 编译产物与增量缓存，可重新 cargo build 生成": [
     "Rust build artifacts and incremental cache; rerun cargo build to regenerate",
     "Rust ビルド成果物と増分キャッシュ。cargo build で再生成できます"
   ]
   ```

   - JSON **键顺序**决定 `R00N` 编号；新增条目请追加在文件末尾，避免打乱已有 ID。
   - 英文不得混入中文（CJK）；日文需与中文语义一致。

3. **运行生成脚本**（仓库根目录）：

   ```bash
   # 新规则仍用中文字符串时：生成枚举并 patch 源文件为 RuleDescription::Rxxx
   python3 scripts/generate-rule-descriptions.py --patch

   # 仅刷新 rule_description.rs（源文件已是 RuleDescription::Rxxx 时）
   python3 scripts/generate-rule-descriptions.py
   ```

4. **验证**：

   ```bash
   cargo test -p clv-core
   ```

   `rule_description_translations_avoid_mixed_language` 会检查所有英文翻译不含 CJK。

### 修改已有规则的描述文案

1. 在 `scripts/rule-description-translations.json` 中修改对应中文键的 en/ja。
2. 运行 `python3 scripts/generate-rule-descriptions.py`（无需 `--patch`）。
3. 运行 `cargo test -p clv-core`。

不要直接编辑 `rule_description.rs`（文件头标注 AUTO-GENERATED）。

### Agent 项目识别原因

新增识别逻辑时，在 `agent.rs` / `scanner.rs` 使用 `AgentReasonPart` 枚举（如 `NameContainsPattern`、`LongUnusedProject`），在 `agent_reason.rs` 补充三语文案。不要在 `AgentProject` 上存硬编码中文字符串。

### 禁止事项

- 不要在 `models.rs`、`CleanupBucket` 等领域类型上添加 `label()` / `hint()` 等展示方法。
- 不要用中文 `category` 字符串做业务逻辑判断（使用 `CleanupCategory`）。
- 不要在 `ScanItem` 上存放 UI 选中状态（选中 ID 在 `AppStore.selected_item_ids`）。
- 不要手写或启发式生成 `rule_description.rs` 中的翻译。

<!-- terrain:begin env-overview v4 -->
## AI 工程环境（Terrain）

本仓库由 Terrain 配置了 AI 工程环境。Coding Agent 请遵循以下约定：

- **知识资产**位于本仓库 **`.terrain/`**（Agent 友好的知识资产、人类友好的知识库、私域知识、源码索引；可随 Git 协作）
- **项目登记**在本地 `~/.terrain/registry.json`（仅记录仓库路径，不含知识正文）
- **Skills** 位于 `.agents/skills/` 与 `.claude/skills/`（由 Terrain 注入，可按需重新集成）
- **Agent 工具**约定在 `~/.terrain/bin/`（`rtk` / `codegraph` / `terrain`）；可选本地清单 `.terrain/env/agent-tools.json`（不入库）
- **无 Terrain 安装**时：RTK / CodeGraph 可降级为 `bunx` / `npx`（见 `rtk-skill`、`codegraph-skill`）
- **工作流**：先读知识 → 再查关系 → 最后读源码；shell 输出优先走 RTK
<!-- terrain:end env-overview -->

<!-- terrain:begin knowledge-guide v5 -->
## Terrain 知识资产

Coding Agent **必须先加载** `terrain-knowledge-skill`，并按其中分层策略查询 **`.terrain/`**（仓库内路径，非全局目录）。

| 层级 | 路径 | 何时使用 |
|------|------|----------|
| Agent 友好 | `.terrain/agent/context.md` | 模块划分、核心流程、系统边界 |
| 私域 | `.terrain/knowledge/` | 业务术语、内部框架/API/脚手架 |
| 人类友好 | `.terrain/human/` | Litho 人类友好的知识库（可选参考） |
| 源码 | `.terrain/agent/repomix.md`（见 `repomix-context-skill`） | 实现细节（本地索引，不入库） |
| 关系 | codegraph CLI（见 `codegraph-skill`） | 调用链、依赖关系、影响分析 |

**原则**：先宏观后微观；优先读已生成文档，再 grep 源码索引。

## 知识资产的 Git 协作规则（必读）

`.terrain/` 的 Git 策略由 **`.terrain/.gitignore`** 与 **`.terrain/.gitattributes`** 声明（Terrain 生成并维护，随仓库分发）。

| 类别 | 位置 | Git 处理 |
|------|------|----------|
| 人为维护的私域知识 | `knowledge/` | 入库，正常三方合并 |
| 生成的知识文档 | `agent/context.md`、`human/`、`index.md` | 入库，但 **`-merge`：禁用自动合并** |
| 本机衍生物 | `agent/repomix*`、`agent/meta*.json`、`.meta/`、`env/`、`.litho-agent/`、`.sdd-agent/` | **不入库**，由 scan 本地重建 |

- **不要**把本机衍生物 `git add -f` 进版本库；它们体积大、含时间戳与 baseline git HEAD，入库必然产生冲突。
- `agent/context.md`、`human/**` 由 LLM 生成，**非确定性** —— 同一份代码两次生成措辞与结构都不同。冲突时**不要手工合并**（合并结果会是"既不是 A 也不是 B"的自相矛盾文档）：保留任一版本结束冲突，然后重新运行 Terrain scan 基于合并后的代码重生成。
- 建议知识资产的刷新集中在主干分支（或 CI）进行，feature 分支不提交 `.terrain/agent/`、`.terrain/human/` 的改动 —— 每个分支各带一份生成结果是冲突的结构性来源。`freshness` 本身就能表达"资产落后于代码"，不必每个分支都刷。

## 知识保鲜（必读）

1. 回答架构/模块问题前，优先执行 `~/.terrain/bin/terrain tools freshness --project <slug>`（或 `bunx @terrain-ai/cli tools freshness --project <slug>`）——该命令会按需重算并回写 `.terrain/.meta/freshness.json`，**不要**只静态读取该文件：它是本地缓存的快照，只在有人显式触发重算时才会更新，可能已经落后于当前 HEAD。CLI 不可用时才降级为直接读取该文件。
2. `freshness_score < 70` 时：不得仅凭 `agent/context.md` 下结论，须用 `grep repomix` 或 `codegraph` 交叉验证
3. `freshness_score < 50` 时：宏观架构上下文不可信，以 repomix 源码切片为准
4. 发现矛盾时的优先级：**repomix 源码 > codegraph > agent/context.md > human/**
5. `knowledge/` 私域文档视为人为维护；若 `refs` 指向的源码路径已删除，应降权处理
6. **CodeGraph 的 `<cg> status` 可能误报"最新"**（观察到索引 10 天未更新、期间 24 个提交改了源码，`status` 仍报正常，`query` 却查不到新符号）。做 impact/callers 分析前，先跑 `~/.terrain/bin/terrain tools codegraph-drift --project <slug>` 做独立的基于 git 的交叉验证；`likely_stale: true` 时先 `<cg> sync` 再查询（见 `codegraph-skill`）。
<!-- terrain:end knowledge-guide -->

<!-- terrain:begin skills v2 -->
### 可用 Skills

| Skill | 用途 |
|-------|------|
| `terrain-knowledge-skill` | `.terrain/` 知识分层与查询顺序（先读） |
| `repomix-context-skill` | grep/读取 `repomix.md` 源码切片 |
| `codegraph-skill` | 符号关系；`~/.terrain/bin/codegraph` 或 `bunx codegraph` |
| `rtk-skill` | 冗长 shell 加 rtk 前缀；`~/.terrain/bin/rtk` 或 `bunx @terrain-ai/rtk` |

加载顺序建议：knowledge → codegraph / repomix → rtk（执行命令时）。
<!-- terrain:end skills -->

<!-- terrain:begin tools v3 -->
### 工具链

| 工具 | 约定路径 | 无 Terrain 时降级 |
|------|----------|-------------------|
| RTK | `~/.terrain/bin/rtk` | `bunx @terrain-ai/rtk` 或 `npx @terrain-ai/rtk` |
| CodeGraph | `~/.terrain/bin/codegraph` | `bunx codegraph` 或 `npx codegraph` |
| Terrain CLI | `~/.terrain/bin/terrain` | `bunx @terrain-ai/cli` 或 `npx @terrain-ai/cli` |
| 知识文件 | `.terrain/` 仓库内路径 | 直接 Read/Grep，无需 CLI |

| 场景 | 用法 |
|------|------|
| 架构、私域知识 | 加载 `terrain-knowledge-skill` |
| 源码片段 | `repomix-context-skill`；`<rtk> grep` 搜索 pack |
| 符号关系 | `codegraph-skill`；检查 `~/.terrain/bin/codegraph` 是否存在（见 codegraph-skill） |
| git/test/build | `rtk-skill`；检查 `~/.terrain/bin/rtk` 是否存在（见 rtk-skill） |
| ACP 知识查询 | `~/.terrain/bin/terrain tools …` |
| 知识保鲜重算（自愈，勿只读静态 JSON） | `~/.terrain/bin/terrain tools freshness --project <slug>` |
| CodeGraph 独立过期检测（`<cg> status` 不可信时） | `~/.terrain/bin/terrain tools codegraph-drift --project <slug>` |

### Agent 工具解析（必读）

**一律使用约定路径**（`~/.terrain/bin/…`、`.terrain/…`），**不要**写机器相关的绝对路径（如 `/Users/…` 或 `C:\Users\…`）。

Windows 上工具部署在 `%USERPROFILE%\.terrain\bin\`（Git Bash / PowerShell 7+ 中可写为 `~/.terrain/bin/`），二进制带 `.exe` 后缀。

1. 执行前检查工具是否存在 — 见 `rtk-skill` / `codegraph-skill` 中的跨平台检查表（**不要**在 Windows 上使用 Unix 专用的 `test -x`）
2. 存在 → 用 `~/.terrain/bin/<tool> …`（词首 `~` 在 bash/zsh/Git Bash/PowerShell 7+ 会展开）
3. 不存在 → RTK / CodeGraph 用上表 `bunx` / `npx` 降级；Terrain CLI 请用户通过桌面应用操作
4. 可选参考：`.terrain/env/agent-tools.json`（本地生成、不入库），内容与约定路径一致

**不要**把 manifest 里的 `~` 路径赋给变量再引号调用（`"$VAR"` 不会展开 `~`）。直接写 `~/.terrain/bin/rtk` 或选用 `bunx` 前缀。

### RTK 要点（必读 `rtk-skill`）

- **必须显式**加 rtk 前缀 — Terrain 不启用 `rtk init` 全局 hook
- 内置 Read/Grep 不会自动走 RTK — 大文件用 `<rtk> read`，搜索用 `<rtk> grep`

**注意**：不要运行 `codegraph install` 或 `rtk init`（已由 Terrain + Skills 配置）。
<!-- terrain:end tools -->