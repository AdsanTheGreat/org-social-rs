//! Reply window UI component.

use org_social_lib_rs::reply;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Draw the reply window overlay
pub fn draw_reply_window(f: &mut Frame, area: Rect, reply_state: &reply::ReplyState, cursor_visible: bool) {
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
        Line::from(format!("Replying to: {}", reply_state.reply_to_id)),
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
    let help_text = "Tab/Shift+Tab:switch fields | Enter/Shift+Enter:newline | Ctrl+S:submit | F1:remove last tag | Esc:cancel";
    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(Color::Black).fg(Color::Green));
    f.render_widget(help, reply_chunks[4]);
}

fn draw_content_field(f: &mut Frame, area: Rect, reply_state: &reply::ReplyState, cursor_visible: bool) {
    let content_title = if reply_state.current_field == reply::ReplyField::Content {
        "Content (ACTIVE)"
    } else {
        "Content"
    };
    let content_style = if reply_state.current_field == reply::ReplyField::Content {
        Style::default().bg(Color::Black).fg(Color::Yellow)
    } else {
        Style::default().bg(Color::Black)
    };
    
    let content_lines: Vec<Line> = if reply_state.content.is_empty() {
        if reply_state.current_field == reply::ReplyField::Content && cursor_visible {
            vec![Line::from(vec![
                Span::styled("Type your reply here...", Style::default().fg(Color::Gray)),
                Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray)),
            ])]
        } else {
            vec![Line::from(Span::styled("Type your reply here...", Style::default().fg(Color::Gray)))]
        }
    } else {
        // Handle cursor rendering for content field
        if reply_state.current_field == reply::ReplyField::Content && cursor_visible {
            render_text_with_cursor(&reply_state.content, reply_state.content_cursor)
        } else {
            reply_state.content.lines().map(Line::from).collect()
        }
    };
    
    let content = Paragraph::new(content_lines)
        .block(Block::default().borders(Borders::ALL).title(content_title))
        .wrap(Wrap { trim: true })
        .style(content_style);
    f.render_widget(content, area);
}

fn draw_tags_field(f: &mut Frame, area: Rect, reply_state: &reply::ReplyState, cursor_visible: bool) {
    let tags_title = if reply_state.current_field == reply::ReplyField::Tags {
        "Tags (ACTIVE) - Space separated, # optional"
    } else {
        "Tags"
    };
    let tags_style = if reply_state.current_field == reply::ReplyField::Tags {
        Style::default().bg(Color::Black).fg(Color::Yellow)
    } else {
        Style::default().bg(Color::Black)
    };

    let mut tags_lines = Vec::new();
    
    // Show existing tags
    if !reply_state.tags.is_empty() {
        let tags_display = reply_state.tags.iter()
            .map(|tag| format!("#{tag}"))
            .collect::<Vec<_>>()
            .join(" ");
        tags_lines.push(Line::from(vec![
            Span::styled("Current: ", Style::default().fg(Color::Gray)),
            Span::styled(tags_display, Style::default().fg(Color::Cyan)),
        ]));
    }
    
    // Show input field
    if reply_state.tags_input.is_empty() && reply_state.current_field == reply::ReplyField::Tags {
        let mut input_spans = vec![Span::styled("Type tags here...", Style::default().fg(Color::DarkGray))];
        if cursor_visible {
            input_spans.push(Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray)));
        }
        tags_lines.push(Line::from(vec![
            Span::styled("Input: ", Style::default().fg(Color::Gray)),
        ].into_iter().chain(input_spans).collect::<Vec<_>>()));
    } else {
        let mut input_spans = Vec::new();
        let cursor_pos = reply_state.tags_input_cursor;
        
        if reply_state.current_field == reply::ReplyField::Tags && cursor_visible {
            // Render with cursor
            if cursor_pos > 0 && cursor_pos <= reply_state.tags_input.len() {
                input_spans.push(Span::raw(&reply_state.tags_input[..cursor_pos]));
            }
            
            if cursor_pos < reply_state.tags_input.len() {
                input_spans.push(Span::styled(
                    &reply_state.tags_input[cursor_pos..cursor_pos + 1],
                    Style::default().fg(Color::Black).bg(Color::White)
                ));
                if cursor_pos + 1 < reply_state.tags_input.len() {
                    input_spans.push(Span::raw(&reply_state.tags_input[cursor_pos + 1..]));
                }
            } else {
                // Cursor at end
                input_spans.push(Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray)));
            }
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

