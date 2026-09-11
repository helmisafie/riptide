// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2025 Fezzik the Giant

//! The now-playing bar: art, lyrics, track info and the waveform.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use super::*;
use crate::app::App;

pub(super) fn render_now_playing(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(dim()));
    let inner = block.inner(area); // height = 6 (7 - 1 border)
    f.render_widget(block, area);

    let sections = Layout::vertical([
        Constraint::Min(0),    // art (see below — it also spans the lyrics rows)
        Constraint::Length(1), // gap
        Constraint::Length(3), // lyrics — sits directly above the waveform row
        Constraint::Length(4), // track info / waveform / time and volume
    ])
    .split(inner);

    let cols = Layout::horizontal([
        Constraint::Percentage(35),
        Constraint::Percentage(30),
        Constraint::Percentage(35),
    ])
    .split(sections[3]);

    let art_h = sections[3].y.saturating_sub(inner.y).saturating_sub(1);
    let art_w = (art_h * 2).min(cols[0].width);
    render_now_playing_art(f, app, Rect::new(inner.x, inner.y, art_w, art_h));

    let inset = art_w + 1;
    let lyrics_area = if inner.width > inset * 2 + 20 {
        Rect::new(
            inner.x + inset,
            sections[2].y,
            inner.width - inset * 2,
            sections[2].height,
        )
    } else {
        sections[2]
    };
    render_lyrics(f, app, lyrics_area);

    let track_info: Vec<Line> = match &app.now_playing.track {
        Some(t) => {
            let quality_label: Option<String> = {
                let rate_str = app
                    .now_playing
                    .sample_rate
                    .or_else(|| {
                        app.now_playing
                            .delivered
                            .sample_rate
                            .and_then(|r| u32::try_from(r).ok())
                    })
                    .map(fmt_sample_rate);
                let codec_str = app
                    .now_playing
                    .codec
                    .as_deref()
                    .map(|c| c.split_whitespace().next().unwrap_or(c).to_uppercase());
                let depth_str = app
                    .now_playing
                    .delivered
                    .bit_depth
                    .map(|d| format!("{d}-bit"));
                let parts: Vec<String> = [codec_str, depth_str, rate_str]
                    .into_iter()
                    .flatten()
                    .collect();
                if parts.is_empty() {
                    None
                } else {
                    Some(parts.join(" · "))
                }
            };
            let heart = if app.favorite_track_ids.contains(&t.id) {
                " ❤"
            } else {
                ""
            };
            let mut lines = vec![
                Line::from(Span::styled(
                    format!("{}{}", t.title, heart),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    t.all_artist_names(),
                    Style::default().fg(Color::White),
                )),
                Line::from(Span::styled(
                    t.album.title.as_str(),
                    Style::default().fg(dim()),
                )),
            ];
            if let Some(label) = quality_label {
                lines.push(Line::from(Span::styled(
                    label,
                    Style::default().fg(accent()).add_modifier(Modifier::BOLD),
                )));
            }
            lines
        }
        None => vec![Line::from(Span::styled(
            "No track playing",
            Style::default().fg(dim()),
        ))],
    };
    f.render_widget(Paragraph::new(track_info), cols[0]);

    f.render_widget(render_squib(app, cols[1].width), cols[1]);

    let time_str = if app.now_playing.duration > 0.0 {
        let rem = (app.now_playing.duration - app.now_playing.position).max(0.0) as u32;
        format!(
            "{} / {} (-{}:{:02})",
            app.now_playing.position_display(),
            app.now_playing.duration_display(),
            rem / 60,
            rem % 60
        )
    } else {
        format!(
            "{} / {}",
            app.now_playing.position_display(),
            app.now_playing.duration_display()
        )
    };

    let vol = app.now_playing.volume;
    let head = (vol as usize * 6 / 100).min(6);
    let vol_bar = if head == 0 {
        "[──────]".to_string()
    } else {
        format!("[{}●{}]", "━".repeat(head - 1), "─".repeat(6 - head))
    };
    let volume_str = format!("Vol: {:>3}% {}", vol, vol_bar);

    let mut badge_spans = Vec::new();
    let playing = app.now_playing.active && !app.now_playing.paused;
    let play_badge = if playing {
        "▶ PLAY "
    } else if app.now_playing.active {
        "⏸ PAUSE "
    } else {
        "⏹ STOP "
    };
    badge_spans.push(Span::styled(
        play_badge,
        Style::default().fg(if playing { accent() } else { dim() }),
    ));

    if app.now_playing.shuffle {
        badge_spans.push(Span::styled("⇄ SHUF ", Style::default().fg(accent())));
    }
    if app.autoplay {
        badge_spans.push(Span::styled("∞ AUTO ", Style::default().fg(accent())));
    }
    let vis_badge = match app.visualizer {
        crate::app::VisualizerStyle::Waveform => "[WAVE]",
        crate::app::VisualizerStyle::Equalizer => "[EQ]",
        crate::app::VisualizerStyle::ProgressRail => "[RAIL]",
    };
    badge_spans.push(Span::styled(vis_badge, Style::default().fg(dim())));

    f.render_widget(
        Paragraph::new(vec![
            Line::from(time_str).alignment(Alignment::Right),
            Line::from(volume_str).alignment(Alignment::Right),
            Line::from(badge_spans).alignment(Alignment::Right),
        ])
        .style(Style::default().fg(dim())),
        cols[2],
    );
}

