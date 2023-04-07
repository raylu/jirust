use crate::event::key::Key;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, List, ListItem, Wrap},
    Frame,
};

use crate::jira::tickets::TicketData;

use super::EventState;

enum InputMode {
    Normal,
    Editing,
}

// CommentPopup holds the state of the application
pub struct CommentsPopup {
    /// Current value of the input box
    input: String,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    pub messages: Vec<String>,
    pub push_comment: bool,
}

impl CommentsPopup {
    pub fn draw<B: Backend>(
        &mut self,
        f: &mut Frame<B>,
        selected_ticket: Option<&TicketData>,
    ) -> anyhow::Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(5),
                    Constraint::Min(1),
                ]
                .as_ref(),
            )
            .split(f.size());
        let title = "Add comments";

        let ticket = match selected_ticket {
            None => return Ok(()),
            Some(ticket_data) => ticket_data,
        };

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    Span::raw("Press "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to exit, "),
                    Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to start editing."),
                ],
                Style::default().add_modifier(Modifier::UNDERLINED),
            ),
            InputMode::Editing => (
                vec![
                    Span::raw("Press "),
                    Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to stop editing, "),
                    Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to record the message"),
                ],
                Style::default(),
            ),
        };
        let mut text = Text::from(Spans::from(msg));
        text.patch_style(style);
        let help_message = Paragraph::new(text);
        f.render_widget(help_message, chunks[0]);

        let input = Paragraph::new(self.input.as_ref())
            .wrap(Wrap { trim: true })
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::default().borders(Borders::ALL).title("Input"));
        f.render_widget(input, chunks[1]);
        match self.input_mode {
            InputMode::Normal =>
                // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
                {}

            InputMode::Editing => {
                // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                f.set_cursor(
                    // Put cursor past the end of the input text
                    chunks[1].x + self.input.len() as u16 + 1,
                    // Move one line down, from the border to the input line
                    chunks[1].y + 1,
                )
            }
        }

        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
                ListItem::new(content)
            })
        .collect();
        let messages =
            List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
        f.render_widget(messages, chunks[2]);
        Ok(())
    }
}

impl CommentsPopup {
    pub fn new() -> Self {
        return Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            push_comment: false
        };
    }

    pub fn edit_mode(&mut self) {
        self.input_mode = InputMode::Editing
    }

    pub fn normal_mode(&mut self) {
        self.input_mode = InputMode::Normal
    }

    pub fn event(&mut self, key: Key) -> anyhow::Result<EventState> {
        match self.input_mode {
            InputMode::Normal => match key {
                Key::Char('q') => {}
                Key::Char('e') => {
                    self.edit_mode();
                    return Ok(EventState::Consumed);
                }
                Key::Char('P') => {
                    self.push_comment = true;
                    return Ok(EventState::Consumed);
                }
                _ => {}
            },
            InputMode::Editing => match key {
                Key::Char(c) => {
                    self.input.push(c);
                    return Ok(EventState::Consumed)
                },
                Key::Backspace => {
                    self.input.pop();
                    return Ok(EventState::Consumed)
                },
                Key::Esc => {
                    self.normal_mode();
                    return Ok(EventState::Consumed)
                },
                Key::Enter => {
                    self.messages.push(self.input.clone());
                    self.input.clear();
                    return Ok(EventState::Consumed)
                }
                _ => {}
            }
        }
        return Ok(EventState::NotConsumed);
    }
}