fn draw_mood_field(f: &mut Frame, area: Rect, reply_state: &reply::ReplyState, cursor_visible: bool) {
    let mood_title = if reply_state.current_field == reply::ReplyField::Mood {
        "Mood (ACTIVE)"
    } else {
        "Mood"
    };
    let mood_style = if reply_state.current_field == reply::ReplyField::Mood {
        Style::default().bg(Color::Black).fg(Color::Yellow)
    } else {
        Style::default().bg(Color::Black)
    };
    
    let mood_content = if reply_state.mood.is_empty() && reply_state.current_field == reply::ReplyField::Mood {
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
        if reply_state.current_field == reply::ReplyField::Mood && cursor_visible {
            let cursor_pos = reply_state.mood_cursor;
            let mut mood_spans = Vec::new();
            
            if cursor_pos > 0 && cursor_pos <= reply_state.mood.len() {
                mood_spans.push(Span::raw(&reply_state.mood[..cursor_pos]));
            }
            
            if cursor_pos < reply_state.mood.len() {
                mood_spans.push(Span::styled(
                    &reply_state.mood[cursor_pos..cursor_pos + 1],
                    Style::default().fg(Color::Black).bg(Color::White)
                ));
                if cursor_pos + 1 < reply_state.mood.len() {
                    mood_spans.push(Span::raw(&reply_state.mood[cursor_pos + 1..]));
                }
            } else {
                // Cursor at end
                mood_spans.push(Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray)));
            }
            
            Line::from(mood_spans)
        } else {
            Line::from(reply_state.mood.as_str())
        }
    };
    
    let mood = Paragraph::new(mood_content)
        .block(Block::default().borders(Borders::ALL).title(mood_title))
        .wrap(Wrap { trim: true })
        .style(mood_style);
    f.render_widget(mood, area);
}

fn render_text_with_cursor(text: &str, cursor_pos: usize) -> Vec<Line> {
    let mut char_count = 0;
    let lines: Vec<&str> = text.lines().collect();
    let mut rendered_lines = Vec::new();
    let mut cursor_placed = false;
    
    for line in lines.iter() {
        let line_start = char_count;
        let line_end = char_count + line.len();
        
        if cursor_pos >= line_start && cursor_pos <= line_end && !cursor_placed {
            // Cursor is in this line
            let col_in_line = cursor_pos - line_start;
            let mut line_spans = Vec::new();
            
            if col_in_line > 0 {
                line_spans.push(Span::raw(&line[..col_in_line]));
            }
            
            // Add cursor
            if col_in_line < line.len() {
                line_spans.push(Span::styled(&line[col_in_line..col_in_line + 1], Style::default().fg(Color::Black).bg(Color::White)));
                if col_in_line + 1 < line.len() {
                    line_spans.push(Span::raw(&line[col_in_line + 1..]));
                }
            } else {
                // Cursor at end of line
                line_spans.push(Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray)));
            }
            
            rendered_lines.push(Line::from(line_spans));
            cursor_placed = true;
        } else {
            rendered_lines.push(Line::from(*line));
        }
        
        char_count = line_end + 1; // +1 for newline character
    }
    
    // If cursor is at the very end, add it on a new line
    if !cursor_placed {
        rendered_lines.push(Line::from(Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray))));
    }
    
    rendered_lines
}
