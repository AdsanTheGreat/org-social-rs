//! Status bar widget system for interactive input/output functionality.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Status bar callback identifiers
#[derive(Clone, Debug, PartialEq)]
pub enum StatusBarCallback {
    Search,
    ConfirmAction,
    FilterTypeSelection,
    FilterAuthorSelection,
    FilterTagInput,
    FilterLanguageInput,
    FilterSourceInput,
}

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Widget, Wrap},
};

/// Types of status bar widgets
#[derive(Clone, Debug, PartialEq)]
pub enum StatusBarWidget {
    /// Default status bar showing navigation help
    Default,
    /// Text input widget
    TextInput {
        prompt: String,
        placeholder: String,
        value: String,
        cursor_pos: usize,
    },
    /// Option selection widget
    OptionSelect {
        prompt: String,
        options: Vec<String>,
        selected_index: usize,
    },
    /// Progress/loading widget - perhaps for async network ops
    Progress {
        message: String,
        progress: Option<f64>,
    },
    /// Confirmation dialog - perhaps for destructive actions
    Confirmation {
        message: String,
        yes_text: String,
        no_text: String,
        selected_yes: bool,
    },
    /// Display-only message with timeout
    Message {
        text: String,
        timeout_ms: Option<u64>,
    },
}

/// State management for status bar widgets
#[derive(Clone, Debug)]
pub struct StatusBarState {
    /// Current active widget
    pub current_widget: StatusBarWidget,
    /// Widget history for navigation
    pub widget_history: VecDeque<StatusBarWidget>,
    /// Maximum history size
    pub max_history: usize,
    /// Callback identifier for widget completion
    pub callback_id: Option<StatusBarCallback>,
    /// Timestamp when the current widget was set (for timeouts)
    pub widget_start_time: Option<Instant>,
}

impl Default for StatusBarState {
    fn default() -> Self {
        Self {
            current_widget: StatusBarWidget::Default,
            widget_history: VecDeque::new(),
            max_history: 10,
            callback_id: None,
            widget_start_time: None,
        }
    }
}

impl StatusBarState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the current widget, optionally preserving history
    pub fn set_widget(&mut self, widget: StatusBarWidget, preserve_history: bool) {
        if preserve_history {
            self.widget_history.push_back(self.current_widget.clone());
            if self.widget_history.len() > self.max_history {
                self.widget_history.pop_front();
            }
        }
        self.current_widget = widget;
        self.widget_start_time = Some(Instant::now());
    }

    /// Go back to the previous widget in history
    pub fn go_back(&mut self) -> bool {
        if let Some(previous) = self.widget_history.pop_back() {
            self.current_widget = previous;
            self.widget_start_time = Some(Instant::now()); // Reset timer for previous widget
            true
        } else {
            false
        }
    }

    /// Reset to default widget and clear history
    pub fn reset(&mut self) {
        self.current_widget = StatusBarWidget::Default;
        self.widget_history.clear();
        self.callback_id = None;
        self.widget_start_time = None;
    }

    /// Check if the current widget has expired based on its timeout
    pub fn check_timeout(&mut self) -> bool {
        if let StatusBarWidget::Message { timeout_ms: Some(timeout), .. } = &self.current_widget {
            if let Some(start_time) = self.widget_start_time {
                let elapsed = start_time.elapsed();
                if elapsed >= Duration::from_millis(*timeout) {
                    self.reset();
                    return true;
                }
            }
        }
        false
    }

    /// Check if the current widget is interactive
    pub fn is_interactive(&self) -> bool {
        !matches!(self.current_widget, StatusBarWidget::Default | StatusBarWidget::Message { .. } | StatusBarWidget::Progress { .. })
    }

    /// Check if the current widget captures input
    pub fn captures_input(&self) -> bool {
        matches!(
            self.current_widget,
            StatusBarWidget::TextInput { .. } | StatusBarWidget::OptionSelect { .. } | StatusBarWidget::Confirmation { .. }
        )
    }
}

/// Widget creation helpers
impl StatusBarWidget {
    /// Create a text input widget
    pub fn text_input(prompt: impl Into<String>, placeholder: impl Into<String>) -> Self {
        Self::TextInput {
            prompt: prompt.into(),
            placeholder: placeholder.into(),
            value: String::new(),
            cursor_pos: 0,
        }
    }

