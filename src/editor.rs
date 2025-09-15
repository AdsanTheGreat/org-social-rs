//! Client-side editor functionality for new posts and replies.
//! The library removed editor functionality in v0.4.0, so we implement it here.
//! TODO: allow for using an external editor, to fully skip this mess.

use org_social_lib_rs::{new_post, post::Post};

// Helper functions for character position handling in strings

fn char_pos_to_byte_pos(s: &str, char_pos: usize) -> usize {
    s.char_indices().nth(char_pos).map(|(i, _)| i).unwrap_or(s.len())
}

fn char_count(s: &str) -> usize {
    s.chars().count()
}

fn insert_char_at_pos(s: &mut String, char_pos: usize, c: char) {
    let byte_pos = char_pos_to_byte_pos(s, char_pos);
    s.insert(byte_pos, c);
}

fn remove_char_at_pos(s: &mut String, char_pos: usize) {
    if char_pos > 0 {
        let byte_pos = char_pos_to_byte_pos(s, char_pos - 1);
        if let Some((_, c)) = s[byte_pos..].char_indices().next() {
            let char_len = c.len_utf8();
            s.drain(byte_pos..byte_pos + char_len);
        }
    }
}

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
                insert_char_at_pos(&mut self.post_state.content, self.content_cursor, c);
                self.content_cursor += 1;
            }
            NewPostField::Tags => {
                insert_char_at_pos(&mut self.tags_input, self.tags_input_cursor, c);
                self.tags_input_cursor += 1;
            }
            NewPostField::Mood => {
                insert_char_at_pos(&mut self.post_state.mood, self.mood_cursor, c);
                self.mood_cursor += 1;
            }
            NewPostField::Lang => {
                insert_char_at_pos(&mut self.post_state.lang, self.lang_cursor, c);
                self.lang_cursor += 1;
            }
            NewPostField::PollEnd => {
                if self.post_state.poll_end.is_none() {
                    self.post_state.poll_end = Some(String::new());
                }
                insert_char_at_pos(self.post_state.poll_end.as_mut().unwrap(), self.poll_end_cursor, c);
                self.poll_end_cursor += 1;
            }
            NewPostField::PollOption => {
                if self.post_state.poll_option.is_none() {
                    self.post_state.poll_option = Some(String::new());
                }
                insert_char_at_pos(self.post_state.poll_option.as_mut().unwrap(), self.poll_option_cursor, c);
                self.poll_option_cursor += 1;
            }
        }
    }

    pub fn handle_newline(&mut self) {
        match self.current_field {
            NewPostField::Content => {
                insert_char_at_pos(&mut self.post_state.content, self.content_cursor, '\n');
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
                remove_char_at_pos(&mut self.post_state.content, self.content_cursor);
                if self.content_cursor > 0 {
                    self.content_cursor -= 1;
                }
            }
            NewPostField::Tags => {
                remove_char_at_pos(&mut self.tags_input, self.tags_input_cursor);
                if self.tags_input_cursor > 0 {
                    self.tags_input_cursor -= 1;
                }
            }
            NewPostField::Mood => {
                remove_char_at_pos(&mut self.post_state.mood, self.mood_cursor);
                if self.mood_cursor > 0 {
                    self.mood_cursor -= 1;
                }
            }
            NewPostField::Lang => {
                remove_char_at_pos(&mut self.post_state.lang, self.lang_cursor);
                if self.lang_cursor > 0 {
                    self.lang_cursor -= 1;
                }
            }
            NewPostField::PollEnd => {
                if let Some(ref mut end) = self.post_state.poll_end {
                    remove_char_at_pos(end, self.poll_end_cursor);
                    if self.poll_end_cursor > 0 {
                        self.poll_end_cursor -= 1;
                    }
                    if end.is_empty() {
                        self.post_state.poll_end = None;
                    }
                }
            }
            NewPostField::PollOption => {
                if let Some(ref mut option) = self.post_state.poll_option {
                    remove_char_at_pos(option, self.poll_option_cursor);
                    if self.poll_option_cursor > 0 {
                        self.poll_option_cursor -= 1;
                    }
                    if option.is_empty() {
                        self.post_state.poll_option = None;
                    }
                }
            }
        }
    }

    pub fn handle_delete(&mut self) {
        match self.current_field {
            NewPostField::Content => {
                if self.content_cursor < self.post_state.content.len() {
                    self.post_state.content.remove(self.content_cursor);
                }
            }
            NewPostField::Tags => {
                if self.tags_input_cursor < self.tags_input.len() {
                    self.tags_input.remove(self.tags_input_cursor);
                }
            }
            NewPostField::Mood => {
                if self.mood_cursor < self.post_state.mood.len() {
                    self.post_state.mood.remove(self.mood_cursor);
                }
            }
            NewPostField::Lang => {
                if self.lang_cursor < self.post_state.lang.len() {
                    self.post_state.lang.remove(self.lang_cursor);
                }
            }
            NewPostField::PollEnd => {
                if let Some(ref mut end) = self.post_state.poll_end {
                    if self.poll_end_cursor < end.len() {
                        end.remove(self.poll_end_cursor);
                        if end.is_empty() {
                            self.post_state.poll_end = None;
                        }
                    }
                }
            }
            NewPostField::PollOption => {
                if let Some(ref mut option) = self.post_state.poll_option {
                    if self.poll_option_cursor < option.len() {
                        option.remove(self.poll_option_cursor);
                        if option.is_empty() {
                            self.post_state.poll_option = None;
                        }
                    }
                }
            }
        }
    }

    pub fn move_cursor_left(&mut self) {
        match self.current_field {
            NewPostField::Content => {
                if self.content_cursor > 0 {
                    self.content_cursor -= 1;
                }
            }
            NewPostField::Tags => {
                if self.tags_input_cursor > 0 {
                    self.tags_input_cursor -= 1;
                }
            }
            NewPostField::Mood => {
                if self.mood_cursor > 0 {
                    self.mood_cursor -= 1;
                }
            }
            NewPostField::Lang => {
                if self.lang_cursor > 0 {
                    self.lang_cursor -= 1;
                }
            }
            NewPostField::PollEnd => {
                if self.poll_end_cursor > 0 {
                    self.poll_end_cursor -= 1;
                }
            }
            NewPostField::PollOption => {
                if self.poll_option_cursor > 0 {
                    self.poll_option_cursor -= 1;
                }
            }
        }
    }

    pub fn move_cursor_right(&mut self) {
        match self.current_field {
            NewPostField::Content => {
                if self.content_cursor < char_count(&self.post_state.content) {
                    self.content_cursor += 1;
                }
            }
            NewPostField::Tags => {
                if self.tags_input_cursor < char_count(&self.tags_input) {
                    self.tags_input_cursor += 1;
                }
            }
            NewPostField::Mood => {
                if self.mood_cursor < char_count(&self.post_state.mood) {
                    self.mood_cursor += 1;
                }
            }
            NewPostField::Lang => {
                if self.lang_cursor < char_count(&self.post_state.lang) {
                    self.lang_cursor += 1;
                }
            }
            NewPostField::PollEnd => {
                if let Some(ref end) = self.post_state.poll_end {
                    if self.poll_end_cursor < char_count(end) {
                        self.poll_end_cursor += 1;
                    }
                }
            }
            NewPostField::PollOption => {
                if let Some(ref option) = self.post_state.poll_option {
                    if self.poll_option_cursor < char_count(option) {
                        self.poll_option_cursor += 1;
                    }
                }
            }
        }
    }

    pub fn move_cursor_to_start(&mut self) {
        match self.current_field {
            NewPostField::Content => self.content_cursor = 0,
            NewPostField::Tags => self.tags_input_cursor = 0,
            NewPostField::Mood => self.mood_cursor = 0,
            NewPostField::Lang => self.lang_cursor = 0,
            NewPostField::PollEnd => self.poll_end_cursor = 0,
            NewPostField::PollOption => self.poll_option_cursor = 0,
        }
    }

    pub fn move_cursor_to_end(&mut self) {
        match self.current_field {
            NewPostField::Content => self.content_cursor = char_count(&self.post_state.content),
            NewPostField::Tags => self.tags_input_cursor = char_count(&self.tags_input),
            NewPostField::Mood => self.mood_cursor = char_count(&self.post_state.mood),
            NewPostField::Lang => self.lang_cursor = char_count(&self.post_state.lang),
            NewPostField::PollEnd => {
                self.poll_end_cursor = self.post_state.poll_end.as_ref().map_or(0, |s| char_count(s));
            }
            NewPostField::PollOption => {
                self.poll_option_cursor = self.post_state.poll_option.as_ref().map_or(0, |s| char_count(s));
            }
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.current_field != NewPostField::Content {
            return;
        }

        let content = &self.post_state.content;
        if content.is_empty() {
            return;
        }

        let (current_line, current_col) = Self::get_cursor_line_col(content, self.content_cursor);

        if current_line > 0 {
            let target_line = current_line - 1;
            if let Some(new_cursor) = Self::get_cursor_from_line_col(content, target_line, current_col) {
                self.content_cursor = new_cursor;
            }
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.current_field != NewPostField::Content {
            return;
        }

        let content = &self.post_state.content;
        if content.is_empty() {
            return;
        }

        let (current_line, current_col) = Self::get_cursor_line_col(content, self.content_cursor);
        let total_lines = content.split('\n').count();
        
        if current_line < total_lines - 1 {
            let target_line = current_line + 1;
            if let Some(new_cursor) = Self::get_cursor_from_line_col(content, target_line, current_col) {
                self.content_cursor = new_cursor;
            }
        }
    }

    /// Helper function to get line and column from cursor position (character-based)
    fn get_cursor_line_col(content: &str, cursor_pos: usize) -> (usize, usize) {
        let chars: Vec<char> = content.chars().collect();
        let chars_before_cursor = &chars[..cursor_pos.min(chars.len())];
        
        let mut line = 0;
        let mut col = 0;
        
        for &ch in chars_before_cursor {
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        
        (line, col)
    }

    /// Helper function to get cursor position from line and column (character-based)
    fn get_cursor_from_line_col(content: &str, target_line: usize, target_col: usize) -> Option<usize> {
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        
        if target_line >= lines.len() {
            return None;
        }

        let mut cursor_pos = 0;
        for i in 0..target_line {
            cursor_pos += lines[i].chars().count() + 1; // +1 for the newline character
        }
        
        let line_char_len = lines[target_line].chars().count();
        cursor_pos += target_col.min(line_char_len);
        
        Some(cursor_pos)
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
                insert_char_at_pos(&mut self.post_state.content, self.content_cursor, c);
                self.content_cursor += 1;
            }
            ReplyField::Tags => {
                insert_char_at_pos(&mut self.tags_input, self.tags_input_cursor, c);
                self.tags_input_cursor += 1;
            }
            ReplyField::Mood => {
                insert_char_at_pos(&mut self.post_state.mood, self.mood_cursor, c);
                self.mood_cursor += 1;
            }
            ReplyField::PollOption => {}
        }
    }

    pub fn handle_newline(&mut self) {
        match self.current_field {
            ReplyField::Content => {
                insert_char_at_pos(&mut self.post_state.content, self.content_cursor, '\n');
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
                remove_char_at_pos(&mut self.post_state.content, self.content_cursor);
                if self.content_cursor > 0 {
                    self.content_cursor -= 1;
                }
            }
            ReplyField::Tags => {
                remove_char_at_pos(&mut self.tags_input, self.tags_input_cursor);
                if self.tags_input_cursor > 0 {
                    self.tags_input_cursor -= 1;
                }
            }
            ReplyField::Mood => {
                remove_char_at_pos(&mut self.post_state.mood, self.mood_cursor);
                if self.mood_cursor > 0 {
                    self.mood_cursor -= 1;
                }
            }
            ReplyField::PollOption => {}
        }
    }

    pub fn handle_delete(&mut self) {
        match self.current_field {
            ReplyField::Content => {
                if self.content_cursor < char_count(&self.post_state.content) {
                    // For delete, we remove the character at the current cursor position
                    let current_pos = self.content_cursor;
                    self.content_cursor += 1;
                    remove_char_at_pos(&mut self.post_state.content, current_pos + 1);
                    self.content_cursor = current_pos;
                }
            }
            ReplyField::Tags => {
                if self.tags_input_cursor < char_count(&self.tags_input) {
                    let current_pos = self.tags_input_cursor;
                    self.tags_input_cursor += 1;
                    remove_char_at_pos(&mut self.tags_input, current_pos + 1);
                    self.tags_input_cursor = current_pos;
                }
            }
            ReplyField::Mood => {
                if self.mood_cursor < char_count(&self.post_state.mood) {
                    let current_pos = self.mood_cursor;
                    self.mood_cursor += 1;
                    remove_char_at_pos(&mut self.post_state.mood, current_pos + 1);
                    self.mood_cursor = current_pos;
                }
            }
            ReplyField::PollOption => {}
        }
    }

    pub fn move_cursor_left(&mut self) {
        match self.current_field {
            ReplyField::Content => {
                if self.content_cursor > 0 {
                    self.content_cursor -= 1;
                }
            }
            ReplyField::Tags => {
                if self.tags_input_cursor > 0 {
                    self.tags_input_cursor -= 1;
                }
            }
            ReplyField::Mood => {
                if self.mood_cursor > 0 {
                    self.mood_cursor -= 1;
                }
            }
            ReplyField::PollOption => {}
        }
    }

    pub fn move_cursor_right(&mut self) {
        match self.current_field {
            ReplyField::Content => {
                if self.content_cursor < char_count(&self.post_state.content) {
                    self.content_cursor += 1;
                }
            }
            ReplyField::Tags => {
                if self.tags_input_cursor < char_count(&self.tags_input) {
                    self.tags_input_cursor += 1;
                }
            }
            ReplyField::Mood => {
                if self.mood_cursor < char_count(&self.post_state.mood) {
                    self.mood_cursor += 1;
                }
            }
            ReplyField::PollOption => {}
        }
    }

    pub fn move_cursor_to_start(&mut self) {
        match self.current_field {
            ReplyField::Content => self.content_cursor = 0,
            ReplyField::Tags => self.tags_input_cursor = 0,
            ReplyField::Mood => self.mood_cursor = 0,
            ReplyField::PollOption => {}
        }
    }

    pub fn move_cursor_to_end(&mut self) {
        match self.current_field {
            ReplyField::Content => self.content_cursor = char_count(&self.post_state.content),
            ReplyField::Tags => self.tags_input_cursor = char_count(&self.tags_input),
            ReplyField::Mood => self.mood_cursor = char_count(&self.post_state.mood),
            ReplyField::PollOption => {}
        }
    }

    pub fn move_cursor_up(&mut self) {
        // Only allow vertical movement in content field
        if self.current_field != ReplyField::Content {
            return;
        }

        let content = &self.post_state.content;
        if content.is_empty() {
            return;
        }

        let (current_line, current_col) = Self::get_cursor_line_col(content, self.content_cursor);

        if current_line > 0 {
            let target_line = current_line - 1;
            if let Some(new_cursor) = Self::get_cursor_from_line_col(content, target_line, current_col) {
                self.content_cursor = new_cursor;
            }
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.current_field != ReplyField::Content {
            return;
        }

        let content = &self.post_state.content;
        if content.is_empty() {
            return;
        }

        let (current_line, current_col) = Self::get_cursor_line_col(content, self.content_cursor);

        let total_lines = content.split('\n').count();
        
        if current_line < total_lines - 1 {
            let target_line = current_line + 1;
            if let Some(new_cursor) = Self::get_cursor_from_line_col(content, target_line, current_col) {
                self.content_cursor = new_cursor;
            }
        }
    }

    /// Helper function to get line and column from cursor position (character-based)
    fn get_cursor_line_col(content: &str, cursor_pos: usize) -> (usize, usize) {
        let chars: Vec<char> = content.chars().collect();
        let chars_before_cursor = &chars[..cursor_pos.min(chars.len())];
        
        let mut line = 0;
        let mut col = 0;
        
        for &ch in chars_before_cursor {
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        
        (line, col)
    }

    /// Helper function to get cursor position from line and column (character-based)
    fn get_cursor_from_line_col(content: &str, target_line: usize, target_col: usize) -> Option<usize> {
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        
        if target_line >= lines.len() {
            return None;
        }

        let mut cursor_pos = 0;
        for i in 0..target_line {
            cursor_pos += lines[i].chars().count() + 1; // +1 for the newline character
        }
        
        let line_char_len = lines[target_line].chars().count();
        cursor_pos += target_col.min(line_char_len);
        
        Some(cursor_pos)
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
