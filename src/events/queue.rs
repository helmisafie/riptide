// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2025 Fezzik the Giant

//! Input for the queue panel.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::*;
use crate::app::App;

pub(super) fn handle_queue_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Left | KeyCode::Char('h') => {
            app.unfocus_queue();
        }
        KeyCode::Char('K') => app.move_queue_track_up(),
        KeyCode::Up if !key.modifiers.is_empty() => app.move_queue_track_up(),
        KeyCode::Char('J') => app.move_queue_track_down(),
        KeyCode::Down if !key.modifiers.is_empty() => app.move_queue_track_down(),
        KeyCode::Up => {
            if app.queue_cursor > 0 {
                app.queue_cursor -= 1;
            }
        }
        KeyCode::Down => {
            let len = app.now_playing.queue.len();
            if len > 0 && app.queue_cursor + 1 < len {
                app.queue_cursor += 1;
            }
        }
        KeyCode::Home => app.queue_cursor = 0,
        KeyCode::End => app.queue_cursor = app.now_playing.queue.len().saturating_sub(1),
        KeyCode::Char('.' | 'o') => app.jump_queue_to_playing(),
        KeyCode::PageUp => app.queue_page_up(),
        KeyCode::PageDown => app.queue_page_down(),
        KeyCode::Enter => app.play_from_queue(app.queue_cursor),
        KeyCode::Char('D' | 'X') => app.clear_upcoming_queue(),
        KeyCode::Char('d' | 'x') | KeyCode::Delete => {
            app.remove_from_queue(app.queue_cursor);
        }
        KeyCode::Char('f') => {
            if let Some(track) = app.now_playing.queue.get(app.queue_cursor).cloned() {
                app.toggle_favorite_track(&track);
            }
        }
        KeyCode::Char('c') => {
            if let Some(url) = app
                .now_playing
                .queue
                .get(app.queue_cursor)
                .map(|t| t.share_url())
            {
                app.copy_url(url);
            }
        }
        KeyCode::Char('C') => {
            if let Some(url) = app
                .now_playing
                .queue
                .get(app.queue_cursor)
                .map(|t| t.album.share_url())
            {
                app.copy_url(url);
            }
        }
        // Targets the cursor rather than the playing track: get_selected_track()
        // (used by the global binding) has no notion of the queue cursor and
        // would jump to the wrong artist while the queue is focused.
        KeyCode::Char('g') => {
            if let Some(track) = app.now_playing.queue.get(app.queue_cursor).cloned() {
                app.go_to_artist_from_track(&track);
            } else {
                app.set_status(
                    "No track selected".to_string(),
                    crate::app::StatusLevel::Error,
                );
            }
        }
        KeyCode::Char('G') => {
            if let Some(track) = app.now_playing.queue.get(app.queue_cursor).cloned() {
                app.go_to_album_from_track(&track);
            } else {
                app.set_status(
                    "No track selected".to_string(),
                    crate::app::StatusLevel::Error,
                );
            }
        }
        KeyCode::Char('r') => {
            if let Some(track) = app.now_playing.queue.get(app.queue_cursor).cloned() {
                app.start_track_radio(&track);
            } else {
                app.set_status(
                    "No track selected".to_string(),
                    crate::app::StatusLevel::Error,
                );
            }
        }
        KeyCode::Char('P') => {
            app.move_queue_track_to_play_next(app.queue_cursor);
        }
        // Anything the queue doesn't claim falls through to the global bindings
        // so transport, volume, tabs and help keep working in here.
        _ => {
            handle_global_key(app, key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_support::{test_app, track};

    #[test]
    fn handle_queue_input_reorders_with_j_and_k() {
        let mut t = test_app();
        t.app.play_tracks((1..=4).map(track).collect(), 0);
        t.app.focus_queue();
        t.app.queue_cursor = 1;

        handle_queue_input(&mut t.app, KeyEvent::new(KeyCode::Char('J'), KeyModifiers::SHIFT));
        assert_eq!(t.app.queue_cursor, 2);
        assert_eq!(t.app.now_playing.queue[2].id, 2);

        handle_queue_input(&mut t.app, KeyEvent::new(KeyCode::Char('K'), KeyModifiers::SHIFT));
        assert_eq!(t.app.queue_cursor, 1);
        assert_eq!(t.app.now_playing.queue[1].id, 2);
    }

    #[test]
    fn handle_queue_input_reorders_with_alt_arrows() {
        let mut t = test_app();
        t.app.play_tracks((1..=4).map(track).collect(), 0);
        t.app.focus_queue();
        t.app.queue_cursor = 2;

        handle_queue_input(&mut t.app, KeyEvent::new(KeyCode::Up, KeyModifiers::ALT));
        assert_eq!(t.app.queue_cursor, 1);
        assert_eq!(t.app.now_playing.queue[1].id, 3);

        handle_queue_input(&mut t.app, KeyEvent::new(KeyCode::Down, KeyModifiers::ALT));
        assert_eq!(t.app.queue_cursor, 2);
        assert_eq!(t.app.now_playing.queue[2].id, 3);
    }

    #[test]
    fn handle_queue_input_removes_with_x() {
        let mut t = test_app();
        t.app.play_tracks((1..=3).map(track).collect(), 0);
        t.app.focus_queue();
        t.app.queue_cursor = 1;

        handle_queue_input(&mut t.app, KeyEvent::from(KeyCode::Char('x')));
        assert_eq!(t.app.now_playing.queue.len(), 2);
        assert_eq!(t.app.now_playing.queue[1].id, 3);
    }

    #[test]
    fn handle_queue_input_clears_upcoming_with_x_capital() {
        let mut t = test_app();
        t.app.play_tracks((1..=5).map(track).collect(), 1); // playing track 2
        t.app.focus_queue();

        handle_queue_input(&mut t.app, KeyEvent::new(KeyCode::Char('X'), KeyModifiers::SHIFT));
        assert_eq!(t.app.now_playing.queue.len(), 2);
        assert_eq!(t.app.now_playing.queue[1].id, 2);
    }

    #[test]
    fn handle_queue_input_jumps_to_playing_with_dot() {
        let mut t = test_app();
        t.app.play_tracks((1..=10).map(track).collect(), 4);
        t.app.focus_queue();
        t.app.queue_cursor = 8;

        handle_queue_input(&mut t.app, KeyEvent::from(KeyCode::Char('.')));
        assert_eq!(t.app.queue_cursor, 4);

        handle_queue_input(&mut t.app, KeyEvent::from(KeyCode::Home));
        assert_eq!(t.app.queue_cursor, 0);

        handle_queue_input(&mut t.app, KeyEvent::from(KeyCode::End));
        assert_eq!(t.app.queue_cursor, 9);
    }
}
