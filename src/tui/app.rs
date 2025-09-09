//! Core TUI application state and logic.

use super::{
    activatable::{ActivatableCollector, ActivatableManager},
    events::{self, EventResult},
    modes::{AppMode, ViewMode},
    navigation::Navigator,
    ui::poll_vote::PollVoteState,
};
use crate::editor::{NewPostEditor, ReplyEditor};
use chrono::{Duration as ChronoDuration, Utc};
use org_social_lib_rs::{feed, notifications, parser, poll, threading};
use std::time::Instant;

/// Application state for the TUI
pub struct TUI {
    /// Path to the org file 
    pub file_path: std::path::PathBuf,
    /// All posts to display
    pub posts: Vec<parser::Post>,
    /// Notification feed
    pub notification_feed: notifications::NotificationFeed,
    /// Threaded view of posts
    pub thread_view: threading::ThreadView,
    /// Current view mode (list or threaded)
    pub view_mode: ViewMode,
    /// Navigation state
    pub navigator: Navigator,
    /// Whether to show help overlay
    pub show_help: bool,
    /// Help scroll position
    pub help_scroll: u16,
    /// Current mode (browsing, reply, etc.)
    pub mode: AppMode,
    /// Reply state (when replying to a post)
    pub reply_state: Option<ReplyEditor>,
    /// New post state (when creating a new post)
    pub new_post_state: Option<NewPostEditor>,
    /// Poll vote state (when voting on a poll)
    pub poll_vote_state: Option<PollVoteState>,
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
    /// Persistent new post state (kept between openings)
    pub persistent_new_post_state: Option<NewPostEditor>,
    /// Persistent reply state (kept between openings for same post)
    pub persistent_reply_state: Option<ReplyEditor>,
    /// ID of the post that the persistent reply state is for
    pub persistent_reply_post_id: Option<String>,
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

