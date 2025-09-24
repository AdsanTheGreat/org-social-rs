//! Interactive status bar UI component.

use ratatui::{
    layout::Rect,
    Frame,
};

use super::super::{
    modes::{AppMode, ViewMode},
    status_bar_widget::{StatusBarWidget, StatusBarState, StatusBarContext},
};

/// Draw the interactive status area
pub fn draw_status_area(
    f: &mut Frame,
    area: Rect,
    mode: &AppMode,
    view_mode: &ViewMode,
    status_bar_state: &StatusBarState,
) {
    match mode {
        AppMode::StatusBarWidget => {
            // Render the active status bar widget
            f.render_widget(&status_bar_state.current_widget, area);
        }
        _ => {
            if matches!(status_bar_state.current_widget, StatusBarWidget::Default) {
                render_default_status(f, area, mode, view_mode);
            } else {
                f.render_widget(&status_bar_state.current_widget, area);
            }
        }
    }
}

/// Render the default status bar for different modes
fn render_default_status(
    f: &mut Frame,
    area: Rect,
    mode: &AppMode,
    view_mode: &ViewMode,
) {
    match mode {
        AppMode::Browsing => {
            let context = StatusBarContext { view_mode: Some(view_mode) };
            let widget = StatusBarWidget::Default;
            widget.render_with_context(area, f.buffer_mut(), &context);
        }
        AppMode::Reply => {
            let widget = StatusBarWidget::message("In reply mode - see reply window", None);
            f.render_widget(&widget, area);
        }
        AppMode::NewPost => {
            let widget = StatusBarWidget::message("In new post mode - see new post window", None);
            f.render_widget(&widget, area);
        }
        AppMode::Help => {
            let widget = StatusBarWidget::message("Showing help - press h or Esc to close", None);
            f.render_widget(&widget, area);
        }
        AppMode::PollVote => {
            let widget = StatusBarWidget::message("Poll voting mode - use j/k to select, Enter to vote, Esc to cancel", None);
            f.render_widget(&widget, area);
        }
        AppMode::StatusBarWidget => {
            let widget = StatusBarWidget::Default;
            let context = StatusBarContext { view_mode: Some(view_mode) };
            widget.render_with_context(area, f.buffer_mut(), &context);
        }
    }
}