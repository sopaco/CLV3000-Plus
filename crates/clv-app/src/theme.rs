//! CLV3000 Plus design tokens and multi-theme skin system.

use clv_core::ThemePreference;
use gpui::{px, rgb, App, Hsla, Pixels};
use gpui_component::{scroll::ScrollbarShow, Theme, ThemeColor, ThemeMode};
use std::sync::{LazyLock, RwLock};

/// Active palette — updated when the user switches themes.
static PALETTE: LazyLock<RwLock<ThemePalette>> =
    LazyLock::new(|| RwLock::new(ThemePalette::defender()));

#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub accent: u32,
    pub accent_bg: u32,
    pub accent_bg_hover: u32,
    pub accent_bg_pressed: u32,
    pub accent_active: u32,
    pub accent_pressed: u32,
    pub accent_secondary: u32,
    pub bg_app: u32,
    pub bg_sidebar: u32,
    pub bg_titlebar: u32,
    pub bg_card: u32,
    pub border: u32,
    pub panel_divider: u32,
    pub safe_green: u32,
    pub green: u32,
    pub warn_orange: u32,
    pub red: u32,
    pub red_bg: u32,
    pub red_border: u32,
    pub text_primary: u32,
    pub text_secondary: u32,
    pub text_muted: u32,
    pub gradient_hero_start: u32,
    pub gradient_hero_alt_start: u32,
    pub gradient_sidebar_start: u32,
    pub gradient_sidebar_end: u32,
    pub gradient_content_start: u32,
    pub gradient_content_end: u32,
    pub gradient_titlebar_end: u32,
    pub gradient_warm_start: u32,
    pub status_bar_bg: u32,
    pub risk_safe_bg: u32,
    pub risk_safe_border: u32,
    pub risk_caution_bg: u32,
    pub risk_caution_border: u32,
    pub risk_caution_fg: u32,
    pub corner_control: f32,
    pub corner_md: f32,
}

impl ThemePalette {
    fn defender() -> Self {
        Self {
            accent: 0x3aa0e0,
            accent_bg: 0x113247,
            accent_bg_hover: 0x1a4360,
            accent_bg_pressed: 0x0d2840,
            accent_active: 0x2d8bc4,
            accent_pressed: 0x2478ad,
            accent_secondary: 0xa78bfa,
            bg_app: 0x0b1220,
            bg_sidebar: 0x0a1628,
            bg_titlebar: 0x0c1a2e,
            bg_card: 0x132337,
            border: 0x1e3a5f,
            panel_divider: 0x152238,
            safe_green: 0x34d399,
            green: 0x22c55e,
            warn_orange: 0xfb923c,
            red: 0xef4444,
            red_bg: 0x2a1416,
            red_border: 0x502024,
            text_primary: 0xf0f9ff,
            text_secondary: 0x94a3b8,
            text_muted: 0x64748b,
            gradient_hero_start: 0x1e3a5f,
            gradient_hero_alt_start: 0x0f172a,
            gradient_sidebar_start: 0x0c1222,
            gradient_sidebar_end: 0x101c30,
            gradient_content_start: 0x0b1120,
            gradient_content_end: 0x111b2e,
            gradient_titlebar_end: 0x0d1b2e,
            gradient_warm_start: 0x818cf8,
            status_bar_bg: 0x0a1628,
            risk_safe_bg: 0x0f2a1a,
            risk_safe_border: 0x1a4d2e,
            risk_caution_bg: 0x2a2414,
            risk_caution_border: 0x4d3d1a,
            risk_caution_fg: 0xf59e0b,
            corner_control: 6.,
            corner_md: 8.,
        }
    }

