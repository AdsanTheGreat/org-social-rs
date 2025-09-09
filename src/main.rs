use clap::Parser;
use cli::Cli;
use org_social_lib_rs::parser;
use std::fs;

mod cli;
mod config;
mod editor;
mod formatting;
mod tui;

#[tokio::main]
async fn main() {
    // Load configuration
    let config = match config::Config::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Warning: Failed to load configuration: {}", e);
            eprintln!("Using default configuration...");
            config::Config::default()
        }
    };

    // Create default config file if it doesn't exist
    if let Err(e) = config::Config::create_default_if_missing() {
        eprintln!("Warning: Failed to create default config: {}", e);
    }

    let args = Cli::parse();
    
    // Merge config with CLI arguments (CLI takes precedence)
    let effective_config = config.merge_with_cli(
        &args
    );
    
    // Configure color output based on the --color flag
    args.configure_colors();


    // Read the user's .org file
    let file_content = match fs::read_to_string(&effective_config.social_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {:?}: {}", effective_config.social_file, e);
            std::process::exit(1);
        }
    };

    let file_path = effective_config.social_file.to_string_lossy().to_string();
    let (user_profile, user_posts) = parser::parse_file(&file_content, Some(file_path.clone()));

    args.handle_command(&user_profile, user_posts, &effective_config).await;
}
