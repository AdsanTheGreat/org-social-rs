use colored::*;
use org_social_lib_rs::{parser, profile::Profile};

/// Format a profile with colors for CLI display
pub fn format_profile_colored(profile: &Profile) -> String {
    let mut output = Vec::new();
    
    output.push(format!("{} {}", "Title:".green().bold(), profile.title().cyan()));
    output.push(format!("{} {}", "Nick:".green().bold(), profile.nick().yellow()));
    
    if !profile.description().is_empty() {
        output.push(format!("{} {}", "Description:".green().bold(), profile.description()));
    }
    
    if let Some(avatar) = profile.avatar() {
        output.push(format!("{} {}", "Avatar:".green().bold(), avatar.blue()));
    }
    
    if let Some(links) = profile.link() {
        if !links.is_empty() {
            if links.len() == 1 {
                output.push(format!("{} {}", "Link:".green().bold(), links[0].blue().underline()));
            } else {
                output.push(format!("{}", "Links:".green().bold()));
                for (i, link) in links.iter().enumerate() {
                    output.push(format!("  {}. {}", (i + 1).to_string().bright_black(), link.blue().underline()));
                }
            }
        }
    }
    
    if let Some(follows) = profile.follow() {
        if !follows.is_empty() {
            output.push(format!("{} {} {}", 
                "Following:".green().bold(), 
                follows.len().to_string().yellow().bold(),
                if follows.len() == 1 { "user" } else { "users" }.bright_black()
            ));
            for (i, (name, url)) in follows.iter().enumerate() {
                output.push(format!("  {}. {} - {}", 
                    (i + 1).to_string().bright_black(),
                    name.cyan().bold(), 
                    url.blue().underline()
                ));
            }
        }
    }
    
    if let Some(contacts) = profile.contact() {
        if !contacts.is_empty() {
            if contacts.len() == 1 {
                output.push(format!("{} {}", "Contact:".green().bold(), contacts[0].magenta()));
            } else {
                output.push(format!("{}", "Contact:".green().bold()));
                for (i, contact) in contacts.iter().enumerate() {
                    output.push(format!("  {}. {}", (i + 1).to_string().bright_black(), contact.magenta()));
                }
            }
        }
    }
    
    if let Some(source) = profile.source() {
        output.push(format!("{} {}", "Source:".green().bold(), source.bright_black()));
    }
    
    output.join("\n")
}

/// Format a post with colors for CLI display
pub fn format_post_colored(post: &parser::Post, profile: Option<&Profile>) -> String {
    let mut output = String::new();

    // Build header line with username, tags, and time
    let mut header = if let Some(author) = post.author() {
        author.green().bold().to_string()
    } else {
        "unknown".bright_black().to_string()
    };

    // Add language as first tag if present
    if let Some(lang) = post.lang() {
        header.push_str(&format!(" {}", format!("#{}", lang).blue()));
    }

    // Add other tags
    if let Some(tags) = post.tags() {
        for tag in tags {
            header.push_str(&format!(" {}", format!("#{}", tag).blue()));
        }
    }

    // Add timestamp if available
    if let Some(time) = post.time() {
        header.push_str(&format!(" {} {}", "•".bright_black(), 
            time.format("%Y-%m-%d %H:%M").to_string().bright_black()));
    }

    output.push_str(&format!("{} {}\n", 
        "---".bright_black(),
        format!("{} ---", header)));

    // Collect additional metadata for display
    let mut metadata = Vec::new();

    if let Some(client) = post.client() {
        metadata.push(format!("{} {}", "Client:".bright_black(), client.yellow()));
    }

    if let Some(reply_to) = post.reply_to() {
        // Extract the post ID from the reply_to URL 
        let reply_id = if let Some(hash_pos) = reply_to.rfind('#') {
            &reply_to[hash_pos + 1..]
        } else {
            reply_to
        };
        
        // Try to map the URL to a nickname from the profile's follow list
        let reply_display = if let Some(profile) = profile {
            if let Some(follows) = profile.follow() {
                // Extract the base URL (without the fragment)
                let base_url = if let Some(hash_pos) = reply_to.rfind('#') {
                    &reply_to[..hash_pos]
                } else {
                    reply_to
                };
                
                // Normalize URLs by removing trailing slashes for comparison - they might be included by mistake
                let normalized_base = base_url.trim_end_matches('/');
                
                // Find the nickname for this URL
                if let Some((nick, _)) = follows.iter().find(|(_, url)| url.trim_end_matches('/') == normalized_base) {
                    format!("{nick}#{reply_id}")
                } else {
                    // No nickname found, use url#ID format
                    format!("{base_url}#{reply_id}")
                }
            } else {
                // No follow list, use url#ID format
                format!("{}#{}", 
                    if let Some(hash_pos) = reply_to.rfind('#') {
                        &reply_to[..hash_pos]
                    } else {
                        reply_to
                    }, 
                    reply_id)
            }
        } else {
            // No profile, use url#ID format
            format!("{}#{}", 
                if let Some(hash_pos) = reply_to.rfind('#') {
                    &reply_to[..hash_pos]
                } else {
                    reply_to
                }, 
                reply_id)
        };
        
        metadata.push(format!("{} {}", "Reply to:".bright_black(), reply_display.magenta()));
    }

    if let Some(mood) = post.mood() {
        metadata.push(format!("{} {}", "Mood:".bright_black(), mood.yellow()));
    }

    if let Some(poll_end) = post.poll_end() {
        metadata.push(format!("{} {}", "Poll ends:".bright_black(), poll_end.yellow()));
    }

    if let Some(poll_option) = post.poll_option() {
        metadata.push(format!("{} {}", "Poll option:".bright_black(), poll_option.yellow()));
    }

    // Display metadata if any exists
    if !metadata.is_empty() {
        output.push_str(&format!("{}\n", metadata.join(" | ")));
    }

    // Add post content
    output.push_str(post.content());

    output
}
