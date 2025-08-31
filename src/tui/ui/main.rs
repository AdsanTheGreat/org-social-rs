//! Main UI layout and coordination.

use super::super::activatable::{ActivatableCollector, ActivatableManager};
use super::super::modes::{AppMode, ViewMode};
use super::super::navigation::Navigator;
use super::{content, help, new_post, post_list, reply, status};
use org_social_lib_rs::{new_post as new_post_module, notifications, parser, reply as reply_module, threading};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

/// Draw the main UI based on current mode
pub fn draw_ui(
    f: &mut Frame,
    mode: &AppMode,
    view_mode: &ViewMode,
    posts: &[parser::Post],
    notification_feed: &notifications::NotificationFeed,
    thread_view: &threading::ThreadView,
    navigator: &Navigator,
    current_post: Option<&parser::Post>,
    reply_state: &Option<reply_module::ReplyState>,
    new_post_state: &Option<new_post_module::NewPostState>,
    status_message: &Option<String>,
    cursor_visible: bool,
    help_scroll: u16,
    collector: &ActivatableCollector,
    activatable_manager: Option<&ActivatableManager>,
) {
    let size = f.area();

    match mode {
        AppMode::Help => {
            help::draw_help(f, size, help_scroll);
        }
        AppMode::Reply => {
            if let Some(reply_state) = reply_state {
                reply::draw_reply_window(f, size, reply_state, cursor_visible);
            }
        }
        AppMode::NewPost => {
            if let Some(new_post_state) = new_post_state {
                new_post::draw_new_post_window(f, size, new_post_state, cursor_visible);
            }
        }
        _ => {
            draw_main_ui(f, size, view_mode, posts, notification_feed, thread_view, navigator, current_post, mode, status_message, collector, activatable_manager);
        }
    }
}

fn draw_main_ui(
    f: &mut Frame,
    area: Rect,
    view_mode: &ViewMode,
    posts: &[parser::Post],
    notification_feed: &notifications::NotificationFeed,
    thread_view: &threading::ThreadView,
    navigator: &Navigator,
    current_post: Option<&parser::Post>,
    mode: &AppMode,
    status_message: &Option<String>,
    collector: &ActivatableCollector,
    activatable_manager: Option<&ActivatableManager>,
) {
    // Split the screen into three areas
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
        .split(area);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(main_chunks[0]);

    // Draw post list (or notification list)
    post_list::draw_post_list(f, content_chunks[0], view_mode, posts, notification_feed, thread_view, navigator);

    // Draw post content
    content::draw_post_content(f, content_chunks[1], current_post, navigator.scroll_offset, collector, activatable_manager);

    // Draw status area
    status::draw_status_area(f, main_chunks[1], mode, view_mode, status_message);
}
