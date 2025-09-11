//! Event handling and input processing.

use super::modes::AppMode;
use crate::editor::{NewPostEditor, NewPostField, ReplyEditor, ReplyField};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    ReplyDelete,
    ReplyCursorLeft,
    ReplyCursorRight,
    ReplyCursorUp,
    ReplyCursorDown,
    ReplyCursorStart,
    ReplyCursorEnd,
    NextReplyField,
    PrevReplyField,
    FinalizeTags,
    RemoveLastTag,
    SubmitReply,
    NewPostInput(char),
    NewPostNewline,
    NewPostBackspace,
    NewPostDelete,
    NewPostCursorLeft,
    NewPostCursorRight,
    NewPostCursorUp,
    NewPostCursorDown,
    NewPostCursorStart,
    NewPostCursorEnd,
    NextNewPostField,
    PrevNewPostField,
    SubmitNewPost,
    NextLink,
    PrevLink,
    ActivateLink,
    CountPollVotes,
    StartPollVote,
    PollVoteUp,
    PollVoteDown,
    SubmitPollVote,
    ResetFields,
}

pub fn handle_key_event(key: KeyEvent, mode: &AppMode) -> EventResult {
    match mode {
        AppMode::Browsing => handle_browsing_input(key),
        AppMode::Reply => handle_reply_input(key),
        AppMode::NewPost => handle_new_post_input(key),
        AppMode::Help => handle_help_input(key),
        AppMode::PollVote => handle_poll_vote_input(key),
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
        KeyCode::Char('v') => EventResult::CountPollVotes, // Count votes for poll in current post
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
            } else if c == 'a' && key.modifiers.contains(KeyModifiers::CONTROL) {
                EventResult::ReplyCursorStart
            } else if c == 'e' && key.modifiers.contains(KeyModifiers::CONTROL) {
                EventResult::ReplyCursorEnd
            } else {
                EventResult::ReplyInput(c)
            }
        }
        KeyCode::Backspace => EventResult::ReplyBackspace,
        KeyCode::Delete => EventResult::ReplyDelete,
        KeyCode::Left => EventResult::ReplyCursorLeft,
        KeyCode::Right => EventResult::ReplyCursorRight,
        KeyCode::Up => EventResult::ReplyCursorUp,
        KeyCode::Down => EventResult::ReplyCursorDown,
        KeyCode::Home => EventResult::ReplyCursorStart,
        KeyCode::End => EventResult::ReplyCursorEnd,
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
        KeyCode::F(2) => EventResult::ResetFields, // F2 to reset all fields
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
            } else if c == 'a' && key.modifiers.contains(KeyModifiers::CONTROL) {
                EventResult::NewPostCursorStart
            } else if c == 'e' && key.modifiers.contains(KeyModifiers::CONTROL) {
                EventResult::NewPostCursorEnd
            } else {
                EventResult::NewPostInput(c)
            }
        }
        KeyCode::Backspace => EventResult::NewPostBackspace,
        KeyCode::Delete => EventResult::NewPostDelete,
        KeyCode::Left => EventResult::NewPostCursorLeft,
        KeyCode::Right => EventResult::NewPostCursorRight,
        KeyCode::Up => EventResult::NewPostCursorUp,
        KeyCode::Down => EventResult::NewPostCursorDown,
        KeyCode::Home => EventResult::NewPostCursorStart,
        KeyCode::End => EventResult::NewPostCursorEnd,
        KeyCode::Enter => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                EventResult::NewPostNewline
            } else {
                EventResult::FinalizeTags
            }
        }
        KeyCode::Tab => EventResult::NextNewPostField,
        KeyCode::BackTab => EventResult::PrevNewPostField,
        KeyCode::F(1) => EventResult::RemoveLastTag, // F1 to remove last tag
        KeyCode::F(2) => EventResult::ResetFields, // F2 to reset all fields
        KeyCode::Esc => EventResult::Cancel,
        _ => EventResult::Continue,
    }
}

fn handle_help_input(key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Char('h') | KeyCode::Char('?') | KeyCode::Esc => EventResult::ToggleHelp,
        KeyCode::Char('j') | KeyCode::Down => EventResult::ScrollDown,
        KeyCode::Char('k') | KeyCode::Up => EventResult::ScrollUp,
        KeyCode::Char('g') => EventResult::GoToFirst,
        KeyCode::Char('G') => EventResult::GoToLast,
        _ => EventResult::Continue,
    }
}

/// Handle Enter key behavior in reply mode based on current field
pub fn handle_reply_enter(reply_state: &Option<ReplyEditor>) -> EventResult {
    match reply_state.as_ref().map(|rs| &rs.current_field) {
        Some(ReplyField::Tags) => EventResult::FinalizeTags,
        Some(ReplyField::Content) => {
            EventResult::ReplyNewline
        }
        Some(ReplyField::PollOption) => {
            EventResult::SubmitReply
        }
        _ => {
            // In mood field, plain Enter goes to the next field
            EventResult::NextReplyField
        }
    }
}

/// Handle Enter key behavior in new post mode based on current field
pub fn handle_new_post_enter(new_post_state: &Option<NewPostEditor>) -> EventResult {
    match new_post_state.as_ref().map(|nps| &nps.current_field) {
        Some(NewPostField::Tags) => EventResult::FinalizeTags,
        Some(NewPostField::Content) => {
            // In content field, plain Enter adds newline for convenience
            EventResult::NewPostNewline
        }
        _ => {
            // In other fields, plain Enter goes to the next field
            EventResult::NextNewPostField
        }
    }
}

fn handle_poll_vote_input(key: KeyEvent) -> EventResult {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => EventResult::Cancel,
        KeyCode::Char('j') | KeyCode::Down => EventResult::PollVoteDown,
        KeyCode::Char('k') | KeyCode::Up => EventResult::PollVoteUp,
        KeyCode::Enter => EventResult::SubmitPollVote,
        _ => EventResult::Continue,
    }
}
