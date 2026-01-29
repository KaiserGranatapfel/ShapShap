// Configuration file support for ShapShap

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub author: Option<String>,
    pub email: Option<String>,
    pub default_prefix: Option<String>,
    pub default_output: Option<PathBuf>,
    pub send_email: Option<SendEmailConfig>,
    pub validation: Option<ValidationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailConfig {
    pub smtp_server: Option<String>,
    pub smtp_port: Option<u16>,
    pub from_email: Option<String>,
    pub default_to: Option<Vec<String>>,
    pub default_cc: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub check_line_length: Option<bool>,
    pub max_line_length: Option<usize>,
    pub check_trailing_whitespace: Option<bool>,
    pub check_tab_indentation: Option<bool>,
    pub require_sign_off: Option<bool>,
    pub check_commit_message_format: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            author: None,
            email: None,
            default_prefix: Some("PATCH".to_string()),
            default_output: None,
            send_email: None,
            validation: Some(ValidationConfig {
                check_line_length: Some(true),
                max_line_length: Some(80),
                check_trailing_whitespace: Some(true),
                check_tab_indentation: Some(true),
                require_sign_off: Some(true),
                check_commit_message_format: Some(true),
            }),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;
            
            let config: Config = toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;
            
            Ok(config)
        } else {
            // Return default config
            Ok(Config::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }
        
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;
        
        Ok(())
    }

    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to find config directory")?;
        Ok(config_dir.join("shapshap").join("config.toml"))
    }

    pub fn init_config() -> Result<()> {
        let config = Config::default();
        config.save()?;
        println!("✓ Created default config at: {}", Self::config_path()?.display());
        Ok(())
    }
}