    /// Soft rose & lavender — airy cherry-blossom palette with lifted mid-tones.
    fn blossom() -> Self {
        Self {
            accent: 0xfb7185,
            accent_bg: 0x5c3048,
            accent_bg_hover: 0x6e3a56,
            accent_bg_pressed: 0x4a2838,
            accent_active: 0xf43f5e,
            accent_pressed: 0xe11d48,
            accent_secondary: 0xc4b5fd,
            bg_app: 0x2a2030,
            bg_sidebar: 0x261c2c,
            bg_titlebar: 0x2e2438,
            bg_card: 0x3a3040,
            border: 0x7a5a6e,
            panel_divider: 0x4a3848,
            safe_green: 0xfda4af,
            green: 0xfb7185,
            warn_orange: 0xfcd34d,
            red: 0xf87171,
            red_bg: 0x4a2838,
            red_border: 0x7a4050,
            text_primary: 0xfff5f7,
            text_secondary: 0xf0c4d0,
            text_muted: 0xb894a4,
            gradient_hero_start: 0x7a5068,
            gradient_hero_alt_start: 0x3a2838,
            gradient_sidebar_start: 0x241a28,
            gradient_sidebar_end: 0x302438,
            gradient_content_start: 0x2a2030,
            gradient_content_end: 0x363040,
            gradient_titlebar_end: 0x302438,
            gradient_warm_start: 0xe879f9,
            status_bar_bg: 0x261c2c,
            risk_safe_bg: 0x4a3848,
            risk_safe_border: 0x6e5060,
            risk_caution_bg: 0x4a4030,
            risk_caution_border: 0x7a6040,
            risk_caution_fg: 0xfcd34d,
            corner_control: 8.,
            corner_md: 12.,
        }
    }

    /// Electric cyan & purple — high-energy youth aesthetic.
    fn neon() -> Self {
        Self {
            accent: 0x22d3ee,
            accent_bg: 0x1e1b4b,
            accent_bg_hover: 0x2a2560,
            accent_bg_pressed: 0x151238,
            accent_active: 0x06b6d4,
            accent_pressed: 0x0891b2,
            accent_secondary: 0xa855f7,
            bg_app: 0x09090f,
            bg_sidebar: 0x0c0c18,
            bg_titlebar: 0x0e0e1a,
            bg_card: 0x141428,
            border: 0x312e81,
            panel_divider: 0x1a1a30,
            safe_green: 0x4ade80,
            green: 0x22c55e,
            warn_orange: 0xfbbf24,
            red: 0xf43f5e,
            red_bg: 0x2a1020,
            red_border: 0x4a1830,
            text_primary: 0xf0fdfa,
            text_secondary: 0x94a3b8,
            text_muted: 0x64748b,
            gradient_hero_start: 0x312e81,
            gradient_hero_alt_start: 0x0a0a14,
            gradient_sidebar_start: 0x080810,
            gradient_sidebar_end: 0x12122a,
            gradient_content_start: 0x09090f,
            gradient_content_end: 0x141428,
            gradient_titlebar_end: 0x12122a,
            gradient_warm_start: 0xa855f7,
            status_bar_bg: 0x0c0c18,
            risk_safe_bg: 0x0f2a1a,
            risk_safe_border: 0x1a4d2e,
            risk_caution_bg: 0x2a2414,
            risk_caution_border: 0x4d3d1a,
            risk_caution_fg: 0xfbbf24,
            corner_control: 4.,
            corner_md: 6.,
        }
    }

    /// Fresh mint & sky — light sage greens with bright teal accents.
    fn aurora() -> Self {
        Self {
            accent: 0x5eead4,
            accent_bg: 0x1e5e54,
            accent_bg_hover: 0x267060,
            accent_bg_pressed: 0x184840,
            accent_active: 0x2dd4bf,
            accent_pressed: 0x14b8a6,
            accent_secondary: 0x7dd3fc,
            bg_app: 0x1a302a,
            bg_sidebar: 0x162a24,
            bg_titlebar: 0x1c3430,
            bg_card: 0x243e38,
            border: 0x3d6b60,
            panel_divider: 0x2a4a40,
            safe_green: 0x86efac,
            green: 0x4ade80,
            warn_orange: 0xfcd34d,
            red: 0xf87171,
            red_bg: 0x3a2828,
            red_border: 0x604040,
            text_primary: 0xf0fdf9,
            text_secondary: 0xa8d4c4,
            text_muted: 0x78a898,
            gradient_hero_start: 0x2a6e60,
            gradient_hero_alt_start: 0x1a3028,
            gradient_sidebar_start: 0x142820,
            gradient_sidebar_end: 0x203830,
            gradient_content_start: 0x1a302a,
            gradient_content_end: 0x243e38,
            gradient_titlebar_end: 0x203830,
            gradient_warm_start: 0x7dd3fc,
            status_bar_bg: 0x162a24,
            risk_safe_bg: 0x1a4030,
            risk_safe_border: 0x2a6050,
            risk_caution_bg: 0x3a3420,
            risk_caution_border: 0x5a5030,
            risk_caution_fg: 0xfcd34d,
            corner_control: 8.,
            corner_md: 10.,
        }
    }

