//! Core TUI application state and logic.

use super::{
    activatable::{ActivatableCollector, ActivatableManager},
    events::{self, EventResult},
    modes::{AppMode, ViewMode},
    navigation::Navigator,
};
use chrono::{Duration as ChronoDuration, Utc};
use org_social_lib_rs::{feed, new_post, parser, reply, threading};
use std::time::Instant;

/// Application state for the TUI
pub struct TUI {
    /// All posts to display
    pub posts: Vec<parser::Post>,
    /// Threaded view of posts
    pub thread_view: threading::ThreadView,
    /// Current view mode (list or threaded)
    pub view_mode: ViewMode,
    /// Navigation state
    pub navigator: Navigator,
    /// Whether to show help overlay
    pub show_help: bool,
    /// Current mode (browsing, reply, etc.)
    pub mode: AppMode,
    /// Reply state (when replying to a post)
    pub reply_state: Option<reply::ReplyState>,
    /// Reply manager for saving replies
    pub reply_manager: reply::ReplyManager,
    /// New post state (when creating a new post)
    pub new_post_state: Option<new_post::NewPostState>,
    /// New post manager for saving new posts
    pub new_post_manager: new_post::NewPostManager,
    /// Status message to display
    pub status_message: Option<String>,
    /// Cursor blink state (true = visible, false = hidden)
    pub cursor_visible: bool,
    /// Last time cursor blink state changed
    pub last_cursor_blink: Instant,
    /// Activatable elements manager for tracking and interacting with links and blocks
    pub activatable_manager: ActivatableManager,
    /// Activatable elements collector for gathering elements during rendering
    pub activatable_collector: ActivatableCollector,
}

impl TUI {
    pub async fn new(
        file_path: &std::path::Path,
        user_profile: &parser::Profile,
        user_posts: Vec<parser::Post>,
        user_only: bool,
        source_filter: Option<String>,
        days_filter: Option<u32>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let feed = if user_only {
            feed::Feed::create_user_feed(user_profile, user_posts.clone())
        } else {
            match feed::Feed::create_combined_feed(user_profile, user_posts.clone()).await {
                Ok(feed) => feed,
                Err(_) => {
                    // Fallback to user posts only
                    feed::Feed::create_user_feed(user_profile, user_posts.clone())
                }
            }
        };

        let mut posts: Vec<parser::Post> = feed.posts.into_iter().collect();

        // Apply source filter
        if let Some(source) = &source_filter {
            posts.retain(|post| {
                    post.source().as_ref().map(|s| s == source).unwrap_or(false)
                });
        }

        // Apply days filter
        if let Some(days) = days_filter {
            let cutoff = Utc::now() - ChronoDuration::try_days(days as i64).unwrap_or_default();
            posts.retain(|post| {
                    if let Some(post_time) = post.time() {
                        post_time.naive_utc() > cutoff.naive_utc()
                    } else {
                        false
                    }
                });
        }

        // Create threaded view from posts
        let thread_view = threading::ThreadView::from_posts(posts.clone());

        let mut app = TUI {
            posts,
            thread_view,
            view_mode: ViewMode::List,
            navigator: Navigator::new(),
            show_help: false,
            mode: AppMode::Browsing,
            reply_state: None,
            reply_manager: reply::ReplyManager::new(file_path),
            new_post_state: None,
            new_post_manager: new_post::NewPostManager::new(file_path),
            status_message: None,
            cursor_visible: true,
            last_cursor_blink: Instant::now(),
            activatable_manager: ActivatableManager::new(),
            activatable_collector: ActivatableManager::create_collector(),
        };

        // Process the initial post content
        app.process_current_post_content();

        Ok(app)
    }

