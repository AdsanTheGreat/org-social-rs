//! Post content display UI component.

use crate::tui::activatable::{self, ActivatableCollector, ActivatableManager};
use org_social_lib_rs::parser;
use org_social_lib_rs::tokenizer::Token;
use org_social_lib_rs::blocks::ActivatableElement;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Process tokens from a post and convert them to Lines with proper styling and position tracking
fn process_post_tokens(
    post: &parser::Post,
    collector: &ActivatableCollector,
    activatable_manager: Option<&ActivatableManager>,
    scroll_offset: usize,
) -> Vec<Line<'static>> {
    let mut lines: Vec<Vec<Span<'static>>> = vec![];
    let mut current_line: Vec<Span<'static>> = vec![];
    let mut current_line_num = 0;
    let mut current_col = 0;

    // Process each token from the post
    for token in post.tokens() {
        let token_spans = token_to_spans(
            token.clone(),
            collector,
            activatable_manager,
            current_line_num,
            &mut current_col,
        );

        for span in token_spans {
            // Check if this span contains newlines
            let text = &span.content;
            if text.contains('\n') {
                // Split span by newlines
                let lines_in_text: Vec<&str> = text.split('\n').collect();
                for (i, line_text) in lines_in_text.iter().enumerate() {
                    if i == 0 {
                        // First part goes to current line
                        if !line_text.is_empty() {
                            current_line.push(Span::styled(line_text.to_string(), span.style));
                        }
                    } else {
                        // Complete current line and start a new one
                        lines.push(current_line);
                        current_line = vec![];
                        current_line_num += 1;
                        current_col = 0;
                        
                        if !line_text.is_empty() {
                            current_line.push(Span::styled(line_text.to_string(), span.style));
                            current_col += line_text.len();
                        }
                    }
                }
            } else {
                // No newlines, just add to current line
                current_col += text.len();
                current_line.push(span);
            }
        }
    }

    // Don't forget the last line if it has content
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    // Apply block styling and handle collapsed/expanded blocks
    let blocks = post.blocks();
    let styled_lines = apply_block_styling(lines, blocks, post, activatable_manager, collector);

    // Apply scrolling
    styled_lines
        .into_iter()
        .skip(scroll_offset)
        .map(Line::from)
        .collect()
}

/// Convert a single token to one or more styled spans
fn token_to_spans(
    token: Token,
    collector: &ActivatableCollector,
    activatable_manager: Option<&ActivatableManager>,
    line_num: usize,
    col_offset: &mut usize,
) -> Vec<Span<'static>> {
    match token {
        Token::PlainText(text) => {
            *col_offset += text.len();
            vec![Span::raw(text)]
        }
        Token::Bold(text) => {
            *col_offset += text.len();
            vec![Span::styled(text, Style::default().add_modifier(Modifier::BOLD))]
        }
        Token::Italic(text) => {
            *col_offset += text.len();
            vec![Span::styled(text, Style::default().add_modifier(Modifier::ITALIC))]
        }
        Token::BoldItalic(text) => {
            *col_offset += text.len();
            vec![Span::styled(
                text,
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::ITALIC),
            )]
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
            vec![activatable::create_hyperlink_span(
                display_text,
                &url,
                activatable_manager,
            )]
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
            vec![activatable::create_mention_span(
                display_text,
                &url,
                activatable_manager,
            )]
        }
        Token::InlineCode(text) => {
            *col_offset += text.len();
            vec![Span::styled(
                text,
                Style::default().fg(Color::White).bg(Color::DarkGray),
            )]
        }
        Token::Strikethrough(text) => {
            *col_offset += text.len();
            vec![Span::styled(
                text,
                Style::default().add_modifier(Modifier::CROSSED_OUT),
            )]
        }
        Token::Underline(text) => {
            *col_offset += text.len();
            vec![Span::styled(
                text,
                Style::default().add_modifier(Modifier::UNDERLINED),
            )]
        }
    }
}

