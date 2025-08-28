//! Post content display UI component.

use crate::tui::activatable::{self, ActivatableCollector, ActivatableManager};
use org_social_lib_rs::parser;
use org_social_lib_rs::tokenizer::{Token, Tokenizer};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Convert a token to a styled span for ratatui, collecting hyperlinks
fn token_to_span(token: Token, collector: &ActivatableCollector, activatable_manager: Option<&ActivatableManager>, line_num: usize, col_offset: &mut usize) -> Span<'static> {
    match token {
        Token::PlainText(text) => {
            let span = Span::raw(text.clone());
            *col_offset += text.len();
            span
        }
        Token::Bold(text) => {
            let span = Span::styled(text.clone(), Style::default().add_modifier(Modifier::BOLD));
            *col_offset += text.len();
            span
        }
        Token::Italic(text) => {
            let span = Span::styled(text.clone(), Style::default().add_modifier(Modifier::ITALIC));
            *col_offset += text.len();
            span
        }
        Token::BoldItalic(text) => {
            let span = Span::styled(
                text.clone(), 
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::ITALIC)
            );
            *col_offset += text.len();
            span
        }
        Token::Link { url, description } => {
            let display_text = description.unwrap_or(url.clone());
            let start_col = *col_offset;
            let end_col = start_col + display_text.len();
            
            // Add link to collector
            activatable::collect_hyperlink(
                collector,
                url.clone(),
                display_text.clone(),
                line_num,
                start_col,
                end_col,
            );
            
            *col_offset += display_text.len();
            
            // Create styled span for the hyperlink with focus checking
            activatable::create_hyperlink_span(display_text, &url, activatable_manager)
        }
        Token::Mention { url, username } => {
            let display_text = if username.starts_with("@") {
                username.clone()
            } else {
                format!("@{username}")
            };
            let start_col = *col_offset;
            let end_col = start_col + display_text.len();

            // Add mention to collector
            activatable::collect_mention(
                collector,
                url.clone(),
                username.clone(),
                line_num,
                start_col,
                end_col,
            );

            *col_offset += display_text.len();

            // Create styled span for the mention with focus checking
            activatable::create_mention_span(display_text, &url, activatable_manager)
        }
        Token::InlineCode(text) => {
            let span = Span::styled(
                text.clone(),
                Style::default()
                    .fg(Color::White)
                    .bg(Color::DarkGray)
            );
            *col_offset += text.len();
            span
        }
        Token::Strikethrough(text) => {
            let span = Span::styled(
                text.clone(),
                Style::default().add_modifier(Modifier::CROSSED_OUT)
            );
            *col_offset += text.len();
            span
        }
        Token::Underline(text) => {
            let span = Span::styled(
                text.clone(),
                Style::default().add_modifier(Modifier::UNDERLINED)
            );
            *col_offset += text.len();
            span
        }

    }
}

/// Parse a line of text using the tokenizer and convert to styled spans
fn parse_line_to_spans(line: &str, collector: &ActivatableCollector, activatable_manager: Option<&ActivatableManager>, line_num: usize) -> Vec<Span<'static>> {
    let mut tokenizer = Tokenizer::new(line.to_string());
    let tokens = tokenizer.tokenize();
    let mut spans = Vec::new();
    let mut col_offset = 0;
    
    for token in tokens {
        spans.push(token_to_span(token, collector, activatable_manager, line_num, &mut col_offset));
    }
    
    spans
}

