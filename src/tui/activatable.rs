//! Activatable elements handling for TUI - visual styling and interaction logic.
//!
//! This module manages interactive elements in the TUI including hyperlinks,
//! collapsible org-mode blocks, and other activatable content.

use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};
use org_social_lib_rs::blocks::ActivatableElement;

/// Represents an activatable element's position in the rendered content
#[derive(Debug, Clone)]
pub struct ActivatablePosition {
    pub element_type: ActivatableType,
    pub line: usize,
    pub start_col: usize,
    pub end_col: usize,
    pub original_line: usize, // Line number in original content before processing
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActivatableType {
    Hyperlink { url: String, display_text: String },
    Mention { url: String, username: String },
    Block { block_type: String, is_collapsed: bool },
}

/// Shared state for collecting activatable elements during rendering
pub type ActivatableCollector = Arc<Mutex<Vec<(ActivatableType, usize, usize, usize, usize)>>>;

/// Manager for tracking and interacting with activatable elements in the TUI
#[derive(Debug)]
pub struct ActivatableManager {
    /// All activatable elements in the current view, indexed by a unique ID
    elements: HashMap<usize, ActivatablePosition>,
    /// Currently focused/selected element ID
    focused_element: Option<usize>,
    /// Next ID to assign to an element
    next_id: usize,
    /// Block collapse state (original line number -> is_collapsed)
    collapsed_blocks: HashMap<usize, bool>,
    /// Processed content with collapsed blocks
    processed_content: Option<String>,
    /// Original activatable elements from the content
    content_elements: Vec<ActivatableElement>,
}

impl Default for ActivatableManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ActivatableManager {
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
            focused_element: None,
            next_id: 0,
            collapsed_blocks: HashMap::new(),
            processed_content: None,
            content_elements: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.elements.clear();
        self.focused_element = None;
        self.next_id = 0;
    }

    pub fn process_post(&mut self, post: &org_social_lib_rs::parser::Post) {
        // Clear current elements but keep focus info
        let current_focused_type = self.focused_element()
            .map(|pos| match &pos.element_type {
                ActivatableType::Hyperlink { url, .. } => format!("hyperlink:{url}"),
                ActivatableType::Mention { url, .. } => format!("mention:{url}"),
                ActivatableType::Block { block_type, .. } => format!("block:{}:{}", block_type, pos.original_line),
            });

        self.clear();

        // Process blocks from the post
        for element in post.blocks() {
            match element {
                org_social_lib_rs::blocks::ActivatableElement::Block(block) => {
                    let is_collapsed = element.is_collapsed();
                    self.add_block_element(
                        element.start_line(),
                        element.end_line(),
                        0,
                        block.block_type.clone(),
                        is_collapsed,
                    );
                }
            }
        }

        // Try to restore focus
        if let Some(focus_key) = current_focused_type {
            self.restore_focus(&focus_key);
        }
    }

    pub fn update_from_collector(&mut self, collector: &ActivatableCollector) {
        if let Ok(elements_data) = collector.lock() {
            for (element_type, line, start_col, end_col, original_line) in elements_data.iter() {
                match element_type {
                    ActivatableType::Hyperlink { url, display_text } => {
                        self.add_hyperlink(url.clone(), display_text.clone(), *line, *start_col, *end_col);
                    }
                    ActivatableType::Mention { url, username } => {
                        self.add_mention(url.clone(), username.clone(), *line, *start_col, *end_col);
                    }
                    ActivatableType::Block { block_type, is_collapsed } => {
                        self.add_block_element(*original_line, *line, *start_col, block_type.clone(), *is_collapsed);
                    }
                }
            }
        }
    }

    pub fn create_collector() -> ActivatableCollector {
        Arc::new(Mutex::new(Vec::new()))
    }

    pub fn add_hyperlink(&mut self, url: String, display_text: String, line: usize, start_col: usize, end_col: usize) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        
        self.elements.insert(id, ActivatablePosition {
            element_type: ActivatableType::Hyperlink { url, display_text },
            line,
            start_col,
            end_col,
            original_line: line,
        });
        
