//! New post window UI component.

use super::text_input::{self, ContentFieldConfig, SingleLineFieldConfig};
use crate::editor::{NewPostEditor, NewPostField};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

fn draw_content_field(f: &mut Frame, area: Rect, new_post_state: &NewPostEditor, cursor_visible: bool) {
    let config = ContentFieldConfig {
        text: &new_post_state.post_state.content,
        cursor_pos: new_post_state.content_cursor,
        is_active: new_post_state.current_field == NewPostField::Content,
        cursor_visible,
        placeholder: "Type your post content here...",
        title_active: "Content (ACTIVE)",
        title_inactive: "Content",
    };
    
    text_input::draw_content_field(f, area, config);
}

/// Draw the new post window overlay
pub fn draw_new_post_window(f: &mut Frame, area: Rect, new_post_state: &NewPostEditor, cursor_visible: bool) {
    // Create centered new post window
    let new_post_area = Rect {
        x: area.width / 8,
        y: area.height / 12,
        width: (area.width * 3) / 4,
        height: (area.height * 5) / 6,
    };

    // Split new post window into sections - more compact layout
    let new_post_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(8),     // Content
            Constraint::Length(4),  // Tags and Mood (side by side)
            Constraint::Length(3),  // Language and Poll End (side by side)
            Constraint::Length(3),  // Poll Option
            Constraint::Length(3),  // Help
        ].as_ref())
        .split(new_post_area);

    // Header
    let header_text = vec![
        Line::from("Creating a new post"),
    ];
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("New Post"))
        .style(Style::default().bg(Color::Black));
    f.render_widget(header, new_post_chunks[0]);

    // Content field
    draw_content_field(f, new_post_chunks[1], new_post_state, cursor_visible);

    // Tags and Mood side by side
    let tags_mood_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(new_post_chunks[2]);
    
    draw_tags_field(f, tags_mood_chunks[0], new_post_state, cursor_visible);
    draw_mood_field(f, tags_mood_chunks[1], new_post_state, cursor_visible);

    // Language and Poll End side by side
    let lang_poll_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(new_post_chunks[3]);
    
    draw_lang_field(f, lang_poll_chunks[0], new_post_state, cursor_visible);
    draw_poll_end_field(f, lang_poll_chunks[1], new_post_state, cursor_visible);

    // Poll Option field
    draw_poll_option_field(f, new_post_chunks[4], new_post_state, cursor_visible);

    // Help/Controls
    let help_text = "Tab/Shift+Tab:switch fields | Enter/Shift+Enter:newline | Ctrl+S:submit | F1:remove last tag | F2:reset fields | Esc:cancel";
    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(Color::Black).fg(Color::Green));
    f.render_widget(help, new_post_chunks[5]);
}

fn draw_tags_field(f: &mut Frame, area: Rect, new_post_state: &NewPostEditor, cursor_visible: bool) {
    let tags_title = if new_post_state.current_field == NewPostField::Tags {
        "Tags (ACTIVE)"
    } else {
        "Tags"
    };
    let tags_style = if new_post_state.current_field == NewPostField::Tags {
        Style::default().bg(Color::Black).fg(Color::Yellow)
    } else {
        Style::default().bg(Color::Black)
    };
    
    let mut tags_content_parts = Vec::new();
    
    // Show existing tags
    if !new_post_state.post_state.tags.is_empty() {
        for tag in &new_post_state.post_state.tags {
            tags_content_parts.push(Span::raw(format!("#{tag} ")));
        }
    }
    
    // Show current input with proper cursor handling
    if !new_post_state.tags_input.is_empty() || new_post_state.current_field == NewPostField::Tags {
        if new_post_state.current_field == NewPostField::Tags && cursor_visible {
            let input_line = text_input::render_single_line_with_cursor(&new_post_state.tags_input, new_post_state.tags_input_cursor);
            tags_content_parts.extend(input_line.spans);
        } else {
            tags_content_parts.push(Span::raw(&new_post_state.tags_input));
        }
    }
    
    // If no content, show placeholder
    let tags_line = if tags_content_parts.is_empty() {
        Line::from(Span::styled("Space separated, # optional", Style::default().fg(Color::Gray)))
    } else {
        Line::from(tags_content_parts)
    };
    
    let tags = Paragraph::new(vec![tags_line])
        .block(Block::default().borders(Borders::ALL).title(tags_title))
        .wrap(Wrap { trim: true })
        .style(tags_style);
    f.render_widget(tags, area);
}

fn draw_mood_field(f: &mut Frame, area: Rect, new_post_state: &NewPostEditor, cursor_visible: bool) {
    let config = SingleLineFieldConfig {
        text: &new_post_state.post_state.mood,
        cursor_pos: new_post_state.mood_cursor,
        is_active: new_post_state.current_field == NewPostField::Mood,
        cursor_visible,
        placeholder: "Your mood",
        title_active: "Mood (ACTIVE)",
        title_inactive: "Mood",
    };
    
    text_input::draw_single_line_field(f, area, config);
}

fn draw_lang_field(f: &mut Frame, area: Rect, new_post_state: &NewPostEditor, cursor_visible: bool) {
    let config = SingleLineFieldConfig {
        text: &new_post_state.post_state.lang,
        cursor_pos: new_post_state.lang_cursor,
        is_active: new_post_state.current_field == NewPostField::Lang,
        cursor_visible,
        placeholder: "e.g., en, es",
        title_active: "Language (ACTIVE)",
        title_inactive: "Language",
    };
    
    text_input::draw_single_line_field(f, area, config);
}

fn draw_poll_end_field(f: &mut Frame, area: Rect, new_post_state: &NewPostEditor, cursor_visible: bool) {
    let config = SingleLineFieldConfig {
        text: new_post_state.post_state.poll_end.as_deref().unwrap_or(""),
        cursor_pos: new_post_state.poll_end_cursor,
        is_active: new_post_state.current_field == NewPostField::PollEnd,
        cursor_visible,
        placeholder: "ISO date",
        title_active: "Poll End (ACTIVE)",
        title_inactive: "Poll End",
    };
    
    text_input::draw_single_line_field(f, area, config);
}

fn draw_poll_option_field(f: &mut Frame, area: Rect, new_post_state: &NewPostEditor, cursor_visible: bool) {
    let config = SingleLineFieldConfig {
        text: new_post_state.post_state.poll_option.as_deref().unwrap_or(""),
        cursor_pos: new_post_state.poll_option_cursor,
        is_active: new_post_state.current_field == NewPostField::PollOption,
        cursor_visible,
        placeholder: "Poll option text",
        title_active: "Poll Option (ACTIVE)",
        title_inactive: "Poll Option",
    };
    
    text_input::draw_single_line_field(f, area, config);
}
