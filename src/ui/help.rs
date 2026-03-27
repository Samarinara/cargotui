use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};
use crate::app::AppMode;

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

fn bindings_for_mode(mode: &AppMode) -> Vec<(&'static str, &'static str)> {
    match mode {
        AppMode::Menu => vec![
            ("↑ / k", "Move up"),
            ("↓ / j", "Move down"),
            ("Enter", "Select item"),
            ("Esc / q", "Go back"),
            ("?", "Open help"),
        ],
        AppMode::Input(_) => vec![
            ("Enter", "Submit input"),
            ("Esc", "Cancel"),
            ("Ctrl+A", "Move cursor to start"),
            ("Ctrl+E", "Move cursor to end"),
            ("Ctrl+U", "Clear input"),
        ],
        AppMode::Running => vec![
            ("Ctrl+C", "Kill running command"),
            ("j / ↓", "Scroll output down"),
            ("k / ↑", "Scroll output up"),
            ("c", "Clear output"),
        ],
        AppMode::Help => vec![
            ("?", "Close help"),
            ("Esc / q", "Close help"),
        ],
        AppMode::Confirm(_) => vec![
            ("Enter", "Confirm action"),
            ("Esc / q", "Cancel"),
        ],
        AppMode::Error(_) => vec![
            ("Esc / q", "Dismiss error"),
        ],
        AppMode::DepBrowser(_) => vec![
            ("↑ / k", "Move up"),
            ("↓ / j", "Move down"),
            ("Enter", "Open docs in browser"),
            ("Esc / q", "Go back"),
        ],
    }
}

pub fn render_help(mode: &AppMode, frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(60, 60, area);

    // Clear the background behind the popup
    frame.render_widget(Clear, popup_area);

    let bindings = bindings_for_mode(mode);

    let lines: Vec<Line> = bindings
        .iter()
        .map(|(key, desc)| {
            Line::from(vec![
                Span::styled(
                    format!("  {:20}", key),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(*desc),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help (? / Esc / q to close)"),
        )
        .style(Style::default().fg(Color::White).bg(Color::Black));

    frame.render_widget(paragraph, popup_area);
}
