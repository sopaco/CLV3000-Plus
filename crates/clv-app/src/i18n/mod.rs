//! Application UI internationalization (zh / en / ja).

mod labels;

pub use labels::*;

use clv_core::{
    AgentReasonPart, AppSettings, CleanupBucket, CleanupReport, Language, RuleDescription,
    format_agent_reason, format_bytes, resolve_language, tr,
};

use crate::app::state::{AppPage, CleanupFilter};

#[derive(Debug, Clone, Copy)]
pub struct I18n {
    pub lang: Language,
}

impl I18n {
    pub fn from_settings(settings: &AppSettings) -> Self {
        Self {
            lang: resolve_language(settings.language),
        }
    }

    pub fn t(&self, zh: &'static str, en: &'static str, ja: &'static str) -> &'static str {
        tr(self.lang, zh, en, ja)
    }

    // ── Navigation & pages ───────────────────────────────────────────────

    pub fn nav_home(&self) -> &'static str {
        self.t("首页", "Home", "ホーム")
    }

    pub fn nav_cleanup(&self) -> &'static str {
        self.t("清理", "Clean", "クリーン")
    }

    pub fn nav_agent(&self) -> &'static str {
        "Agent"
    }

    pub fn nav_startup(&self) -> &'static str {
        self.t("启动", "Startup", "起動")
    }

    pub fn nav_process(&self) -> &'static str {
        self.t("进程", "Process", "プロセス")
    }

    pub fn nav_settings(&self) -> &'static str {
        self.t("设置", "Settings", "設定")
    }

    pub fn page_title(&self, page: AppPage) -> &'static str {
        match page {
            AppPage::Dashboard => self.t("首页", "Home", "ホーム"),
            AppPage::Cleanup => self.t("智能清理", "Smart Cleanup", "スマートクリーン"),
            AppPage::Agent => self.t("Agent 项目", "Agent Projects", "Agent プロジェクト"),
            AppPage::Startup => self.t("启动项管理", "Startup Items", "起動項目"),
            AppPage::Process => self.t("进程管理", "Processes", "プロセス管理"),
            AppPage::Settings => self.t("设置", "Settings", "設定"),
            AppPage::Onboarding => self.t("欢迎", "Welcome", "ようこそ"),
        }
    }

    // ── Common actions ───────────────────────────────────────────────────

    pub fn open_location(&self) -> &'static str {
        self.t("打开位置", "Open Location", "場所を開く")
    }

    pub fn details(&self) -> &'static str {
        self.t("详情", "Details", "詳細")
    }

    pub fn refresh(&self) -> &'static str {
        self.t("刷新", "Refresh", "更新")
    }

    pub fn back(&self) -> &'static str {
        self.t("上一步", "Back", "戻る")
    }

    pub fn next(&self) -> &'static str {
        self.t("下一步", "Next", "次へ")
    }

    pub fn select_all(&self) -> &'static str {
        self.t("全选", "Select All", "すべて選択")
    }

    pub fn deselect_all(&self) -> &'static str {
        self.t("取消全选", "Deselect All", "選択解除")
    }

    pub fn kill_process(&self) -> &'static str {
        self.t("结束", "End", "終了")
    }

    pub fn clean_cache(&self) -> &'static str {
        self.t("清理缓存", "Clean Cache", "キャッシュ削除")
    }

    pub fn save(&self) -> &'static str {
        self.t("保存", "Save", "保存")
    }

    pub fn unknown(&self) -> &'static str {
        self.t("未知", "Unknown", "不明")
    }

    // ── Settings ───────────────────────────────────────────────────────

    pub fn settings_title(&self) -> &'static str {
        self.t("设置", "Settings", "設定")
    }

    pub fn settings_subtitle(&self) -> &'static str {
        self.t(
            "个性化扫描与清理行为",
            "Customize scan and cleanup behavior",
            "スキャンとクリーンの動作をカスタマイズ",
        )
    }

    pub fn language_section_title(&self) -> &'static str {
        self.t("界面语言", "Language", "表示言語")
    }

    pub fn language_section_desc(&self) -> &'static str {
        self.t(
            "默认跟随系统，也可手动选择",
            "Follow system by default, or choose manually",
            "既定はシステムに従い、手動選択も可能",
        )
    }

    pub fn language_system(&self) -> &'static str {
        self.t("跟随系统", "System", "システム")
    }

    pub fn language_zh(&self) -> &'static str {
        "中文"
    }

    pub fn language_en(&self) -> &'static str {
        "English"
    }

    pub fn language_ja(&self) -> &'static str {
        "日本語"
    }

    pub fn theme_section_title(&self) -> &'static str {
        self.t("界面主题", "Appearance", "テーマ")
    }

    pub fn theme_section_desc(&self) -> &'static str {
        self.t(
            "选择你喜欢的视觉风格，立即生效",
            "Pick a visual style — changes apply instantly",
            "好みのスタイルを選択 — すぐに反映",
        )
    }

    pub fn theme_defender_name(&self) -> &'static str {
        self.t("守护蓝", "Defender Blue", "ディフェンダーブルー")
    }

    pub fn theme_defender_desc(&self) -> &'static str {
        self.t(
            "专业安全软件风格，沉稳可靠",
            "Professional security aesthetic — calm and reliable",
            "プロのセキュリティ風 — 落ち着いた信頼感",
        )
    }

    pub fn theme_blossom_name(&self) -> &'static str {
        self.t("樱花粉", "Cherry Blossom", "桜ピンク")
    }

    pub fn theme_blossom_desc(&self) -> &'static str {
        self.t(
            "清透樱花粉与薰衣草，明快不沉闷",
            "Airy cherry blossom and lavender — light and fresh",
            "桜ピンクとラベンダー — 明るく軽やか",
        )
    }

    pub fn theme_neon_name(&self) -> &'static str {
        self.t("霓虹活力", "Neon Pulse", "ネオンパルス")
    }

    pub fn theme_neon_desc(&self) -> &'static str {
        self.t(
            "电光青紫，年轻潮流感十足",
            "Electric cyan and purple — bold youth energy",
            "シアンとパープル — 若々しいエネルギー",
        )
    }

    pub fn theme_aurora_name(&self) -> &'static str {
        self.t("极光薄荷", "Aurora Mint", "オーロラミント")
    }

    pub fn theme_aurora_desc(&self) -> &'static str {
        self.t(
            "明亮薄荷与天蓝，通透清爽",
            "Bright mint and sky blue — crisp and refreshing",
            "明るいミントとスカイブルー — すっきり爽やか",
        )
    }

    pub fn theme_label(&self, theme: clv_core::ThemePreference) -> &'static str {
        match theme {
            clv_core::ThemePreference::Defender => self.theme_defender_name(),
            clv_core::ThemePreference::Blossom => self.theme_blossom_name(),
            clv_core::ThemePreference::Neon => self.theme_neon_name(),
            clv_core::ThemePreference::Aurora => self.theme_aurora_name(),
        }
    }

    pub fn theme_desc(&self, theme: clv_core::ThemePreference) -> &'static str {
        match theme {
            clv_core::ThemePreference::Defender => self.theme_defender_desc(),
            clv_core::ThemePreference::Blossom => self.theme_blossom_desc(),
            clv_core::ThemePreference::Neon => self.theme_neon_desc(),
            clv_core::ThemePreference::Aurora => self.theme_aurora_desc(),
        }
    }

    pub fn expert_mode_label(&self) -> &'static str {
        self.t("专家模式", "Expert Mode", "エキスパートモード")
    }

    pub fn expert_mode_desc(&self) -> &'static str {
        self.t(
            "显示完整路径、受保护项与更多可清理项",
            "Show full paths, protected items, and more cleanable targets",
            "フルパス・保護項目・追加の削除対象を表示",
        )
    }

    pub fn soft_delete_label(&self) -> &'static str {
        self.t(
            "软删除（推荐）",
            "Soft Delete (Recommended)",
            "ソフト削除（推奨）",
        )
    }

    pub fn soft_delete_desc(&self) -> &'static str {
        self.t(
            "清理的文件移入回收区，7 天后自动清除",
            "Moved to trash and auto-purged after 7 days",
            "ゴミ箱に移動し、7 日後に自動削除",
        )
    }

    pub fn agent_heuristics_label(&self) -> &'static str {
        self.t(
            "Agent 项目识别",
            "Agent Project Detection",
            "Agent プロジェクト検出",
        )
    }

    pub fn agent_heuristics_desc(&self) -> &'static str {
        self.t(
            "根据目录名与 .agents/.claude/.cursor/.trae/.opencode 等标记识别 Agent 试验项目",
            "Detect agent trial projects via directory names and markers like .agents/.claude/.cursor/.trae/.opencode",
            "ディレクトリ名と .agents/.claude/.cursor/.trae/.opencode 等のマーカーで Agent 試験プロジェクトを検出",
        )
    }

    pub fn scan_paths_title(&self) -> &'static str {
        self.t("扫描目录", "Scan Directories", "スキャン対象")
    }

    pub fn scan_paths_desc(&self) -> &'static str {
        self.t(
            "每行一个路径，保存后下次扫描生效",
            "One path per line; takes effect on next scan",
            "1 行に 1 パス。保存後、次回スキャンから有効",
        )
    }

    pub fn scan_paths_placeholder(&self) -> &'static str {
        self.t(
            "每行一个目录路径，支持 ~/Projects",
            "One directory per line, ~/Projects supported",
            "1 行に 1 ディレクトリ。~/Projects 可",
        )
    }

    pub fn save_scan_paths(&self) -> &'static str {
        self.t("保存扫描目录", "Save Scan Paths", "スキャン対象を保存")
    }

    pub fn scan_paths_hint(&self) -> &'static str {
        self.t(
            "留空行会被忽略；支持 ~/ 展开",
            "Blank lines are ignored; ~ expands to home",
            "空行は無視。~ はホームに展開",
        )
    }

    pub fn supported_stacks_title(&self) -> &'static str {
        self.t(
            "支持清理的技术栈",
            "Supported Tech Stacks",
            "対応技術スタック",
        )
    }

    pub fn supported_stacks_list(&self) -> &'static str {
        "Rust · Node.js/Web · Android · iOS · Flutter · KMP · Java · Python · .NET · C/C++ · Go · Ruby · PHP · Unity · Terraform"
    }

    pub fn app_version(&self) -> String {
        format!("CLV3000 Plus v{}", env!("CARGO_PKG_VERSION"))
    }

    // ── Dashboard ────────────────────────────────────────────────────────

    pub fn scan_now(&self) -> &'static str {
        self.t("立即体检", "Scan Now", "今すぐ診断")
    }

    pub fn scanning_health(&self) -> &'static str {
        self.t("体检中…", "Scanning…", "診断中…")
    }

    pub fn health_score_label(&self) -> &'static str {
        self.t("清爽分", "Health Score", "快適スコア")
    }

    pub fn realtime_guard(&self) -> &'static str {
        self.t("实时守护中", "Real-time Protection", "リアルタイム保護")
    }

    pub fn hero_title(&self) -> &'static str {
        self.t("让电脑保持轻快", "Keep Your PC Light", "PC を快適に")
    }

    pub fn hero_subtitle(&self) -> &'static str {
        self.t(
            "智能找出 Agent 试验项目与开发缓存，一键释放空间",
            "Find agent trial projects and dev caches, free space in one click",
            "Agent 試験プロジェクトと開発キャッシュを検出し、ワンクリックで解放",
        )
    }

    pub fn hero_scan_hint(&self) -> &'static str {
        self.t(
            "极速扫描，可继续操作其他功能",
            "Fast scan — keep using the app",
            "高速スキャン — 他の機能も利用可能",
        )
    }

    pub fn disk_usage(&self) -> &'static str {
        self.t("磁盘使用", "Disk Usage", "ディスク使用量")
    }

    pub fn reclaimable_space(&self) -> &'static str {
        self.t("可释放空间", "Reclaimable", "解放可能")
    }

    pub fn cleanable_items_count(&self, count: usize) -> String {
        match self.lang {
            Language::Zh => format!("{count} 个可清理项"),
            Language::En => format!("{count} cleanable item(s)"),
            Language::Ja => format!("{count} 件削除可能"),
        }
    }

    pub fn agent_projects_tile(&self) -> &'static str {
        self.t("Agent 项目", "Agent Projects", "Agent プロジェクト")
    }

    pub fn approx_size(&self, size: &str) -> String {
        match self.lang {
            Language::Zh => format!("约 {size}"),
            Language::En => format!("~{size}"),
            Language::Ja => format!("約 {size}"),
        }
    }

    pub fn startup_items(&self) -> &'static str {
        self.t("启动项", "Startup Items", "起動項目")
    }

    pub fn manage_startup(&self) -> &'static str {
        self.t("管理开机启动", "Manage login items", "ログイン時起動を管理")
    }

    pub fn system_status(&self) -> &'static str {
        self.t("系统状态", "System Status", "システム状態")
    }

    pub fn disk_usage_metric(&self) -> &'static str {
        self.t("磁盘占用", "Disk Used", "ディスク使用率")
    }

    pub fn optimizable_space(&self) -> &'static str {
        self.t("可优化空间", "Optimizable Space", "最適化可能")
    }

    pub fn free_space(&self, size: &str) -> String {
        match self.lang {
            Language::Zh => format!("可用空间 {size}"),
            Language::En => format!("{size} free"),
            Language::Ja => format!("空き {size}"),
        }
    }

    pub fn expert_mode_short(&self) -> &'static str {
        self.t("专家模式", "Expert", "エキスパート")
    }

    pub fn simple_mode_short(&self) -> &'static str {
        self.t("简单模式", "Simple", "シンプル")
    }

    pub fn protection_features_title(&self) -> &'static str {
        self.t("全方位守护", "All-round Protection", "総合保護")
    }

    pub fn feature_agent_detect(&self) -> &'static str {
        self.t(
            "识别 Claude / Cursor / Codex / Trae / OpenCode 等 Agent 试验项目",
            "Detect agent trial projects (Claude, Cursor, Codex, Trae, OpenCode, …)",
            "Claude / Cursor / Codex / Trae / OpenCode 等の Agent 試験プロジェクトを検出",
        )
    }

    pub fn feature_safe_cleanup(&self) -> &'static str {
        self.t(
            "安全清理多技术栈构建产物与依赖缓存",
            "Safely clean build artifacts and dependency caches",
            "複数スタックのビルド成果物と依存キャッシュを安全に削除",
        )
    }

    pub fn feature_startup(&self) -> &'static str {
        self.t(
            "管理登录启动项，减轻开机负担",
            "Manage login startup items",
            "ログイン起動項目を管理し、起動負荷を軽減",
        )
    }

    pub fn feature_process(&self) -> &'static str {
        self.t(
            "查看高占用进程，一键释放系统资源",
            "View heavy processes and free resources",
            "高負荷プロセスを表示し、リソースを解放",
        )
    }

    pub fn last_cleanup_freed(&self, size: &str) -> String {
        match self.lang {
            Language::Zh => format!("上次清理已释放 {size}"),
            Language::En => format!("Last cleanup freed {size}"),
            Language::Ja => format!("前回の削除で {size} を解放"),
        }
    }

    // ── Cleanup history ────────────────────────────────────────────────

    pub fn cleanup_history_title(&self) -> &'static str {
        self.t("清理趋势", "Cleanup Trend", "削除トレンド")
    }

    pub fn no_cleanup_history(&self) -> String {
        match self.lang {
            Language::Zh => "尚无清理记录".to_string(),
            Language::En => "No cleanup history yet".to_string(),
            Language::Ja => "削除履歴なし".to_string(),
        }
    }

    pub fn history_summary(&self, freed_7d: &str, cleanups: usize) -> String {
        match self.lang {
            Language::Zh => format!("过去 7 天释放 {freed_7d}，共 {cleanups} 次清理"),
            Language::En => format!("Freed {freed_7d} in the last 7 days, {cleanups} cleanup(s)"),
            Language::Ja => format!("過去 7 日で {freed_7d} を解放、{cleanups} 回の削除"),
        }
    }

    pub fn history_7d_freed(&self) -> &'static str {
        self.t("近 7 天", "Last 7 Days", "過去 7 日")
    }

    pub fn history_30d_freed(&self) -> &'static str {
        self.t("近 30 天", "Last 30 Days", "過去 30 日")
    }

    pub fn history_cleanups_count(&self, count: usize) -> String {
        match self.lang {
            Language::Zh => format!("{count} 次清理"),
            Language::En => format!("{count} cleanup(s)"),
            Language::Ja => format!("{count} 回削除"),
        }
    }

    pub fn history_7d_detail(&self, success: usize, failed: usize) -> String {
        if failed == 0 {
            match self.lang {
                Language::Zh => format!("全部成功，共清理 {success} 项"),
                Language::En => format!("All {success} item(s) cleaned successfully"),
                Language::Ja => format!("すべて成功、{success} 件を削除"),
            }
        } else {
            match self.lang {
                Language::Zh => format!("成功 {success} 项，失败 {failed} 项"),
                Language::En => format!("{success} succeeded, {failed} failed"),
                Language::Ja => format!("{success} 件成功、{failed} 件失敗"),
            }
        }
    }

    // ── Health messages ──────────────────────────────────────────────────

    pub fn health_scanning(&self) -> &'static str {
        self.t("正在为你体检…", "Scanning your system…", "診断中…")
    }

    pub fn health_no_scan(&self) -> &'static str {
        self.t(
            "点一下，马上知道电脑状态",
            "Tap to check your system health",
            "タップして PC の状態を確認",
        )
    }

    pub fn health_excellent(&self) -> &'static str {
        self.t(
            "状态很棒，继续保持",
            "Excellent — keep it up",
            "良好 — この調子で",
        )
    }

    pub fn health_good(&self) -> &'static str {
        self.t(
            "整体不错，还能更轻快",
            "Good — room to improve",
            "まずまず — さらに快適に",
        )
    }

    pub fn health_fair(&self) -> &'static str {
        self.t(
            "清理一下会更流畅",
            "Cleanup will help",
            "削除するとさらに快適",
        )
    }

    pub fn health_poor(&self) -> &'static str {
        self.t(
            "建议尽快体检清理",
            "Scan and clean soon",
            "早めの診断・削除を推奨",
        )
    }

    // ── Cleanup ──────────────────────────────────────────────────────────

    pub fn filter_label(&self) -> &'static str {
        self.t("筛选", "Filter", "フィルター")
    }

    pub fn filter_all(&self) -> &'static str {
        self.t("全部", "All", "すべて")
    }

    pub fn filter_safe(&self) -> &'static str {
        self.t("仅安全清理项", "Safe Only", "安全な項目のみ")
    }

    pub fn filter_project(&self) -> &'static str {
        labels::cleanup_bucket_label(self.lang, CleanupBucket::ProjectBuildCache)
    }

    pub fn filter_shared(&self) -> &'static str {
        labels::cleanup_bucket_label(self.lang, CleanupBucket::SharedToolCache)
    }

    pub fn filter_dev_env(&self) -> &'static str {
        labels::cleanup_bucket_label(self.lang, CleanupBucket::DevEnvironment)
    }

    pub fn filter_ai(&self) -> &'static str {
        labels::cleanup_bucket_label(self.lang, CleanupBucket::AiGenerated)
    }

    pub fn cleanup_bucket_label(&self, bucket: CleanupBucket) -> &'static str {
        labels::cleanup_bucket_label(self.lang, bucket)
    }

    pub fn rule_description_label(&self, description: RuleDescription) -> &'static str {
        labels::rule_description_label(self.lang, description)
    }

    pub fn agent_reason_text(&self, parts: &[AgentReasonPart]) -> String {
        format_agent_reason(self.lang, parts)
    }

    pub fn cleanup_bucket_hint(&self, bucket: CleanupBucket) -> &'static str {
        labels::cleanup_bucket_hint(self.lang, bucket)
    }

    pub fn cleanup_filter_label(&self, filter: CleanupFilter) -> &'static str {
        match filter {
            CleanupFilter::All => self.filter_all(),
            CleanupFilter::SafeOnly => self.filter_safe(),
            CleanupFilter::ProjectBuildCache => self.filter_project(),
            CleanupFilter::SharedToolCache => self.filter_shared(),
            CleanupFilter::DevEnvironment => self.filter_dev_env(),
            CleanupFilter::AiGenerated => self.filter_ai(),
        }
    }

    pub fn items_selected_summary(&self, total: usize, selected: usize) -> String {
        match self.lang {
            Language::Zh => format!("{total} 项 · 已选 {selected} 项"),
            Language::En => format!("{total} item(s) · {selected} selected"),
            Language::Ja => format!("{total} 件 · {selected} 件選択"),
        }
    }

    pub fn not_scanned_yet(&self) -> &'static str {
        self.t("尚未扫描", "Not scanned yet", "未スキャン")
    }

    pub fn rescan(&self) -> &'static str {
        self.t("重新扫描", "Rescan", "再スキャン")
    }

    pub fn start_scan(&self) -> &'static str {
        self.t("开始扫描", "Start Scan", "スキャン開始")
    }

    pub fn clean_selected(&self) -> &'static str {
        self.t("清理选中项", "Clean Selected", "選択項目を削除")
    }

    pub fn confirm_cleanup_title(&self) -> &'static str {
        self.t("确认清理", "Confirm Cleanup", "削除の確認")
    }

    pub fn confirm_cleanup_body(&self, count: usize, bytes: u64) -> String {
        let size = format_bytes(bytes);
        match self.lang {
            Language::Zh => format!("即将清理 {count} 项，预计释放 {size}"),
            Language::En => format!("Clean {count} item(s), ~{size} to free"),
            Language::Ja => format!("{count} 件を削除、約 {size} を解放"),
        }
    }

    pub fn empty_filter_title(&self) -> &'static str {
        self.t(
            "当前筛选下暂无项目",
            "No items in this filter",
            "このフィルターに該当なし",
        )
    }

    pub fn empty_filter_hint(&self) -> &'static str {
        self.t(
            "尝试切换左侧筛选条件，或重新扫描",
            "Try another filter or rescan",
            "フィルターを変更するか再スキャンしてください",
        )
    }

    pub fn scanning_title(&self) -> &'static str {
        self.t("正在扫描", "Scanning", "スキャン中")
    }

    pub fn scan_progress_detail(
        &self,
        phase: &str,
        found: usize,
        bytes: u64,
        path: Option<&str>,
    ) -> String {
        let size = format_bytes(bytes);
        let base = match self.lang {
            Language::Zh => format!("{phase} · 已发现 {found} 项（{size}）"),
            Language::En => format!("{phase} · {found} found ({size})"),
            Language::Ja => format!("{phase} · {found} 件検出（{size}）"),
        };
        if let Some(path) = path {
            format!("{base}\n{path}")
        } else {
            base
        }
    }

    pub fn no_scan_results_title(&self) -> &'static str {
        self.t("还没有扫描结果", "No scan results yet", "スキャン結果なし")
    }

    pub fn no_scan_results_hint(&self) -> &'static str {
        self.t(
            "扫描后将列出可安全清理的缓存与构建产物",
            "Scan to list caches and build artifacts you can clean",
            "スキャン後、削除可能なキャッシュとビルド成果物を表示",
        )
    }

    pub fn scan_now_short(&self) -> &'static str {
        self.t("立即扫描", "Scan Now", "今すぐスキャン")
    }

    pub fn selected_summary(&self, count: usize, bytes: u64) -> String {
        let size = format_bytes(bytes);
        match self.lang {
            Language::Zh => format!("已选 {count} 项 · 预计释放 {size}"),
            Language::En => format!("{count} selected · ~{size} to free"),
            Language::Ja => format!("{count} 件選択 · 約 {size} を解放"),
        }
    }

    // ── Agent ────────────────────────────────────────────────────────────

    pub fn agent_page_title(&self) -> &'static str {
        self.t(
            "Agent 试验项目",
            "Agent Trial Projects",
            "Agent 試験プロジェクト",
        )
    }

    pub fn agent_search_placeholder(&self) -> &'static str {
        self.t(
            "搜索名称、路径、技术栈或原因…",
            "Search name, path, stack, or reason…",
            "名前・パス・スタック・理由で検索…",
        )
    }

    pub fn agent_projects_found(&self, count: usize, total: usize) -> String {
        match self.lang {
            Language::Zh => format!("搜索到 {count} / {total} 个项目"),
            Language::En => format!("{count} / {total} projects found"),
            Language::Ja => format!("{count} / {total} 件が見つかりました"),
        }
    }

    pub fn agent_projects_total(&self, total: usize) -> String {
        match self.lang {
            Language::Zh => format!("共 {total} 个 Agent 项目"),
            Language::En => format!("{total} agent project(s)"),
            Language::Ja => format!("Agent プロジェクト {total} 件"),
        }
    }

    pub fn agent_page_subtitle_default(&self) -> &'static str {
        self.t(
            "识别 Codex / Claude / Cursor 等 Agent 可能创建的项目",
            "Projects likely created by Codex, Claude, Cursor, etc.",
            "Codex / Claude / Cursor 等が作成した可能性のあるプロジェクト",
        )
    }

    pub fn scan_agent_projects(&self) -> &'static str {
        self.t(
            "扫描 Agent 项目",
            "Scan Agent Projects",
            "Agent プロジェクトをスキャン",
        )
    }

    pub fn scanning_ellipsis(&self) -> &'static str {
        self.t("扫描中…", "Scanning…", "スキャン中…")
    }

    pub fn agent_empty_title(&self) -> &'static str {
        self.t(
            "暂无 Agent 项目",
            "No Agent Projects",
            "Agent プロジェクトなし",
        )
    }

    pub fn agent_empty_hint(&self) -> &'static str {
        self.t(
            "点击上方「扫描 Agent 项目」，将识别含 .agents / .claude 等标记的目录",
            "Tap “Scan Agent Projects” to find directories with .agents, .claude, etc.",
            "上の「Agent プロジェクトをスキャン」で .agents / .claude 等を検出",
        )
    }

    pub fn no_matching_projects(&self) -> &'static str {
        self.t(
            "没有匹配的项目",
            "No matching projects",
            "一致するプロジェクトなし",
        )
    }

    pub fn days_inactive(&self, days: i64) -> String {
        match self.lang {
            Language::Zh => format!("{days} 天未使用"),
            Language::En => format!("Inactive {days}d"),
            Language::Ja => format!("{days} 日未使用"),
        }
    }

    // ── Startup ──────────────────────────────────────────────────────────

    pub fn startup_page_title(&self) -> &'static str {
        self.t("启动项管理", "Startup Items", "起動項目管理")
    }

    pub fn startup_page_subtitle(&self, count: usize, high: usize) -> String {
        match self.lang {
            Language::Zh => format!("共 {count} 项 · {high} 项高影响启动项"),
            Language::En => format!("{count} item(s) · {high} high impact"),
            Language::Ja => format!("{count} 件 · 高影響 {high} 件"),
        }
    }

    pub fn startup_toggle_failed(&self, err: &str) -> String {
        match self.lang {
            Language::Zh => format!("启动项操作失败：{err}"),
            Language::En => format!("Startup toggle failed: {err}"),
            Language::Ja => format!("起動項目の操作に失敗: {err}"),
        }
    }

    pub fn startup_impact(&self, label: &str) -> String {
        match self.lang {
            Language::Zh => format!("影响: {label}"),
            Language::En => format!("Impact: {label}"),
            Language::Ja => format!("影響: {label}"),
        }
    }

    pub fn loading_startup_items(&self) -> &'static str {
        self.t(
            "正在加载启动项…",
            "Loading startup items…",
            "起動項目を読み込み中…",
        )
    }

    pub fn startup_empty_title(&self) -> &'static str {
        self.t("未检测到启动项", "No Startup Items", "起動項目なし")
    }

    pub fn startup_empty_hint(&self) -> &'static str {
        self.t(
            "当前平台暂不支持，或列表为空",
            "Unsupported on this platform, or list is empty",
            "このプラットフォーム非対応、またはリストが空",
        )
    }

    // ── Process ──────────────────────────────────────────────────────────

    pub fn process_page_title(&self) -> &'static str {
        self.t("进程管理", "Processes", "プロセス管理")
    }

    pub fn process_search_found(&self, count: usize, total: usize) -> String {
        match self.lang {
            Language::Zh => format!("搜索到 {count} / {total} 个进程"),
            Language::En => format!("{count} / {total} processes found"),
            Language::Ja => format!("{count} / {total} プロセスが見つかりました"),
        }
    }

    pub fn process_list_summary(&self, shown: usize, total: usize) -> String {
        match self.lang {
            Language::Zh => format!("显示前 {shown} 个进程 · 共 {total} 个"),
            Language::En => format!("Showing top {shown} · {total} total"),
            Language::Ja => format!("上位 {shown} 件を表示 · 合計 {total} 件"),
        }
    }

    pub fn process_search_placeholder(&self) -> &'static str {
        self.t(
            "搜索名称、PID 或类别…",
            "Search name, PID, or category…",
            "名前・PID・カテゴリで検索…",
        )
    }

    pub fn sort_by_memory(&self) -> &'static str {
        self.t("按内存", "By Memory", "メモリ順")
    }

    pub fn sort_by_cpu(&self) -> &'static str {
        self.t("按 CPU", "By CPU", "CPU 順")
    }

    pub fn sort_by_name(&self) -> &'static str {
        self.t("按名称", "By Name", "名前順")
    }

    pub fn col_pid(&self) -> &'static str {
        "PID"
    }

    pub fn col_name(&self) -> &'static str {
        self.t("名称", "Name", "名前")
    }

    pub fn col_memory(&self) -> &'static str {
        self.t("内存", "Memory", "メモリ")
    }

    pub fn col_category(&self) -> &'static str {
        self.t("类别", "Category", "カテゴリ")
    }

    pub fn no_matching_processes(&self) -> &'static str {
        self.t(
            "没有匹配的进程",
            "No matching processes",
            "一致するプロセスなし",
        )
    }

    pub fn loading_processes(&self) -> &'static str {
        self.t(
            "正在加载进程列表…",
            "Loading processes…",
            "プロセスを読み込み中…",
        )
    }

    pub fn no_processes_to_show(&self) -> &'static str {
        self.t(
            "没有可显示的进程",
            "No processes to show",
            "表示するプロセスなし",
        )
    }

    // ── Onboarding ───────────────────────────────────────────────────────

    pub fn welcome_title(&self) -> &'static str {
        self.t(
            "欢迎使用 CLV3000 Plus",
            "Welcome to CLV3000 Plus",
            "CLV3000 Plus へようこそ",
        )
    }

    pub fn welcome_subtitle(&self) -> &'static str {
        self.t(
            "您的电脑安全管家",
            "Your PC maintenance companion",
            "PC メンテナンスの相棒",
        )
    }

    pub fn onboard_feature_1(&self) -> &'static str {
        self.t(
            "智能清理 Agent 与开发项目的缓存和依赖",
            "Clean agent and dev project caches and dependencies",
            "Agent と開発プロジェクトのキャッシュ・依存を削除",
        )
    }

    pub fn onboard_feature_2(&self) -> &'static str {
        self.feature_startup()
    }

    pub fn onboard_feature_3(&self) -> &'static str {
        self.t(
            "查看并结束高占用进程",
            "View and end heavy processes",
            "高負荷プロセスの表示と終了",
        )
    }

    pub fn choose_mode(&self) -> &'static str {
        self.t("选择使用模式：", "Choose a mode:", "モードを選択:")
    }

    pub fn simple_mode_title(&self) -> &'static str {
        self.t(
            "简单模式（推荐）",
            "Simple Mode (Recommended)",
            "シンプルモード（推奨）",
        )
    }

    pub fn simple_mode_desc(&self) -> &'static str {
        self.t(
            "用人话解释每一项，默认只清理安全内容",
            "Plain-language explanations; safe items only by default",
            "わかりやすい説明。既定は安全な項目のみ削除",
        )
    }

    pub fn expert_mode_onboard_title(&self) -> &'static str {
        self.expert_mode_label()
    }

    pub fn expert_mode_onboard_desc(&self) -> &'static str {
        self.t(
            "显示完整路径，可清理更多项目",
            "Full paths and more cleanable items",
            "フルパス表示。より多くの項目を削除可能",
        )
    }

    pub fn scan_dirs_intro(&self) -> &'static str {
        self.t(
            "将扫描以下常见目录：",
            "Will scan common directories:",
            "次の一般的なディレクトリをスキャン:",
        )
    }

    pub fn default_scan_dirs(&self) -> &'static str {
        "~/Projects · ~/Documents · ~/Desktop · ~/Developer etc."
    }

    pub fn start_health_scan(&self) -> &'static str {
        self.scan_now()
    }

    // ── Status bar & scan ────────────────────────────────────────────────

    pub fn status_cleaning(&self) -> &'static str {
        self.t(
            "正在清理选中项，请稍候…",
            "Cleaning selected items…",
            "選択項目を削除中…",
        )
    }

    pub fn status_scanning(&self) -> &'static str {
        self.t(
            "极速扫描中 · 可切换页面继续操作",
            "Fast scan in progress — switch pages freely",
            "高速スキャン中 — ページ切替可能",
        )
    }

    pub fn status_protected(&self, reclaimable: &str, count: usize) -> String {
        match self.lang {
            Language::Zh => format!("保护中 · 可释放 {reclaimable} · {count} 项待清理"),
            Language::En => format!("Protected · {reclaimable} reclaimable · {count} pending"),
            Language::Ja => format!("保護中 · {reclaimable} 解放可能 · {count} 件待ち"),
        }
    }

    pub fn status_idle(&self) -> &'static str {
        self.t(
            "实时防护已开启 — 点击首页「立即体检」",
            "Protection on — tap “Scan Now” on Home",
            "保護オン — ホームで「今すぐ診断」をタップ",
        )
    }

    pub fn scan_start_message(&self) -> String {
        #[cfg(target_os = "macos")]
        {
            self.t(
                "正在极速扫描（若长时间无响应，请在「系统设置 → 隐私与安全性 → 完全磁盘访问权限」中授权本应用）",
                "Fast scan in progress (if it stalls, grant Full Disk Access in System Settings → Privacy & Security)",
                "高速スキャン中（応答がない場合は「システム設定 → プライバシーとセキュリティ → フルディスクアクセス」で本アプリを許可）",
            )
            .to_string()
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.t(
                "正在极速扫描，可继续浏览其他页面",
                "Fast scan in progress — browse other pages",
                "高速スキャン中 — 他のページも利用可能",
            )
            .to_string()
        }
    }

    pub fn scan_preparing(&self) -> &'static str {
        self.t("准备扫描…", "Preparing scan…", "スキャン準備中…")
    }

    pub fn scan_interrupted(&self) -> &'static str {
        self.t("扫描已中断", "Scan interrupted", "スキャンが中断されました")
    }

    pub fn scan_complete(&self) -> &'static str {
        self.t("扫描完成", "Scan complete", "スキャン完了")
    }

    pub fn select_items_first(&self) -> &'static str {
        self.t(
            "请先选择要清理的项目",
            "Select items to clean first",
            "削除する項目を選択してください",
        )
    }

    pub fn cleanup_in_progress(&self) -> &'static str {
        self.t(
            "正在清理，请稍候…",
            "Cleaning, please wait…",
            "削除中、お待ちください…",
        )
    }

    pub fn cleanup_progress_title(&self) -> &'static str {
        self.t("正在清理", "Cleaning", "削除中")
    }

    pub fn cleanup_progress_detail(
        &self,
        completed: usize,
        total: usize,
        freed_bytes: u64,
        path: Option<&str>,
    ) -> String {
        let size = format_bytes(freed_bytes);
        let base = match self.lang {
            Language::Zh => format!("已完成 {completed}/{total} · 已释放 {size}"),
            Language::En => format!("{completed}/{total} done · {size} freed"),
            Language::Ja => format!("{completed}/{total} 完了 · {size} 解放"),
        };
        if let Some(path) = path {
            format!("{base}\n{path}")
        } else {
            base
        }
    }

    pub fn cleanup_status_detail(&self, completed: usize, total: usize) -> String {
        match self.lang {
            Language::Zh => format!("正在清理 {completed}/{total}…"),
            Language::En => format!("Cleaning {completed}/{total}…"),
            Language::Ja => format!("削除中 {completed}/{total}…"),
        }
    }

    pub fn cleanup_switch_pages_hint(&self) -> &'static str {
        self.t(
            "可切换页面，清理不会中断",
            "Switch pages — cleanup continues",
            "ページ切替可 — 削除は継続",
        )
    }

    pub fn cleanup_interrupted(&self) -> &'static str {
        self.t("清理已中断", "Cleanup interrupted", "削除が中断されました")
    }

    pub fn cleanup_complete_title(&self) -> &'static str {
        self.t("清理完成", "Cleanup Complete", "削除完了")
    }

    pub fn cleanup_complete_notification(&self, report: &CleanupReport) -> String {
        let size = format_bytes(report.freed_bytes);
        let failed_suffix = if report.failed.is_empty() {
            String::new()
        } else {
            match self.lang {
                Language::Zh => format!("，{} 项失败", report.failed.len()),
                Language::En => format!(", {} failed", report.failed.len()),
                Language::Ja => format!("、{} 件失敗", report.failed.len()),
            }
        };
        match self.lang {
            Language::Zh => format!(
                "成功释放 {size}，共清理 {} 项{failed_suffix}",
                report.success_count,
            ),
            Language::En => format!(
                "Freed {size}, {} item(s) cleaned{failed_suffix}",
                report.success_count,
            ),
            Language::Ja => format!(
                "{size} を解放、{} 件を削除{failed_suffix}",
                report.success_count,
            ),
        }
    }

    pub fn cleanup_summary(&self, report: &CleanupReport) -> String {
        let failed_suffix = if report.failed.is_empty() {
            String::new()
        } else {
            match self.lang {
                Language::Zh => format!("，失败 {} 项", report.failed.len()),
                Language::En => format!(", {} failed", report.failed.len()),
                Language::Ja => format!("、{} 件失敗", report.failed.len()),
            }
        };
        match self.lang {
            Language::Zh => format!(
                "已释放 {}，成功 {} 项{failed_suffix}",
                format_bytes(report.freed_bytes),
                report.success_count,
            ),
            Language::En => format!(
                "Freed {}, {} item(s) cleaned{failed_suffix}",
                format_bytes(report.freed_bytes),
                report.success_count,
            ),
            Language::Ja => format!(
                "{} を解放、{} 件を削除{failed_suffix}",
                format_bytes(report.freed_bytes),
                report.success_count,
            ),
        }
    }

    pub fn killing_process(&self, pid: u32) -> String {
        match self.lang {
            Language::Zh => format!("正在结束进程 {pid}…"),
            Language::En => format!("Ending process {pid}…"),
            Language::Ja => format!("プロセス {pid} を終了中…"),
        }
    }

    pub fn process_killed(&self, pid: u32) -> String {
        match self.lang {
            Language::Zh => format!("已结束进程 {pid}"),
            Language::En => format!("Ended process {pid}"),
            Language::Ja => format!("プロセス {pid} を終了しました"),
        }
    }

    pub fn kill_process_failed(&self, err: &str) -> String {
        match self.lang {
            Language::Zh => format!("结束进程失败：{err}"),
            Language::En => format!("Failed to end process: {err}"),
            Language::Ja => format!("プロセス終了に失敗: {err}"),
        }
    }

    pub fn kill_process_internal_error(&self) -> &'static str {
        self.t(
            "结束进程时发生内部错误",
            "Internal error while ending process",
            "プロセス終了中に内部エラー",
        )
    }

    // ── Scan progress bar ────────────────────────────────────────────────

    pub fn fast_scanning(&self) -> &'static str {
        self.t("正在极速扫描", "Fast Scanning", "高速スキャン中")
    }

    pub fn scan_switch_pages_hint(&self) -> &'static str {
        self.t(
            "可切换页面，扫描不会中断",
            "Switch pages — scan continues",
            "ページ切替可 — スキャンは継続",
        )
    }

    pub fn scan_bar_detail(
        &self,
        phase: &str,
        found: usize,
        bytes: u64,
        path: Option<&str>,
    ) -> String {
        self.scan_progress_detail(phase, found, bytes, path)
    }
}