/// Draw the current post content
pub fn draw_post_content(f: &mut Frame, area: Rect, post: Option<&parser::Post>, scroll_offset: usize, collector: &ActivatableCollector, activatable_manager: Option<&ActivatableManager>) {
    // Clear the collector for new content
    if let Ok(mut elements) = collector.lock() {
        elements.clear();
    }
    
    if let Some(post) = post {
        // Create header with post metadata
        let author = post.author().as_ref().map(|s| s.as_str()).unwrap_or("unknown");
        let time_str = if let Some(time) = post.time() {
            time.format("%Y-%m-%d %H:%M").to_string()
        } else {
            "no time".to_string()
        };

        let mut header_lines = vec![
            Line::from(vec![
                Span::styled("Author: ", Style::default().fg(Color::Gray)),
                Span::styled(author, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Time: ", Style::default().fg(Color::Gray)),
                Span::styled(time_str, Style::default().fg(Color::Blue)),
            ]),
            Line::from(vec![
                Span::styled("ID: ", Style::default().fg(Color::Gray)),
                Span::styled(post.id(), Style::default().fg(Color::Yellow)),
            ]),
        ];

        // Add tags if present
        if let Some(tags) = post.tags() {
            if !tags.is_empty() {
                header_lines.push(Line::from(vec![
                    Span::styled("Tags: ", Style::default().fg(Color::Gray)),
                    Span::styled(tags.join(" "), Style::default().fg(Color::Cyan)),
                ]));
            }
        }

        // Add reply info if present
        if let Some(reply_to) = post.reply_to() {
            header_lines.push(Line::from(vec![
                Span::styled("Reply to: ", Style::default().fg(Color::Gray)),
                Span::styled(reply_to, Style::default().fg(Color::Magenta)),
            ]));
        }

        header_lines.push(Line::from(""));

        // Split content area for header and content
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(header_lines.len() as u16 + 1), Constraint::Min(1)].as_ref())
            .split(area);

        // Render header
        let header = Paragraph::new(header_lines)
            .block(Block::default().borders(Borders::ALL).title("Post Details"))
            .wrap(Wrap { trim: true });

        f.render_widget(header, content_chunks[0]);

        // Process content with blocks and get the content to display
        let display_content = if let Some(manager) = activatable_manager {
            // Get processed content from the manager or process it now
            let content = post.content();
            if let Some(processed) = manager.get_processed_content() {
                processed.to_string()
            } else {
                // This shouldn't happen in normal flow as manager should have processed content
                content.to_string()
            }
        } else {
            post.content().to_string()
        };

        // Render post content with scrolling using tokenized org-mode markup
        let content_lines: Vec<Line> = display_content
            .lines()
            .skip(scroll_offset)
            .enumerate()
            .map(|(line_num, line)| {
                let actual_line_num = line_num + scroll_offset;
                
                // Check if this line contains a collapsed block indicator
                if line.trim_start().starts_with("[+]") {
                    // This is a collapsed block, create a special styled span
                    
                    // Extract block type from the line (format: "[+] Code (rust) [...]")
                    let block_type = if line.contains("Code") {
                        "src".to_string()
                    } else if line.contains("Quote") {
                        "quote".to_string()
                    } else if line.contains("Example") {
                        "example".to_string()
                    } else if line.contains("Verse") {
                        "verse".to_string()
                    } else {
                        "block".to_string()
                    };
                    
                    // Add the collapsed block to the collector so it can be navigated to
                    activatable::collect_block(
                        collector,
                        block_type,
                        true, // is_collapsed
                        actual_line_num,
                        0, // start_col
                        line.len(), // end_col
                        actual_line_num, // original_line (same in this case)
                    );
                    
                    vec![activatable::create_block_span(line.to_string(), actual_line_num, activatable_manager)]
                } else {
                    // Check if this line is part of an expanded block
                    let is_block_begin = line.trim_start().starts_with("#+begin_") || 
                                        line.trim_start().starts_with("#+BEGIN_");
                    let is_block_end = line.trim_start().starts_with("#+end_") || 
                                      line.trim_start().starts_with("#+END_");
                    
                    if is_block_begin {
                        // This is a block begin line - add to collector only once here
                        let begin_prefix = if line.trim_start().starts_with("#+begin_") {
                            "#+begin_"
                        } else {
                            "#+BEGIN_"
                        };
                        let after_begin = line.trim_start().strip_prefix(begin_prefix).unwrap_or("");
                        let block_type = after_begin.split_whitespace().next().unwrap_or("block").to_lowercase();
                        
                        // Add the expanded block to the collector so it can be navigated to
                        // Only add it once on the begin line, not on every line of the block
                        activatable::collect_block(
                            collector,
                            block_type.clone(),
                            false, // is_collapsed = false (expanded)
                            actual_line_num,
                            0, // start_col
                            line.len(), // end_col
                            actual_line_num, // original_line
                        );
                        
                        // Check if this block is focused and apply highlighting
                        let mut spans = parse_line_to_spans(line, collector, activatable_manager, actual_line_num);
                        if let Some(manager) = activatable_manager {
                            if manager.is_block_focused(actual_line_num) {
                                // Apply block focus styling to the entire line
                                spans = vec![activatable::create_block_span(line.to_string(), actual_line_num, Some(manager))];
                            }
                        }
                        spans
                    } else if is_block_end {
                        // This is an end line - highlight if we're in a focused block
                        let mut spans = parse_line_to_spans(line, collector, activatable_manager, actual_line_num);
                        if let Some(manager) = activatable_manager {
                            if manager.is_line_in_focused_block(actual_line_num) {
                                spans = vec![activatable::create_block_span(line.to_string(), actual_line_num, Some(manager))];
                            }
                        }
                        spans
                    } else {
                        // Regular line - check if it's within a focused expanded block
                        let spans = parse_line_to_spans(line, collector, activatable_manager, actual_line_num);
                        
                        // If this line is within a focused expanded block, we still want to preserve
                        // all the normal formatting (links, bold, etc.) but add a subtle background
                        if let Some(manager) = activatable_manager {
                            if manager.is_line_in_focused_block(actual_line_num) {
                                // Apply subtle background to all spans in the line
                                spans.into_iter().map(|mut span| {
                                    span.style = span.style.bg(Color::Rgb(40, 40, 50)); // Very dark blue background
                                    span
                                }).collect()
                            } else {
                                spans
                            }
                        } else {
                            spans
                        }
                    }
                }
            })
            .map(Line::from)
            .collect();

        let content = Paragraph::new(content_lines)
            .block(Block::default().borders(Borders::ALL).title("Content"))
            .wrap(Wrap { trim: true });

        f.render_widget(content, content_chunks[1]);
    } else {
        let no_posts = Paragraph::new("No posts available")
            .block(Block::default().borders(Borders::ALL).title("Content"))
            .style(Style::default().fg(Color::Gray));

        f.render_widget(no_posts, area);
    }
}