    /// Create a text input widget with pre-filled value
    pub fn text_input_with_value(prompt: impl Into<String>, placeholder: impl Into<String>, value: impl Into<String>) -> Self {
        let value_str = value.into();
        let cursor_pos = value_str.len(); // Position cursor at the end of pre-filled text
        Self::TextInput {
            prompt: prompt.into(),
            placeholder: placeholder.into(),
            value: value_str,
            cursor_pos,
        }
    }

    /// Create an option select widget
    pub fn option_select(prompt: impl Into<String>, options: Vec<String>) -> Self {
        Self::OptionSelect {
            prompt: prompt.into(),
            options,
            selected_index: 0,
        }
    }

    /// Create a progress widget
    pub fn progress(message: impl Into<String>, progress: Option<f64>) -> Self {
        Self::Progress {
            message: message.into(),
            progress,
        }
    }

    /// Create a confirmation dialog
    pub fn confirmation(
        message: impl Into<String>,
        yes_text: impl Into<String>,
        no_text: impl Into<String>,
    ) -> Self {
        Self::Confirmation {
            message: message.into(),
            yes_text: yes_text.into(),
            no_text: no_text.into(),
            selected_yes: true,
        }
    }

    /// Create a message widget
    pub fn message(text: impl Into<String>, timeout_ms: Option<u64>) -> Self {
        Self::Message {
            text: text.into(),
            timeout_ms,
        }
    }
}

/// Input handling for widgets
impl StatusBarWidget {
    /// Handle character input for text input widget
    pub fn handle_char_input(&mut self, c: char) -> bool {
        match self {
            StatusBarWidget::TextInput { value, cursor_pos, .. } => {
                value.insert(*cursor_pos, c);
                *cursor_pos += 1;
                true
            }
            _ => false,
        }
    }

    /// Handle backspace for text input widget
    pub fn handle_backspace(&mut self) -> bool {
        match self {
            StatusBarWidget::TextInput { value, cursor_pos, .. } => {
                if *cursor_pos > 0 {
                    *cursor_pos -= 1;
                    value.remove(*cursor_pos);
                }
                true
            }
            _ => false,
        }
    }

    /// Handle delete for text input widget
    pub fn handle_delete(&mut self) -> bool {
        match self {
            StatusBarWidget::TextInput { value, cursor_pos, .. } => {
                if *cursor_pos < value.len() {
                    value.remove(*cursor_pos);
                }
                true
            }
            _ => false,
        }
    }

    /// Handle cursor left for text input widget
    pub fn handle_cursor_left(&mut self) -> bool {
        match self {
            StatusBarWidget::TextInput { cursor_pos, .. } => {
                if *cursor_pos > 0 {
                    *cursor_pos -= 1;
                }
                true
            }
            _ => false,
        }
    }

    /// Handle cursor right for text input widget
    pub fn handle_cursor_right(&mut self) -> bool {
        match self {
            StatusBarWidget::TextInput { value, cursor_pos, .. } => {
                if *cursor_pos < value.len() {
                    *cursor_pos += 1;
                }
                true
            }
            _ => false,
        }
    }

    /// Handle cursor start (Home key) for text input widget
    pub fn handle_cursor_start(&mut self) -> bool {
        match self {
            StatusBarWidget::TextInput { cursor_pos, .. } => {
                *cursor_pos = 0;
                true
            }
            _ => false,
        }
    }

    /// Handle cursor end (End key) for text input widget
    pub fn handle_cursor_end(&mut self) -> bool {
        match self {
            StatusBarWidget::TextInput { value, cursor_pos, .. } => {
                *cursor_pos = value.len();
                true
            }
            _ => false,
        }
    }

    /// Handle up/previous for option select and confirmation widgets
    pub fn handle_up(&mut self) -> bool {
        match self {
            StatusBarWidget::OptionSelect { options, selected_index, .. } => {
                if *selected_index > 0 {
                    *selected_index -= 1;
                } else {
                    *selected_index = options.len().saturating_sub(1);
                }
                true
            }
            StatusBarWidget::Confirmation { selected_yes, .. } => {
                *selected_yes = !*selected_yes;
                true
            }
            _ => false,
        }
    }

