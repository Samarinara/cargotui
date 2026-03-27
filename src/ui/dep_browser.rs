use crate::cargo::metadata::PackageInfo;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

/// Returns the docs.rs URL for a given package name and version.
pub fn build_doc_url(name: &str, version: &str) -> String {
    format!("https://docs.rs/{}/{}", name, version)
}

/// Returns the display string for a package row in the list.
pub fn format_package_row(pkg: &PackageInfo) -> String {
    format!("{} v{}", pkg.name, pkg.version)
}

/// Returns the status message shown after successfully opening docs.
pub fn format_open_success_message(name: &str, version: &str) -> String {
    format!("Opening docs for {} v{}…", name, version)
}

/// Opens a URL in the system default browser.
pub fn open_url(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .spawn()?;
    }
    Ok(())
}

pub fn render_dep_browser(state: &crate::app::DepBrowserState, frame: &mut Frame, area: Rect) {
    use crate::app::DepBrowserStatus;

    // Split area vertically when there's a message: 90% content, 10% message
    let (content_area, message_area) = if state.message.is_some() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(90), Constraint::Min(1)])
            .split(area);
        (chunks[0], Some(chunks[1]))
    } else {
        (area, None)
    };

    let block = Block::default().borders(Borders::ALL).title("Dependencies");

    match &state.status {
        DepBrowserStatus::Loading => {
            let paragraph = Paragraph::new("Loading dependencies…")
                .block(block)
                .alignment(ratatui::layout::Alignment::Center);
            frame.render_widget(paragraph, content_area);
        }
        DepBrowserStatus::Error(msg) => {
            let paragraph = Paragraph::new(msg.as_str()).block(block);
            frame.render_widget(paragraph, content_area);
        }
        DepBrowserStatus::Loaded => {
            if state.packages.is_empty() {
                let paragraph = Paragraph::new("No packages found").block(block);
                frame.render_widget(paragraph, content_area);
            } else {
                let items: Vec<ListItem> = state
                    .packages
                    .iter()
                    .map(|pkg| ListItem::new(format_package_row(pkg)))
                    .collect();

                let list = List::new(items).block(block).highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );

                let mut list_state = ListState::default();
                list_state.select(Some(state.selected));

                frame.render_stateful_widget(list, content_area, &mut list_state);
            }
        }
    }

    // Render message in the bottom area if present
    if let (Some(msg), Some(msg_area)) = (&state.message, message_area) {
        let paragraph = Paragraph::new(msg.as_str());
        frame.render_widget(paragraph, msg_area);
    }
}
