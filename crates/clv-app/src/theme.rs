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
    pub is_light: bool,
    pub ring_center_bg: u32,
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
            is_light: false,
            ring_center_bg: 0x0a2540,
        }
    }

    /// Soft rose & lavender — light cherry-blossom palette on airy white-pink bases.
    fn blossom() -> Self {
        Self {
            accent: 0xf43f5e,
            accent_bg: 0xfff1f5,
            accent_bg_hover: 0xffe4e6,
            accent_bg_pressed: 0xfecdd3,
            accent_active: 0xe11d48,
            accent_pressed: 0xbe123c,
            accent_secondary: 0xc4b5fd,
            bg_app: 0xfff7f9,
            bg_sidebar: 0xfff1f5,
            bg_titlebar: 0xfffafb,
            bg_card: 0xffffff,
            border: 0xfbcfe8,
            panel_divider: 0xfce7f3,
            safe_green: 0x10b981,
            green: 0x059669,
            warn_orange: 0xf59e0b,
            red: 0xef4444,
            red_bg: 0xfff1f2,
            red_border: 0xfecaca,
            text_primary: 0x1f2937,
            text_secondary: 0x6b7280,
            text_muted: 0x9ca3af,
            gradient_hero_start: 0xffe4e6,
            gradient_hero_alt_start: 0xfff1f5,
            gradient_sidebar_start: 0xfffafb,
            gradient_sidebar_end: 0xfff1f5,
            gradient_content_start: 0xfff7f9,
            gradient_content_end: 0xfff1f5,
            gradient_titlebar_end: 0xfff1f5,
            gradient_warm_start: 0xf9a8d4,
            status_bar_bg: 0xfff1f5,
            risk_safe_bg: 0xd1fae5,
            risk_safe_border: 0xa7f3d0,
            risk_caution_bg: 0xfef3c7,
            risk_caution_border: 0xfde68a,
            risk_caution_fg: 0xd97706,
            corner_control: 8.,
            corner_md: 12.,
            is_light: true,
            ring_center_bg: 0xffffff,
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
            is_light: false,
            ring_center_bg: 0x0a2540,
        }
    }

    /// Fresh mint & sky — light airy greens on white-mint bases.
    fn aurora() -> Self {
        Self {
            accent: 0x14b8a6,
            accent_bg: 0xccfbf1,
            accent_bg_hover: 0x99f6e4,
            accent_bg_pressed: 0x5eead4,
            accent_active: 0x0d9488,
            accent_pressed: 0x0f766e,
            accent_secondary: 0x38bdf8,
            bg_app: 0xf0fdf9,
            bg_sidebar: 0xecfdf5,
            bg_titlebar: 0xf5fffe,
            bg_card: 0xffffff,
            border: 0xa7f3d0,
            panel_divider: 0xd1fae5,
            safe_green: 0x10b981,
            green: 0x059669,
            warn_orange: 0xf59e0b,
            red: 0xef4444,
            red_bg: 0xfff1f2,
            red_border: 0xfecaca,
            text_primary: 0x134e4a,
            text_secondary: 0x4b5563,
            text_muted: 0x6b7280,
            gradient_hero_start: 0xccfbf1,
            gradient_hero_alt_start: 0xecfdf5,
            gradient_sidebar_start: 0xf5fffe,
            gradient_sidebar_end: 0xecfdf5,
            gradient_content_start: 0xf0fdf9,
            gradient_content_end: 0xe6fffa,
            gradient_titlebar_end: 0xecfdf5,
            gradient_warm_start: 0x7dd3fc,
            status_bar_bg: 0xecfdf5,
            risk_safe_bg: 0xd1fae5,
            risk_safe_border: 0xa7f3d0,
            risk_caution_bg: 0xfef3c7,
            risk_caution_border: 0xfde68a,
            risk_caution_fg: 0xd97706,
            corner_control: 8.,
            corner_md: 10.,
            is_light: true,
            ring_center_bg: 0xffffff,
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

    pub fn is_light() -> bool {
        super::palette().is_light
    }

    pub fn ring_center_bg() -> Hsla {
        let p = super::palette();
        if p.is_light {
            super::hex(p.ring_center_bg)
        } else {
            super::hex(p.ring_center_bg).opacity(0.75)
        }
    }

    /// Elevated card / glass surface border.
    pub fn glass_border() -> Hsla {
        let p = super::palette();
        if p.is_light {
            super::hex(p.border).opacity(0.65)
        } else {
            super::hex(0xffffff).opacity(0.1)
        }
    }

    /// Elevated card / glass surface background.
    pub fn glass_bg() -> Hsla {
        let p = super::palette();
        if p.is_light {
            super::hex(p.bg_card)
        } else {
            super::hex(0xffffff).opacity(0.06)
        }
    }

    pub fn glass_bg_hover() -> Hsla {
        let p = super::palette();
        if p.is_light {
            super::hex(p.accent_bg_hover)
        } else {
            super::hex(0xffffff).opacity(0.09)
        }
    }

    pub fn glass_bg_active() -> Hsla {
        let p = super::palette();
        if p.is_light {
            super::hex(p.accent_bg)
        } else {
            super::hex(0xffffff).opacity(0.05)
        }
    }

    pub fn glass_bg_soft() -> Hsla {
        let p = super::palette();
        if p.is_light {
            super::hex(p.bg_app)
        } else {
            super::hex(0xffffff).opacity(0.04)
        }
    }

    pub fn from_hex(v: u32) -> Hsla {
        super::hex(v)
    }
}

fn build_gpui_palette(p: ThemePalette) -> ThemeColor {
    let mut t = if p.is_light {
        *ThemeColor::light()
    } else {
        *ThemeColor::dark()
    };
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
    t.success_foreground = if p.is_light {
        hex(p.green)
    } else {
        hex(0xffffff)
    };
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
    t.switch = if p.is_light {
        hex(0xd1d5db)
    } else {
        hex(p.border)
    };
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
    Theme::change(
        if palette.is_light {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        },
        None,
        cx,
    );
    let theme = Theme::global_mut(cx);
    theme.colors = build_gpui_palette(palette);
    theme.radius = corner_control();
    theme.radius_lg = corner_md();
    theme.scrollbar_show = ScrollbarShow::Hover;
    theme.font_size = font_base();
    theme.shadow = !palette.is_light;
}

/// Apply the default defender theme (alias for startup).
pub fn apply_clv_theme(cx: &mut App) {
    apply_theme(ThemePreference::Defender, cx);
}