        // Create notification feed from all posts for the user
        let all_posts_for_notifications = if user_only {
            // If user_only, we only have user posts, so no notifications
            Vec::new()
        } else {
            feed.posts.clone()
        };
        let notification_feed = notifications::NotificationFeed::create_notification_feed(
            user_profile,
            &user_posts,
            all_posts_for_notifications,
        );

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
            file_path: file_path.to_path_buf(),
            posts,
            notification_feed,
            thread_view,
            view_mode: ViewMode::List,
            navigator: Navigator::new(),
            show_help: false,
            help_scroll: 0,
            mode: AppMode::Browsing,
            reply_state: None,
            new_post_state: None,
            poll_vote_state: None,
            status_message: None,
            cursor_visible: true,
            last_cursor_blink: Instant::now(),
            activatable_manager: ActivatableManager::new(),
            activatable_collector: ActivatableManager::create_collector(),
            persistent_new_post_state: None,
            persistent_reply_state: None,
            persistent_reply_post_id: None,
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
                self.navigator.next_post(&self.view_mode, &self.posts, &self.thread_view, Some(&self.notification_feed));
                self.process_current_post_content();
            }
            EventResult::PrevPost => {
                self.navigator.prev_post(&self.view_mode, &self.posts, &self.thread_view, Some(&self.notification_feed));
                self.process_current_post_content();
            }
            EventResult::ScrollDown => {
                if self.mode == AppMode::Help {
                    self.scroll_help_down();
                } else {
                    self.navigator.scroll_down(&self.posts);
                }
            }
            EventResult::ScrollUp => {
                if self.mode == AppMode::Help {
                    self.scroll_help_up();
                } else {
                    self.navigator.scroll_up();
                }
            }
            EventResult::GoToFirst => {
                if self.mode == AppMode::Help {
                    self.help_scroll = 0;
                } else {
                    self.navigator.go_to_first(&self.posts);
                }
            }
            EventResult::GoToLast => {
                if self.mode == AppMode::Help {
                    self.scroll_help_to_bottom();
                } else {
                    self.navigator.go_to_last(&self.posts);
                }
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
            EventResult::CountPollVotes => {
                self.count_poll_votes();
            }
            EventResult::StartPollVote => {
                self.start_poll_vote();
            }
            EventResult::PollVoteUp => {
                self.poll_vote_up();
            }
            EventResult::PollVoteDown => {
                self.poll_vote_down();
            }
            EventResult::SubmitPollVote => {
                self.submit_poll_vote();
            }
            EventResult::ResetFields => {
                self.reset_fields();
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
        let (post_id, _initial_tags) = if let Some(post) = self.current_post() {
            (post.full_id(), post.tags().clone())
        } else {
            return;
        };
        
        self.mode = AppMode::Reply;
        
        // Check if we have a persistent reply state for the same post
        if let Some(persistent_post_id) = &self.persistent_reply_post_id {
            if *persistent_post_id == post_id && self.persistent_reply_state.is_some() {
                // Reuse the existing state for the same post
                self.reply_state = self.persistent_reply_state.take();
            } else {
                // Different post, reset and create new state
                self.persistent_reply_state = None;
                self.persistent_reply_post_id = Some(post_id.clone());
                self.reply_state = Some(ReplyEditor::new(self.current_post().unwrap()));
            }
        } else {
            // First time replying, create new state
            self.persistent_reply_post_id = Some(post_id.clone());
            self.reply_state = Some(ReplyEditor::new(self.current_post().unwrap()));
        }
        
        self.status_message = Some(format!("Replying to post {post_id}"));
    }

    /// Cancel current action and return to browsing
    pub fn cancel(&mut self) {
        // Save current states to persistent storage before canceling
        if let Some(reply_state) = self.reply_state.take() {
            self.persistent_reply_state = Some(reply_state);
        }
        if let Some(new_post_state) = self.new_post_state.take() {
            self.persistent_new_post_state = Some(new_post_state);
        }
        
        self.mode = AppMode::Browsing;
        self.poll_vote_state = None;
        self.show_help = false;
        self.status_message = None;
    }

    /// Toggle help display
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        if self.show_help {
            self.mode = AppMode::Help;
            self.help_scroll = 0; // Reset scroll when opening help
        } else {
            self.mode = AppMode::Browsing;
        }
    }

    pub fn scroll_help_down(&mut self) {
        self.help_scroll = self.help_scroll.saturating_add(1);
    }

    pub fn scroll_help_up(&mut self) {
        self.help_scroll = self.help_scroll.saturating_sub(1);
    }

    pub fn scroll_help_to_bottom(&mut self) {
        // Set to a large value, it will be clamped during rendering
        self.help_scroll = 1000;
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
        if let Some(reply_state_mut) = &mut self.reply_state {
            let post = reply_state_mut.create_post();
            if let Some(path_str) = self.file_path.to_str() {
                match post.save_post(path_str) {
                    Ok(_) => {
                        self.status_message = Some("Reply saved successfully!".to_string());
                        // Clear persistent state on successful submission
                        self.persistent_reply_state = None;
                        self.persistent_reply_post_id = None;
                        self.cancel();
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Error saving reply: {}", e));
                    }
                }
            } else {
                self.status_message = Some("Failed to save post: invalid file path".to_string());
            }
        }
    }

    /// Start creating a new post
    pub fn start_new_post(&mut self) {
        self.mode = AppMode::NewPost;
        
        // Check if we have a persistent new post state
        if let Some(persistent_state) = self.persistent_new_post_state.take() {
            // Reuse the existing state
            self.new_post_state = Some(persistent_state);
        } else {
            // Create new state
            self.new_post_state = Some(NewPostEditor::new());
        }
        
        self.status_message = Some("Creating new post".to_string());
    }

    /// Reset all fields in the current form
    pub fn reset_fields(&mut self) {
        match self.mode {
            AppMode::Reply => {
                if let Some(post) = self.current_post() {
                    self.reply_state = Some(ReplyEditor::new(post));
                    self.persistent_reply_state = None; // Clear persistent state
                    self.status_message = Some("Reply fields reset".to_string());
                }
            }
            AppMode::NewPost => {
                self.new_post_state = Some(NewPostEditor::new());
                self.persistent_new_post_state = None; // Clear persistent state
                self.status_message = Some("New post fields reset".to_string());
            }
            _ => {}
        }
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
        if let Some(new_post_state) = self.new_post_state.as_mut() {
            let post = new_post_state.create_post();
            if let Some(path_str) = self.file_path.to_str() {
                match post.save_post(path_str) {
                    Ok(_) => {
                        self.status_message = Some("New post saved successfully!".to_string());
                        // Clear persistent state on successful submission
                        self.persistent_new_post_state = None;
                        self.cancel();
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Error saving new post: {}", e));
                    }
                }
            } else {
                self.status_message = Some("Failed to save post: invalid file path".to_string());
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
            ViewMode::Notifications => {
                // Get the post from the notification at the selected index
                self.notification_feed.notifications
                    .get(self.navigator.selected_post)
                    .map(|notification| &notification.post)
            }
        }
    }

    pub fn process_current_post_content(&mut self) {
        if let Some(post) = self.current_post().cloned() {
            self.activatable_manager.process_post(&post);
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
                    super::activatable::ActivatableType::Mention { url, username } => {
                        self.status_message = Some(format!("Mention: {username} ({url})"));
                    }
                    super::activatable::ActivatableType::Block { block_type, is_collapsed } => {
                        let state = if *is_collapsed { "collapsed" } else { "expanded" };
                        self.status_message = Some(format!("Block: {block_type} ({state})"));
                    }
                    super::activatable::ActivatableType::Poll { post_title: _, vote_counts, total_votes, status } => {
                        let poll_status = if let Some(counts) = vote_counts {
                            // Display vote counts if available
                            let options_summary = if counts.len() <= 3 {
                                counts.iter()
                                    .map(|(option, votes)| format!("{}: {}", option, votes))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            } else {
                                format!("{} options", counts.len())
                            };
                            format!("Poll: {} votes ({}), Status: {}", total_votes, options_summary, status)
                        } else {
                            // Fallback to basic poll info
                            format!("Poll: Press 'v' to count votes")
                        };
                        self.status_message = Some(poll_status);
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
                    super::activatable::ActivatableType::Mention { url, username } => {
                        self.status_message = Some(format!("Mention: {username} ({url})"));
                    }
                    super::activatable::ActivatableType::Block { block_type, is_collapsed } => {
                        let state = if *is_collapsed { "collapsed" } else { "expanded" };
                        self.status_message = Some(format!("Block: {block_type} ({state})"));
                    }
                    super::activatable::ActivatableType::Poll { post_title: _, vote_counts, total_votes, status } => {
                        let poll_status = if let Some(counts) = vote_counts {
                            // Display vote counts if available
                            let options_summary = if counts.len() <= 3 {
                                counts.iter()
                                    .map(|(option, votes)| format!("{}: {}", option, votes))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            } else {
                                format!("{} options", counts.len())
                            };
                            format!("Poll: {} votes ({}), Status: {}", total_votes, options_summary, status)
                        } else {
                            // Fallback to basic poll info
                            format!("Poll: Press 'v' to count votes")
                        };
                        self.status_message = Some(poll_status);
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

        if let Some(result_message) = self.activatable_manager.activate_focused(&self.view_mode) {
            // Check if we need to start poll voting
            if result_message == "StartPollVote" {
                self.start_poll_vote();
            } else {
                self.status_message = Some(result_message);
            }
            
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

    /// Count votes for the poll in the current post (only available in threaded view)
    pub fn count_poll_votes(&mut self) {
        // This functionality is only available in threaded view with access to ThreadNode
        if self.view_mode != ViewMode::Threaded {
            self.status_message = Some("Vote counting only available in threaded view (press 't' to switch)".to_string());
            return;
        }

        // Get the current post and thread node
        let (current_post, thread_node) = match self.get_current_thread_node() {
            Some((post, node)) => (post, node),
            None => {
                self.status_message = Some("No post selected".to_string());
                return;
            }
        };

        // Check if the current post has a poll
        if !poll::is_poll_post(current_post) {
            self.status_message = Some("Current post does not contain a poll".to_string());
            return;
        }

        // Get all reply posts from the thread node to count votes
        let reply_posts: Vec<parser::Post> = thread_node.replies
            .iter()
            .flat_map(|reply_node| {
                let mut posts = vec![reply_node.post.clone()];
                posts.extend(self.collect_all_replies_recursive(reply_node));
                posts
            })
            .collect();

        // Count the votes using the org-social-lib-rs poll module
        match poll::count_poll_votes(current_post, &reply_posts) {
            Some(poll_result) => {
                // Update the activatable manager with the poll results
                self.activatable_manager.update_poll_results(&poll_result);
                
                // Display the poll results
                let vote_summary = format!(
                    "Poll Results: {} total votes, Status: {:?}",
                    poll_result.total_votes,
                    poll_result.status
                );
                
                // You could also display individual option counts here
                let mut detailed_results = vec![vote_summary];
                for option in &poll_result.options {
                    detailed_results.push(format!(
                        "  • {}: {} votes",
                        option.text,
                        option.votes
                    ));
                }
                
                self.status_message = Some(detailed_results.join(" | "));
                
                // Force reprocessing of the current post content to update the display
                self.process_current_post_content();
            }
            None => {
                self.status_message = Some("Failed to count poll votes - invalid poll format".to_string());
            }
        }
    }

    /// Get the current thread node and post when in threaded view
    fn get_current_thread_node(&self) -> Option<(&parser::Post, &threading::ThreadNode)> {
        if self.thread_view.is_empty() {
            return None;
        }

        let current_thread = &self.thread_view.roots[self.navigator.selected_thread];
        let thread_posts = current_thread.flatten();
        let current_post = thread_posts.get(self.navigator.selected_thread_post)?;
        
        // For simplicity, we return the root thread node
        // In a more sophisticated implementation, you might want to find the exact node
        Some((current_post, current_thread))
    }

    /// Recursively collect all reply posts from a thread node
    fn collect_all_replies_recursive(&self, node: &threading::ThreadNode) -> Vec<parser::Post> {
        let mut posts = Vec::new();
        for reply in &node.replies {
            posts.push(reply.post.clone());
            posts.extend(self.collect_all_replies_recursive(reply));
        }
        posts
    }

    /// Start poll voting mode for the currently focused poll
    pub fn start_poll_vote(&mut self) {
        // Update activatable manager from collector first
        self.activatable_manager.update_from_collector(&self.activatable_collector);

        if let Some(focused) = self.activatable_manager.focused_element() {
            if let super::activatable::ActivatableType::Poll { post_title, vote_counts, .. } = &focused.element_type {
                // Extract poll options from vote_counts or parse from current post
                let poll_options = if let Some(counts) = vote_counts {
                    counts.iter().map(|(option, _)| option.clone()).collect()
                } else {
                    // If no vote counts, try to parse options from the current post
                    if let Some(current_post) = self.current_post() {
                        if let Some(poll) = poll::parse_poll_from_post(current_post) {
                            poll.options.into_iter().map(|opt| opt.text).collect()
                        } else {
                            vec!["Option 1".to_string(), "Option 2".to_string()] // Fallback
                        }
                    } else {
                        vec!["Option 1".to_string(), "Option 2".to_string()] // Fallback
                    }
                };

                // Get the current post ID for creating the vote reply
                let poll_post_id = if let Some(current_post) = self.current_post() {
                    current_post.full_id()
                } else {
                    "unknown".to_string()
                };

                self.poll_vote_state = Some(PollVoteState::new(
                    format!("Poll in: {}", post_title), // Use the post title as context
                    poll_options,
                    poll_post_id,
                ));
                self.mode = AppMode::PollVote;
                self.status_message = Some("Select a poll option to vote for".to_string());
            }
        } else {
            self.status_message = Some("No poll focused".to_string());
        }
    }

    /// Move selection up in poll vote mode
    pub fn poll_vote_up(&mut self) {
        if let Some(poll_state) = &mut self.poll_vote_state {
            poll_state.move_up();
        }
    }

    /// Move selection down in poll vote mode
    pub fn poll_vote_down(&mut self) {
        if let Some(poll_state) = &mut self.poll_vote_state {
            poll_state.move_down();
        }
    }

    /// Submit the poll vote and create a vote reply
    pub fn submit_poll_vote(&mut self) {
        let selected_option = if let Some(poll_state) = &self.poll_vote_state {
            poll_state.get_selected_option().cloned()
        } else {
            None
        };

        if let Some(selected_option) = selected_option {
            // Create a reply state for the poll vote with the selected option set in poll_option
            let mut vote_reply_state = ReplyEditor::new(self.current_post().unwrap());

            vote_reply_state.set_poll_option(selected_option.clone());

            // Switch to reply mode with the poll option pre-filled
            self.reply_state = Some(vote_reply_state);
            self.mode = AppMode::Reply;
            self.poll_vote_state = None;
            self.status_message = Some(format!(
                "Vote for '{}' set in poll option field. Add content or press Ctrl+S to submit.",
                selected_option
            ));
        } else {
            // No option selected or no poll state, return to browsing mode
            self.mode = AppMode::Browsing;
            self.poll_vote_state = None;
            self.status_message = Some("No option selected".to_string());
        }
    }
}
