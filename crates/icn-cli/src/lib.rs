//! ICN Command Line Interface Library
//!
//! This library provides the core functionality for the ICN CLI.
//! It includes client implementations, data models, and utility functions.

mod client;

use std::path::PathBuf;
use std::error::Error;
use std::fs;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use log::{debug, info, error};

pub use client::{
    IcnClient, IcnClientError, 
    Identity, Cooperative, Resource, Proposal, 
    NetworkPeer, NetworkStatus, PingResult,
    PeerStatistics
};

/// CLI configuration error types
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read configuration file: {0}")]
    ReadError(String),
    
    #[error("Failed to parse configuration: {0}")]
    ParseError(String),
    
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
    
    #[error("Missing required configuration: {0}")]
    MissingValue(String),
}

/// Configuration for the ICN CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// API configuration
    #[serde(default)]
    pub api: ApiConfig,
    
    /// Network configuration
    #[serde(default)]
    pub network: NetworkConfig,
    
    /// Output configuration
    #[serde(default)]
    pub output: OutputConfig,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig::default(),
            network: NetworkConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API URL
    #[serde(default = "default_api_url")]
    pub url: String,
    
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            url: default_api_url(),
            timeout: default_timeout(),
        }
    }
}

fn default_api_url() -> String {
    "http://localhost:8082".to_string()
}

fn default_timeout() -> u64 {
    30
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Preferred peers
    #[serde(default)]
    pub preferred_peers: Vec<String>,
    
    /// Connect timeout in seconds
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            preferred_peers: Vec::new(),
            connect_timeout: default_connect_timeout(),
        }
    }
}

fn default_connect_timeout() -> u64 {
    15
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Default output format
    #[serde(default = "default_format")]
    pub default_format: String,
    
    /// Enable color output
    #[serde(default = "default_color")]
    pub color: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            default_format: default_format(),
            color: default_color(),
        }
    }
}

fn default_format() -> String {
    "table".to_string()
}

fn default_color() -> bool {
    true
}

/// Load configuration from file and environment
pub fn load_config(config_path: Option<&str>) -> Result<CliConfig, Box<dyn Error>> {
    // Default config path
    let config_path = if let Some(path) = config_path {
        PathBuf::from(path)
    } else if let Some(home) = dirs::home_dir() {
        home.join(".icn").join("config.toml")
    } else {
        return Err(ConfigError::ReadError("Could not determine home directory".to_string()).into());
    };
    
    debug!("Loading configuration from {:?}", config_path);
    
    // Start with default config
    let mut config = CliConfig::default();
    
    // Load from file if it exists
    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(content) => {
                match toml::from_str::<CliConfig>(&content) {
                    Ok(file_config) => {
                        info!("Loaded configuration from {:?}", config_path);
                        config = file_config;
                    },
                    Err(e) => {
                        error!("Failed to parse configuration file: {}", e);
                        return Err(ConfigError::ParseError(e.to_string()).into());
                    }
                }
            },
            Err(e) => {
                error!("Failed to read configuration file: {}", e);
                return Err(ConfigError::ReadError(e.to_string()).into());
            }
        }
    } else {
        debug!("Configuration file {:?} not found, using defaults", config_path);
    }
    
    // Override with environment variables
    if let Ok(url) = std::env::var("ICN_API_URL") {
        config.api.url = url;
    }
    
    if let Ok(timeout) = std::env::var("ICN_API_TIMEOUT") {
        if let Ok(timeout) = timeout.parse::<u64>() {
            config.api.timeout = timeout;
        }
    }
    
    if let Ok(format) = std::env::var("ICN_OUTPUT_FORMAT") {
        config.output.default_format = format;
    }
    
    if let Ok(color) = std::env::var("ICN_OUTPUT_COLOR") {
        if let Ok(color) = color.parse::<bool>() {
            config.output.color = color;
        }
    }
    
    debug!("Final configuration: {:?}", config);
    Ok(config)
}

/// Format duration for human-readable output
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else if seconds < 86400 {
        format!("{}h {}m {}s", seconds / 3600, (seconds % 3600) / 60, seconds % 60)
    } else {
        format!("{}d {}h {}m {}s", 
            seconds / 86400, 
            (seconds % 86400) / 3600, 
            (seconds % 3600) / 60, 
            seconds % 60
        )
    }
}

/// Format bytes for human-readable output
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    
    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes < TB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3600), "1h 0m 0s");
        assert_eq!(format_duration(86400), "1d 0h 0m 0s");
        assert_eq!(format_duration(90061), "1d 1h 1m 1s");
    }
    
    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1500), "1.46 KB");
        assert_eq!(format_bytes(1500000), "1.43 MB");
        assert_eq!(format_bytes(1500000000), "1.40 GB");
        assert_eq!(format_bytes(1500000000000), "1.36 TB");
    }
    
    #[test]
    fn test_default_config() {
        let config = CliConfig::default();
        assert_eq!(config.api.url, "http://localhost:8082");
        assert_eq!(config.api.timeout, 30);
        assert_eq!(config.output.default_format, "table");
        assert_eq!(config.output.color, true);
    }
}
