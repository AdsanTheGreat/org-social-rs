//! New post window UI component.

use org_social_lib_rs::new_post;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Render text with cursor for text input fields
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

/// Render single-line text with cursor positioned correctly
fn render_single_line_with_cursor(text: &str, cursor_pos: usize) -> Line {
    if text.is_empty() {
        return Line::from(Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray)));
    }
    
    let mut line_spans = Vec::new();
    
    if cursor_pos > 0 && cursor_pos <= text.len() {
        line_spans.push(Span::raw(&text[..cursor_pos]));
    }
    
    if cursor_pos < text.len() {
        // Highlight the character at cursor position
        line_spans.push(Span::styled(&text[cursor_pos..cursor_pos + 1], Style::default().fg(Color::Black).bg(Color::White)));
        if cursor_pos + 1 < text.len() {
            line_spans.push(Span::raw(&text[cursor_pos + 1..]));
        }
    } else {
        // Cursor at end of text
        line_spans.push(Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray)));
    }
    
    Line::from(line_spans)
}

/// Draw the new post window overlay
pub fn draw_new_post_window(f: &mut Frame, area: Rect, new_post_state: &new_post::NewPostState, cursor_visible: bool) {
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
    let help_text = "Tab/Shift+Tab:switch fields | Enter/Shift+Enter:newline | Ctrl+S:submit | F1:remove last tag | Esc:cancel | n:new post";
    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(Color::Black).fg(Color::Green));
    f.render_widget(help, new_post_chunks[5]);
}

fn draw_content_field(f: &mut Frame, area: Rect, new_post_state: &new_post::NewPostState, cursor_visible: bool) {
    let content_title = if new_post_state.current_field == new_post::NewPostField::Content {
        "Content (ACTIVE)"
    } else {
        "Content"
    };
    let content_style = if new_post_state.current_field == new_post::NewPostField::Content {
        Style::default().bg(Color::Black).fg(Color::Yellow)
    } else {
        Style::default().bg(Color::Black)
    };
    
    let content_lines: Vec<Line> = if new_post_state.content.is_empty() {
        if new_post_state.current_field == new_post::NewPostField::Content && cursor_visible {
            vec![Line::from(vec![
                Span::styled("Type your post content here...", Style::default().fg(Color::Gray)),
                Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray)),
            ])]
        } else {
            vec![Line::from(Span::styled("Type your post content here...", Style::default().fg(Color::Gray)))]
        }
    } else {
        // Handle cursor rendering for content field
        if new_post_state.current_field == new_post::NewPostField::Content && cursor_visible {
            render_text_with_cursor(&new_post_state.content, new_post_state.content_cursor)
        } else {
            new_post_state.content.lines().map(Line::from).collect()
        }
    };
    
    let content = Paragraph::new(content_lines)
        .block(Block::default().borders(Borders::ALL).title(content_title))
        .wrap(Wrap { trim: true })
        .style(content_style);
    f.render_widget(content, area);
}

