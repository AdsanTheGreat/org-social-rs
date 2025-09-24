//! Main UI layout and coordination.

use super::super::activatable::{ActivatableCollector, ActivatableManager};
use super::super::modes::{AppMode, ViewMode};
use super::super::navigation::Navigator;
use super::super::status_bar_widget::StatusBarState;
use super::{content, help, interactive_status, new_post, poll_vote, post_list, reply};
use crate::editor::{NewPostEditor, ReplyEditor};
use org_social_lib_rs::{feed, notifications, parser, threading};
use std::rc::Rc;
use std::cell::RefCell;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

/// Draw the main UI based on current mode
pub fn draw_ui(
    f: &mut Frame,
    mode: &AppMode,
    view_mode: &ViewMode,
    simple_feed: &feed::SimpleFeed,
    notification_feed: &notifications::NotificationFeed,
    thread_view: &threading::ThreadView,
    navigator: &Navigator,
    current_post: Option<Rc<RefCell<parser::Post>>>,
    reply_state: &Option<ReplyEditor>,
    new_post_state: &Option<NewPostEditor>,
    poll_vote_state: &Option<poll_vote::PollVoteState>,
    cursor_visible: bool,
    help_scroll: u16,
    collector: &ActivatableCollector,
    activatable_manager: Option<&ActivatableManager>,
    status_bar_state: &StatusBarState,
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
        AppMode::PollVote => {
            if let Some(poll_vote_state) = poll_vote_state {
                poll_vote::render_poll_vote(f, size, poll_vote_state);
            }
        }
        _ => {
            draw_main_ui(f, size, view_mode, simple_feed, notification_feed, thread_view, navigator, current_post, mode, collector, activatable_manager, status_bar_state);
        }
    }
}

fn draw_main_ui(
    f: &mut Frame,
    area: Rect,
    view_mode: &ViewMode,
    simple_feed: &feed::SimpleFeed,
    notification_feed: &notifications::NotificationFeed,
    thread_view: &threading::ThreadView,
    navigator: &Navigator,
    current_post: Option<Rc<RefCell<parser::Post>>>,
    mode: &AppMode,
    collector: &ActivatableCollector,
    activatable_manager: Option<&ActivatableManager>,
    status_bar_state: &StatusBarState,
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
    post_list::draw_post_list(f, content_chunks[0], view_mode, simple_feed, notification_feed, thread_view, navigator);

    // Draw post content
    let current_post_ref = current_post.as_ref().map(|p| p.borrow());
    let current_post_borrowed = current_post_ref.as_ref().map(|p| &**p);
    content::draw_post_content(f, content_chunks[1], current_post_borrowed, navigator.scroll_offset, collector, activatable_manager);

    // Draw status area
    interactive_status::draw_status_area(f, main_chunks[1], mode, view_mode, status_bar_state);
}