pub(super) fn render_now_playing_art(f: &mut Frame, app: &App, area: Rect) {
    let np = &app.now_playing;
    if area.width == 0 || area.height == 0 {
        return;
    }

    if let Some(bytes) = np.art_bytes() {
        if !render_image(f, bytes, area) {
            f.render_widget(
                Paragraph::new("No artwork")
                    .style(Style::default().fg(dim()))
                    .alignment(Alignment::Center),
                area,
            );
        }
    } else if np.art_loading {
        f.render_widget(
            Paragraph::new(spinner_char(app.tick).to_string())
                .style(Style::default().fg(dim()))
                .alignment(Alignment::Center),
            area,
        );
    }
}

pub(super) fn render_lyrics(f: &mut Frame, app: &App, area: Rect) {
    let np = &app.now_playing;

    if np.lyrics_loading {
        let spinner = spinner_char(app.tick);
        f.render_widget(
            Paragraph::new(spinner.to_string())
                .style(Style::default().fg(dim()))
                .alignment(Alignment::Center),
            Rect::new(area.x, area.y + 1, area.width, 1),
        );
        return;
    }

    let lines: &[(f64, String)];
    let plain_buf: Vec<(f64, String)>;

    if !np.lyrics_synced.is_empty() {
        lines = &np.lyrics_synced;
    } else if !np.lyrics_plain.is_empty() {
        let n = np.lyrics_plain.len() as f64;
        let dur = if np.duration > 0.0 { np.duration } else { n };
        plain_buf = np
            .lyrics_plain
            .iter()
            .enumerate()
            .map(|(i, t)| (i as f64 / n * dur, t.clone()))
            .collect();
        lines = &plain_buf;
    } else {
        return;
    }

    let pos = np.position;
    let cur = lines.partition_point(|(t, _)| *t <= pos).saturating_sub(1);

    let show: [Option<usize>; 3] = [
        cur.checked_sub(1),
        Some(cur),
        if cur + 1 < lines.len() {
            Some(cur + 1)
        } else {
            None
        },
    ];

    for (row, opt) in show.iter().enumerate() {
        if let Some(idx) = opt {
            let (_, text) = &lines[*idx];
            let is_cur = row == 1;
            let style = if is_cur {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(dim())
            };
            let y = area.y + row as u16;
            if y < area.y + area.height {
                f.render_widget(
                    Paragraph::new(text.as_str())
                        .style(style)
                        .alignment(Alignment::Center),
                    Rect::new(area.x, y, area.width, 1),
                );
            }
        }
    }
}

pub(super) fn render_squib(app: &App, width: u16) -> Paragraph<'static> {
    const WAVE: [&str; 8] = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
    const BARS: [&str; 8] = [" ", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

    let ratio = app.now_playing.progress_ratio();
    let played_w = ((width as f64 * ratio) as u16).min(width);
    let playing = app.now_playing.active && !app.now_playing.paused;

    let spans: Vec<Span<'static>> = (0..width)
        .map(|i| {
            let color = if i < played_w { accent() } else { dim() };
            let ch: &'static str = match app.visualizer {
                crate::app::VisualizerStyle::Waveform => {
                    if playing {
                        let phase = i as f64 * 0.8 + app.tick as f64 * 0.35;
                        let t = (phase.sin() + 1.0) / 2.0;
                        WAVE[(t * 7.99) as usize]
                    } else {
                        "▄"
                    }
                }
                crate::app::VisualizerStyle::Equalizer => {
                    if playing {
                        let p1 = i as f64 * 0.7 + app.tick as f64 * 0.45;
                        let p2 = i as f64 * 1.4 - app.tick as f64 * 0.3;
                        let t = ((p1.sin() + p2.cos() + 2.0) / 4.0).clamp(0.0, 1.0);
                        BARS[(t * 7.99) as usize]
                    } else {
                        " "
                    }
                }
                crate::app::VisualizerStyle::ProgressRail => {
                    if i + 1 == played_w {
                        "●"
                    } else if i < played_w {
                        "━"
                    } else {
                        "─"
                    }
                }
            };
            Span::styled(ch, Style::default().fg(color))
        })
        .collect();

    Paragraph::new(Line::from(spans))
}