fn draw_tags_field(f: &mut Frame, area: Rect, new_post_state: &new_post::NewPostState, cursor_visible: bool) {
    let tags_title = if new_post_state.current_field == new_post::NewPostField::Tags {
        "Tags (ACTIVE)"
    } else {
        "Tags"
    };
    let tags_style = if new_post_state.current_field == new_post::NewPostField::Tags {
        Style::default().bg(Color::Black).fg(Color::Yellow)
    } else {
        Style::default().bg(Color::Black)
    };
    
    let mut tags_content_parts = Vec::new();
    
    // Show existing tags
    if !new_post_state.tags.is_empty() {
        for tag in &new_post_state.tags {
            tags_content_parts.push(Span::raw(format!("#{tag} ")));
        }
    }
    
    // Show current input with proper cursor handling
    if !new_post_state.tags_input.is_empty() || new_post_state.current_field == new_post::NewPostField::Tags {
        if new_post_state.current_field == new_post::NewPostField::Tags && cursor_visible {
            let input_line = render_single_line_with_cursor(&new_post_state.tags_input, new_post_state.tags_input_cursor);
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

fn draw_mood_field(f: &mut Frame, area: Rect, new_post_state: &new_post::NewPostState, cursor_visible: bool) {
    let mood_title = if new_post_state.current_field == new_post::NewPostField::Mood {
        "Mood (ACTIVE)"
    } else {
        "Mood"
    };
    let mood_style = if new_post_state.current_field == new_post::NewPostField::Mood {
        Style::default().bg(Color::Black).fg(Color::Yellow)
    } else {
        Style::default().bg(Color::Black)
    };
    
    let mood_line = if new_post_state.mood.is_empty() {
        if new_post_state.current_field == new_post::NewPostField::Mood && cursor_visible {
            Line::from(vec![
                Span::styled("Your mood", Style::default().fg(Color::Gray)),
                Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray)),
            ])
        } else {
            Line::from(Span::styled("Your mood", Style::default().fg(Color::Gray)))
        }
    } else {
        if new_post_state.current_field == new_post::NewPostField::Mood && cursor_visible {
            render_single_line_with_cursor(&new_post_state.mood, new_post_state.mood_cursor)
        } else {
            Line::from(new_post_state.mood.clone())
        }
    };
    
    let mood = Paragraph::new(vec![mood_line])
        .block(Block::default().borders(Borders::ALL).title(mood_title))
        .style(mood_style);
    f.render_widget(mood, area);
}

fn draw_lang_field(f: &mut Frame, area: Rect, new_post_state: &new_post::NewPostState, cursor_visible: bool) {
    let lang_title = if new_post_state.current_field == new_post::NewPostField::Lang {
        "Language (ACTIVE)"
    } else {
        "Language"
    };
    let lang_style = if new_post_state.current_field == new_post::NewPostField::Lang {
        Style::default().bg(Color::Black).fg(Color::Yellow)
    } else {
        Style::default().bg(Color::Black)
    };
    
    let lang_line = if new_post_state.lang.is_empty() {
        if new_post_state.current_field == new_post::NewPostField::Lang && cursor_visible {
            Line::from(vec![
                Span::styled("e.g., en, es", Style::default().fg(Color::Gray)),
                Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray)),
            ])
        } else {
            Line::from(Span::styled("e.g., en, es", Style::default().fg(Color::Gray)))
        }
    } else {
        if new_post_state.current_field == new_post::NewPostField::Lang && cursor_visible {
            render_single_line_with_cursor(&new_post_state.lang, new_post_state.lang_cursor)
        } else {
            Line::from(new_post_state.lang.clone())
        }
    };
    
    let lang = Paragraph::new(vec![lang_line])
        .block(Block::default().borders(Borders::ALL).title(lang_title))
        .style(lang_style);
    f.render_widget(lang, area);
}

fn draw_poll_end_field(f: &mut Frame, area: Rect, new_post_state: &new_post::NewPostState, cursor_visible: bool) {
    let poll_end_title = if new_post_state.current_field == new_post::NewPostField::PollEnd {
        "Poll End (ACTIVE)"
    } else {
        "Poll End"
    };
    let poll_end_style = if new_post_state.current_field == new_post::NewPostField::PollEnd {
        Style::default().bg(Color::Black).fg(Color::Yellow)
    } else {
        Style::default().bg(Color::Black)
    };
    
    let poll_end_line = if new_post_state.poll_end.is_empty() {
        if new_post_state.current_field == new_post::NewPostField::PollEnd && cursor_visible {
            Line::from(vec![
                Span::styled("ISO date", Style::default().fg(Color::Gray)),
                Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray)),
            ])
        } else {
            Line::from(Span::styled("ISO date", Style::default().fg(Color::Gray)))
        }
    } else {
        if new_post_state.current_field == new_post::NewPostField::PollEnd && cursor_visible {
            render_single_line_with_cursor(&new_post_state.poll_end, new_post_state.poll_end_cursor)
        } else {
            Line::from(new_post_state.poll_end.clone())
        }
    };
    
    let poll_end = Paragraph::new(vec![poll_end_line])
        .block(Block::default().borders(Borders::ALL).title(poll_end_title))
        .style(poll_end_style);
    f.render_widget(poll_end, area);
}

fn draw_poll_option_field(f: &mut Frame, area: Rect, new_post_state: &new_post::NewPostState, cursor_visible: bool) {
    let poll_option_title = if new_post_state.current_field == new_post::NewPostField::PollOption {
        "Poll Option (ACTIVE)"
    } else {
        "Poll Option"
    };
    let poll_option_style = if new_post_state.current_field == new_post::NewPostField::PollOption {
        Style::default().bg(Color::Black).fg(Color::Yellow)
    } else {
        Style::default().bg(Color::Black)
    };
    
    let poll_option_line = if new_post_state.poll_option.is_empty() {
        if new_post_state.current_field == new_post::NewPostField::PollOption && cursor_visible {
            Line::from(vec![
                Span::styled("Poll option text", Style::default().fg(Color::Gray)),
                Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray)),
            ])
        } else {
            Line::from(Span::styled("Poll option text", Style::default().fg(Color::Gray)))
        }
    } else {
        if new_post_state.current_field == new_post::NewPostField::PollOption && cursor_visible {
            render_single_line_with_cursor(&new_post_state.poll_option, new_post_state.poll_option_cursor)
        } else {
            Line::from(new_post_state.poll_option.clone())
        }
    };
    
    let poll_option = Paragraph::new(vec![poll_option_line])
        .block(Block::default().borders(Borders::ALL).title(poll_option_title))
        .style(poll_option_style);
    f.render_widget(poll_option, area);
}
