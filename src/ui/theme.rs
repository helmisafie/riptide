// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2025 Fezzik the Giant

//! Colours, spacing constants and small formatting helpers.

use std::sync::atomic::{AtomicU8, Ordering};
use ratatui::style::{Color, Modifier, Style};
use crate::app::ThemePreset;

#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub accent: Color,
    pub dim: Color,
    pub highlight_bg: Color,
    pub highlight_fg: Color,
    pub highlight_dim: Color,
    pub highlight_accent: Color,
    pub select_bg: Color,
    pub select_fg: Color,
}

impl ThemePreset {
    pub fn colors(&self) -> ThemeColors {
        match self {
            Self::Tidal => ThemeColors {
                accent: Color::Cyan,
                dim: Color::DarkGray,
                highlight_bg: Color::Rgb(40, 40, 55),
                highlight_fg: Color::Rgb(236, 236, 245),
                highlight_dim: Color::Rgb(150, 150, 170),
                highlight_accent: Color::Rgb(120, 210, 232),
                select_bg: Color::Rgb(30, 100, 200),
                select_fg: Color::Rgb(236, 236, 245),
            },
            Self::Catppuccin => ThemeColors {
                accent: Color::Rgb(203, 166, 247), // Mauve
                dim: Color::Rgb(108, 112, 134),    // Overlay0
                highlight_bg: Color::Rgb(49, 50, 68), // Surface0
                highlight_fg: Color::Rgb(205, 214, 244), // Text
                highlight_dim: Color::Rgb(166, 173, 200), // Subtext0
                highlight_accent: Color::Rgb(245, 194, 231), // Pink
                select_bg: Color::Rgb(137, 180, 250), // Blue
                select_fg: Color::Rgb(17, 17, 27),
            },
            Self::TokyoNight => ThemeColors {
                accent: Color::Rgb(122, 162, 247), // Blue
                dim: Color::Rgb(86, 95, 137),      // Comment
                highlight_bg: Color::Rgb(41, 46, 66),
                highlight_fg: Color::Rgb(192, 202, 245),
                highlight_dim: Color::Rgb(115, 126, 170),
                highlight_accent: Color::Rgb(187, 154, 247), // Purple
                select_bg: Color::Rgb(65, 72, 104),
                select_fg: Color::Rgb(236, 236, 245),
            },
            Self::Nord => ThemeColors {
                accent: Color::Rgb(136, 192, 208), // Frost
                dim: Color::Rgb(76, 86, 106),
                highlight_bg: Color::Rgb(59, 66, 82),
                highlight_fg: Color::Rgb(236, 239, 244),
                highlight_dim: Color::Rgb(143, 188, 187),
                highlight_accent: Color::Rgb(129, 161, 193),
                select_bg: Color::Rgb(94, 129, 172),
                select_fg: Color::Rgb(236, 239, 244),
            },
            Self::Gruvbox => ThemeColors {
                accent: Color::Rgb(250, 189, 47), // Yellow
                dim: Color::Rgb(146, 131, 116),    // Gray
                highlight_bg: Color::Rgb(60, 56, 54),
                highlight_fg: Color::Rgb(235, 219, 178),
                highlight_dim: Color::Rgb(168, 153, 132),
                highlight_accent: Color::Rgb(254, 128, 25), // Orange
                select_bg: Color::Rgb(184, 187, 38), // Green
                select_fg: Color::Rgb(40, 40, 40),
            },
        }
    }
}

static ACTIVE_THEME: AtomicU8 = AtomicU8::new(0);

pub(super) fn set_active_theme(preset: ThemePreset) {
    let idx = ThemePreset::ALL.iter().position(|&p| p == preset).unwrap_or(0);
    ACTIVE_THEME.store(idx as u8, Ordering::Relaxed);
}

pub(super) fn current_theme_preset() -> ThemePreset {
    ThemePreset::ALL
        .get(ACTIVE_THEME.load(Ordering::Relaxed) as usize)
        .copied()
        .unwrap_or(ThemePreset::Tidal)
}

pub(super) fn theme() -> ThemeColors {
    current_theme_preset().colors()
}

pub(super) fn accent() -> Color {
    theme().accent
}

pub(super) fn dim() -> Color {
    theme().dim
}

#[allow(dead_code)]
pub(super) fn highlight_bg() -> Color {
    theme().highlight_bg
}

pub(super) fn highlight_fg() -> Color {
    theme().highlight_fg
}

pub(super) fn highlight_dim() -> Color {
    theme().highlight_dim
}

pub(super) fn select_bg() -> Color {
    theme().select_bg
}

pub(super) fn select_fg() -> Color {
    theme().select_fg
}

pub(super) fn fmt_sample_rate(hz: u32) -> String {
    match hz {
        44100 => "44.1 kHz".into(),
        88200 => "88.2 kHz".into(),
        176400 => "176.4 kHz".into(),
        _ => {
            let khz = hz / 1000;
            format!("{khz} kHz")
        }
    }
}

pub(super) fn row_style(is_selected: bool) -> Style {
    let t = theme();
    if is_selected {
        Style::default()
            .bg(t.highlight_bg)
            .fg(t.highlight_fg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    }
}

pub(super) fn row_dim_style(is_selected: bool) -> Style {
    let t = theme();
    if is_selected {
        Style::default().bg(t.highlight_bg).fg(t.highlight_dim)
    } else {
        Style::default().fg(t.dim)
    }
}

pub(super) fn row_accent_style(is_selected: bool) -> Style {
    let t = theme();
    let style = Style::default().add_modifier(Modifier::BOLD);
    if is_selected {
        style.bg(t.highlight_bg).fg(t.highlight_accent)
    } else {
        style.fg(t.accent)
    }
}

pub(super) fn responsive_queue_width(total_width: u16) -> u16 {
    if total_width < 90 {
        24
    } else if total_width < 120 {
        28
    } else if total_width < 160 {
        32
    } else {
        36
    }
}

pub(super) const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

pub(super) fn spinner_char(tick: u64) -> char {
    SPINNER[(tick / 3) as usize % SPINNER.len()]
}

pub(super) fn cursor_char(tick: u64) -> &'static str {
    if (tick / 30).is_multiple_of(2) { "█" } else { " " }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn responsive_queue_width_scales_with_terminal_width() {
        assert_eq!(responsive_queue_width(80), 24);
        assert_eq!(responsive_queue_width(100), 28);
        assert_eq!(responsive_queue_width(140), 32);
        assert_eq!(responsive_queue_width(180), 36);
    }
}
