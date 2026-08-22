use clv_core::{Language, RiskLevel, TechStack, tr};
use clv_platform::{ProcessCategory, StartupImpact, StartupKind};

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
        StartupKind::StartupFolder => tr(lang, "启动文件夹", "Startup Folder", "スタートアップフォルダ"),
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

/// Translate common scan-item category strings from rules.
pub fn scan_category_label(lang: Language, category: &str) -> &str {
    if lang == Language::Zh {
        return category;
    }
    match category {
        "编译缓存" => tr(lang, "编译缓存", "Build Cache", "ビルドキャッシュ"),
        "依赖包" => tr(lang, "依赖包", "Dependencies", "依存パッケージ"),
        "构建缓存" => tr(lang, "构建缓存", "Build Cache", "ビルドキャッシュ"),
        "构建产物" => tr(lang, "构建产物", "Build Output", "ビルド成果物"),
        "全局缓存" => tr(lang, "全局缓存", "Global Cache", "グローバルキャッシュ"),
        "工具链" => tr(lang, "工具链", "Toolchain", "ツールチェーン"),
        "Agent 会话" => tr(lang, "Agent 会话", "Agent Session", "Agent セッション"),
        "Agent 缓存" => tr(lang, "Agent 缓存", "Agent Cache", "Agent キャッシュ"),
        _ => category,
    }
}
