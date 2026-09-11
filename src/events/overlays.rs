// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2025 Fezzik the Giant

//! Input for the modal overlays: command palette, sort picker, help, artist picker.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::*;
use crate::app::{App, SortPalette, Tab};

pub(super) fn handle_command_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.command.active = false;
        }
        KeyCode::Enter => {
            let matches = app.command.matches();
            let typed = app.command.input.trim().to_string();
            let cmd = matches
                .get(app.command.selected)
                .or_else(|| matches.first())
                .copied()
                .unwrap_or(&typed);
            execute_command(app, cmd);
        }
        KeyCode::Tab => {
            // Accept ghost-text completion.
            if let Some(&first) = app.command.matches().first() {
                app.command.input = first.to_string();
                app.command.selected = 0;
            }
        }
        KeyCode::Up => {
            if app.command.selected > 0 {
                app.command.selected -= 1;
            }
        }
        KeyCode::Down => {
            let len = app.command.matches().len();
            if app.command.selected + 1 < len {
                app.command.selected += 1;
            }
        }
        KeyCode::Backspace => {
            app.command.input.pop();
            app.command.selected = 0;
        }
        KeyCode::Char(c) => {
            app.command.input.push(c);
            app.command.selected = 0;
        }
        _ => {}
    }
}

pub(super) fn execute_command(app: &mut App, cmd: &str) {
    app.command.active = false;
    app.command.input.clear();
    let cleanup = |app: &App| {
        if leaving_album(app) {
            kitty_delete_album_art();
        }
        if leaving_artist(app) {
            kitty_delete_artist_art();
        }
    };
    match cmd {
        "home" => {
            cleanup(app);
            app.set_tab(Tab::Home);
        }
        "tracks" => {
            cleanup(app);
            app.set_tab(Tab::Favorites);
        }
        "artists" => {
            cleanup(app);
            app.set_tab(Tab::Artists);
        }
        "albums" => {
            cleanup(app);
            app.set_tab(Tab::Albums);
        }
        "playlists" => {
            cleanup(app);
            app.set_tab(Tab::Playlists);
        }
        "search" => {
            cleanup(app);
            app.set_tab(Tab::Search);
            app.search.modal_open = true;
            app.search.query.clear();
        }
        "art" => {
            app.enter_art_fullscreen();
        }
        "lyrics" => {
            app.enter_lyrics_view();
        }
        "theme" => {
            app.cycle_theme();
        }
        "visualizer" => {
            app.cycle_visualizer();
        }
        c if c.starts_with("theme:") || c.starts_with("theme ") => {
            app.set_theme_by_name(&c[6..]);
        }
        c if c.starts_with("visualizer:") || c.starts_with("visualizer ") => {
            app.set_visualizer_by_name(&c[11..]);
        }
        "clear" | "clear-queue" => {
            app.clear_queue();
        }
        "clear-upcoming" => {
            app.clear_upcoming_queue();
        }
        "autoplay" => {
            app.toggle_autoplay();
        }
        "autoplay:on" | "autoplay on" => {
            app.set_autoplay(true);
        }
        "autoplay:off" | "autoplay off" => {
            app.set_autoplay(false);
        }
        _ => {}
    }
}

pub(super) fn handle_sort_palette_input(app: &mut App, key: KeyEvent) {
    let options = SortPalette::get_options(app.current_tab);
    let count = options.len();
    match key.code {
        KeyCode::Esc => {
            app.sort_palette.active = false;
        }
        KeyCode::Up => {
            if app.sort_palette.selected > 0 {
                app.sort_palette.selected -= 1;
            }
        }
        KeyCode::Down => {
            if app.sort_palette.selected + 1 < count {
                app.sort_palette.selected += 1;
            }
        }
        KeyCode::Enter => {
            if let Some((_, field)) = options.get(app.sort_palette.selected) {
                app.apply_sort(*field);
            }
        }
        _ => {}
    }
}

