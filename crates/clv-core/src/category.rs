use crate::models::CleanupBucket;
use serde::{Deserialize, Serialize};

/// Typed cleanup item category — drives bucket classification and i18n keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupCategory {
    AgentCache,
    AgentSession,
    BuildCache,
    BuildDir,
    BuildOutput,
    BytecodeCache,
    CompileCache,
    Dependency,
    Dependencies,
    DependencyCache,
    EditorCache,
    GlobalCache,
    GradleCache,
    IntermediateOutput,
    LintCache,
    Logs,
    PluginCache,
    ProviderCache,
    SystemTemp,
    TempFiles,
    TestCache,
    TestEnv,
    TestOutput,
    ToolCache,
    Toolchain,
    TypeCheckCache,
    VirtualEnv,
    XcodeCache,
    AppBinary,
    AppSupport,
    AppCache,
    AppPreference,
    AppContainer,
    AppGroupContainer,
    AppSavedState,
    AppWebData,
    AppLaunchItem,
    AppLog,
    StrayExecutable,
    OrphanResidue,
    /// Windows: `HKCU\Software\<vendor>` keys left behind (M2). Removed only
    /// after a `reg export` backup; `HKLM` counterparts are `Protected`.
    RegistryLeftover,
    /// Windows: Start Menu / Desktop `.lnk` shortcuts (M2).
    AppShortcut,
}

impl CleanupCategory {
    pub fn cleanup_bucket(self) -> CleanupBucket {
        match self {
            Self::AgentCache | Self::AgentSession => CleanupBucket::AiGenerated,
            Self::GlobalCache | Self::SystemTemp => CleanupBucket::SharedToolCache,
            Self::Toolchain
            | Self::VirtualEnv
            | Self::TestEnv
            | Self::Dependencies
            | Self::Dependency
            | Self::DependencyCache
            | Self::ProviderCache => CleanupBucket::DevEnvironment,
            Self::CompileCache
            | Self::BuildCache
            | Self::BuildOutput
            | Self::BytecodeCache
            | Self::IntermediateOutput
            | Self::XcodeCache
            | Self::BuildDir
            | Self::TestCache
            | Self::ToolCache
            | Self::TestOutput
            | Self::LintCache
            | Self::TypeCheckCache
            | Self::GradleCache
            | Self::PluginCache
            | Self::EditorCache
            | Self::TempFiles
            | Self::Logs => CleanupBucket::ProjectBuildCache,
            // App-uninstaller categories never appear in the rule-driven scan;
            // they only materialize inside the apps view's own report.
            Self::AppBinary
            | Self::AppSupport
            | Self::AppCache
            | Self::AppPreference
            | Self::AppContainer
            | Self::AppGroupContainer
            | Self::AppSavedState
            | Self::AppWebData
            | Self::AppLaunchItem
            | Self::AppLog
            | Self::StrayExecutable
            | Self::OrphanResidue
            | Self::RegistryLeftover
            | Self::AppShortcut => CleanupBucket::AppResidue,
        }
    }
}
