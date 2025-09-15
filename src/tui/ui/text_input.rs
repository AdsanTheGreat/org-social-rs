//! Shared text input rendering utilities for forms.

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, layout::Rect,
};

// Helper functions for character position handling in strings

fn char_pos_to_byte_pos(s: &str, char_pos: usize) -> usize {
    s.char_indices().nth(char_pos).map(|(i, _)| i).unwrap_or(s.len())
}

fn substr_by_chars(s: &str, start: usize, end: usize) -> &str {
    let start_byte = char_pos_to_byte_pos(s, start);
    let end_byte = char_pos_to_byte_pos(s, end);
    &s[start_byte..end_byte]
}

fn char_at_pos(s: &str, char_pos: usize) -> Option<char> {
    s.chars().nth(char_pos)
}

/// Render multiline text with cursor for text input fields
pub fn render_text_with_cursor(text: &str, cursor_pos: usize) -> Vec<Line> {
    let mut char_count = 0;
    let lines: Vec<&str> = text.lines().collect();
    let mut rendered_lines = Vec::new();
    let mut cursor_placed = false;
    
    for line in lines.iter() {
        let line_start = char_count;
        let line_char_len = line.chars().count();
        let line_end = char_count + line_char_len;
        
        if cursor_pos >= line_start && cursor_pos <= line_end && !cursor_placed {
            // Cursor is in this line
            let col_in_line = cursor_pos - line_start;
            let mut line_spans = Vec::new();
            
            if col_in_line > 0 {
                line_spans.push(Span::raw(substr_by_chars(line, 0, col_in_line)));
            }
            
            // Add cursor
            if col_in_line < line_char_len {
                if let Some(char_at_cursor) = char_at_pos(line, col_in_line) {
                    line_spans.push(Span::styled(char_at_cursor.to_string(), Style::default().fg(Color::Black).bg(Color::White)));
                }
                if col_in_line + 1 < line_char_len {
                    line_spans.push(Span::raw(substr_by_chars(line, col_in_line + 1, line_char_len)));
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
pub fn render_single_line_with_cursor(text: &str, cursor_pos: usize) -> Line {
    if text.is_empty() {
        return Line::from(Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray)));
    }
    
    let mut line_spans = Vec::new();
    let text_char_len = text.chars().count();
    
    if cursor_pos > 0 && cursor_pos <= text_char_len {
        line_spans.push(Span::raw(substr_by_chars(text, 0, cursor_pos)));
    }
    
    if cursor_pos < text_char_len {
        // Highlight the character at cursor position
        if let Some(char_at_cursor) = char_at_pos(text, cursor_pos) {
            line_spans.push(Span::styled(char_at_cursor.to_string(), Style::default().fg(Color::Black).bg(Color::White)));
        }
        if cursor_pos + 1 < text_char_len {
            line_spans.push(Span::raw(substr_by_chars(text, cursor_pos + 1, text_char_len)));
        }
    } else {
        // Cursor at end of text
        line_spans.push(Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray)));
    }
    
    Line::from(line_spans)
}

/// Configuration for a content text input field
pub struct ContentFieldConfig<'a> {
    pub text: &'a str,
    pub cursor_pos: usize,
    pub is_active: bool,
    pub cursor_visible: bool,
    pub placeholder: &'a str,
    pub title_active: &'a str,
    pub title_inactive: &'a str,
}

/// Draw a multiline content text input field with consistent styling
pub fn draw_content_field(f: &mut Frame, area: Rect, config: ContentFieldConfig) {
    let title = if config.is_active {
        config.title_active
    } else {
        config.title_inactive
    };
    
    let style = if config.is_active {
        Style::default().bg(Color::Black).fg(Color::Yellow)
    } else {
        Style::default().bg(Color::Black)
    };
    
    let content_lines: Vec<Line> = if config.text.is_empty() {
        if config.is_active && config.cursor_visible {
            vec![Line::from(vec![
                Span::styled(config.placeholder, Style::default().fg(Color::Gray)),
                Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray)),
            ])]
        } else {
            vec![Line::from(Span::styled(config.placeholder, Style::default().fg(Color::Gray)))]
        }
    } else {
        // Handle cursor rendering for content field
        if config.is_active && config.cursor_visible {
            render_text_with_cursor(config.text, config.cursor_pos)
        } else {
            config.text.lines().map(Line::from).collect()
        }
    };
    
    let content = Paragraph::new(content_lines)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: true })
        .style(style);
    f.render_widget(content, area);
}

/// Configuration for a single-line text input field
pub struct SingleLineFieldConfig<'a> {
    pub text: &'a str,
    pub cursor_pos: usize,
    pub is_active: bool,
    pub cursor_visible: bool,
    pub placeholder: &'a str,
    pub title_active: &'a str,
    pub title_inactive: &'a str,
}

/// Draw a single-line text input field with consistent styling
pub fn draw_single_line_field(f: &mut Frame, area: Rect, config: SingleLineFieldConfig) {
    let title = if config.is_active {
        config.title_active
    } else {
        config.title_inactive
    };
    
    let style = if config.is_active {
        Style::default().bg(Color::Black).fg(Color::Yellow)
    } else {
        Style::default().bg(Color::Black)
    };
    
    let line = if config.text.is_empty() {
        if config.is_active && config.cursor_visible {
            Line::from(vec![
                Span::styled(config.placeholder, Style::default().fg(Color::Gray)),
                Span::styled("█", Style::default().fg(Color::White).bg(Color::Gray)),
            ])
        } else {
            Line::from(Span::styled(config.placeholder, Style::default().fg(Color::Gray)))
        }
    } else {
        if config.is_active && config.cursor_visible {
            render_single_line_with_cursor(config.text, config.cursor_pos)
        } else {
            Line::from(config.text)
        }
    };
    
    let field = Paragraph::new(vec![line])
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: true })
        .style(style);
    f.render_widget(field, area);
}
