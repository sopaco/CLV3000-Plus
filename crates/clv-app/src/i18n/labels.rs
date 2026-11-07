use clv_core::{
    CleanupBucket, CleanupCategory, Language, RiskLevel, RuleDescription, TechStack,
    localized_text_matches_query, tr,
};
use clv_platform::{ProcessCategory, StartupImpact, StartupKind};

fn localized_label_matches_query(query: &str, label: impl Fn(Language) -> &'static str) -> bool {
    localized_text_matches_query(query, label)
}

pub fn tech_stack_matches_query(stack: TechStack, query: &str) -> bool {
    localized_label_matches_query(query, |lang| tech_stack_label(lang, stack))
}

pub fn process_category_matches_query(category: &ProcessCategory, query: &str) -> bool {
    localized_label_matches_query(query, |lang| process_category_label(lang, category))
}

pub fn risk_label(lang: Language, risk: RiskLevel) -> &'static str {
    match risk {
        RiskLevel::Safe => tr(lang, "安全", "Safe", "安全"),
        RiskLevel::Caution => tr(lang, "建议确认", "Review", "要確認"),
        RiskLevel::Protected => tr(lang, "受保护", "Protected", "保護"),
    }
}

pub fn tech_stack_label(lang: Language, stack: TechStack) -> &'static str {
    match stack {
        TechStack::Rust => "Rust",
        TechStack::NodeWeb => "Node.js / Web",
        TechStack::Android => "Android",
        TechStack::Ios => "iOS",
        TechStack::Flutter => "Flutter",
        TechStack::Kmp => "KMP",
        TechStack::Java => "Java",
        TechStack::Python => "Python",
        TechStack::DotNet => ".NET",
        TechStack::Cpp => "C/C++",
        TechStack::Go => "Go",
        TechStack::Ruby => "Ruby",
        TechStack::Php => "PHP",
        TechStack::Unity => "Unity",
        TechStack::Infra => tr(lang, "基础设施", "Infrastructure", "インフラ"),
        TechStack::Agent => tr(lang, "Agent 项目", "Agent Project", "Agent プロジェクト"),
        TechStack::System => tr(lang, "系统缓存", "System Cache", "システムキャッシュ"),
        TechStack::Other => tr(lang, "其他", "Other", "その他"),
    }
}

pub fn startup_impact_label(lang: Language, impact: StartupImpact) -> &'static str {
    match impact {
        StartupImpact::Low => tr(lang, "低", "Low", "低"),
        StartupImpact::Medium => tr(lang, "中", "Medium", "中"),
        StartupImpact::High => tr(lang, "高", "High", "高"),
    }
}

pub fn startup_kind_label(lang: Language, kind: &StartupKind) -> &'static str {
    match kind {
        StartupKind::LoginItem => tr(lang, "登录项", "Login Item", "ログイン項目"),
        StartupKind::LaunchAgent => "LaunchAgent",
        StartupKind::LaunchDaemon => tr(lang, "后台服务", "Launch Daemon", "デーモン"),
        StartupKind::ScheduledTask => tr(lang, "计划任务", "Scheduled Task", "スケジュールタスク"),
        StartupKind::RegistryRun => tr(lang, "注册表启动", "Registry Run", "レジストリ起動"),
        StartupKind::StartupFolder => tr(
            lang,
            "启动文件夹",
            "Startup Folder",
            "スタートアップフォルダ",
        ),
        StartupKind::Service => tr(lang, "系统服务", "System Service", "システムサービス"),
    }
}

pub fn process_category_label(lang: Language, category: &ProcessCategory) -> &'static str {
    match category {
        ProcessCategory::System => tr(lang, "系统", "System", "システム"),
        ProcessCategory::User => tr(lang, "用户", "User", "ユーザー"),
        ProcessCategory::Dev => tr(lang, "开发", "Development", "開発"),
        ProcessCategory::Agent => "Agent",
    }
}

