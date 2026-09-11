// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2025 Fezzik the Giant

//! Command palette and the artist-picker modal.

// ── Artist selection modal ────────────────────────────────────────────────────

#[derive(Default)]
pub struct ArtistSelection {
    pub active: bool,
    pub artist_names: Vec<String>,
    pub selected: usize,
    pub searching_for: Option<String>,
}

// ── Command palette ───────────────────────────────────────────────────────────

#[derive(Default)]
pub struct CommandState {
    pub active: bool,
    pub input: String,
    pub selected: usize,
}

impl CommandState {
    /// Destinations and presentation modes offered in the palette.
    pub const COMMANDS: &'static [&'static str] = &[
        "home",
        "tracks",
        "artists",
        "albums",
        "playlists",
        "search",
        "art",
        "lyrics",
        "theme",
        "theme:catppuccin",
        "theme:gruvbox",
        "theme:nord",
        "theme:tidal",
        "theme:tokyo-night",
        "visualizer",
        "visualizer:equalizer",
        "visualizer:progress-rail",
        "visualizer:waveform",
        "clear",
        "clear-upcoming",
        "autoplay",
        "autoplay:off",
        "autoplay:on",
    ];

    pub fn matches(&self) -> Vec<&'static str> {
        let q = self.input.to_lowercase();
        let mut matches: Vec<&'static str> = Self::COMMANDS
            .iter()
            .filter(|&&c| c.starts_with(q.as_str()))
            .copied()
            .collect();
        // "art" is a prefix of "artists", so without this typing the whole word
        // leaves the longer command selected and Enter runs that one instead.
        if let Some(exact) = matches.iter().position(|&c| c == q) {
            let cmd = matches.remove(exact);
            matches.insert(0, cmd);
        }
        matches
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_state_matches_prefix() {
        let mut cmd = CommandState::default();
        cmd.input = "tra".to_string();
        let matches = cmd.matches();
        assert!(matches.contains(&"tracks"));
        assert!(!matches.contains(&"artists"));
    }

    #[test]
    fn command_state_ranks_exact_match_first() {
        let partial = CommandState {
            input: "ar".to_string(),
            ..CommandState::default()
        };
        assert_eq!(partial.matches(), vec!["artists", "art"]);

        let exact = CommandState {
            input: "art".to_string(),
            ..CommandState::default()
        };
        assert_eq!(exact.matches(), vec!["art", "artists"]);
    }

    #[test]
    fn command_state_empty_input_matches_all() {
        let cmd = CommandState::default();
        let matches = cmd.matches();
        assert_eq!(matches.len(), CommandState::COMMANDS.len());
    }

    #[test]
    fn command_state_no_match_returns_empty() {
        let mut cmd = CommandState::default();
        cmd.input = "zzz".to_string();
        assert!(cmd.matches().is_empty());
    }

    #[test]
    fn command_state_theme_and_visualizer_match() {
        let mut cmd = CommandState::default();
        cmd.input = "theme:".to_string();
        let matches = cmd.matches();
        assert!(matches.contains(&"theme:catppuccin"));
        assert!(matches.contains(&"theme:nord"));

        cmd.input = "theme".to_string();
        let matches = cmd.matches();
        assert_eq!(matches[0], "theme");
    }
}
