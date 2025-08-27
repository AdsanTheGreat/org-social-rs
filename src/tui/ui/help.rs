//! Help overlay UI component.

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Draw the help overlay
pub fn draw_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled("Org-Social TUI Help", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  j/↓  - Move to next post"),
        Line::from("  k/↑  - Move to previous post"),
        Line::from("  d    - Scroll down in current post"),
        Line::from("  u    - Scroll up in current post"),
        Line::from("  g    - Go to first post"),
        Line::from("  G    - Go to last post"),
        Line::from(""),
        Line::from("View Modes:"),
        Line::from("  t    - Toggle between list and threaded view"),
        Line::from("  List View: Shows all posts chronologically"),
        Line::from("  Threaded View: Shows posts organized by conversations"),
        Line::from(""),
        Line::from("Actions:"),
        Line::from("  r    - Reply to current post"),
        Line::from("  n    - Create new post"),
        Line::from("  q    - Quit application"),
        Line::from(""),
        Line::from("Hyperlinks:"),
        Line::from("  l    - Navigate to next link"),
        Line::from("  L    - Navigate to previous link"),
        Line::from("  Enter - Open focused link (browser or clipboard)"),
        Line::from(""),
        Line::from("Other:"),
        Line::from("  h/?  - Show/hide this help"),
        Line::from("  Esc  - Cancel current action"),
        Line::from(""),
        Line::from("In Reply Mode:"),
        Line::from("  Type to compose reply"),
        Line::from("  Enter/Shift+Enter - Add newline"),
        Line::from("  Ctrl+S - Submit reply"),
        Line::from("  Tab/Shift+Tab - Switch fields"),
        Line::from("  F1 - Remove last tag"),
        Line::from("  Esc - Cancel reply"),
        Line::from(""),
        Line::from("In New Post Mode:"),
        Line::from("  Type to compose post"),
        Line::from("  Enter/Shift+Enter - Add newline"),
        Line::from("  Ctrl+S - Submit post"),
        Line::from("  Tab/Shift+Tab - Switch fields"),
        Line::from("  F1 - Remove last tag"),
        Line::from("  Esc - Cancel post"),
        Line::from(""),
        Line::from(Span::styled("Press h or Esc to close help", Style::default().fg(Color::Green))),
    ];

    let help_area = Rect {
        x: area.width / 6,
        y: area.height / 8,
        width: (area.width * 2) / 3,
        height: (area.height * 3) / 4,
    };

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(Color::Black));

    f.render_widget(help, help_area);
}
