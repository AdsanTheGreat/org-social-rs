//! Application mode definitions and state management.

#[derive(Clone, PartialEq)]
pub enum AppMode {
    Browsing,
    Reply,
    NewPost,
    Help,
}

#[derive(Clone, PartialEq)]
pub enum ViewMode {
    List,
    Threaded,
}

impl ViewMode {
    pub fn toggle(&self) -> Self {
        match self {
            ViewMode::List => ViewMode::Threaded,
            ViewMode::Threaded => ViewMode::List,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ViewMode::List => "List View",
            ViewMode::Threaded => "Threaded View",
        }
    }
}
