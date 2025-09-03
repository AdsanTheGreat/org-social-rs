use crate::{formatting, tui};
use chrono::{Duration, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use org_social_lib_rs::{feed, network, parser};
use std::path::PathBuf;

#[derive(Clone, ValueEnum)]
pub enum ColorOption {
    /// Automatically detect if colors should be used (default)
    Auto,
    /// Always use colors, even when piping
    Always,
    /// Never use colors
    Never,
}

#[derive(Parser)]
#[command(name = "org-social-rs")]
#[command(about = "An org-social reader")]
#[command(version)]
pub struct Cli {
    /// Path to the user's .org social file (overrides config)
    #[arg(short, long)]
    pub file: Option<PathBuf>,
    
    /// Enable verbose output (overrides config)
    #[arg(short, long)]
    pub verbose: Option<bool>,
    
    /// Control colored output
    #[arg(long, value_enum, default_value = "auto")]
    pub color: ColorOption,
    
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Read posts from your feed
    Feed {
        /// Number of posts to show (uses config default if not specified)
        #[arg(short, long)]
        count: Option<usize>,
        
        /// Show only user's own posts (don't fetch from followed users)
        #[arg(long)]
        user_only: bool,
        
        /// Show posts from a specific source URL
        #[arg(long)]
        source: Option<String>,
        
        /// Show posts from the last N days
        #[arg(long)]
        days: Option<u32>,
    },
    
    /// Show profile information
    Profile,
    
    /// List followed users
    Following,
    
    /// Show feed statistics
    Stats,
    
    /// Launch TUI interface
    Tui {
        /// Show only user's own posts (don't fetch from followed users)
        #[arg(long)]
        user_only: bool,
        
        /// Show posts from a specific source URL
        #[arg(long)]
        source: Option<String>,
        
        /// Show posts from the last N days
        #[arg(long)]
        days: Option<u32>,
    },
}

impl Cli {
    /// Configure color output based on the color flag
    pub fn configure_colors(&self) {
        match self.color {
            ColorOption::Always => {
                colored::control::set_override(true);
            }
            ColorOption::Never => {
                colored::control::set_override(false);
            }
            ColorOption::Auto => {
                // Don't override, let colored crate auto-detect
                // This is the default behavior
            }
        }
    }

    /// Get file override from CLI args
    pub fn file_override(&self) -> Option<PathBuf> {
        self.file.clone()
    }
    
    pub async fn handle_command(&self, user_profile: &parser::Profile, user_posts: Vec<parser::Post>, config: &crate::config::Config) {
        let verbose = self.verbose.unwrap_or(false);
        match &self.command {
            Commands::Feed { count, user_only, source, days } => {
                let effective_count = count.unwrap_or(config.default_feed_count);
                handle_feed_command(user_profile, user_posts, effective_count, *user_only, source.clone(), *days, verbose).await;
            }
            Commands::Profile => {
                handle_profile_command(user_profile);
            }
            Commands::Following => {
                handle_following_command(user_profile);
            }
            Commands::Stats => {
                handle_stats_command(user_profile, &user_posts, verbose).await;
            }
            Commands::Tui { user_only, source, days } => {
                handle_tui_command(&config.social_file, user_profile, user_posts, *user_only, source.clone(), *days).await;
            }
        }
    }
}

async fn handle_feed_command(
    user_profile: &parser::Profile,
    user_posts: Vec<parser::Post>,
    count: usize,
    user_only: bool,
    source_filter: Option<String>,
    days_filter: Option<u32>,
    verbose: bool,
) {
    if verbose {
        println!("{}", "Creating feed...".bright_black());
    }
    
    let feed = if user_only {
        feed::Feed::create_user_feed(user_profile, user_posts)
    } else {
        match feed::Feed::create_combined_feed(user_profile, user_posts).await {
            Ok(feed) => feed,
            Err(e) => {
                eprintln!("{} {}", "Warning:".yellow().bold(), format!("Failed to fetch remote feeds: {e}").red());
                println!("{}", "Showing user posts only...".yellow());
                let user_posts = parser::parse_file(&std::fs::read_to_string("social.org").unwrap_or_default(), None).1;
                feed::Feed::create_user_feed(user_profile, user_posts)
            }
        }
    };
    
    let mut posts_to_show: Vec<&parser::Post> = feed.posts.iter().collect();
    
    // Apply source filter
    if let Some(source) = &source_filter {
        posts_to_show.retain(|post| {
                post.source().as_ref().map(|s| s == source).unwrap_or(false)
            });
    }
    
    // Apply days filter
    if let Some(days) = days_filter {
        let cutoff = Utc::now() - Duration::try_days(days as i64).unwrap_or_default();
        posts_to_show.retain(|post| {
                if let Some(post_time) = post.time() {
                    post_time.naive_utc() > cutoff.naive_utc()
                } else {
                    false
                }
            });
    }
    
    // Take only the requested count
    posts_to_show.truncate(count);
    
    println!("{}", "=== Feed ===".cyan().bold());
    println!("{}", format!("Showing {} posts", posts_to_show.len()).bright_black());
    for (i, post) in posts_to_show.iter().enumerate() {
        println!("{}", formatting::format_post_colored(post, Some(user_profile)));
        if i < posts_to_show.len() - 1 {
            println!();
        }
    }
}

fn handle_profile_command(user_profile: &parser::Profile) {
    println!("{}", "=== Profile ===".cyan().bold());
    println!("{}", formatting::format_profile_colored(user_profile));
}

fn handle_following_command(user_profile: &parser::Profile) {
    println!("{}", "=== Following ===".cyan().bold());
    match user_profile.follow() {
        Some(follows) => {
            if follows.is_empty() {
                println!("{}", "Not following anyone yet.".yellow());
            } else {
                for (i, (name, url)) in follows.iter().enumerate() {
                    println!("{}. {} - {}", 
                        format!("{}", i + 1).bright_black(),
                        name.green().bold(), 
                        url.blue().underline());
                }
            }
        }
        None => {
            println!("{}", "Not following anyone yet.".yellow());
        }
    }
}

async fn handle_stats_command(
    user_profile: &parser::Profile,
    user_posts: &[parser::Post],
    verbose: bool,
) {
    println!("{}", "=== Statistics ===".cyan().bold());
    println!("{} {}", "User posts:".green(), user_posts.len().to_string().yellow().bold());
    
    if let Some(follows) = user_profile.follow() {
        println!("{} {}", "Following:".green(), format!("{} users", follows.len()).yellow().bold());
        
        if verbose {
            println!("{}", "Fetching remote feed statistics...".bright_black());
            let feeds = network::get_feeds_from_profile_with_timeout(user_profile).await;
            let total_remote_posts: usize = feeds.iter().map(|(_, posts, _)| posts.len()).sum();
            println!("{} {}", "Total remote posts:".green(), total_remote_posts.to_string().yellow().bold());
            println!("{} {}", "Total posts in combined feed:".green(), 
                (user_posts.len() + total_remote_posts).to_string().yellow().bold());
        }
    } else {
        println!("{} {}", "Following:".green(), "0 users".yellow().bold());
    }
}

async fn handle_tui_command(
    file_path: &std::path::Path,
    user_profile: &parser::Profile,
    user_posts: Vec<parser::Post>,
    user_only: bool,
    source_filter: Option<String>,
    days_filter: Option<u32>,
) {
    match tui::run_tui(file_path, user_profile, user_posts, user_only, source_filter, days_filter).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{} {}", "Error running TUI:".red().bold(), e);
            std::process::exit(1);
        }
    }
}