    pub fn for_preference(pref: ThemePreference) -> Self {
        match pref {
            ThemePreference::Defender => Self::defender(),
            ThemePreference::Blossom => Self::blossom(),
            ThemePreference::Neon => Self::neon(),
            ThemePreference::Aurora => Self::aurora(),
        }
    }

    /// Preview swatches for the settings theme picker (accent, secondary, card bg).
    pub fn preview_swatches(self) -> [u32; 3] {
        [self.accent, self.accent_secondary, self.bg_card]
    }
}

fn palette() -> ThemePalette {
    *PALETTE.read().unwrap_or_else(|e| e.into_inner())
}

fn hex(v: u32) -> Hsla {
    rgb(v).into()
}

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
    px(palette().corner_md)
}

/// gpui-component control radius (Switch, etc.).
pub fn corner_control() -> Pixels {
    px(palette().corner_control)
}

/// Base UI font size (desktop-friendly on Windows / macOS).
pub fn font_base() -> Pixels {
    px(16.)
}

/// Design token colors — read from the active theme palette.
pub mod colors {
    use gpui::Hsla;

    pub fn bg_app() -> Hsla {
        super::hex(super::palette().bg_app)
    }
    pub fn bg_sidebar() -> Hsla {
        super::hex(super::palette().bg_sidebar)
    }
    pub fn bg_titlebar() -> Hsla {
        super::hex(super::palette().bg_titlebar)
    }
    pub fn bg_card() -> Hsla {
        super::hex(super::palette().bg_card)
    }
    pub fn border() -> Hsla {
        super::hex(super::palette().border)
    }
    pub fn panel_divider() -> Hsla {
        super::hex(super::palette().panel_divider)
    }
    pub fn accent_blue() -> Hsla {
        super::hex(super::palette().accent)
    }
    pub fn accent_cyan() -> Hsla {
        accent_blue()
    }
    pub fn accent_secondary() -> Hsla {
        super::hex(super::palette().accent_secondary)
    }
    pub fn accent_blue_bg() -> Hsla {
        super::hex(super::palette().accent_bg)
    }
    pub fn accent_blue_bg_hover() -> Hsla {
        super::hex(super::palette().accent_bg_hover)
    }
    pub fn accent_blue_bg_pressed() -> Hsla {
        super::hex(super::palette().accent_bg_pressed)
    }
    pub fn accent_blue_pressed() -> Hsla {
        super::hex(super::palette().accent_pressed)
    }
    pub fn safe_green() -> Hsla {
        super::hex(super::palette().safe_green)
    }
    pub fn green() -> Hsla {
        super::hex(super::palette().green)
    }
    pub fn warn_orange() -> Hsla {
        super::hex(super::palette().warn_orange)
    }
    pub fn red() -> Hsla {
        super::hex(super::palette().red)
    }
    pub fn red_bg() -> Hsla {
        super::hex(super::palette().red_bg)
    }
    pub fn red_border() -> Hsla {
        super::hex(super::palette().red_border)
    }
    pub fn text_primary() -> Hsla {
        super::hex(super::palette().text_primary)
    }
    pub fn text_secondary() -> Hsla {
        super::hex(super::palette().text_secondary)
    }
    pub fn text_muted() -> Hsla {
        super::hex(super::palette().text_muted)
    }
    pub fn status_bar_bg() -> Hsla {
        super::hex(super::palette().status_bar_bg)
    }
    pub fn risk_safe_bg() -> Hsla {
        super::hex(super::palette().risk_safe_bg)
    }
    pub fn risk_safe_border() -> Hsla {
        super::hex(super::palette().risk_safe_border)
    }
    pub fn risk_caution_bg() -> Hsla {
        super::hex(super::palette().risk_caution_bg)
    }
    pub fn risk_caution_border() -> Hsla {
        super::hex(super::palette().risk_caution_border)
    }
    pub fn risk_caution_fg() -> Hsla {
        super::hex(super::palette().risk_caution_fg)
    }
    pub fn gradient_hero_start() -> Hsla {
        super::hex(super::palette().gradient_hero_start)
    }
    pub fn gradient_hero_alt_start() -> Hsla {
        super::hex(super::palette().gradient_hero_alt_start)
    }
    pub fn gradient_sidebar_start() -> Hsla {
        super::hex(super::palette().gradient_sidebar_start)
    }
    pub fn gradient_sidebar_end() -> Hsla {
        super::hex(super::palette().gradient_sidebar_end)
    }
    pub fn gradient_content_start() -> Hsla {
        super::hex(super::palette().gradient_content_start)
    }
    pub fn gradient_content_end() -> Hsla {
        super::hex(super::palette().gradient_content_end)
    }
    pub fn gradient_titlebar_end() -> Hsla {
        super::hex(super::palette().gradient_titlebar_end)
    }
    pub fn gradient_warm_start() -> Hsla {
        super::hex(super::palette().gradient_warm_start)
    }

