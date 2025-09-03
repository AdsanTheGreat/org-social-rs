//! Status bar UI component.

use ratatui::{
    layout::Rect,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::super::modes::{AppMode, ViewMode};

/// Draw the status/input area
pub fn draw_status_area(f: &mut Frame, area: Rect, mode: &AppMode, view_mode: &ViewMode, status_message: &Option<String>) {
    let text = match mode {
        AppMode::Browsing => {
            let view_info = view_mode.display_name();
            
            if let Some(msg) = status_message {
                Text::from(vec![
                    Line::from(msg.clone()),
                    Line::from(format!("{view_info} | q:quit | j/k:nav | d/u:scroll | g/G:top/bottom | t:toggle view | r:reply | n:new post | h:help")),
                ])
            } else {
                Text::from(format!("{view_info} | q:quit | j/k:navigate | d/u:scroll | g/G:top/bottom | t:toggle view | r:reply | n:new post | h:help"))
            }
        }
        AppMode::Reply => {
            Text::from("In reply mode - see reply window")
        }
        AppMode::NewPost => {
            Text::from("In new post mode - see new post window")
        }
        AppMode::Help => {
            Text::from("Showing help - press h or Esc to close")
        }
        AppMode::PollVote => {
            Text::from("Poll voting mode - use j/k to select, Enter to vote, Esc to cancel")
        }
    };

    let status = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .wrap(Wrap { trim: true });

    f.render_widget(status, area);
}