        id
    }

    pub fn add_mention(&mut self, url: String, username: String, line: usize, start_col: usize, end_col: usize) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        
        self.elements.insert(id, ActivatablePosition {
            element_type: ActivatableType::Mention { url, username },
            line,
            start_col,
            end_col,
            original_line: line,
        });
        
        id
    }

    pub fn add_block_element(&mut self, original_line: usize, display_line: usize, start_col: usize, block_type: String, is_collapsed: bool) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        
        // Estimate end column based on collapsed state
        let end_col = if is_collapsed {
            start_col + format!("[+] {} [...]", self.get_block_summary(&block_type)).len()
        } else {
            start_col + block_type.len() + 10 // Rough estimate for begin line
        };
        
        self.elements.insert(id, ActivatablePosition {
            element_type: ActivatableType::Block { block_type, is_collapsed },
            line: display_line,
            start_col,
            end_col,
            original_line,
        });
        
        id
    }

    fn get_block_summary(&self, block_type: &str) -> String {
        match block_type.to_lowercase().as_str() {
            "src" => "Code",
            "quote" => "Quote", 
            "example" => "Example",
            "verse" => "Verse",
            _ => "Block",
        }.to_string()
    }

    pub fn focused_element(&self) -> Option<&ActivatablePosition> {
        self.focused_element.and_then(|id| self.elements.get(&id))
    }

    pub fn focus_next(&mut self) -> bool {
        if self.elements.is_empty() {
            return false;
        }

        let current_id = self.focused_element.unwrap_or(usize::MAX);
        let mut element_ids: Vec<_> = self.elements.keys().cloned().collect();
        element_ids.sort();

        let next_index = if let Some(pos) = element_ids.iter().position(|&id| id > current_id) {
            pos
        } else {
            0 // Wrap to first
        };

        self.focused_element = Some(element_ids[next_index]);
        true
    }

    pub fn focus_prev(&mut self) -> bool {
        if self.elements.is_empty() {
            return false;
        }

        let current_id = self.focused_element.unwrap_or(0);
        let mut element_ids: Vec<_> = self.elements.keys().cloned().collect();
        element_ids.sort();

        let prev_index = if let Some(pos) = element_ids.iter().rposition(|&id| id < current_id) {
            pos
        } else {
            element_ids.len() - 1 // Wrap to last
        };

        self.focused_element = Some(element_ids[prev_index]);
        true
    }

    pub fn is_focused(&self, element_id: usize) -> bool {
        self.focused_element == Some(element_id)
    }

    pub fn is_url_focused(&self, url: &str) -> bool {
        if let Some(focused) = self.focused_element() {
            match &focused.element_type {
                ActivatableType::Hyperlink { url: focused_url, .. } => focused_url == url,
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn is_mention_focused(&self, url: &str) -> bool {
        if let Some(focused) = self.focused_element() {
            match &focused.element_type {
                ActivatableType::Mention { url: focused_url, .. } => focused_url == url,
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn is_block_focused(&self, original_line: usize) -> bool {
        if let Some(focused) = self.focused_element() {
            match &focused.element_type {
                ActivatableType::Block { .. } => focused.original_line == original_line,
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn get_focused_block_info(&self) -> Option<(usize, usize, bool)> {
        if let Some(focused) = self.focused_element() {
            if let ActivatableType::Block { is_collapsed, .. } = &focused.element_type {
                // Find the corresponding block element to get the end line
                for element in &self.content_elements {
                    if element.start_line() == focused.original_line {
                        return Some((element.start_line(), element.end_line(), *is_collapsed));
                    }
                }
            }
        }
        None
    }

    pub fn is_line_in_focused_block(&self, line: usize) -> bool {
        if let Some((start_line, end_line, _)) = self.get_focused_block_info() {
            line >= start_line && line <= end_line
        } else {
            false
        }
    }

    pub fn activate_focused(&mut self) -> Option<String> {
        if let Some(focused) = self.focused_element.and_then(|id| self.elements.get(&id).cloned()) {
            match &focused.element_type {
                ActivatableType::Hyperlink { url, .. } => {
                    Some(self.activate_link(url))
                }
                ActivatableType::Mention { url, .. } => {
                    Some(self.activate_link(url))
                }
                ActivatableType::Block { .. } => {
                    self.toggle_block_at_line(focused.original_line);
                    Some(format!("Toggled block at line {}", focused.original_line + 1))
                }
            }
        } else {
            None
        }
    }

    pub fn toggle_block_at_line(&mut self, original_line: usize) {
        let current_state = self.collapsed_blocks.get(&original_line).copied().unwrap_or(false);
        self.collapsed_blocks.insert(original_line, !current_state);
    }

    pub fn get_processed_content(&self) -> Option<&str> {
        self.processed_content.as_deref()
    }

    pub fn get_collapsed_blocks(&self) -> &HashMap<usize, bool> {
        &self.collapsed_blocks
    }

    /// Activate a hyperlink by opening it
    fn activate_link(&self, url: &str) -> String {
        // Try to open the URL in the default browser with suppressed output
        let result = if cfg!(target_os = "linux") {
            Command::new("xdg-open")
                .arg(url)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
        } else if cfg!(target_os = "macos") { // Hell yeah, why not extra OSes? no idea if they work, I know I once saw something like this
            Command::new("open")
                .arg(url)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
        } else if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", "start", url])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Unsupported OS"))
        };

        match result {
            Ok(_) => format!("Opened link: {url}"),
            Err(_) => {
                // TODO: Implement fallback behavior, maybe copy to clipboard?
                format!("Failed to open link: {url}")
            }
        }
    }

    /// Restore focus to a previously focused element
    fn restore_focus(&mut self, focus_key: &str) {
        for (&id, pos) in &self.elements {
            let key = match &pos.element_type {
                ActivatableType::Hyperlink { url, .. } => format!("hyperlink:{url}"),
                ActivatableType::Mention { url, .. } => format!("mention:{url}"),
                ActivatableType::Block { block_type, .. } => format!("block:{}:{}", block_type, pos.original_line),
            };
            if key == focus_key {
                self.focused_element = Some(id);
                break;
            }
        }
    }

    /// Get debug information about current state
    pub fn debug_info(&self) -> String {
        format!(
            "Elements: {}, Focused: {:?}, Collapsed blocks: {}",
            self.elements.len(),
            self.focused_element,
            self.collapsed_blocks.len()
        )
    }
}

/// Create a styled span for a hyperlink with proper focus highlighting
pub fn create_hyperlink_span<'a>(text: String, url: &str, activatable_manager: Option<&ActivatableManager>) -> Span<'a> {
    let is_focused = activatable_manager
        .map(|manager| manager.is_url_focused(url))
        .unwrap_or(false);

    let style = if is_focused {
        Style::default()
            .fg(Color::LightBlue)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::UNDERLINED)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::UNDERLINED)
    };

    Span::styled(text, style)
}

/// Create a styled span for a mention with proper focus highlighting
pub fn create_mention_span<'a>(text: String, url: &str, activatable_manager: Option<&ActivatableManager>) -> Span<'a> {
    let is_focused = activatable_manager
        .map(|manager| manager.is_mention_focused(url))
        .unwrap_or(false);

    let style = if is_focused {
        Style::default()
            .fg(Color::LightCyan)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::UNDERLINED)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::UNDERLINED)
    };

    Span::styled(text, style)
}

/// Create a styled span for a collapsed block with proper focus highlighting
pub fn create_block_span<'a>(text: String, original_line: usize, activatable_manager: Option<&ActivatableManager>) -> Span<'a> {
    let is_focused = activatable_manager
        .map(|manager| manager.is_block_focused(original_line))
        .unwrap_or(false);

    let style = if is_focused {
        Style::default()
            .fg(Color::Yellow)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::DIM)
    };

    Span::styled(text, style)
}

/// Add a hyperlink to the collector during rendering
pub fn collect_hyperlink(collector: &ActivatableCollector, url: String, display_text: String, line: usize, start_col: usize, end_col: usize) {
    if let Ok(mut elements) = collector.lock() {
        elements.push((
            ActivatableType::Hyperlink { url, display_text },
            line,
            start_col,
            end_col,
            line, // original_line same as line for hyperlinks
        ));
    }
}

/// Add a mention to the collector during rendering
pub fn collect_mention(collector: &ActivatableCollector, url: String, username: String, line: usize, start_col: usize, end_col: usize) {
    if let Ok(mut elements) = collector.lock() {
        elements.push((
            ActivatableType::Mention { url, username },
            line,
            start_col,
            end_col,
            line, // original_line same as line for mentions
        ));
    }
}

/// Add a block element to the collector during rendering
pub fn collect_block(collector: &ActivatableCollector, block_type: String, is_collapsed: bool, line: usize, start_col: usize, end_col: usize, original_line: usize) {
    if let Ok(mut elements) = collector.lock() {
        elements.push((
            ActivatableType::Block { block_type, is_collapsed },
            line,
            start_col,
            end_col,
            original_line,
        ));
    }
}
