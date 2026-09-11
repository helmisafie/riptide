// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2025 Fezzik the Giant

//! Sort fields, the sort palette, and the preferences persisted to config.

use super::*;

// ── Sort palette ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum SortField {
    #[default]
    Alphabetical,
    LastAdded,
    ByArtist,
}

impl SortField {
    /// Short label for the sort indicator shown in list titles.
    pub fn label(self) -> &'static str {
        match self {
            SortField::Alphabetical => "A-Z",
            SortField::LastAdded => "Recent",
            SortField::ByArtist => "Artist",
        }
    }
}

// ── Themes and Visualizer ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemePreset {
    #[default]
    Tidal,
    Catppuccin,
    TokyoNight,
    Nord,
    Gruvbox,
}

impl ThemePreset {
    pub const ALL: &'static [ThemePreset] = &[
        ThemePreset::Tidal,
        ThemePreset::Catppuccin,
        ThemePreset::TokyoNight,
        ThemePreset::Nord,
        ThemePreset::Gruvbox,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Tidal => "tidal",
            Self::Catppuccin => "catppuccin",
            Self::TokyoNight => "tokyo-night",
            Self::Nord => "nord",
            Self::Gruvbox => "gruvbox",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Tidal => "Tidal Cyan",
            Self::Catppuccin => "Catppuccin Mocha",
            Self::TokyoNight => "Tokyo Night",
            Self::Nord => "Nord",
            Self::Gruvbox => "Gruvbox Dark",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Tidal => Self::Catppuccin,
            Self::Catppuccin => Self::TokyoNight,
            Self::TokyoNight => Self::Nord,
            Self::Nord => Self::Gruvbox,
            Self::Gruvbox => Self::Tidal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VisualizerStyle {
    #[default]
    Waveform,
    Equalizer,
    ProgressRail,
}

impl VisualizerStyle {
    pub const ALL: &'static [VisualizerStyle] = &[
        VisualizerStyle::Waveform,
        VisualizerStyle::Equalizer,
        VisualizerStyle::ProgressRail,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Waveform => "waveform",
            Self::Equalizer => "equalizer",
            Self::ProgressRail => "progress-rail",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Waveform => "Waveform",
            Self::Equalizer => "Equalizer",
            Self::ProgressRail => "Progress Rail",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Waveform => Self::Equalizer,
            Self::Equalizer => Self::ProgressRail,
            Self::ProgressRail => Self::Waveform,
        }
    }
}

// ── Persisted preferences ────────────────────────────────────────────────────

/// User choices that survive restarts, stored inside `Config`.
///
/// Every field carries a serde default so configs written by older versions
/// still load — an absent key falls back to the value the app used before this
/// existed, rather than failing the whole parse and logging the user out.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Preferences {
    #[serde(default)]
    pub tracks_sort: Option<SortField>,
    #[serde(default)]
    pub artists_sort: Option<SortField>,
    #[serde(default)]
    pub fav_albums_sort: Option<SortField>,
    #[serde(default)]
    pub playlists_sort: Option<SortField>,
    #[serde(default = "default_volume")]
    pub volume: u8,
    #[serde(default)]
    pub shuffle: bool,
    #[serde(default = "default_true")]
    pub queue_visible: bool,
    #[serde(default)]
    pub theme: ThemePreset,
    #[serde(default)]
    pub visualizer: VisualizerStyle,
    #[serde(default = "default_true")]
    pub autoplay: bool,
}

fn default_volume() -> u8 {
    100
}
fn default_true() -> bool {
    true
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            tracks_sort: None,
            artists_sort: None,
            fav_albums_sort: None,
            playlists_sort: None,
            volume: default_volume(),
            shuffle: false,
            queue_visible: default_true(),
            theme: ThemePreset::default(),
            visualizer: VisualizerStyle::default(),
            autoplay: default_true(),
        }
    }
}

#[derive(Default)]
pub struct SortPalette {
    pub active: bool,
    pub selected: usize,
}

impl SortPalette {
    pub fn get_options(current_tab: Tab) -> &'static [(&'static str, SortField)] {
        match current_tab {
            Tab::Home => &[],
            Tab::Artists | Tab::Playlists => &[
                ("Alphabetical", SortField::Alphabetical),
                ("Last Added", SortField::LastAdded),
            ],
            Tab::Albums | Tab::Favorites => &[
                ("Alphabetical", SortField::Alphabetical),
                ("By Artist", SortField::ByArtist),
                ("Last Added", SortField::LastAdded),
            ],
            Tab::Search => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_preset_cycles_through_all_variants() {
        let mut t = ThemePreset::Tidal;
        for &expected in ThemePreset::ALL {
            assert_eq!(t, expected);
            t = t.next();
        }
        assert_eq!(t, ThemePreset::Tidal);
    }

    #[test]
    fn visualizer_style_cycles_through_all_variants() {
        let mut v = VisualizerStyle::Waveform;
        for &expected in VisualizerStyle::ALL {
            assert_eq!(v, expected);
            v = v.next();
        }
        assert_eq!(v, VisualizerStyle::Waveform);
    }

    #[test]
    fn preferences_deserializes_missing_fields_with_defaults() {
        let json = r#"{"volume":80,"shuffle":true}"#;
        let prefs: Preferences = serde_json::from_str(json).unwrap();
        assert_eq!(prefs.volume, 80);
        assert!(prefs.shuffle);
        assert_eq!(prefs.theme, ThemePreset::Tidal);
        assert_eq!(prefs.visualizer, VisualizerStyle::Waveform);
        assert!(prefs.autoplay);
    }

    #[test]
    fn preferences_roundtrip_preserves_theme_and_visualizer() {
        let prefs = Preferences {
            theme: ThemePreset::Catppuccin,
            visualizer: VisualizerStyle::Equalizer,
            autoplay: false,
            ..Default::default()
        };
        let serialized = serde_json::to_string(&prefs).unwrap();
        let deserialized: Preferences = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.theme, ThemePreset::Catppuccin);
        assert_eq!(deserialized.visualizer, VisualizerStyle::Equalizer);
        assert!(!deserialized.autoplay);
    }
}
