use crate::app::{CratePickerState, CratePickerStatus};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

pub fn render_crate_picker(state: &CratePickerState, frame: &mut Frame, area: Rect) {
    // Clear the overlay area first
    frame.render_widget(Clear, area);

    let outer_block = Block::default().borders(Borders::ALL).title("Crate Picker");
    let inner_area = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    // Layout: filter row (1 line) | list (fill) | hint row (1 line)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(inner_area);

    let filter_area = chunks[0];
    let list_area = chunks[1];
    let hint_area = chunks[2];

    // Filter row
    let filter_text = format!("Filter: {}", state.filter);
    let filter_paragraph = Paragraph::new(filter_text.as_str());
    frame.render_widget(filter_paragraph, filter_area);

    // Hint row
    let hint = Paragraph::new("Enter: Select   Esc: Cancel");
    frame.render_widget(hint, hint_area);

    // List area — depends on status
    match &state.status {
        CratePickerStatus::Loading => {
            let paragraph = Paragraph::new("Loading…");
            frame.render_widget(paragraph, list_area);
        }
        CratePickerStatus::Error(msg) => {
            let paragraph = Paragraph::new(msg.as_str());
            frame.render_widget(paragraph, list_area);
        }
        CratePickerStatus::Loaded => {
            let filtered = state.filtered_packages();
            if filtered.is_empty() {
                let paragraph = Paragraph::new("No packages found");
                frame.render_widget(paragraph, list_area);
            } else {
                let items: Vec<ListItem> = filtered
                    .iter()
                    .map(|pkg| ListItem::new(format!("{} v{}", pkg.name, pkg.version)))
                    .collect();

                let list = List::new(items).highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );

                let mut list_state = ListState::default();
                list_state.select(Some(state.selected));

                frame.render_stateful_widget(list, list_area, &mut list_state);
            }
        }
    }
}
