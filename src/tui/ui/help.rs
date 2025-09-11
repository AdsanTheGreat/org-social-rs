//! Help overlay UI component.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Draw the help overlay
pub fn draw_help(f: &mut Frame, area: Rect, scroll_offset: u16) {
    let help_text = vec![
        // Whitespace before text is ignored, it's just internal structure
        Line::from(""),
        Line::from(Span::styled("Org-Social TUI Help", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))),
        Line::from(""),
        Line::from(Span::styled("Help Navigation: j/k or ↓/↑ to scroll, g/G for top/bottom", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  j/↓ - Move to next post"),
        Line::from("  k/↑ - Move to previous post"),
        Line::from("  d - Scroll down in current post"),
        Line::from("  u - Scroll up in current post"),
        Line::from("  g - Go to first post"),
        Line::from("  G - Go to last post"),
        Line::from(""),
        Line::from("View Modes:"),
        Line::from("  t - Toggle between modes: List -> Threaded -> Notifications -> List"),
        Line::from("  List View: Shows all posts chronologically"),
        Line::from("  Threaded View: Shows posts organized by conversations"),
        Line::from("  Notifications View: Shows mentions and replies targeted at the user"),
        Line::from(""),
        Line::from("Actions:"),
        Line::from("  r - Reply to current post"),
        Line::from("  n - Create new post"),
        Line::from("  v - Count votes for poll (threaded view only)"),
        Line::from("  q - Quit application"),
        Line::from(""),
        Line::from("Hyperlinks/Mentions/Blocks:"),
        Line::from("  l - Navigate to next link/mention/block"),
        Line::from("  L - Navigate to previous link/mention/block"),
        Line::from("  Enter - Depending on the type:"),
        Line::from("    Link: Open in browser"),
        Line::from("    Mention: Open user's social.org in browser"),
        Line::from("    Block: Toggle block"),
        Line::from(""),
        Line::from("Other:"),
        Line::from("  h/? - Show/hide this help"),
        Line::from("  Esc - Cancel current action"),
        Line::from(""),
        Line::from("In New Post/Reply Mode:"),
        Line::from("  Enter/Shift+Enter - Add newline (content field only)"),
        Line::from("  Backspace/Delete - Delete character before/at cursor"),
        Line::from("  ←/→ - Move cursor left/right"),
        Line::from("  ↑/↓ - Move cursor up/down (content field only)"),
        Line::from("  Home|Ctrl+A/End|Ctrl+E - Jump to start/end of field"),
        Line::from("  Tab/Shift+Tab - Switch between fields"),
        Line::from("  Ctrl+S - Submit"),
        Line::from("  F1 - Remove last tag"),
        Line::from("  F2 - Reset all fields"),
        Line::from("  Esc - Cancel reply"),
        Line::from(""),
        Line::from(Span::styled("Press h or Esc to close help", Style::default().fg(Color::Green))),
    ];

    let help_area = Rect {
        x: area.width / 6,
        y: area.height / 8,
        width: (area.width * 2) / 3,
        height: (area.height * 3) / 4,
    };

    // Calculate the maximum scroll offset based on content height and widget height
    let content_height = help_text.len() as u16;
    let widget_height = help_area.height.saturating_sub(2); // Subtract 2 for borders
    let max_scroll = content_height.saturating_sub(widget_height);
    let actual_scroll = scroll_offset.min(max_scroll);

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true })
        .scroll((actual_scroll, 0))
        .style(Style::default().bg(Color::Black));

    f.render_widget(help, help_area);
}
