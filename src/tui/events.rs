//! Event handling and input processing.

use super::modes::AppMode;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use org_social_lib_rs::{new_post, reply};

/// Handle keyboard events and return appropriate actions
pub enum EventResult {
    Quit,
    Continue,
    NextPost,
    PrevPost,
    ScrollDown,
    ScrollUp,
    GoToFirst,
    GoToLast,
    ToggleView,
    StartReply,
    StartNewPost,
    ToggleHelp,
    Cancel,
    ReplyInput(char),
    ReplyNewline,
    ReplyBackspace,
    NextReplyField,
    PrevReplyField,
    FinalizeTags,
    RemoveLastTag,
    SubmitReply,
    NewPostInput(char),
    NewPostNewline,
    NewPostBackspace,
    NextNewPostField,
    PrevNewPostField,
    SubmitNewPost,
    NextLink,
    PrevLink,
    ActivateLink,
}

pub fn handle_key_event(key: KeyEvent, mode: &AppMode) -> EventResult {
    match mode {
        AppMode::Browsing => handle_browsing_input(key),
        AppMode::Reply => handle_reply_input(key),
        AppMode::NewPost => handle_new_post_input(key),
        AppMode::Help => handle_help_input(key),
    }
}

fn handle_browsing_input(key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Char('q') => EventResult::Quit,
        KeyCode::Char('j') | KeyCode::Down => EventResult::NextPost,
        KeyCode::Char('k') | KeyCode::Up => EventResult::PrevPost,
        KeyCode::Char('d') | KeyCode::PageDown => EventResult::ScrollDown,
        KeyCode::Char('u') | KeyCode::PageUp => EventResult::ScrollUp,
        KeyCode::Char('g') => EventResult::GoToFirst,
        KeyCode::Char('G') => EventResult::GoToLast,
        KeyCode::Char('t') => EventResult::ToggleView,
        KeyCode::Char('r') => EventResult::StartReply,
        KeyCode::Char('n') => EventResult::StartNewPost,
        KeyCode::Char('h') | KeyCode::Char('?') => EventResult::ToggleHelp,
        KeyCode::Char('l') => EventResult::NextLink,      // Navigate to next activatable element
        KeyCode::Char('L') => EventResult::PrevLink,      // Navigate to previous activatable element
        KeyCode::Enter | KeyCode::Tab => EventResult::ActivateLink, // Activate focused element (link or block)
        KeyCode::Esc => EventResult::Cancel,
        _ => EventResult::Continue,
    }
}

fn handle_reply_input(key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Char(c) => {
            // Handle Ctrl+S for submission
            if c == 's' && key.modifiers.contains(KeyModifiers::CONTROL) {
                EventResult::SubmitReply
            } else {
                EventResult::ReplyInput(c)
            }
        }
        KeyCode::Backspace => EventResult::ReplyBackspace,
        KeyCode::Enter => {
            // Handle different Enter key combinations
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                // Shift+Enter for newline
                EventResult::ReplyNewline
            } else {
                // Plain Enter behavior depends on current field - this will be handled in the app
                EventResult::FinalizeTags
            }
        }
        KeyCode::Tab => EventResult::NextReplyField,
        KeyCode::BackTab => EventResult::PrevReplyField,
        KeyCode::F(1) => EventResult::RemoveLastTag, // F1 to remove last tag
        KeyCode::Esc => EventResult::Cancel,
        _ => EventResult::Continue,
    }
}

fn handle_new_post_input(key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Char(c) => {
            // Handle Ctrl+S for submission
            if c == 's' && key.modifiers.contains(KeyModifiers::CONTROL) {
                EventResult::SubmitNewPost
            } else {
                EventResult::NewPostInput(c)
            }
        }
        KeyCode::Backspace => EventResult::NewPostBackspace,
        KeyCode::Enter => {
            // Handle different Enter key combinations
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                // Shift+Enter for newline
                EventResult::NewPostNewline
            } else {
                // Plain Enter behavior depends on current field - this will be handled in the app
                EventResult::FinalizeTags
            }
        }
        KeyCode::Tab => EventResult::NextNewPostField,
        KeyCode::BackTab => EventResult::PrevNewPostField,
        KeyCode::F(1) => EventResult::RemoveLastTag, // F1 to remove last tag
        KeyCode::Esc => EventResult::Cancel,
        _ => EventResult::Continue,
    }
}

fn handle_help_input(key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Char('h') | KeyCode::Char('?') | KeyCode::Esc => EventResult::ToggleHelp,
        _ => EventResult::Continue,
    }
}

/// Handle Enter key behavior in reply mode based on current field
pub fn handle_reply_enter(reply_state: &Option<reply::ReplyState>) -> EventResult {
    match reply_state.as_ref().map(|rs| &rs.current_field) {
        Some(reply::ReplyField::Tags) => EventResult::FinalizeTags,
        Some(reply::ReplyField::Content) => {
            // In content field, plain Enter adds newline for convenience
            EventResult::ReplyNewline
        }
        _ => {
            // In mood field, plain Enter submits
            EventResult::SubmitReply
        }
    }
}

/// Handle Enter key behavior in new post mode based on current field
pub fn handle_new_post_enter(new_post_state: &Option<new_post::NewPostState>) -> EventResult {
    match new_post_state.as_ref().map(|nps| &nps.current_field) {
        Some(new_post::NewPostField::Tags) => EventResult::FinalizeTags,
        Some(new_post::NewPostField::Content) => {
            // In content field, plain Enter adds newline for convenience
            EventResult::NewPostNewline
        }
        _ => {
            // In other fields, plain Enter submits
            EventResult::SubmitNewPost
        }
    }
}
