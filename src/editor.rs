//! Client-side editor functionality for new posts and replies.
//! The library removed editor functionality in v0.4.0, so we implement it here.

use org_social_lib_rs::{new_post, post::Post};

/// Enum for tracking which field is currently being edited in new post creation
#[derive(Debug, Clone, PartialEq)]
pub enum NewPostField {
    Content,
    Tags,
    Mood,
    Lang,
    PollEnd,
    PollOption,
}

/// Client-side state for editing a new post
#[derive(Clone)]
pub struct NewPostEditor {
    /// The library's post state
    pub post_state: new_post::NewPostState,
    /// Current field being edited
    pub current_field: NewPostField,
    /// Content cursor position
    pub content_cursor: usize,
    /// Tags input field
    pub tags_input: String,
    /// Tags input cursor position
    pub tags_input_cursor: usize,
    /// Mood cursor position
    pub mood_cursor: usize,
    /// Lang cursor position
    pub lang_cursor: usize,
    /// Poll end cursor position
    pub poll_end_cursor: usize,
    /// Poll option cursor position
    pub poll_option_cursor: usize,
}

impl NewPostEditor {
    pub fn new() -> Self {
        Self {
            post_state: new_post::NewPostState::new(),
            current_field: NewPostField::Content,
            content_cursor: 0,
            tags_input: String::new(),
            tags_input_cursor: 0,
            mood_cursor: 0,
            lang_cursor: 0,
            poll_end_cursor: 0,
            poll_option_cursor: 0,
        }
    }

    pub fn handle_input(&mut self, c: char) {
        match self.current_field {
            NewPostField::Content => {
                self.post_state.content.insert(self.content_cursor, c);
                self.content_cursor += 1;
            }
            NewPostField::Tags => {
                self.tags_input.insert(self.tags_input_cursor, c);
                self.tags_input_cursor += 1;
            }
            NewPostField::Mood => {
                self.post_state.mood.insert(self.mood_cursor, c);
                self.mood_cursor += 1;
            }
            NewPostField::Lang => {
                self.post_state.lang.insert(self.lang_cursor, c);
                self.lang_cursor += 1;
            }
            NewPostField::PollEnd => {
                if self.post_state.poll_end.is_none() {
                    self.post_state.poll_end = Some(String::new());
                }
                self.post_state.poll_end.as_mut().unwrap().insert(self.poll_end_cursor, c);
                self.poll_end_cursor += 1;
            }
            NewPostField::PollOption => {
                if self.post_state.poll_option.is_none() {
                    self.post_state.poll_option = Some(String::new());
                }
                self.post_state.poll_option.as_mut().unwrap().insert(self.poll_option_cursor, c);
                self.poll_option_cursor += 1;
            }
        }
    }

    pub fn handle_newline(&mut self) {
        match self.current_field {
            NewPostField::Content => {
                self.post_state.content.insert(self.content_cursor, '\n');
                self.content_cursor += 1;
            }
            _ => {
                // Only allow newlines in content field
            }
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.current_field {
            NewPostField::Content => {
                if self.content_cursor > 0 {
                    self.content_cursor -= 1;
                    self.post_state.content.remove(self.content_cursor);
                }
            }
            NewPostField::Tags => {
                if self.tags_input_cursor > 0 {
                    self.tags_input_cursor -= 1;
                    self.tags_input.remove(self.tags_input_cursor);
                }
            }
            NewPostField::Mood => {
                if self.mood_cursor > 0 {
                    self.mood_cursor -= 1;
                    self.post_state.mood.remove(self.mood_cursor);
                }
            }
            NewPostField::Lang => {
                if self.lang_cursor > 0 {
                    self.lang_cursor -= 1;
                    self.post_state.lang.remove(self.lang_cursor);
                }
            }
            NewPostField::PollEnd => {
                if self.poll_end_cursor > 0 {
                    self.poll_end_cursor -= 1;
                    if let Some(ref mut end) = self.post_state.poll_end {
                        end.remove(self.poll_end_cursor);
                        if end.is_empty() {
                            self.post_state.poll_end = None;
                        }
                    }
                }
            }
            NewPostField::PollOption => {
                if self.poll_option_cursor > 0 {
                    self.poll_option_cursor -= 1;
                    if let Some(ref mut option) = self.post_state.poll_option {
                        option.remove(self.poll_option_cursor);
                        if option.is_empty() {
                            self.post_state.poll_option = None;
                        }
                    }
                }
            }
        }
    }

    pub fn next_field(&mut self) {
        self.current_field = match self.current_field {
            NewPostField::Content => NewPostField::Tags,
            NewPostField::Tags => NewPostField::Mood,
            NewPostField::Mood => NewPostField::Lang,
            NewPostField::Lang => NewPostField::PollEnd,
            NewPostField::PollEnd => NewPostField::PollOption,
            NewPostField::PollOption => NewPostField::Content,
        };
    }

    pub fn prev_field(&mut self) {
        self.current_field = match self.current_field {
            NewPostField::Content => NewPostField::PollOption,
            NewPostField::Tags => NewPostField::Content,
            NewPostField::Mood => NewPostField::Tags,
            NewPostField::Lang => NewPostField::Mood,
            NewPostField::PollEnd => NewPostField::Lang,
            NewPostField::PollOption => NewPostField::PollEnd,
        };
    }

    pub fn finalize_tags_input(&mut self) {
        if !self.tags_input.is_empty() {
            let new_tags: Vec<String> = self.tags_input
                .split_whitespace()
                .map(|tag| tag.trim_start_matches('#').to_string())
                .filter(|tag| !tag.is_empty())
                .collect();
            
            for tag in new_tags {
                if !self.post_state.tags.contains(&tag) {
                    self.post_state.tags.push(tag);
                }
            }
            
            self.tags_input.clear();
            self.tags_input_cursor = 0;
        }
    }

    pub fn remove_last_tag(&mut self) {
        self.post_state.tags.pop();
    }

    pub fn is_ready_to_submit(&self) -> bool {
        !self.post_state.is_empty()
    }

    pub fn create_post(&mut self) -> Post {
        self.finalize_tags_input();
        self.post_state.create_post("org-social-rs")
    }
}

impl Default for NewPostEditor {
    fn default() -> Self {
        Self::new()
    }
}

/// Enum for tracking which field is currently being edited in reply creation
#[derive(Debug, Clone, PartialEq)]
pub enum ReplyField {
    Content,
    Tags,
    Mood,
    PollOption,
}

/// Client-side state for editing a reply
#[derive(Debug, Clone)]
pub struct ReplyEditor {
    /// The library's post state
    pub post_state: new_post::NewPostState,
    /// Current field being edited
    pub current_field: ReplyField,
    /// Content cursor position
    pub content_cursor: usize,
    /// Tags input field
    pub tags_input: String,
    /// Tags input cursor position
    pub tags_input_cursor: usize,
    /// Mood cursor position
    pub mood_cursor: usize,
}

impl ReplyEditor {
    pub fn new(target_post: &Post) -> Self {
        let post_state = new_post::NewPostState::reply_to_post(target_post.clone());
        Self {
            post_state,
            current_field: ReplyField::Content,
            content_cursor: 0,
            tags_input: String::new(),
            tags_input_cursor: 0,
            mood_cursor: 0,
        }
    }

