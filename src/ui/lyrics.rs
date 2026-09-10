// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2025 Fezzik the Giant

//! Full-window presentation of lyrics with synced auto-scroll.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::*;
use crate::app::App;

pub(super) fn render_lyrics_view(f: &mut Frame, app: &App, area: Rect) {
    if area.height < 6 || area.width == 0 {
        return;
    }

    let (header_area, body_area, hud_area) = lyrics_view_layout(area);
    render_lyrics_header(f, app, header_area);
    render_lyrics_body(f, app, body_area);
    render_lyrics_hud(f, app, hud_area);
}

fn lyrics_view_layout(area: Rect) -> (Rect, Rect, Rect) {
    let [header, body, hud] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(4),
    ])
    .areas(area);
    (header, body, hud)
}

fn render_lyrics_header(f: &mut Frame, app: &App, area: Rect) {
    if area.is_empty() {
        return;
    }

    let (title, metadata) = match &app.now_playing.track {
        Some(track) => (
            track.title.as_str(),
            format!("{} · {}", track.all_artist_names(), track.album.title),
        ),
        None => ("No track playing", String::new()),
    };

    f.render_widget(
        Paragraph::new(title)
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center),
        Rect::new(area.x, area.y, area.width, 1),
    );

    if area.height > 1 {
        f.render_widget(
            Paragraph::new(metadata)
                .style(Style::default().fg(dim()))
                .alignment(Alignment::Center),
            Rect::new(area.x, area.y + 1, area.width, 1),
        );
    }

    if area.height > 2 {
        f.render_widget(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(dim())),
            Rect::new(area.x, area.y + 2, area.width, 1),
        );
    }
}

fn render_lyrics_body(f: &mut Frame, app: &App, area: Rect) {
    if area.is_empty() {
        return;
    }

    let np = &app.now_playing;

    if np.lyrics_loading {
        let msg = format!("{} Loading lyrics...", spinner_char(app.tick));
        f.render_widget(
            Paragraph::new(msg)
                .style(Style::default().fg(dim()))
                .alignment(Alignment::Center),
            Rect::new(area.x, area.y + area.height / 2, area.width, 1),
        );
        return;
    }

    let total = app.lyrics_line_count();
    if total == 0 {
        f.render_widget(
            Paragraph::new("No lyrics available")
                .style(Style::default().fg(dim()))
                .alignment(Alignment::Center),
            Rect::new(area.x, area.y + area.height / 2, area.width, 1),
        );
        return;
    }

    let active_idx = app.current_lyrics_line();
    let display_center = app.lyrics_scroll.unwrap_or(active_idx);
    let center_row = area.y + area.height / 2;

    let get_line = |idx: usize| -> &str {
        if !np.lyrics_synced.is_empty() {
            &np.lyrics_synced[idx].1
        } else {
            &np.lyrics_plain[idx]
        }
    };

    for idx in 0..total {
        let diff = idx as i32 - display_center as i32;
        let target_y = center_row as i32 + diff;

        if target_y < area.y as i32 || target_y >= (area.y + area.height) as i32 {
            continue;
        }

        let is_active = idx == active_idx;
        let is_selected = app.lyrics_scroll == Some(idx);

        let style = if is_active {
            Style::default()
                .fg(highlight_fg())
                .add_modifier(Modifier::BOLD)
        } else if diff.abs() <= 2 {
            Style::default().fg(highlight_dim())
        } else {
            Style::default().fg(dim())
        };

        let text = get_line(idx);
        let line_content = if is_selected && app.lyrics_scroll.is_some() {
            format!("▸ {text}")
        } else {
            text.to_string()
        };

        f.render_widget(
            Paragraph::new(line_content)
                .style(style)
                .alignment(Alignment::Center),
            Rect::new(area.x, target_y as u16, area.width, 1),
        );
    }
}

