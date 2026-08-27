//! Semantic icon mapping — Lucide SVG via gpui-component-assets + app assets.

use gpui_component::{Icon, IconName};

/// Custom asset paths (see `assets/icons/`).
pub const ICON_BROOM: &str = "icons/broom.svg";
pub const ICON_APP_LOGO: &str = "icons/icon_app.png";

/// Sidebar navigation
pub const NAV_HOME: IconName = IconName::LayoutDashboard;

pub fn nav_cleanup_icon() -> Icon {
    Icon::empty().path(ICON_BROOM)
}

pub const NAV_AGENT: IconName = IconName::FolderOpen;
pub const NAV_STARTUP: IconName = IconName::Building2;
pub const NAV_PROCESS: IconName = IconName::SquareTerminal;
pub const NAV_LARGE_FILES: IconName = IconName::File;
pub const NAV_SETTINGS: IconName = IconName::Settings;

/// Brand / status
pub const BRAND_SHIELD: IconName = IconName::CircleCheck;
pub const STATUS_OK: IconName = IconName::CircleCheck;
pub const STATUS_INFO: IconName = IconName::Info;
pub const STATUS_LOADING: IconName = IconName::LoaderCircle;

/// Actions
pub const ACTION_SCAN: IconName = IconName::Search;
pub const ACTION_CLEAN: IconName = IconName::Delete;
pub const ACTION_OPEN_FOLDER: IconName = IconName::FolderOpen;
pub const ACTION_REFRESH: IconName = IconName::Loader;

/// Empty states (single large icon only)
pub const EMPTY_SCAN: IconName = IconName::Search;
pub const EMPTY_AGENT: IconName = IconName::FolderOpen;
pub const EMPTY_STARTUP: IconName = IconName::Building2;
pub const EMPTY_GENERIC: IconName = IconName::Inbox;