    /// Handle down/next for option select and confirmation widgets
    pub fn handle_down(&mut self) -> bool {
        match self {
            StatusBarWidget::OptionSelect { options, selected_index, .. } => {
                if *selected_index < options.len().saturating_sub(1) {
                    *selected_index += 1;
                } else {
                    *selected_index = 0;
                }
                true
            }
            StatusBarWidget::Confirmation { selected_yes, .. } => {
                *selected_yes = !*selected_yes;
                true
            }
            _ => false,
        }
    }

    /// Get the current value/selection from the widget
    pub fn get_value(&self) -> Option<String> {
        match self {
            StatusBarWidget::TextInput { value, .. } => Some(value.clone()),
            StatusBarWidget::OptionSelect { options, selected_index, .. } => {
                options.get(*selected_index).cloned()
            }
            StatusBarWidget::Confirmation { selected_yes, yes_text, no_text, .. } => {
                Some(if *selected_yes { yes_text.clone() } else { no_text.clone() })
            }
            _ => None,
        }
    }

    /// Get the selected boolean value for confirmation widgets
    pub fn get_confirmation(&self) -> Option<bool> {
        match self {
            StatusBarWidget::Confirmation { selected_yes, .. } => Some(*selected_yes),
            _ => None,
        }
    }
}

/// Context for rendering the default status bar
pub struct StatusBarContext<'a> {
    pub view_mode: Option<&'a crate::tui::modes::ViewMode>,
    pub current_filter: Option<&'a crate::tui::app::FilterType>,
    pub filter_mode_active: bool,
}

impl<'a> Default for StatusBarContext<'a> {
    fn default() -> Self {
        Self { 
            view_mode: None,
            current_filter: None,
            filter_mode_active: false,
        }
    }
}

/// Render the status bar widget to a given area
impl Widget for &StatusBarWidget {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        self.render_with_context(area, buf, &StatusBarContext::default());
    }
}

impl StatusBarWidget {
    /// Render with additional context information
    pub fn render_with_context(&self, area: Rect, buf: &mut ratatui::buffer::Buffer, context: &StatusBarContext) {
        match self {
            StatusBarWidget::Default => {
                let mut status_parts = Vec::new();
                
                // Add view mode info
                if let Some(view_mode) = context.view_mode {
                    status_parts.push(view_mode.display_name().to_string());
                }
                
                // Add filter info
                if let Some(filter) = context.current_filter {
                    status_parts.push(format!("Filtered: {}", filter.display_name()));
                }
                
                let prefix = if !status_parts.is_empty() {
                    format!("{} | ", status_parts.join(" | "))
                } else {
                    String::new()
                };
                
                let controls = if context.current_filter.is_some() {
                    "f:change filter | other keys: navigate/interact"
                } else {
                    "t:toggle view | r:reply | n:new post | f:filters | h:help | q:quit"
                };
                
                let text = Text::from(format!("{}{}", prefix, controls));
                let paragraph = Paragraph::new(text)
                    .block(Block::default().borders(Borders::ALL).title("Status"))
                    .wrap(Wrap { trim: true });
                paragraph.render(area, buf);
            }
            StatusBarWidget::TextInput { prompt, placeholder, value, cursor_pos } => {
                render_text_input(area, buf, prompt, placeholder, value, *cursor_pos);
            }
            StatusBarWidget::OptionSelect { prompt, options, selected_index } => {
                render_option_select(area, buf, prompt, options, *selected_index);
            }
            StatusBarWidget::Progress { message, progress } => {
                render_progress(area, buf, message, *progress);
            }
            StatusBarWidget::Confirmation { message, yes_text, no_text, selected_yes } => {
                render_confirmation(area, buf, message, yes_text, no_text, *selected_yes);
            }
            StatusBarWidget::Message { text, .. } => {
                render_message(area, buf, text);
            }
        }
    }
}

