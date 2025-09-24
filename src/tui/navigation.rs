//! Navigation logic for posts and threads.

use super::modes::ViewMode;
use org_social_lib_rs::{notifications, threading};
use org_social_lib_rs::feed::SimpleFeed;


pub struct Navigator {
    pub selected_post: usize,
    pub selected_thread: usize,
    pub selected_thread_post: usize,
    pub scroll_offset: usize,
}

impl Navigator {
    pub fn new() -> Self {
        Self {
            selected_post: 0,
            selected_thread: 0,
            selected_thread_post: 0,
            scroll_offset: 0,
        }
    }

    pub fn next_post(&mut self, view_mode: &ViewMode, simple_feed: &SimpleFeed, thread_view: &threading::ThreadView, notification_feed: Option<&notifications::NotificationFeed>) {
        match view_mode {
            ViewMode::List => {
                    let posts = &simple_feed.posts;
                if !posts.is_empty() && self.selected_post < posts.len().saturating_sub(1) {
                    self.selected_post += 1;
                    self.scroll_offset = 0;
                }
            }
            ViewMode::Threaded => {
                // Use ThreadView
                // thread_view: Rc<RefCell<ThreadView>>
                let thread_view = thread_view;
                self.next_threaded_post(thread_view);
            }
            ViewMode::Notifications => {
                if let Some(notification_feed) = notification_feed {
                    if !notification_feed.notifications.is_empty() && 
                       self.selected_post < notification_feed.notifications.len().saturating_sub(1) {
                        self.selected_post += 1;
                        self.scroll_offset = 0;
                    }
                }
            }
        }
    }

    pub fn prev_post(&mut self, view_mode: &ViewMode, simple_feed: &SimpleFeed, thread_view: &threading::ThreadView, notification_feed: Option<&notifications::NotificationFeed>) {
        match view_mode {
            ViewMode::List => {
                    let posts = &simple_feed.posts;
                if !posts.is_empty() && self.selected_post > 0 {
                    self.selected_post -= 1;
                    self.scroll_offset = 0;
                }
            }
            ViewMode::Threaded => {
                let thread_view = thread_view;
                self.prev_threaded_post(thread_view);
            }
            ViewMode::Notifications => {
                if let Some(notification_feed) = notification_feed {
                    if !notification_feed.notifications.is_empty() && self.selected_post > 0 {
                        self.selected_post -= 1;
                        self.scroll_offset = 0;
                    }
                }
            }
        }
    }

    fn next_threaded_post(&mut self, thread_view: &threading::ThreadView) {
        // Refactor for Rc<RefCell<ThreadView>>
        let view = thread_view;
        if view.is_empty() {
            return;
        }
        let current_thread = &view.roots[self.selected_thread];
        let thread_posts = current_thread.flatten();
        if self.selected_thread_post < thread_posts.len().saturating_sub(1) {
            self.selected_thread_post += 1;
        } else if self.selected_thread < view.thread_count().saturating_sub(1) {
            self.selected_thread += 1;
            self.selected_thread_post = 0;
        }
        self.scroll_offset = 0;
    }

    fn prev_threaded_post(&mut self, thread_view: &threading::ThreadView) {
        let view = thread_view;
        if view.is_empty() {
            return;
        }
        if self.selected_thread_post > 0 {
            self.selected_thread_post -= 1;
        } else if self.selected_thread > 0 {
            self.selected_thread -= 1;
            let current_thread = &view.roots[self.selected_thread];
            let thread_posts = current_thread.flatten();
            self.selected_thread_post = thread_posts.len().saturating_sub(1);
        }
        self.scroll_offset = 0;
    }

    pub fn go_to_first(&mut self, simple_feed: &SimpleFeed) {
        let posts = &simple_feed.posts;
        if !posts.is_empty() {
            self.selected_post = 0;
            self.scroll_offset = 0;
        }
    }

    pub fn go_to_last(&mut self, simple_feed: &SimpleFeed) {
        let posts = &simple_feed.posts;
        if !posts.is_empty() {
            self.selected_post = posts.len() - 1;
            self.scroll_offset = 0;
        }
    }

    pub fn scroll_down(&mut self, simple_feed: &SimpleFeed) {
        let posts = &simple_feed.posts;
        if let Some(post) = posts.get(self.selected_post) {
            let content_lines = post.borrow().content().lines().count();
            if self.scroll_offset < content_lines.saturating_sub(1) {
                self.scroll_offset += 1;
            }
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn reset_scroll(&mut self) {
        self.scroll_offset = 0;
    }
}
