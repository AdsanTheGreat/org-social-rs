//! TUI module for the org-social reader.
//!
//! This module provides a terminal-based user interface using ratatui,
//! allowing users to scroll through posts, and in the future do some actions on them.

use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

pub mod activatable;
pub mod app;
pub mod events;
pub mod modes;
pub mod navigation;
pub mod ui;

pub use app::TUI;
use crate::parser;

/// Launch the TUI application
pub async fn run_tui(
    file_path: &std::path::Path,
    user_profile: &parser::Profile,
    user_posts: Vec<parser::Post>,
    user_only: bool,
    source_filter: Option<String>,
    days_filter: Option<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = TUI::new(file_path, user_profile, user_posts, user_only, source_filter, days_filter).await?;

    // Run the event loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        println!("{err:?}");
    }

    Ok(())
}

/// Main event loop for the TUI
async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut TUI,
) -> io::Result<()> {
    loop {
        // Update cursor blink state
        app.update_cursor_blink();
        
        terminal.draw(|f| {
            ui::draw_ui(
                f,
                &app.mode,
                &app.view_mode,
                &app.posts,
                &app.notification_feed,
                &app.thread_view,
                &app.navigator,
                app.current_post(),
                &app.reply_state,
                &app.new_post_state,
                &app.poll_vote_state,
                &app.status_message,
                app.cursor_visible,
                app.help_scroll,
                &app.activatable_collector,
                Some(&app.activatable_manager),
            )
        })?;

        // Use poll to check for events with timeout for cursor blinking
        if crossterm::event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    crossterm::event::KeyCode::Char('q') if app.mode == modes::AppMode::Browsing => {
                        return Ok(());
                    }
                    _ => {
                        app.handle_event(key);
                    }
                }
            }
        }
    }
}
