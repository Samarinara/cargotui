use crate::app::{MenuLevel, MenuState};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

pub fn render_menu(menu: &MenuState, frame: &mut Frame, area: Rect) {
    let level: &MenuLevel = match menu.stack.last() {
        Some(l) => l,
        None => return,
    };

    // Split area: 80% list, 20% description
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(area);

    // Build list items from node names
    let items: Vec<ListItem> = level
        .nodes
        .iter()
        .map(|node| ListItem::new(node.name))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Commands"))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ListState::default();
    list_state.select(Some(level.selected));

    frame.render_stateful_widget(list, chunks[0], &mut list_state);

    // Description of the currently selected node
    let description = level
        .nodes
        .get(level.selected)
        .map(|node| node.description)
        .unwrap_or("");

    let paragraph = Paragraph::new(Text::raw(description))
        .block(Block::default().borders(Borders::ALL).title("Description"))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, chunks[1]);
}
