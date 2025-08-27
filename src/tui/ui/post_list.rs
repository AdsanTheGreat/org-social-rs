//! Post list UI component (both list and threaded views).

use super::super::modes::ViewMode;
use super::super::navigation::Navigator;
use org_social_lib_rs::{parser, threading};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

/// Draw the post list (either list or threaded view)
pub fn draw_post_list(f: &mut Frame, area: Rect, view_mode: &ViewMode, posts: &[parser::Post], thread_view: &threading::ThreadView, navigator: &Navigator) {
    match view_mode {
        ViewMode::List => {
            draw_list_view(f, area, posts, navigator);
        }
        ViewMode::Threaded => {
            draw_threaded_view(f, area, thread_view, navigator);
        }
    }
}

fn draw_list_view(f: &mut Frame, area: Rect, posts: &[parser::Post], navigator: &Navigator) {
    if posts.is_empty() {
        let no_posts = List::new(vec![ListItem::new("No posts available")])
            .block(Block::default().borders(Borders::ALL).title("Posts (0/0)"))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(no_posts, area);
        return;
    }

    let items: Vec<ListItem> = posts
        .iter()
        .enumerate()
        .map(|(i, post)| {
            let style = if i == navigator.selected_post {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let author = post.author().as_ref().map(|s| s.as_str()).unwrap_or("unknown");
            let time_str = if let Some(time) = post.time() {
                time.format("%m-%d %H:%M").to_string()
            } else {
                "no time".to_string()
            };

            let content_preview = post.content()
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(25)
                .collect::<String>();

            let line = Line::from(vec![
                Span::styled(format!("{author}: "), style.fg(Color::Green)),
                Span::styled(content_preview, style),
                Span::styled(format!(" ({time_str})"), style.fg(Color::Blue)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(navigator.selected_post));

    let posts_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Posts ({}/{})", navigator.selected_post + 1, posts.len()))
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(posts_list, area, &mut list_state);
}

fn draw_threaded_view(f: &mut Frame, area: Rect, thread_view: &threading::ThreadView, navigator: &Navigator) {
    if thread_view.is_empty() {
        let no_posts = List::new(vec![ListItem::new("No posts available")])
            .block(Block::default().borders(Borders::ALL).title("Threads (0/0)"))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(no_posts, area);
        return;
    }

    let mut items = Vec::new();
    let mut global_index = 0;
    let mut selected_global_index = 0;

    // Flatten all threads into a list with proper indentation
    for (thread_idx, thread) in thread_view.roots.iter().enumerate() {
        let thread_posts = thread.flatten();
        
        for (post_idx, post) in thread_posts.iter().enumerate() {
            if thread_idx == navigator.selected_thread && post_idx == navigator.selected_thread_post {
                selected_global_index = global_index;
            }

            let depth = if post_idx == 0 { 0 } else { 
                // Calculate depth based on reply relationships
                thread.replies.iter()
                    .find_map(|reply| find_post_depth(reply, post.id(), 1))
                    .unwrap_or(1)
            };

            let indent = "  ".repeat(depth);
            let style = if thread_idx == navigator.selected_thread && post_idx == navigator.selected_thread_post {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let author = post.author().as_ref().map(|s| s.as_str()).unwrap_or("unknown");
            let time_str = if let Some(time) = post.time() {
                time.format("%m-%d %H:%M").to_string()
            } else {
                "no time".to_string()
            };

            let content_preview = post.content()
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(25 - indent.len()) // Account for indentation
                .collect::<String>();

            let line = Line::from(vec![
                Span::styled(indent.to_string(), style),
                Span::styled(format!("{author}: "), style.fg(Color::Green)),
                Span::styled(content_preview, style),
                Span::styled(format!(" ({time_str})"), style.fg(Color::Blue)),
            ]);

            items.push(ListItem::new(line));
            global_index += 1;
        }
    }

    let mut list_state = ListState::default();
    list_state.select(Some(selected_global_index));

    let posts_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Threads ({}/{} - {} total posts)", 
                    navigator.selected_thread + 1, 
                    thread_view.thread_count(),
                    thread_view.total_posts()))
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_stateful_widget(posts_list, area, &mut list_state);
}

// Helper function to find the depth of a post in the reply tree
fn find_post_depth(node: &threading::ThreadNode, target_id: &str, current_depth: usize) -> Option<usize> {
    if node.post.id() == target_id {
        return Some(current_depth);
    }
    
    for reply in &node.replies {
        if let Some(depth) = find_post_depth(reply, target_id, current_depth + 1) {
            return Some(depth);
        }
    }
    
    None
}
