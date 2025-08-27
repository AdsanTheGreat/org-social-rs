use clap::Parser;
use cli::Cli;
use org_social_lib_rs::parser;
use std::fs;

mod cli;
mod formatting;
mod tui;

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    
    // Configure color output based on the --color flag
    args.configure_colors();

    if args.verbose {
        println!("Reading file: {:?}", args.file);
    }

    // Read the user's .org file
    let file_content = match fs::read_to_string(&args.file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {:?}: {}", args.file, e);
            std::process::exit(1);
        }
    };

    let file_path = args.file.to_string_lossy().to_string();
    let (user_profile, user_posts) = parser::parse_file(&file_content, Some(file_path.clone()));

    args.handle_command(&user_profile, user_posts).await;
}
