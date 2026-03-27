use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::collections::VecDeque;
use std::process::ExitStatus;

pub struct OutputBuffer {
    pub history: VecDeque<CommandOutput>,
    pub current: usize,
    pub scroll_offset: usize,
    pub auto_scroll: bool,
}

pub struct CommandOutput {
    pub command_label: String,
    pub lines: Vec<String>,
    pub exit_status: Option<ExitStatus>,
}

impl OutputBuffer {
    pub fn new() -> Self {
        OutputBuffer {
            history: VecDeque::new(),
            current: 0,
            scroll_offset: 0,
            auto_scroll: true,
        }
    }

    /// Appends a line to the current (last) CommandOutput in history.
    /// If history is empty, creates a new CommandOutput.
    /// If auto_scroll is true, scrolls to bottom.
    pub fn push_line(&mut self, line: String) {
        if self.history.is_empty() {
            self.history.push_back(CommandOutput {
                command_label: String::new(),
                lines: Vec::new(),
                exit_status: None,
            });
        }
        if let Some(entry) = self.history.back_mut() {
            entry.lines.push(line);
        }
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    /// Starts a new command output entry. Caps history at 10.
    pub fn start_command(&mut self, label: String) {
        if self.history.len() >= 10 {
            self.history.pop_front();
        }
        self.history.push_back(CommandOutput {
            command_label: label,
            lines: Vec::new(),
            exit_status: None,
        });
        self.scroll_offset = 0;
    }

    /// Sets the exit_status on the last entry.
    pub fn finish_command(&mut self, exit_status: ExitStatus) {
        if let Some(entry) = self.history.back_mut() {
            entry.exit_status = Some(exit_status);
        }
    }

    /// Decreases scroll_offset by amount (clamped to 0), disables auto_scroll.
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
        self.auto_scroll = false;
    }

    /// Increases scroll_offset by amount.
    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_add(amount);
    }

    /// Sets scroll_offset to usize::MAX (render will clamp) and enables auto_scroll.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = usize::MAX;
        self.auto_scroll = true;
    }

    /// Clears the lines of the current (last) entry.
    pub fn clear_current(&mut self) {
        if let Some(entry) = self.history.back_mut() {
            entry.lines.clear();
        }
    }
}

pub fn render_output(buffer: &OutputBuffer, frame: &mut Frame, area: Rect) {
    let entry = buffer.history.back();

    let mut lines: Vec<Line> = match entry {
        Some(e) => e.lines.iter().map(|l| Line::from(l.as_str())).collect(),
        None => vec![],
    };

    // Append exit status line if present
    if let Some(e) = entry {
        if let Some(status) = &e.exit_status {
            let (msg, color) = if status.success() {
                (format!("✓ Process exited successfully (0)"), Color::Green)
            } else {
                let code = status.code().unwrap_or(-1);
                (format!("✗ Process exited with code {}", code), Color::Red)
            };
            lines.push(Line::from(Span::styled(msg, Style::default().fg(color))));
        }
    }

    let total_lines = lines.len();
    // visible height minus borders
    let visible_height = area.height.saturating_sub(2) as usize;

    let offset = if buffer.auto_scroll {
        total_lines.saturating_sub(visible_height)
    } else {
        let max_offset = total_lines.saturating_sub(visible_height);
        buffer.scroll_offset.min(max_offset)
    };

    let title = entry
        .map(|e| e.command_label.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("Output");

    let block = Block::default().borders(Borders::ALL).title(title);
    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((offset as u16, 0));

    frame.render_widget(paragraph, area);

    // Render auto-scroll paused indicator in top-right corner
    if !buffer.auto_scroll {
        let indicator = " ↑ Auto-scroll paused ";
        let indicator_len = indicator.len() as u16;
        if area.width > indicator_len + 2 && area.height > 0 {
            let indicator_area = Rect {
                x: area.x + area.width - indicator_len - 1,
                y: area.y,
                width: indicator_len,
                height: 1,
            };
            let indicator_widget =
                Paragraph::new(Span::styled(indicator, Style::default().fg(Color::Yellow)));
            frame.render_widget(indicator_widget, indicator_area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Feature: cargo-tui, Property 4: Output buffer max 10 entries
        #[test]
        fn prop_output_buffer_max_10_entries(n in 11usize..=30usize) {
            let mut buffer = OutputBuffer::new();
            for i in 0..n {
                buffer.start_command(format!("command_{}", i));
            }
            prop_assert!(buffer.history.len() <= 10);
        }

        // Feature: cargo-tui, Property 5: Auto-scroll disables on scroll-up
        // Validates: Requirements 9.3, 9.4
        #[test]
        fn prop_auto_scroll_disables_on_scroll_up(amount in 1..=100usize) {
            let mut buffer = OutputBuffer::new();
            buffer.auto_scroll = true;
            buffer.scroll_up(amount);
            prop_assert_eq!(buffer.auto_scroll, false);
        }

        // Feature: cargo-tui, Property 6: Auto-scroll round trip
        // Validates: Requirements 9.3, 9.4
        #[test]
        fn prop_auto_scroll_round_trip(amount in 1..=100usize) {
            let mut buffer = OutputBuffer::new();
            buffer.auto_scroll = true;
            buffer.scroll_up(amount);
            prop_assert_eq!(buffer.auto_scroll, false);
            buffer.scroll_to_bottom();
            prop_assert_eq!(buffer.auto_scroll, true);
        }
    }
}
