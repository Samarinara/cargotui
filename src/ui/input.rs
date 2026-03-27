use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub struct InputState {
    pub value: String,
    pub cursor: usize,
    pub spec: InputSpec,
    pub error: Option<String>,
}

pub struct InputSpec {
    pub prompt: &'static str,
    pub required: bool,
    pub placeholder: &'static str,
}

impl InputState {
    pub fn new(spec: InputSpec) -> Self {
        InputState {
            value: String::new(),
            cursor: 0,
            spec,
            error: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        self.error = None;

        match (key.code, key.modifiers) {
            (KeyCode::Backspace, _) => {
                if self.cursor > 0 {
                    let prev_boundary = self.value[..self.cursor]
                        .char_indices()
                        .last()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.value.remove(prev_boundary);
                    self.cursor = prev_boundary;
                }
            }
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                self.cursor = 0;
            }
            (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                self.cursor = self.value.len();
            }
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                self.value.clear();
                self.cursor = 0;
            }
            (KeyCode::Char(c), mods)
                if mods == KeyModifiers::NONE || mods == KeyModifiers::SHIFT =>
            {
                self.value.insert(self.cursor, c);
                self.cursor += c.len_utf8();
            }
            _ => {}
        }

        debug_assert!(self.cursor <= self.value.len());
    }

    pub fn validate(&self) -> Option<String> {
        if self.spec.required && self.value.trim().is_empty() {
            Some("This field is required".to_string())
        } else {
            None
        }
    }
}

pub fn render_input(state: &InputState, frame: &mut Frame, area: Rect) {
    // Split: top line for input field, bottom line for error
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    let input_area = chunks[0];
    let error_area = chunks[1];

    // Build the input line content
    let input_line = if state.value.is_empty() {
        Line::from(vec![
            Span::raw(format!("{}: ", state.spec.prompt)),
            Span::styled(
                state.spec.placeholder,
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            ),
        ])
    } else {
        let before = &state.value[..state.cursor];
        let after = &state.value[state.cursor..];
        Line::from(vec![
            Span::raw(format!("{}: {}", state.spec.prompt, before)),
            Span::styled("|", Style::default().fg(Color::Yellow)),
            Span::raw(after.to_string()),
        ])
    };

    let input_widget =
        Paragraph::new(input_line).block(Block::default().borders(Borders::ALL).title("Input"));

    frame.render_widget(input_widget, input_area);

    // Render error message if present
    if let Some(err) = &state.error {
        let error_widget =
            Paragraph::new(Span::styled(err.as_str(), Style::default().fg(Color::Red)));
        frame.render_widget(error_widget, error_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // Feature: cargo-tui, Property 8: Cursor invariant
        // Validates: Requirements 10.2
        #[test]
        fn prop_cursor_invariant(
            keys in {
                let key_strategy = prop_oneof![
                    Just(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
                    Just(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)),
                    Just(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::CONTROL)),
                    Just(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL)),
                    (0u8..=127u8).prop_map(|c| KeyEvent::new(KeyCode::Char(c as char), KeyModifiers::NONE)),
                ];
                proptest::collection::vec(key_strategy, 1..=20)
            }
        ) {
            let mut state = InputState::new(InputSpec {
                prompt: "test",
                required: false,
                placeholder: "",
            });
            for key in keys {
                state.handle_key(key);
                prop_assert!(state.cursor <= state.value.len());
            }
        }

        // Feature: cargo-tui, Property 7: Empty input rejected
        // Validates: Requirements 10.5
        #[test]
        fn prop_empty_input_rejected(whitespace in r"[ \t\n\r]*") {
            let mut state = InputState::new(InputSpec {
                prompt: "test",
                required: true,
                placeholder: "",
            });
            state.value = whitespace.clone();
            state.cursor = state.value.len();
            let result = state.validate();
            prop_assert!(result.is_some());
        }
    }
}
