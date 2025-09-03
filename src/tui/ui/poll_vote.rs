//! Poll voting UI component for selecting and voting on poll options.

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Poll voting state to track user selection
#[derive(Debug, Clone)]
pub struct PollVoteState {
    pub poll_context: String, // Context description (e.g., "Poll in: Post summary...")
    pub poll_options: Vec<String>,
    pub selected_option: usize,
    pub poll_post_id: String,
}

impl PollVoteState {
    pub fn new(poll_context: String, poll_options: Vec<String>, poll_post_id: String) -> Self {
        Self {
            poll_context,
            poll_options,
            selected_option: 0,
            poll_post_id,
        }
    }

    pub fn move_up(&mut self) {
        if self.selected_option > 0 {
            self.selected_option -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_option < self.poll_options.len() - 1 {
            self.selected_option += 1;
        }
    }

    pub fn get_selected_option(&self) -> Option<&String> {
        self.poll_options.get(self.selected_option)
    }
}

/// Render the poll voting interface
pub fn render_poll_vote(frame: &mut Frame, area: ratatui::layout::Rect, poll_state: &PollVoteState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Question block
            Constraint::Min(5),    // Options list
            Constraint::Length(3), // Instructions
        ])
        .split(area);

    // Render poll question
    let question_block = Block::default()
        .title("Poll Context")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let question_paragraph = Paragraph::new(poll_state.poll_context.clone())
        .block(question_block)
        .wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(question_paragraph, chunks[0]);

    // Render poll options
    let options_block = Block::default()
        .title("Vote Options (use ↑/↓ to select, Enter to start reply)")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Green));

    let options: Vec<ListItem> = poll_state
        .poll_options
        .iter()
        .enumerate()
        .map(|(i, option)| {
            let style = if i == poll_state.selected_option {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let indicator = if i == poll_state.selected_option { "► " } else { "  " };
            
            ListItem::new(Line::from(vec![
                Span::styled(indicator, style),
                Span::styled(option.clone(), style),
            ]))
        })
        .collect();

    let options_list = List::new(options)
        .block(options_block)
        .highlight_style(Style::default());

    let mut list_state = ListState::default();
    list_state.select(Some(poll_state.selected_option));

    frame.render_stateful_widget(options_list, chunks[1], &mut list_state);

    // Render instructions
    let instructions_block = Block::default()
        .title("Instructions")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    let instructions = vec![
        Line::from("↑/↓ or j/k: Navigate options"),
        Line::from("Enter: Start reply with selected option pre-filled"),
        Line::from("Esc or q: Cancel voting and return to browsing"),
    ];

    let instructions_paragraph = Paragraph::new(instructions)
        .block(instructions_block);

    frame.render_widget(instructions_paragraph, chunks[2]);
}