/// Translate scan-item category for display.
pub fn scan_category_label(lang: Language, category: CleanupCategory) -> &'static str {
    use CleanupCategory::*;
    match category {
        CompileCache => tr(lang, "编译缓存", "Compile Cache", "コンパイルキャッシュ"),
        Dependencies => tr(lang, "依赖包", "Dependencies", "依存パッケージ"),
        BuildCache => tr(lang, "构建缓存", "Build Cache", "ビルドキャッシュ"),
        BuildOutput => tr(lang, "构建产物", "Build Output", "ビルド成果物"),
        GlobalCache => tr(lang, "全局缓存", "Global Cache", "グローバルキャッシュ"),
        Toolchain => tr(lang, "工具链", "Toolchain", "ツールチェーン"),
        AgentSession => tr(lang, "Agent 会话", "Agent Session", "Agent セッション"),
        AgentCache => tr(lang, "Agent 缓存", "Agent Cache", "Agent キャッシュ"),
        BuildDir => tr(lang, "构建目录", "Build Directory", "ビルドディレクトリ"),
        BytecodeCache => tr(
            lang,
            "字节码缓存",
            "Bytecode Cache",
            "バイトコードキャッシュ",
        ),
        IntermediateOutput => tr(lang, "中间产物", "Intermediate Output", "中間成果物"),
        XcodeCache => tr(lang, "Xcode 缓存", "Xcode Cache", "Xcode キャッシュ"),
        TestCache => tr(lang, "测试缓存", "Test Cache", "テストキャッシュ"),
        ToolCache => tr(lang, "工具缓存", "Tool Cache", "ツールキャッシュ"),
        TestOutput => tr(lang, "测试产物", "Test Output", "テスト成果物"),
        LintCache => tr(lang, "Lint 缓存", "Lint Cache", "Lint キャッシュ"),
        TypeCheckCache => tr(
            lang,
            "类型检查缓存",
            "Type-check Cache",
            "型チェックキャッシュ",
        ),
        GradleCache => tr(lang, "Gradle 缓存", "Gradle Cache", "Gradle キャッシュ"),
        PluginCache => tr(lang, "插件缓存", "Plugin Cache", "プラグインキャッシュ"),
        EditorCache => tr(lang, "编辑器缓存", "Editor Cache", "エディタキャッシュ"),
        TempFiles => tr(lang, "临时文件", "Temp Files", "一時ファイル"),
        Logs => tr(lang, "日志", "Logs", "ログ"),
        Dependency => tr(lang, "依赖", "Dependencies", "依存"),
        DependencyCache => tr(lang, "依赖缓存", "Dependency Cache", "依存キャッシュ"),
        SystemTemp => tr(lang, "系统临时", "System Temp", "システム一時"),
        VirtualEnv => tr(lang, "虚拟环境", "Virtual Env", "仮想環境"),
        TestEnv => tr(lang, "测试环境", "Test Env", "テスト環境"),
        ProviderCache => tr(
            lang,
            "Provider 缓存",
            "Provider Cache",
            "Provider キャッシュ",
        ),
        AppBinary => tr(lang, "应用本体", "App Bundle", "アプリ本体"),
        AppSupport => tr(lang, "配置与数据", "App Data", "設定とデータ"),
        AppCache => tr(lang, "应用缓存", "App Cache", "アプリキャッシュ"),
        AppPreference => tr(lang, "偏好设置", "App Preferences", "環境設定"),
        AppContainer => tr(lang, "应用容器", "App Container", "アプリコンテナ"),
        AppGroupContainer => tr(lang, "组容器", "Group Container", "グループコンテナ"),
        AppSavedState => tr(lang, "窗口状态", "Saved State", "ウィンドウ状態"),
        AppWebData => tr(lang, "网页数据", "App Web Data", "ウェブデータ"),
        AppLaunchItem => tr(lang, "启动项", "Launch Item", "起動項目"),
        AppLog => tr(lang, "应用日志", "App Logs", "アプリログ"),
        StrayExecutable => tr(lang, "失效链接", "Broken Link", "無効リンク"),
        OrphanResidue => tr(lang, "游离残留", "Orphan Residue", "孤立残留"),
        RegistryLeftover => tr(lang, "注册表残留", "Registry Residue", "レジストリ残留"),
        AppShortcut => tr(lang, "快捷方式", "Shortcut", "ショートカット"),
    }
}

pub fn rule_description_label(lang: Language, description: RuleDescription) -> &'static str {
    description.text(lang)
}

pub fn cleanup_bucket_label(lang: Language, bucket: CleanupBucket) -> &'static str {
    match bucket {
        CleanupBucket::ProjectBuildCache => tr(
            lang,
            "项目临时产物",
            "Project Build Cache",
            "プロジェクト一時ファイル",
        ),
        CleanupBucket::SharedToolCache => tr(
            lang,
            "构建下载缓存",
            "Shared Tool Cache",
            "共有ツールキャッシュ",
        ),
        CleanupBucket::DevEnvironment => tr(lang, "环境与依赖", "Dev Environment", "環境と依存"),
        CleanupBucket::AiGenerated => tr(lang, "AI 工具数据", "AI Tool Data", "AI ツールデータ"),
        CleanupBucket::AppResidue => tr(lang, "应用残留", "App Residue", "アプリ残留"),
    }
}