pub(super) fn handle_help_input(app: &mut App, key: KeyEvent) {
    use crate::app::KeybindGroup;

    const HELP_QUERY_MAX: usize = 48;

    let content_h = app.help_content_h.get();
    let max_scroll =
        |query: &str| KeybindGroup::total_help_lines_filtered(query).saturating_sub(content_h);

    match key.code {
        KeyCode::Esc => {
            if app.help_query.is_empty() {
                app.help_active = false;
            }
            app.help_query.clear();
            app.help_scroll = 0;
        }
        KeyCode::Up => {
            app.help_scroll = app.help_scroll.saturating_sub(1);
        }
        KeyCode::Down => {
            let max = max_scroll(&app.help_query);
            app.help_scroll = (app.help_scroll + 1).min(max);
        }
        KeyCode::PageUp => {
            app.help_scroll = app.help_scroll.saturating_sub(10);
        }
        KeyCode::PageDown => {
            let max = max_scroll(&app.help_query);
            app.help_scroll = (app.help_scroll + 10).min(max);
        }
        KeyCode::Backspace => {
            if app.help_query.pop().is_some() {
                app.help_scroll = 0;
            }
        }
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) && (c == 'u' || c == 'U') {
                if !app.help_query.is_empty() {
                    app.help_query.clear();
                    app.help_scroll = 0;
                }
            } else if !key.modifiers.contains(KeyModifiers::CONTROL)
                && !key.modifiers.contains(KeyModifiers::ALT)
            {
                // Refusing a leading space keeps the query never-whitespace-only,
                // which is what lets everything else test it with `is_empty`:
                // the filter trims, so " " would match everything while the box
                // looked filled and Esc read as clear-not-close.
                let leading_space = app.help_query.is_empty() && c.is_whitespace();
                if !leading_space && app.help_query.chars().count() < HELP_QUERY_MAX {
                    app.help_query.push(c);
                    app.help_scroll = 0;
                }
            }
        }
        _ => {}
    }
}

pub(super) fn handle_artist_selection_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.artist_selection.active = false;
        }
        KeyCode::Up => {
            if app.artist_selection.selected > 0 {
                app.artist_selection.selected -= 1;
            }
        }
        KeyCode::Down => {
            let len = app.artist_selection.artist_names.len();
            if app.artist_selection.selected + 1 < len {
                app.artist_selection.selected += 1;
            }
        }
        KeyCode::Enter => {
            app.open_selected_artist_from_selection();
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{ThemePreset, VisualizerStyle};
    use crate::app::test_support::test_app;

    #[test]
    fn execute_command_cycles_and_sets_theme() {
        let mut t = test_app();
        assert_eq!(t.app.theme, ThemePreset::Tidal);

        execute_command(&mut t.app, "theme");
        assert_eq!(t.app.theme, ThemePreset::Catppuccin);

        execute_command(&mut t.app, "theme:nord");
        assert_eq!(t.app.theme, ThemePreset::Nord);
    }

    #[test]
    fn execute_command_cycles_and_sets_visualizer() {
        let mut t = test_app();
        assert_eq!(t.app.visualizer, VisualizerStyle::Waveform);

        execute_command(&mut t.app, "visualizer");
        assert_eq!(t.app.visualizer, VisualizerStyle::Equalizer);

        execute_command(&mut t.app, "visualizer:progress-rail");
        assert_eq!(t.app.visualizer, VisualizerStyle::ProgressRail);
    }

    #[test]
    fn handle_command_input_sets_theme_by_space_syntax() {
        let mut t = test_app();
        t.app.command.active = true;
        t.app.command.input = "theme gruvbox".to_string();

        handle_command_input(&mut t.app, KeyEvent::from(KeyCode::Enter));
        assert_eq!(t.app.theme, ThemePreset::Gruvbox);
        assert!(!t.app.command.active);
    }

    #[test]
    fn execute_command_clears_queue_and_upcoming() {
        use crate::app::test_support::track;
        let mut t = test_app();
        t.app.play_tracks((1..=5).map(track).collect(), 1);

        execute_command(&mut t.app, "clear-upcoming");
        assert_eq!(t.app.now_playing.queue.len(), 2);

        execute_command(&mut t.app, "clear");
        assert!(t.app.now_playing.queue.is_empty());
        assert!(t.app.now_playing.track.is_none());
    }

    #[test]
    fn execute_command_toggles_and_sets_autoplay() {
        let mut t = test_app();
        t.app.autoplay = true;

        execute_command(&mut t.app, "autoplay");
        assert!(!t.app.autoplay);

        execute_command(&mut t.app, "autoplay:on");
        assert!(t.app.autoplay);

        execute_command(&mut t.app, "autoplay:off");
        assert!(!t.app.autoplay);
    }
}