    pub fn from_hex(v: u32) -> Hsla {
        super::hex(v)
    }
}

fn build_gpui_palette(p: ThemePalette) -> ThemeColor {
    let mut t = *ThemeColor::dark();
    let accent = hex(p.accent);
    let accent_bg = hex(p.accent_bg);
    let accent_active = hex(p.accent_active);
    let accent_pressed = hex(p.accent_pressed);
    let accent_bg_hover = hex(p.accent_bg_hover);
    let accent_bg_pressed = hex(p.accent_bg_pressed);

    t.background = hex(p.bg_app);
    t.sidebar = hex(p.bg_sidebar);
    t.foreground = hex(p.text_primary);
    t.muted_foreground = hex(p.text_secondary);
    t.popover = hex(p.bg_card);
    t.popover_foreground = hex(p.text_primary);
    t.border = hex(p.border);
    t.input = hex(p.border);
    t.muted = hex(p.bg_card);
    t.secondary = accent_bg;
    t.secondary_foreground = hex(p.text_secondary);
    t.secondary_hover = accent_bg_hover;
    t.secondary_active = accent_bg_pressed;
    t.primary = accent;
    t.primary_foreground = hex(0xffffff);
    t.primary_hover = accent_active;
    t.primary_active = accent_pressed;
    t.accent = accent_bg;
    t.accent_foreground = accent;
    t.info = accent;
    t.info_foreground = hex(0xffffff);
    t.info_hover = accent_active;
    t.info_active = accent_pressed;
    t.success = hex(p.safe_green);
    t.success_foreground = hex(0xffffff);
    t.success_hover = hex(p.green);
    t.danger = hex(p.red);
    t.danger_foreground = hex(0xffffff);
    t.danger_hover = hex(p.red);
    t.danger_active = hex(p.red_bg);
    t.sidebar_foreground = hex(p.text_secondary);
    t.sidebar_accent = accent_bg;
    t.sidebar_accent_foreground = accent;
    t.sidebar_primary = accent;
    t.sidebar_primary_foreground = hex(0xffffff);
    t.sidebar_border = hex(p.panel_divider);
    t.list = hex(p.bg_card);
    t.list_hover = accent_bg;
    t.list_active = accent_bg_pressed;
    t.list_active_border = accent;
    t.table = hex(p.bg_card);
    t.table_head = hex(p.bg_app);
    t.table_hover = accent_bg;
    t.table_active = accent_bg_pressed;
    t.table_active_border = accent;
    t.scrollbar = hex(p.bg_sidebar);
    t.scrollbar_thumb = hex(p.border);
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
    t.switch = hex(p.border);
    t.switch_thumb = hex(0xffffff);
    t.slider_bar = accent;
    t.slider_thumb = hex(0xffffff);
    t.chart_1 = accent;
    t.chart_2 = accent_active;
    t.chart_3 = accent_bg_hover;
    t.chart_4 = accent_bg;
    t.chart_5 = hex(p.border);
    t.blue = accent;
    t.blue_light = accent;
    t.cyan = accent;
    t.cyan_light = accent;
    t.title_bar = hex(p.bg_titlebar);
    t.title_bar_border = hex(p.panel_divider);
    t
}

/// Apply the selected visual theme to gpui-component and internal tokens.
pub fn apply_theme(preference: ThemePreference, cx: &mut App) {
    let palette = ThemePalette::for_preference(preference);
    if let Ok(mut guard) = PALETTE.write() {
        *guard = palette;
    }
    Theme::change(ThemeMode::Dark, None, cx);
    let theme = Theme::global_mut(cx);
    theme.colors = build_gpui_palette(palette);
    theme.radius = corner_control();
    theme.radius_lg = corner_md();
    theme.scrollbar_show = ScrollbarShow::Hover;
    theme.font_size = font_base();
    theme.shadow = true;
}

/// Apply the default defender theme (alias for startup).
pub fn apply_clv_theme(cx: &mut App) {
    apply_theme(ThemePreference::Defender, cx);
}
