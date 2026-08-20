//! CLV3000 Plus design tokens — security-software aesthetic.

use gpui::{px, rgb, App, Hsla, Pixels};
use gpui_component::{scroll::ScrollbarShow, Theme, ThemeColor, ThemeMode};

/// Square corners for cards, panels, and primary buttons.
pub fn corner() -> Pixels {
    px(0.)
}

/// Subtle radius for tags, badges, and labels.
pub fn corner_sm() -> Pixels {
    px(4.)
}

/// Soft radius for sidebar icon containers.
pub fn corner_md() -> Pixels {
    px(8.)
}

/// gpui-component control radius (Switch, etc.).
pub fn corner_control() -> Pixels {
    px(6.)
}

/// Base UI font size (desktop-friendly on Windows / macOS).
pub fn font_base() -> Pixels {
    px(16.)
}

fn hex(v: u32) -> Hsla {
    rgb(v).into()
}

/// Primary accent — icons, progress, selected states.
const ACCENT: u32 = 0x3AA0E0;
/// Accent surface — hover / active backgrounds.
const ACCENT_BG: u32 = 0x113247;
/// Pressed primary controls (slightly darker than accent).
const ACCENT_ACTIVE: u32 = 0x2D8BC4;
/// Lifted accent background for nested list/secondary hover.
const ACCENT_BG_HOVER: u32 = 0x1A4360;

/// Design token colors.
pub mod colors {
    use gpui::Hsla;

    pub fn bg_app() -> Hsla {
        super::hex(0x0b1220)
    }
    pub fn bg_sidebar() -> Hsla {
        super::hex(0x0a1628)
    }
    pub fn bg_titlebar() -> Hsla {
        super::hex(0x0c1a2e)
    }
    pub fn bg_card() -> Hsla {
        super::hex(0x132337)
    }
    pub fn border() -> Hsla {
        super::hex(0x1e3a5f)
    }
    pub fn panel_divider() -> Hsla {
        super::hex(0x152238)
    }
    pub fn accent_blue() -> Hsla {
        super::hex(super::ACCENT)
    }
    /// Alias — progress rings, spinners, and legacy call sites use the same accent.
    pub fn accent_cyan() -> Hsla {
        accent_blue()
    }
    pub fn accent_blue_bg() -> Hsla {
        super::hex(super::ACCENT_BG)
    }
    pub fn safe_green() -> Hsla {
        super::hex(0x34d399)
    }
    pub fn green() -> Hsla {
        super::hex(0x22c55e)
    }
    pub fn warn_orange() -> Hsla {
        super::hex(0xfb923c)
    }
    pub fn red() -> Hsla {
        super::hex(0xef4444)
    }
    pub fn red_bg() -> Hsla {
        super::hex(0x2a1416)
    }
    pub fn red_border() -> Hsla {
        super::hex(0x502024)
    }
    pub fn text_primary() -> Hsla {
        super::hex(0xf0f9ff)
    }
    pub fn text_secondary() -> Hsla {
        super::hex(0x94a3b8)
    }
    pub fn text_muted() -> Hsla {
        super::hex(0x64748b)
    }

    pub fn from_hex(v: u32) -> Hsla {
        super::hex(v)
    }
}

fn build_palette() -> ThemeColor {
    let mut t = *ThemeColor::dark();
    let accent = hex(ACCENT);
    let accent_bg = hex(ACCENT_BG);
    let accent_active = hex(ACCENT_ACTIVE);
    let accent_bg_hover = hex(ACCENT_BG_HOVER);

    t.background = hex(0x0b1220);
    t.sidebar = hex(0x0a1628);
    t.foreground = hex(0xf0f9ff);
    t.muted_foreground = hex(0x94a3b8);
    t.popover = hex(0x132337);
    t.popover_foreground = hex(0xf0f9ff);
    t.border = hex(0x1e3a5f);
    t.input = hex(0x1e3a5f);
    t.muted = hex(0x132337);
    t.secondary = accent_bg;
    t.secondary_foreground = hex(0x94a3b8);
    t.secondary_hover = accent_bg_hover;
    t.secondary_active = accent_bg;
    t.primary = accent;
    t.primary_foreground = hex(0xffffff);
    t.primary_hover = accent_active;
    t.primary_active = accent_active;
    t.accent = accent_bg;
    t.accent_foreground = accent;
    t.info = accent;
    t.info_foreground = hex(0xffffff);
    t.info_hover = accent_active;
    t.info_active = accent_active;
    t.success = hex(0x34d399);
    t.success_foreground = hex(0xffffff);
    t.success_hover = hex(0x4ade80);
    t.danger = hex(0xef4444);
    t.danger_foreground = hex(0xffffff);
    t.danger_hover = hex(0xf87171);
    t.danger_active = hex(0x2a1416);
    t.sidebar_foreground = hex(0x94a3b8);
    t.sidebar_accent = accent_bg;
    t.sidebar_accent_foreground = accent;
    t.sidebar_primary = accent;
    t.sidebar_primary_foreground = hex(0xffffff);
    t.sidebar_border = hex(0x152238);
    t.list = hex(0x132337);
    t.list_hover = accent_bg;
    t.list_active = accent_bg_hover;
    t.list_active_border = accent;
    t.table = hex(0x132337);
    t.table_head = hex(0x0b1220);
    t.table_hover = accent_bg;
    t.table_active = accent_bg_hover;
    t.table_active_border = accent;
    t.scrollbar = hex(0x0a1628);
    t.scrollbar_thumb = hex(0x1e3a5f);
    t.scrollbar_thumb_hover = accent;
    t.ring = accent;
    t.link = accent;
    t.link_hover = accent;
    t.link_active = accent;
    t.progress_bar = accent;
    t.selection = accent_bg;
    t.caret = accent;
    t.drag_border = accent;
    t.drop_target = accent_bg;
    t.switch = hex(0x1e3a5f);
    t.switch_thumb = hex(0xffffff);
    t.slider_bar = accent;
    t.slider_thumb = hex(0xffffff);
    t.chart_1 = accent;
    t.chart_2 = accent_active;
    t.chart_3 = accent_bg_hover;
    t.chart_4 = accent_bg;
    t.chart_5 = hex(0x1e3a5f);
    t.blue = accent;
    t.blue_light = accent;
    t.cyan = accent;
    t.cyan_light = accent;
    t.title_bar = hex(0x0c1a2e);
    t.title_bar_border = hex(0x152238);
    t
}

/// Apply the CLV security-software theme.
pub fn apply_clv_theme(cx: &mut App) {
    Theme::change(ThemeMode::Dark, None, cx);
    let theme = Theme::global_mut(cx);
    theme.colors = build_palette();
    theme.radius = corner_control();
    theme.radius_lg = corner_md();
    theme.scrollbar_show = ScrollbarShow::Hover;
    theme.font_size = font_base();
    theme.shadow = true;
}