/// Apply block styling to lines based on post blocks
fn apply_block_styling(
    lines: Vec<Vec<Span<'static>>>,
    blocks: &[ActivatableElement],
    post: &parser::Post,
    activatable_manager: Option<&ActivatableManager>,
    collector: &ActivatableCollector,
) -> Vec<Vec<Span<'static>>> {
    let mut styled_lines = lines;

    for block in blocks {
        match block {
            ActivatableElement::Block(org_block) => {
                let start_line = block.start_line();
                let end_line = block.end_line();
                let is_collapsed = block.is_collapsed();

                if is_collapsed {
                    // Replace the block lines with a single collapsed line
                    let summary = block.get_summary();
                    
                    // Add collapsed block to collector
                    activatable::collect_block(
                        collector,
                        org_block.block_type.clone(),
                        true,
                        start_line,
                        0,
                        summary.len(),
                        start_line,
                    );

                    // Replace block content with collapsed representation
                    if start_line < styled_lines.len() {
                        let collapsed_span = activatable::create_block_span(
                            summary,
                            start_line,
                            activatable_manager,
                        );
                        
                        styled_lines[start_line] = vec![collapsed_span];
                        
                        // Remove the lines that are collapsed (from start+1 to end)
                        let lines_to_remove = (start_line + 1).min(styled_lines.len())
                            ..=end_line.min(styled_lines.len().saturating_sub(1));
                        for _ in lines_to_remove.clone() {
                            if start_line + 1 < styled_lines.len() {
                                styled_lines.remove(start_line + 1);
                            }
                        }
                    }
                } else {
                    // Block is expanded - add styling to indicate it's a block
                    // Add expanded block to collector (only once, on the start line)
                    activatable::collect_block(
                        collector,
                        org_block.block_type.clone(),
                        false,
                        start_line,
                        0,
                        if start_line < styled_lines.len() {
                            styled_lines[start_line].iter().map(|s| s.content.len()).sum()
                        } else {
                            0
                        },
                        start_line,
                    );

                    // Apply block focus styling if focused
                    if let Some(manager) = activatable_manager {
                        if manager.is_block_focused(start_line) {
                            // Apply subtle background to all lines in the block
                            for line_idx in start_line..=end_line.min(styled_lines.len().saturating_sub(1)) {
                                if line_idx < styled_lines.len() {
                                    for span in &mut styled_lines[line_idx] {
                                        span.style = span.style.bg(Color::Rgb(40, 40, 50));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            ActivatableElement::Poll(_poll) => {
                let start_line = block.start_line();
                let end_line = block.end_line();

                // Check if we have existing poll data in the manager
                let (vote_counts, total_votes, status) = if let Some(manager) = activatable_manager {
                    manager.get_poll_data_for_line(start_line)
                        .unwrap_or((None, 0, "Unknown".to_string()))
                } else {
                    (None, 0, "Unknown".to_string())
                };

                // Polls are not collapsible, just add them as activatable elements
                // Add poll to collector for vote counting activation
                if let Ok(mut elements) = collector.lock() {
                    // Get a short title from the post content (first 30 characters)
                    let post_title = {
                        let content = post.content();
                        if content.len() > 30 {
                            format!("{}...", &content[..30])
                        } else {
                            content.to_string()
                        }
                    };
                    
                    elements.push((
                        super::super::activatable::ActivatableType::Poll { 
                            post_title,
                            vote_counts,
                            total_votes,
                            status,
                        },
                        start_line,
                        0,
                        if start_line < styled_lines.len() {
                            styled_lines[start_line].iter().map(|s| s.content.len()).sum()
                        } else {
                            0
                        },
                        start_line,
                    ));
                }

                // Apply poll focus styling and add vote count information if available
                if let Some(manager) = activatable_manager {
                    if manager.is_poll_focused(start_line) {
                        // Apply subtle background to all lines in the poll
                        for line_idx in start_line..=end_line.min(styled_lines.len().saturating_sub(1)) {
                            if line_idx < styled_lines.len() {
                                for span in &mut styled_lines[line_idx] {
                                    span.style = span.style.bg(Color::Rgb(50, 40, 60)); // Purple-ish for polls
                                }
                            }
                        }
                    }
                    
                    // If poll results are available, append them to the display
                    if let Some(poll_info) = manager.get_poll_display_info(start_line) {
                        // Add the poll results as additional lines after the poll content
                        let poll_lines: Vec<&str> = poll_info.lines().collect();
                        for (i, info_line) in poll_lines.iter().enumerate().skip(1) { // Skip first line (poll question)
                            let result_line_idx = end_line + i;
                            if result_line_idx < styled_lines.len() {
                                // Insert vote count information
                                styled_lines[result_line_idx].push(Span::styled(
                                    format!(" [{}]", info_line),
                                    Style::default().fg(Color::Cyan).add_modifier(Modifier::ITALIC)
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    styled_lines
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

        // Add poll option if present
        if let Some(poll_option) = post.poll_option() {
            header_lines.push(Line::from(vec![
                Span::styled("Poll option: ", Style::default().fg(Color::Gray)),
                Span::styled(poll_option, Style::default().fg(Color::Yellow)),
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

        // Process post content using the new token-based approach
        let content_lines = process_post_tokens(post, collector, activatable_manager, scroll_offset);

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