    pub fn handle_input(&mut self, c: char) {
        match self.current_field {
            ReplyField::Content => {
                self.post_state.content.insert(self.content_cursor, c);
                self.content_cursor += 1;
            }
            ReplyField::Tags => {
                self.tags_input.insert(self.tags_input_cursor, c);
                self.tags_input_cursor += 1;
            }
            ReplyField::Mood => {
                self.post_state.mood.insert(self.mood_cursor, c);
                self.mood_cursor += 1;
            }
            ReplyField::PollOption => {
                // Poll option is handled separately
            }
        }
    }

    pub fn handle_newline(&mut self) {
        match self.current_field {
            ReplyField::Content => {
                self.post_state.content.insert(self.content_cursor, '\n');
                self.content_cursor += 1;
            }
            _ => {
                // Only allow newlines in content field
            }
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.current_field {
            ReplyField::Content => {
                if self.content_cursor > 0 {
                    self.content_cursor -= 1;
                    self.post_state.content.remove(self.content_cursor);
                }
            }
            ReplyField::Tags => {
                if self.tags_input_cursor > 0 {
                    self.tags_input_cursor -= 1;
                    self.tags_input.remove(self.tags_input_cursor);
                }
            }
            ReplyField::Mood => {
                if self.mood_cursor > 0 {
                    self.mood_cursor -= 1;
                    self.post_state.mood.remove(self.mood_cursor);
                }
            }
            ReplyField::PollOption => {
                // Poll option is handled separately
            }
        }
    }

    pub fn next_field(&mut self) {
        self.current_field = match self.current_field {
            ReplyField::Content => ReplyField::Tags,
            ReplyField::Tags => ReplyField::Mood,
            ReplyField::Mood => ReplyField::PollOption,
            ReplyField::PollOption => ReplyField::Content,
        };
    }

    pub fn prev_field(&mut self) {
        self.current_field = match self.current_field {
            ReplyField::Content => ReplyField::PollOption,
            ReplyField::Tags => ReplyField::Content,
            ReplyField::Mood => ReplyField::Tags,
            ReplyField::PollOption => ReplyField::Mood,
        };
    }

    pub fn finalize_tags_input(&mut self) {
        if !self.tags_input.is_empty() {
            let new_tags: Vec<String> = self.tags_input
                .split_whitespace()
                .map(|tag| tag.trim_start_matches('#').to_string())
                .filter(|tag| !tag.is_empty())
                .collect();
            
            for tag in new_tags {
                if !self.post_state.tags.contains(&tag) {
                    self.post_state.tags.push(tag);
                }
            }
            
            self.tags_input.clear();
            self.tags_input_cursor = 0;
        }
    }

    pub fn remove_last_tag(&mut self) {
        self.post_state.tags.pop();
    }

    pub fn is_ready_to_submit(&self) -> bool {
        !self.post_state.content.trim().is_empty()
    }

    pub fn set_poll_option(&mut self, option: String) {
        self.post_state.poll_option = Some(option);
    }

    pub fn create_post(&mut self) -> Post {
        self.finalize_tags_input();
        self.post_state.create_post("org-social-rs")
    }
}
