// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature = "cli")]
use ratatui::style::Color;

/// Named theme variants available to the user.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeVariant {
    #[default]
    Dark,
    Light,
    Rust,
    Navy,
    Coal,
    Ayu,
}

#[cfg(feature = "cli")]
impl std::fmt::Display for ThemeVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dark => write!(f, "Dark"),
            Self::Light => write!(f, "Light"),
            Self::Rust => write!(f, "Rust"),
            Self::Navy => write!(f, "Navy"),
            Self::Coal => write!(f, "Coal"),
            Self::Ayu => write!(f, "Ayu"),
        }
    }
}

#[cfg(feature = "cli")]
pub const ALL_THEME_VARIANTS: &[ThemeVariant] = &[
    ThemeVariant::Dark,
    ThemeVariant::Light,
    ThemeVariant::Rust,
    ThemeVariant::Navy,
    ThemeVariant::Coal,
    ThemeVariant::Ayu,
];

/// A resolved color palette used by the TUI renderer.
#[cfg(feature = "cli")]
#[derive(Debug, Clone)]
pub struct ThemePalette {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub border: Color,
    pub ok: Color,
    pub warn: Color,
    pub err: Color,
    pub muted: Color,
    pub tab_active_bg: Color,
    pub tab_active_fg: Color,
    pub input_bg: Color,
    pub chart_1: Color,
    pub chart_2: Color,
    pub logo_gradient: [(u8, u8, u8); 6],
}

#[cfg(feature = "cli")]
impl ThemePalette {
    pub fn from_variant(variant: ThemeVariant) -> Self {
        match variant {
            ThemeVariant::Dark => Self {
                bg: Color::Rgb(18, 18, 28),
                fg: Color::Rgb(220, 220, 235),
                accent: Color::Rgb(130, 170, 255),
                border: Color::Rgb(80, 80, 120),
                ok: Color::Rgb(80, 200, 120),
                warn: Color::Rgb(240, 180, 60),
                err: Color::Rgb(220, 60, 80),
                muted: Color::Rgb(100, 100, 130),
                tab_active_bg: Color::Rgb(40, 40, 70),
                tab_active_fg: Color::Rgb(130, 170, 255),
                input_bg: Color::Rgb(24, 24, 38),
                chart_1: Color::Rgb(130, 170, 255),
                chart_2: Color::Rgb(80, 200, 120),
                logo_gradient: [
                    (255, 80, 180),
                    (220, 110, 200),
                    (180, 140, 230),
                    (140, 170, 245),
                    (100, 210, 250),
                    (60, 230, 240),
                ],
            },
            ThemeVariant::Light => Self {
                bg: Color::Rgb(248, 248, 252),
                fg: Color::Rgb(30, 30, 50),
                accent: Color::Rgb(60, 100, 200),
                border: Color::Rgb(180, 180, 210),
                ok: Color::Rgb(30, 140, 70),
                warn: Color::Rgb(180, 120, 0),
                err: Color::Rgb(180, 30, 50),
                muted: Color::Rgb(140, 140, 160),
                tab_active_bg: Color::Rgb(220, 225, 245),
                tab_active_fg: Color::Rgb(60, 100, 200),
                input_bg: Color::Rgb(240, 240, 248),
                chart_1: Color::Rgb(60, 100, 200),
                chart_2: Color::Rgb(30, 140, 70),
                logo_gradient: [
                    (200, 50, 150),
                    (170, 70, 170),
                    (130, 100, 200),
                    (90, 130, 210),
                    (50, 170, 220),
                    (20, 190, 210),
                ],
            },
            ThemeVariant::Rust => Self {
                bg: Color::Rgb(20, 14, 10),
                fg: Color::Rgb(235, 220, 200),
                accent: Color::Rgb(222, 100, 42),
                border: Color::Rgb(110, 60, 30),
                ok: Color::Rgb(120, 180, 80),
                warn: Color::Rgb(240, 170, 50),
                err: Color::Rgb(210, 50, 40),
                muted: Color::Rgb(110, 90, 75),
                tab_active_bg: Color::Rgb(50, 28, 14),
                tab_active_fg: Color::Rgb(222, 100, 42),
                input_bg: Color::Rgb(24, 16, 10),
                chart_1: Color::Rgb(222, 100, 42),
                chart_2: Color::Rgb(120, 180, 80),
                logo_gradient: [
                    (222, 100, 42),
                    (200, 110, 50),
                    (185, 125, 65),
                    (170, 140, 80),
                    (155, 160, 95),
                    (140, 180, 110),
                ],
            },
            ThemeVariant::Navy => Self {
                bg: Color::Rgb(10, 18, 38),
                fg: Color::Rgb(210, 220, 240),
                accent: Color::Rgb(80, 160, 240),
                border: Color::Rgb(40, 70, 130),
                ok: Color::Rgb(60, 200, 150),
                warn: Color::Rgb(240, 190, 60),
                err: Color::Rgb(220, 70, 80),
                muted: Color::Rgb(80, 100, 140),
                tab_active_bg: Color::Rgb(20, 40, 80),
                tab_active_fg: Color::Rgb(80, 160, 240),
                input_bg: Color::Rgb(12, 22, 46),
                chart_1: Color::Rgb(80, 160, 240),
                chart_2: Color::Rgb(60, 200, 150),
                logo_gradient: [
                    (80, 160, 240),
                    (90, 170, 240),
                    (100, 180, 235),
                    (110, 190, 230),
                    (120, 200, 220),
                    (130, 210, 210),
                ],
            },
            ThemeVariant::Coal => Self {
                bg: Color::Rgb(14, 14, 14),
                fg: Color::Rgb(200, 200, 200),
                accent: Color::Rgb(160, 160, 200),
                border: Color::Rgb(60, 60, 60),
                ok: Color::Rgb(100, 180, 100),
                warn: Color::Rgb(200, 170, 80),
                err: Color::Rgb(200, 70, 70),
                muted: Color::Rgb(90, 90, 90),
                tab_active_bg: Color::Rgb(30, 30, 30),
                tab_active_fg: Color::Rgb(160, 160, 200),
                input_bg: Color::Rgb(18, 18, 18),
                chart_1: Color::Rgb(160, 160, 200),
                chart_2: Color::Rgb(100, 180, 100),
                logo_gradient: [
                    (180, 130, 240),
                    (170, 140, 235),
                    (160, 150, 220),
                    (150, 160, 210),
                    (140, 170, 200),
                    (130, 180, 195),
                ],
            },
            ThemeVariant::Ayu => Self {
                bg: Color::Rgb(14, 17, 23),
                fg: Color::Rgb(200, 200, 185),
                accent: Color::Rgb(255, 167, 89),
                border: Color::Rgb(50, 60, 80),
                ok: Color::Rgb(145, 208, 110),
                warn: Color::Rgb(255, 186, 0),
                err: Color::Rgb(255, 85, 85),
                muted: Color::Rgb(80, 90, 110),
                tab_active_bg: Color::Rgb(28, 34, 46),
                tab_active_fg: Color::Rgb(255, 167, 89),
                input_bg: Color::Rgb(18, 22, 30),
                chart_1: Color::Rgb(255, 167, 89),
                chart_2: Color::Rgb(145, 208, 110),
                logo_gradient: [
                    (255, 167, 89),
                    (255, 180, 100),
                    (245, 190, 110),
                    (200, 200, 110),
                    (155, 210, 110),
                    (110, 220, 120),
                ],
            },
        }
    }
}
