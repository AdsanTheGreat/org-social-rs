use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::cli;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to the default social.org file
    pub social_file: PathBuf,
    /// Default number of posts to show in feed
    pub default_feed_count: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            social_file: PathBuf::from("social.org"),
            default_feed_count: 10,
        }
    }
}

impl Config {
    /// Load configuration from file or create default
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;
        
        let settings = config::Config::builder()
            .add_source(config::Config::try_from(&Config::default())?)
            .add_source(config::File::from(config_path).required(false))
            .add_source(config::Environment::with_prefix("ORG_SOCIAL"))
            .build()?;

        Ok(settings.try_deserialize()?)
    }

    /// Get the configuration file path
    fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Unable to determine config directory")?;
        
        let app_config_dir = config_dir.join("org-social-rs");
        std::fs::create_dir_all(&app_config_dir)?;
        
        Ok(app_config_dir.join("config.toml"))
    }

    /// Save the current configuration to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;
        let toml_string = toml::to_string_pretty(self)?;
        std::fs::write(config_path, toml_string)?;
        Ok(())
    }

    /// Create a default configuration file if it doesn't exist
    pub fn create_default_if_missing() -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;
        
        if !config_path.exists() {
            let default_config = Config::default();
            default_config.save()?;
            println!("Created default configuration at: {}", config_path.display());
        }
        
        Ok(())
    }

    /// Merge CLI options with config, CLI takes precedence
    pub fn merge_with_cli(&self, cli: &cli::Cli) -> Self {
        Self {
            social_file: cli.file_override().unwrap_or_else(|| self.social_file.clone()),
            // Keep other fields from config
            default_feed_count: self.default_feed_count,
        }
    }
}
