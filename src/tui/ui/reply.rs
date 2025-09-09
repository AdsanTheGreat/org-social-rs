//! Reply window UI component.

use super::text_input::{self, ContentFieldConfig};
use crate::editor::{ReplyEditor, ReplyField};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Draw the reply window overlay
pub fn draw_reply_window(f: &mut Frame, area: Rect, reply_state: &ReplyEditor, cursor_visible: bool) {
    // Create centered reply window
    let reply_area = Rect {
        x: area.width / 8,
        y: area.height / 8,
        width: (area.width * 3) / 4,
        height: (area.height * 3) / 4,
    };

    // Split reply window into sections
    let reply_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(8),     // Content
            Constraint::Length(4),  // Tags
            Constraint::Length(3),  // Mood
            Constraint::Length(3),  // Help
        ].as_ref())
        .split(reply_area);

    // Header - show what we're replying to
    let header_text = vec![
        Line::from(format!(
            "Replying to: {}",
            reply_state.post_state.reply_to.as_deref().unwrap_or("<unknown>")
        )),
    ];
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title("Reply"))
        .style(Style::default().bg(Color::Black));
    f.render_widget(header, reply_chunks[0]);

    // Content field
    draw_content_field(f, reply_chunks[1], reply_state, cursor_visible);

    // Tags field
    draw_tags_field(f, reply_chunks[2], reply_state, cursor_visible);

    // Mood field
    draw_mood_field(f, reply_chunks[3], reply_state, cursor_visible);

    // Help/Controls
    let help_text = "Tab/Shift+Tab:switch fields | Enter/Shift+Enter:newline | Ctrl+S:submit | F1:remove last tag | F2:reset fields | Esc:cancel";
    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(Color::Black).fg(Color::Green));
    f.render_widget(help, reply_chunks[4]);
}

fn draw_content_field(f: &mut Frame, area: Rect, reply_state: &ReplyEditor, cursor_visible: bool) {
    let config = ContentFieldConfig {
        text: &reply_state.post_state.content,
        cursor_pos: reply_state.content_cursor,
        is_active: reply_state.current_field == ReplyField::Content,
        cursor_visible,
        placeholder: "Type your reply here...",
        title_active: "Content (ACTIVE)",
        title_inactive: "Content",
    };
    
    text_input::draw_content_field(f, area, config);
}

fn draw_tags_field(f: &mut Frame, area: Rect, reply_state: &ReplyEditor, cursor_visible: bool) {
    let tags_title = if reply_state.current_field == ReplyField::Tags {
        "Tags (ACTIVE) - Space separated, # optional"
    } else {
        "Tags"
    };
    let tags_style = if reply_state.current_field == ReplyField::Tags {
        Style::default().bg(Color::Black).fg(Color::Yellow)
    } else {
        Style::default().bg(Color::Black)
    };

    let mut tags_lines = Vec::new();
    
    // Show existing tags
    if !reply_state.post_state.tags.is_empty() {
        let tags_display = reply_state.post_state.tags.iter()
            .map(|tag| format!("#{tag}"))
            .collect::<Vec<_>>()
            .join(" ");
        tags_lines.push(Line::from(vec![
            Span::styled("Current: ", Style::default().fg(Color::Gray)),
            Span::styled(tags_display, Style::default().fg(Color::Cyan)),
        ]));
    }
    
    // Show input field
    if reply_state.tags_input.is_empty() && reply_state.current_field == ReplyField::Tags {
        let mut input_spans = vec![Span::styled("Type tags here...", Style::default().fg(Color::DarkGray))];
        if cursor_visible {
            input_spans.push(Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray)));
        }
        tags_lines.push(Line::from(vec![
            Span::styled("Input: ", Style::default().fg(Color::Gray)),
        ].into_iter().chain(input_spans).collect::<Vec<_>>()));
    } else {
        let mut input_spans = Vec::new();
        
        if reply_state.current_field == ReplyField::Tags && cursor_visible {
            let input_line = text_input::render_single_line_with_cursor(&reply_state.tags_input, reply_state.tags_input_cursor);
            input_spans.extend(input_line.spans);
        } else {
            // No cursor
            input_spans.push(Span::raw(&reply_state.tags_input));
        }
        
        tags_lines.push(Line::from(vec![
            Span::styled("Input: ", Style::default().fg(Color::Gray)),
        ].into_iter().chain(input_spans).collect::<Vec<_>>()));
    }

    let tags = Paragraph::new(tags_lines)
        .block(Block::default().borders(Borders::ALL).title(tags_title))
        .wrap(Wrap { trim: true })
        .style(tags_style);
    f.render_widget(tags, area);
}

fn draw_mood_field(f: &mut Frame, area: Rect, reply_state: &ReplyEditor, cursor_visible: bool) {
    let mood_title = if reply_state.current_field == ReplyField::Mood {
        "Mood (ACTIVE)"
    } else {
        "Mood"
    };
    let mood_style = if reply_state.current_field == ReplyField::Mood {
        Style::default().bg(Color::Black).fg(Color::Yellow)
    } else {
        Style::default().bg(Color::Black)
    };

    let mood_content = if reply_state.post_state.mood.is_empty() && reply_state.current_field == ReplyField::Mood {
        if cursor_visible {
            Line::from(vec![
                Span::styled("Enter mood (optional)...", Style::default().fg(Color::DarkGray)),
                Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray)),
            ])
        } else {
            Line::from(Span::styled("Enter mood (optional)...", Style::default().fg(Color::DarkGray)))
        }
    } else {
        // Render mood with cursor if active
        if reply_state.current_field == ReplyField::Mood && cursor_visible {
            text_input::render_single_line_with_cursor(&reply_state.post_state.mood, reply_state.mood_cursor)
        } else {
            Line::from(reply_state.post_state.mood.as_str())
        }
    };
    
    let mood = Paragraph::new(mood_content)
        .block(Block::default().borders(Borders::ALL).title(mood_title))
        .wrap(Wrap { trim: true })
        .style(mood_style);
    f.render_widget(mood, area);
}

