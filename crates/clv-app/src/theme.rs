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
        super::hex(0x38bdf8)
    }
    pub fn accent_cyan() -> Hsla {
        super::hex(0x22d3ee)
    }
    pub fn accent_blue_bg() -> Hsla {
        super::hex(0x0c4a6e)
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

fn hex(v: u32) -> Hsla {
    rgb(v).into()
}

fn build_palette() -> ThemeColor {
    let mut t = *ThemeColor::dark();
    t.background = hex(0x0b1220);
    t.sidebar = hex(0x0a1628);
    t.foreground = hex(0xf0f9ff);
    t.muted_foreground = hex(0x94a3b8);
    t.popover = hex(0x132337);
    t.popover_foreground = hex(0xf0f9ff);
    t.border = hex(0x1e3a5f);
    t.input = hex(0x1e3a5f);
    t.muted = hex(0x132337);
    t.secondary = hex(0x0c4a6e);
    t.secondary_foreground = hex(0x94a3b8);
    t.secondary_hover = hex(0x155e75);
    t.secondary_active = hex(0x0c4a6e);
    t.primary = hex(0x0ea5e9);
    t.primary_foreground = hex(0xffffff);
    t.primary_hover = hex(0x38bdf8);
    t.primary_active = hex(0x0284c7);
    t.accent = hex(0x0c4a6e);
    t.accent_foreground = hex(0x22d3ee);
    t.success = hex(0x34d399);
    t.success_foreground = hex(0xffffff);
    t.success_hover = hex(0x4ade80);
    t.danger = hex(0xef4444);
    t.danger_foreground = hex(0xffffff);
    t.danger_hover = hex(0xf87171);
    t.danger_active = hex(0x2a1416);
    t.sidebar_foreground = hex(0x94a3b8);
    t.sidebar_accent = hex(0x0c4a6e);
    t.sidebar_accent_foreground = hex(0x22d3ee);
    t.sidebar_primary = hex(0x0ea5e9);
    t.sidebar_primary_foreground = hex(0xffffff);
    t.sidebar_border = hex(0x152238);
    t.list = hex(0x132337);
    t.list_hover = hex(0x0c4a6e);
    t.list_active = hex(0x155e75);
    t.table = hex(0x132337);
    t.table_head = hex(0x0b1220);
    t.scrollbar = hex(0x0a1628);
    t.scrollbar_thumb = hex(0x1e3a5f);
    t.scrollbar_thumb_hover = hex(0x38bdf8);
    t.ring = hex(0x22d3ee);
    t.link = hex(0x38bdf8);
    t.link_hover = hex(0x7dd3fc);
    t.progress_bar = hex(0x0ea5e9);
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
