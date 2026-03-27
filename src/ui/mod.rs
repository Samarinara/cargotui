pub mod menu;
pub mod output;
pub mod status_bar;
pub mod input;
pub mod help;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
};
use crate::app::{App, AppMode};
use menu::render_menu;
use output::render_output;
use status_bar::render_status_bar;
use input::render_input;
use help::render_help;

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

pub fn render(app: &mut App, frame: &mut Frame) {
    let full_area = frame.area();

    // Split into main area and status bar (last row)
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(full_area);

    let main_area = vertical[0];
    let status_area = vertical[1];

    // Split main area: 30% menu / 70% output
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(main_area);

    let menu_area = horizontal[0];
    let output_area = horizontal[1];

    render_menu(&app.menu, frame, menu_area);
    render_output(&app.output, frame, output_area);
    render_status_bar(app, frame, status_area);

    // Overlays
    match &app.mode {
        AppMode::Input(_) => {
            // Render input overlay centered in the output area
            let overlay = centered_rect(80, 30, output_area);
            if let Some(input_state) = &app.input {
                render_input(input_state, frame, overlay);
            }
        }
        AppMode::Help => {
            render_help(&app.mode, frame, full_area);
        }
        AppMode::Confirm(ctx) => {
            let overlay = centered_rect(50, 20, full_area);
            frame.render_widget(Clear, overlay);
            let message = ctx.message.clone();
            let paragraph = Paragraph::new(vec![
                Line::from(message.as_str()),
                Line::from(""),
                Line::from("Enter: Confirm   Esc/q: Cancel"),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Confirm"),
            )
            .style(Style::default().fg(Color::White).bg(Color::Black));
            frame.render_widget(paragraph, overlay);
        }
        _ => {}
    }
}
