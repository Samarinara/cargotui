use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use crate::app::{App, AppMode};

pub struct KeyBinding {
    pub key: &'static str,
    pub description: &'static str,
}

fn mode_name(mode: &AppMode) -> &'static str {
    match mode {
        AppMode::Menu => "Menu",
        AppMode::Input(_) => "Input",
        AppMode::Running => "Running",
        AppMode::Help => "Help",
        AppMode::Confirm(_) => "Confirm",
        AppMode::Error(_) => "Error",
    }
}

fn mode_hints(mode: &AppMode) -> &'static str {
    match mode {
        AppMode::Menu => "↑↓/jk: Navigate  Enter: Select  Esc/q: Back  ?: Help",
        AppMode::Input(_) => "Enter: Submit  Esc: Cancel  Ctrl+A/E: Start/End  Ctrl+U: Clear",
        AppMode::Running => "Ctrl+C: Kill  j/k: Scroll  c: Clear",
        AppMode::Help => "?: Close  Esc/q: Close",
        AppMode::Confirm(_) => "Enter: Confirm  Esc/q: Cancel",
        AppMode::Error(_) => "Esc/q: Dismiss",
    }
}

pub fn render_status_bar(app: &App, frame: &mut Frame, area: Rect) {
    let workspace_path = app
        .workspace
        .as_ref()
        .map(|w| w.root.display().to_string())
        .unwrap_or_else(|| "No workspace".to_string());

    let mode = mode_name(&app.mode);
    let hints = mode_hints(&app.mode);

    let dark_bg = Style::default().fg(Color::White).bg(Color::DarkGray);

    let line = Line::from(vec![
        Span::styled(workspace_path, dark_bg),
        Span::styled(" | ", dark_bg),
        Span::styled(mode, dark_bg),
        Span::styled(" | ", dark_bg),
        Span::styled(hints, dark_bg),
    ]);

    let paragraph = Paragraph::new(line).style(dark_bg);

    frame.render_widget(paragraph, area);
}