/// Render text input widget
fn render_text_input(area: Rect, buf: &mut ratatui::buffer::Buffer, prompt: &str, placeholder: &str, value: &str, cursor_pos: usize) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(prompt.len() as u16 + 2),
            Constraint::Min(0),
        ])
        .split(area);

    // Render prompt
    let prompt_widget = Paragraph::new(format!("{}: ", prompt))
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().add_modifier(Modifier::BOLD));
    prompt_widget.render(chunks[0], buf);

    // Render input field
    let display_text = if value.is_empty() && !placeholder.is_empty() {
        placeholder
    } else {
        value
    };

    let input_style = if value.is_empty() && !placeholder.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default()
    };

    // Create text with cursor
    let mut spans = vec![];
    if cursor_pos == 0 {
        spans.push(Span::styled("█", Style::default().bg(Color::White).fg(Color::Black)));
    }
    
    for (i, ch) in display_text.chars().enumerate() {
        if i + 1 == cursor_pos {
            spans.push(Span::styled(ch.to_string(), input_style));
            spans.push(Span::styled("█", Style::default().bg(Color::White).fg(Color::Black)));
        } else {
            spans.push(Span::styled(ch.to_string(), input_style));
        }
    }
    
    if cursor_pos >= display_text.len() && cursor_pos > 0 {
        spans.push(Span::styled("█", Style::default().bg(Color::White).fg(Color::Black)));
    }

    let input_widget = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::ALL))
        .style(input_style);
    input_widget.render(chunks[1], buf);
}

/// Render option select widget
fn render_option_select(area: Rect, buf: &mut ratatui::buffer::Buffer, prompt: &str, options: &[String], selected_index: usize) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Prompt area
            Constraint::Min(0),    // Options area
        ])
        .split(area);

    // Render prompt
    let prompt_widget = Paragraph::new(prompt)
        .block(Block::default().borders(Borders::ALL).title("Select Option"))
        .style(Style::default().add_modifier(Modifier::BOLD));
    prompt_widget.render(chunks[0], buf);

    // Render options
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, option)| {
            let style = if i == selected_index {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(option.as_str()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Blue).fg(Color::White));
    
    let mut list_state = ListState::default();
    list_state.select(Some(selected_index));
    ratatui::widgets::StatefulWidget::render(list, chunks[1], buf, &mut list_state);
}

/// Render progress widget
fn render_progress(area: Rect, buf: &mut ratatui::buffer::Buffer, message: &str, progress: Option<f64>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Message area
            Constraint::Length(3), // Progress bar area
        ])
        .split(area);

    // Render message
    let message_widget = Paragraph::new(message)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().add_modifier(Modifier::BOLD));
    message_widget.render(chunks[0], buf);

    // Render progress bar
    if let Some(progress_value) = progress {
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Blue))
            .percent((progress_value * 100.0) as u16);
        gauge.render(chunks[1], buf);
    } else {
        // Indeterminate progress - show animated dots or spinner
        let dots = "...";
        let spinner_widget = Paragraph::new(dots)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        spinner_widget.render(chunks[1], buf);
    }
}

/// Render confirmation dialog
fn render_confirmation(area: Rect, buf: &mut ratatui::buffer::Buffer, message: &str, yes_text: &str, no_text: &str, selected_yes: bool) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Message area
            Constraint::Length(3), // Buttons area
        ])
        .split(area);

    // Render message
    let message_widget = Paragraph::new(message)
        .block(Block::default().borders(Borders::ALL).title("Confirm"))
        .wrap(Wrap { trim: true })
        .style(Style::default().add_modifier(Modifier::BOLD));
    message_widget.render(chunks[0], buf);

    // Render buttons
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(chunks[1]);

    let yes_style = if selected_yes {
        Style::default().bg(Color::Green).fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let no_style = if !selected_yes {
        Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let yes_widget = Paragraph::new(yes_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center)
        .style(yes_style);
    yes_widget.render(button_chunks[0], buf);

    let no_widget = Paragraph::new(no_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center)
        .style(no_style);
    no_widget.render(button_chunks[1], buf);
}

/// Render message widget
fn render_message(area: Rect, buf: &mut ratatui::buffer::Buffer, text: &str) {
    let message_widget = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Message"))
        .wrap(Wrap { trim: true })
        .style(Style::default().add_modifier(Modifier::BOLD));
    message_widget.render(area, buf);
}