    pub fn handle_event(&mut self, key_event: crossterm::event::KeyEvent) {
        // Reset cursor to visible when user types
        self.reset_cursor();

        let event_result = events::handle_key_event(key_event, &self.mode);
        
        match event_result {
            EventResult::Quit => {
                // This will be handled by the main event loop
            }
            EventResult::Continue => {}
            EventResult::NextPost => {
                self.navigator.next_post(&self.view_mode, &self.posts, &self.thread_view);
                self.process_current_post_content();
            }
            EventResult::PrevPost => {
                self.navigator.prev_post(&self.view_mode, &self.posts, &self.thread_view);
                self.process_current_post_content();
            }
            EventResult::ScrollDown => {
                self.navigator.scroll_down(&self.posts);
            }
            EventResult::ScrollUp => {
                self.navigator.scroll_up();
            }
            EventResult::GoToFirst => {
                self.navigator.go_to_first(&self.posts);
            }
            EventResult::GoToLast => {
                self.navigator.go_to_last(&self.posts);
            }
            EventResult::ToggleView => {
                self.toggle_view_mode();
            }
            EventResult::StartReply => {
                self.start_reply();
            }
            EventResult::StartNewPost => {
                self.start_new_post();
            }
            EventResult::ToggleHelp => {
                self.toggle_help();
            }
            EventResult::Cancel => {
                self.cancel();
            }
            EventResult::ReplyInput(c) => {
                self.handle_reply_input(c);
            }
            EventResult::ReplyNewline => {
                self.handle_reply_newline();
            }
            EventResult::ReplyBackspace => {
                self.handle_reply_backspace();
            }
            EventResult::NextReplyField => {
                self.next_reply_field();
            }
            EventResult::PrevReplyField => {
                self.prev_reply_field();
            }
            EventResult::FinalizeTags => {
                // Handle Enter key behavior based on current mode and field
                match self.mode {
                    AppMode::Reply => {
                        let enter_result = events::handle_reply_enter(&self.reply_state);
                        match enter_result {
                            EventResult::FinalizeTags => self.finalize_tags_input(),
                            EventResult::ReplyNewline => self.handle_reply_newline(),
                            EventResult::SubmitReply => self.submit_reply(),
                            _ => {}
                        }
                    }
                    AppMode::NewPost => {
                        let enter_result = events::handle_new_post_enter(&self.new_post_state);
                        match enter_result {
                            EventResult::FinalizeTags => self.finalize_new_post_tags_input(),
                            EventResult::NewPostNewline => self.handle_new_post_newline(),
                            EventResult::SubmitNewPost => self.submit_new_post(),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            EventResult::RemoveLastTag => {
                self.remove_last_tag();
            }
            EventResult::SubmitReply => {
                self.submit_reply();
            }
            EventResult::NewPostInput(c) => {
                self.handle_new_post_input(c);
            }
            EventResult::NewPostNewline => {
                self.handle_new_post_newline();
            }
            EventResult::NewPostBackspace => {
                self.handle_new_post_backspace();
            }
            EventResult::NextNewPostField => {
                self.next_new_post_field();
            }
            EventResult::PrevNewPostField => {
                self.prev_new_post_field();
            }
            EventResult::SubmitNewPost => {
                self.submit_new_post();
            }
            EventResult::NextLink => {
                self.next_hyperlink();
            }
            EventResult::PrevLink => {
                self.prev_hyperlink();
            }
            EventResult::ActivateLink => {
                self.activate_hyperlink();
            }
        }
    }

    /// Toggle between list and threaded view
    pub fn toggle_view_mode(&mut self) {
        self.view_mode = self.view_mode.toggle();
        self.navigator.reset_scroll();
        
        // Update status message to show current view
        self.status_message = Some(format!("Switched to {}", self.view_mode.display_name().to_lowercase()));
    }

    /// Start replying to the current post
    pub fn start_reply(&mut self) {
        // Extract the required data from the current post first
        let (post_id, initial_tags) = if let Some(post) = self.current_post() {
            (post.id().to_string(), post.tags().clone())
        } else {
            return;
        };
        
        self.mode = AppMode::Reply;
        self.reply_state = Some(reply::ReplyState::new(post_id.clone(), initial_tags));
        self.status_message = Some(format!("Replying to post {post_id}"));
    }

    /// Cancel current action and return to browsing
    pub fn cancel(&mut self) {
        self.mode = AppMode::Browsing;
        self.reply_state = None;
        self.new_post_state = None;
        self.show_help = false;
        self.status_message = None;
    }

    /// Toggle help display
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        if self.show_help {
            self.mode = AppMode::Help;
        } else {
            self.mode = AppMode::Browsing;
        }
    }

    pub fn handle_reply_input(&mut self, c: char) {
        if let Some(reply_state) = &mut self.reply_state {
            reply_state.handle_input(c);
        }
    }

    pub fn handle_reply_newline(&mut self) {
        if let Some(reply_state) = &mut self.reply_state {
            reply_state.handle_newline();
        }
    }

    pub fn handle_reply_backspace(&mut self) {
        if let Some(reply_state) = &mut self.reply_state {
            reply_state.handle_backspace();
        }
    }

    pub fn next_reply_field(&mut self) {
        if let Some(reply_state) = &mut self.reply_state {
            reply_state.next_field();
        }
    }

    pub fn prev_reply_field(&mut self) {
        if let Some(reply_state) = &mut self.reply_state {
            reply_state.prev_field();
        }
    }

    pub fn finalize_tags_input(&mut self) {
        if let Some(reply_state) = &mut self.reply_state {
            reply_state.finalize_tags_input();
        }
    }

    pub fn finalize_new_post_tags_input(&mut self) {
        if let Some(new_post_state) = &mut self.new_post_state {
            new_post_state.finalize_tags_input();
        }
    }

    pub fn remove_last_tag(&mut self) {
        match self.mode {
            AppMode::Reply => {
                if let Some(reply_state) = &mut self.reply_state {
                    reply_state.remove_last_tag();
                }
            }
            AppMode::NewPost => {
                if let Some(new_post_state) = &mut self.new_post_state {
                    new_post_state.remove_last_tag();
                }
            }
            _ => {}
        }
    }

    /// Submit reply
    pub fn submit_reply(&mut self) {
        if let Some(reply_state) = &self.reply_state {
            if reply_state.is_ready_to_submit() {
                match self.reply_manager.save_reply(reply_state) {
                    Ok(success_message) => {
                        self.status_message = Some(success_message);
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Error saving reply: {e}"));
                    }
                }
                self.cancel();
            }
        }
    }

    /// Start creating a new post
    pub fn start_new_post(&mut self) {
        self.mode = AppMode::NewPost;
        self.new_post_state = Some(new_post::NewPostState::new(None));
        self.status_message = Some("Creating new post".to_string());
    }

    pub fn handle_new_post_input(&mut self, c: char) {
        if let Some(new_post_state) = &mut self.new_post_state {
            new_post_state.handle_input(c);
        }
    }

    pub fn handle_new_post_newline(&mut self) {
        if let Some(new_post_state) = &mut self.new_post_state {
            new_post_state.handle_newline();
        }
    }

    pub fn handle_new_post_backspace(&mut self) {
        if let Some(new_post_state) = &mut self.new_post_state {
            new_post_state.handle_backspace();
        }
    }

    pub fn next_new_post_field(&mut self) {
        if let Some(new_post_state) = &mut self.new_post_state {
            new_post_state.next_field();
        }
    }

    pub fn prev_new_post_field(&mut self) {
        if let Some(new_post_state) = &mut self.new_post_state {
            new_post_state.prev_field();
        }
    }

    /// Submit new post
    pub fn submit_new_post(&mut self) {
        if let Some(new_post_state) = &self.new_post_state {
            if new_post_state.is_ready_to_submit() {
                match self.new_post_manager.save_new_post(new_post_state) {
                    Ok(success_message) => {
                        self.status_message = Some(success_message);
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Error saving new post: {e}"));
                    }
                }
                self.cancel();
            }
        }
    }

    pub fn current_post(&self) -> Option<&parser::Post> {
        match self.view_mode {
            ViewMode::List => self.posts.get(self.navigator.selected_post),
            ViewMode::Threaded => {
                if self.thread_view.is_empty() {
                    return None;
                }
                let current_thread = &self.thread_view.roots[self.navigator.selected_thread];
                let thread_posts = current_thread.flatten();
                thread_posts.get(self.navigator.selected_thread_post).copied()
            }
        }
    }

    pub fn process_current_post_content(&mut self) {
        if let Some(post) = self.current_post().cloned() {
            self.activatable_manager.process_content(post.content());
        }
    }

    pub fn current_post_index(&self) -> Option<usize> {
        match self.view_mode {
            ViewMode::List => {
                if self.navigator.selected_post < self.posts.len() {
                    Some(self.navigator.selected_post)
                } else {
                    None
                }
            },
            ViewMode::Threaded => {
                // For threaded view, we'd need more complex logic
                // For now, just return None to keep it simple
                None
            }
        }
    }

    /// Update cursor blink state if enough time has passed
    pub fn update_cursor_blink(&mut self) {
        let now = Instant::now();
        // Blink every 500ms
        if now.duration_since(self.last_cursor_blink) >= std::time::Duration::from_millis(500) {
            self.cursor_visible = !self.cursor_visible;
            self.last_cursor_blink = now;
        }
    }

    /// Reset cursor to visible (called when user types)
    pub fn reset_cursor(&mut self) {
        self.cursor_visible = true;
        self.last_cursor_blink = Instant::now();
    }

    /// Navigate to the next activatable element in the current view
    pub fn next_hyperlink(&mut self) {
        // Update activatable manager from collector first
        self.activatable_manager.update_from_collector(&self.activatable_collector);
        
        if self.activatable_manager.focus_next() {
            if let Some(element) = self.activatable_manager.focused_element() {
                match &element.element_type {
                    super::activatable::ActivatableType::Hyperlink { url, .. } => {
                        self.status_message = Some(format!("Link: {url}"));
                    }
                    super::activatable::ActivatableType::Block { block_type, is_collapsed } => {
                        let state = if *is_collapsed { "collapsed" } else { "expanded" };
                        self.status_message = Some(format!("Block: {block_type} ({state})"));
                    }
                }
            }
        } else {
            self.status_message = Some("No activatable elements found in current view".to_string());
        }
    }

    /// Navigate to the previous activatable element in the current view
    pub fn prev_hyperlink(&mut self) {
        // Update activatable manager from collector first
        self.activatable_manager.update_from_collector(&self.activatable_collector);
        
        if self.activatable_manager.focus_prev() {
            if let Some(element) = self.activatable_manager.focused_element() {
                match &element.element_type {
                    super::activatable::ActivatableType::Hyperlink { url, .. } => {
                        self.status_message = Some(format!("Link: {url}"));
                    }
                    super::activatable::ActivatableType::Block { block_type, is_collapsed } => {
                        let state = if *is_collapsed { "collapsed" } else { "expanded" };
                        self.status_message = Some(format!("Block: {block_type} ({state})"));
                    }
                }
            }
        } else {
            self.status_message = Some("No activatable elements found in current view".to_string());
        }
    }

    /// Activate the currently focused element
    pub fn activate_hyperlink(&mut self) {
        // Update activatable manager from collector first
        self.activatable_manager.update_from_collector(&self.activatable_collector);
        
        if let Some(result_message) = self.activatable_manager.activate_focused() {
            self.status_message = Some(result_message);
            
            // If we activated a block, refresh the processed content
            if let Some(focused) = self.activatable_manager.focused_element() {
                if matches!(focused.element_type, super::activatable::ActivatableType::Block { .. }) {
                    self.process_current_post_content();
                }
            }
        } else {
            self.status_message = Some("No element currently focused".to_string());
        }
    }
}