fn render_lyrics_hud(f: &mut Frame, app: &App, area: Rect) {
    if area.is_empty() {
        return;
    }

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(dim()));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let inset = 4.min(inner.width / 4);
    let rail_area = Rect::new(
        inner.x + inset,
        inner.y,
        inner.width.saturating_sub(inset * 2),
        1,
    );
    let played = progress_columns(rail_area.width, app.now_playing.progress_ratio());
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("━".repeat(played as usize), Style::default().fg(accent())),
            Span::styled(
                "━".repeat(rail_area.width.saturating_sub(played) as usize),
                Style::default().fg(dim()),
            ),
        ])),
        rail_area,
    );

    if inner.height > 1 {
        let left_text = format!(
            "{} / {}",
            app.now_playing.position_display(),
            app.now_playing.duration_display(),
        );

        let right_text = if app.lyrics_scroll.is_some() {
            "Manual scroll · [c] sync | [Esc/L] close".to_string()
        } else {
            "↑↓/jk: Scroll | [Esc/L] close".to_string()
        };

        let half = inner.width / 2;
        f.render_widget(
            Paragraph::new(left_text).style(Style::default().fg(dim())),
            Rect::new(inner.x + inset, inner.y + 1, half.saturating_sub(inset), 1),
        );
        f.render_widget(
            Paragraph::new(right_text)
                .style(Style::default().fg(dim()))
                .alignment(Alignment::Right),
            Rect::new(inner.x + half, inner.y + 1, half.saturating_sub(inset), 1),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::models::{Album, ArtistRef, Track};
    use crate::app::test_support::test_app;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn test_track() -> Track {
        Track {
            id: 1,
            title: "Lyrics Track".to_string(),
            duration: 200,
            artist: Some(ArtistRef {
                name: "Lyrics Artist".to_string(),
            }),
            artists: Vec::new(),
            album: Album {
                id: 2,
                title: "Lyrics Album".to_string(),
                number_of_tracks: None,
                release_date: None,
                cover: None,
                artist: None,
                media_metadata: None,
                added_at: None,
                album_type: None,
            },
            media_metadata: None,
            added_at: None,
        }
    }

    #[test]
    fn lyrics_layout_splits_expected_proportions() {
        let (header, body, hud) = lyrics_view_layout(Rect::new(0, 0, 80, 24));
        assert_eq!(header, Rect::new(0, 0, 80, 3));
        assert_eq!(body, Rect::new(0, 3, 80, 17));
        assert_eq!(hud, Rect::new(0, 20, 80, 4));
    }

    #[test]
    fn lyrics_view_renders_active_line_and_hud() {
        let mut t = test_app();
        t.app.lyrics_view = true;
        t.app.now_playing.track = Some(test_track());
        t.app.now_playing.lyrics_synced = vec![
            (0.0, "First lyric line".to_string()),
            (5.0, "Second lyric line".to_string()),
            (10.0, "Third lyric line".to_string()),
        ];
        t.app.now_playing.position = 6.0;
        t.app.now_playing.duration = 200.0;

        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|f| crate::ui::draw(f, &t.app)).unwrap();

        let rendered: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();

        assert!(rendered.contains("Lyrics Track"));
        assert!(rendered.contains("Lyrics Artist · Lyrics Album"));
        assert!(rendered.contains("Second lyric line"));
        assert!(!rendered.contains("show keybinds"));
    }

    #[test]
    fn lyrics_view_shows_loading_and_empty_states() {
        let mut t = test_app();
        t.app.lyrics_view = true;
        t.app.now_playing.track = Some(test_track());
        t.app.now_playing.lyrics_loading = true;

        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|f| crate::ui::draw(f, &t.app)).unwrap();

        let rendered: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered.contains("Loading lyrics..."));

        t.app.now_playing.lyrics_loading = false;
        terminal.draw(|f| crate::ui::draw(f, &t.app)).unwrap();
        let rendered2: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered2.contains("No lyrics available"));
    }
}
