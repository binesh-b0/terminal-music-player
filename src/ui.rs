use crate::app::App;
use crate::player::{PlaybackState, Player};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;
use std::path::Path;
use std::time::Duration;

pub fn draw(frame: &mut Frame, app: &App, player: &Player) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(frame.size());

    draw_header(frame, root[0], app, player);
    draw_body(frame, root[1], app, player);
    draw_footer(frame, root[2], app, player);

    if app.show_help {
        draw_help(frame, frame.size());
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App, player: &Player) {
    let title = Line::from(vec![
        Span::styled(
            "Terminal Music Player",
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("Shuffle: {}", if app.shuffle { "On" } else { "Off" }),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw("  "),
        Span::styled(
            format!("Repeat: {}", app.repeat.label()),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw("  "),
        Span::styled(
            format!("Vol: {:>3}%", (player.volume() * 100.0).round() as u32),
            Style::default().fg(if player.is_muted() {
                Color::DarkGray
            } else {
                Color::Green
            }),
        ),
    ]);

    let header = Paragraph::new(title).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Now Playing "),
    );
    frame.render_widget(header, area);
}

fn draw_body(frame: &mut Frame, area: Rect, app: &App, player: &Player) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    draw_playlist(frame, columns[0], app);
    draw_now_playing(frame, columns[1], player);
}

fn draw_playlist(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = (0..app.playlist.len())
        .filter_map(|i| app.playlist.track_at(i))
        .map(|path| ListItem::new(file_label(path)))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Playlist "),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("  ");

    let mut state = ratatui::widgets::ListState::default();
    if !app.playlist.is_empty() {
        state.select(Some(app.selection.min(app.playlist.len() - 1)));
    }
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_now_playing(frame: &mut Frame, area: Rect, player: &Player) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let track = player
        .current_track()
        .map(|p| file_label(p.as_path()))
        .unwrap_or_else(|| "—".to_string());

    let state = match player.state() {
        PlaybackState::Playing => "Playing",
        PlaybackState::Paused => "Paused",
        PlaybackState::Stopped => "Stopped",
    };

    let meta = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::styled("Track: ", Style::default().fg(Color::DarkGray)),
            Span::styled(track, Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("State: ", Style::default().fg(Color::DarkGray)),
            Span::styled(state, Style::default().fg(Color::Yellow)),
        ]),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Info "),
    );
    frame.render_widget(meta, rows[0]);

    let pos = player.position();
    let total = player.duration();
    let (label, ratio) = match total {
        Some(total) if total.as_millis() > 0 => {
            let ratio = (pos.as_secs_f64() / total.as_secs_f64()).clamp(0.0, 1.0);
            (format!("{} / {}", fmt_time(pos), fmt_time(total)), ratio)
        }
        _ => (fmt_time(pos), 0.0),
    };

    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Progress "),
        )
        .gauge_style(Style::default().fg(Color::Magenta))
        .label(label)
        .ratio(ratio);
    frame.render_widget(gauge, rows[1]);

    let hints = Paragraph::new(Text::from(vec![
        Line::from("Enter: play selected track"),
        Line::from("Space/P: play/pause   N/B: next/prev   S: stop"),
        Line::from("←/→: seek 5s   H: help   Z: shuffle   R: repeat"),
        Line::from("+/-: volume   M: mute   Q: quit"),
    ]))
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Controls "),
    );
    frame.render_widget(hints, rows[2]);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App, _player: &Player) {
    let status = if app.status.is_empty() {
        "Ready".to_string()
    } else {
        app.status.clone()
    };

    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
        Span::raw(status),
        Span::raw("  "),
        Span::styled("H", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": help"),
    ]));

    frame.render_widget(footer, area);
}

fn draw_help(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(70, 70, area);
    frame.render_widget(Clear, popup);

    let text = Text::from(vec![
        Line::from("Navigation"),
        Line::from("  ↑/↓    move selection"),
        Line::from("  Enter  play selected track"),
        Line::from(""),
        Line::from("Playback"),
        Line::from("  Space/P  play/pause"),
        Line::from("  S        stop"),
        Line::from("  N/B      next/previous"),
        Line::from("  ←/→      seek -/+ 5s"),
        Line::from(""),
        Line::from("Modes"),
        Line::from("  Z        toggle shuffle"),
        Line::from("  R        cycle repeat (Off/All/One)"),
        Line::from(""),
        Line::from("Audio"),
        Line::from("  +/-      volume"),
        Line::from("  M        mute"),
        Line::from(""),
        Line::from("Other"),
        Line::from("  H/?      toggle this help"),
        Line::from("  Q        quit"),
    ]);

    let help = Paragraph::new(text).wrap(Wrap { trim: true }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Help "),
    );
    frame.render_widget(help, popup);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn file_label(path: &Path) -> String {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| path.display().to_string())
}

fn fmt_time(duration: Duration) -> String {
    let total = duration.as_secs();
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h:02}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}
