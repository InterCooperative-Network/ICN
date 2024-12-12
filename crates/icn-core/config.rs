// File: crates/icn-core/src/config.rs
//
// This module provides configuration management for the ICN system, including
// configuration loading, validation, and access to configuration values.

use std::path::PathBuf;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use humantime_serde;
use config::{Config as ConfigFile, File, Environment};

use crate::error::{Error, Result};

/// Main configuration structure for the ICN system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Node identification
    pub node: NodeConfig,
    
    /// Network configuration
    pub network: NetworkConfig,
    
    /// Consensus configuration
    pub consensus: ConsensusConfig,
    
    /// Storage configuration
    pub storage: StorageConfig,
    
    /// Runtime configuration
    pub runtime: RuntimeConfig,
    
    /// Telemetry configuration
    pub telemetry: TelemetryConfig,
}

/// Node-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Unique node identifier
    pub id: String,
    
    /// Node role (validator, observer, etc.)
    pub role: NodeRole,
    
    /// Node data directory
    pub data_dir: PathBuf,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Listen address
    pub listen_addr: String,
    
    /// Listen port
    pub listen_port: u16,
    
    /// Maximum peer connections
    pub max_peers: usize,
    
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<String>,
    
    /// Connection timeout
    #[serde(with = "humantime_serde")]
    pub connection_timeout: Duration,
}

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Minimum number of validators
    pub min_validators: usize,
    
    /// Round timeout
    #[serde(with = "humantime_serde")]
    pub round_timeout: Duration,
    
    /// Consensus threshold (0.0 - 1.0)
    pub consensus_threshold: f64,
    
    /// Maximum consecutive rounds for a validator
    pub max_consecutive_rounds: usize,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Database URL
    pub database_url: String,
    
    /// Maximum database connections
    pub max_connections: u32,
    
    /// Connection timeout
    #[serde(with = "humantime_serde")]
    pub connection_timeout: Duration,
    
    /// Enable database SSL
    pub enable_ssl: bool,
}

/// Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Maximum concurrent tasks
    pub max_tasks: usize,
    
    /// Task stack size
    pub stack_size: usize,
    
    /// Thread pool size
    pub thread_pool_size: usize,
}

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Enable metrics collection
    pub enable_metrics: bool,
    
    /// Metrics endpoint
    pub metrics_endpoint: String,
    
    /// Log level
    pub log_level: LogLevel,
    
    /// Enable debug logs
    pub enable_debug: bool,
}

/// Node roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeRole {
    /// Full validator node
    Validator,
    
    /// Observer node (no validation)
    Observer,
    
    /// Light node
    Light,
}

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Configuration builder for flexible configuration creation
#[derive(Default)]
pub struct ConfigBuilder {
    config_path: Option<PathBuf>,
    env_prefix: Option<String>,
    test_mode: bool,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the configuration file path
    pub fn with_config<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config_path = Some(path.into());
        self
    }

    /// Set the environment variable prefix
    pub fn with_env_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.env_prefix = Some(prefix.into());
        self
    }

    /// Enable test mode with default test configuration
    pub fn with_test_defaults(mut self) -> Self {
        self.test_mode = true;
        self
    }

    /// Build the configuration
    pub fn build(self) -> Result<Config> {
        let mut builder = ConfigFile::builder();

        // Load default configuration
        builder = builder.add_source(File::with_name("config/default"));

        // Load environment-specific configuration
        if let Ok(env) = std::env::var("ICN_ENV") {
            builder = builder.add_source(
                File::with_name(&format!("config/{}", env)).required(false)
            );
        }

        // Load local configuration file if specified
        if let Some(path) = self.config_path {
            builder = builder.add_source(File::from(path));
        }

        // Add environment variable source if prefix specified
        if let Some(prefix) = self.env_prefix {
            builder = builder.add_source(
                Environment::with_prefix(&prefix)
                    .separator("__")
                    .try_parsing(true)
            );
        }

        // Build configuration
        let config = builder.build()
            .map_err(|e| Error::config(format!("Failed to build config: {}", e)))?;

        // Deserialize into our Config struct
        let mut config: Config = config.try_deserialize()
            .map_err(|e| Error::config(format!("Failed to deserialize config: {}", e)))?;

        // Apply test defaults if in test mode
        if self.test_mode {
            config = Self::apply_test_defaults(config);
        }

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Apply test default values
    fn apply_test_defaults(mut config: Config) -> Config {
        config.node.id = "test-node".to_string();
        config.node.role = NodeRole::Observer;
        config.network.listen_port = 0; // Random port
        config.network.max_peers = 10;
        config.consensus.min_validators = 1;
        config.consensus.consensus_threshold = 0.51;
        config
    }
}

impl Config {
    /// Create a new configuration builder
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    /// Validate the configuration
    fn validate(&self) -> Result<()> {
        // Validate node configuration
        if self.node.id.is_empty() {
            return Err(Error::validation("Node ID cannot be empty"));
        }

        // Validate network configuration
        if self.network.max_peers < 1 {
            return Err(Error::validation("Max peers must be greater than 0"));
        }

        if self.network.connection_timeout.as_secs() < 1 {
            return Err(Error::validation("Connection timeout must be at least 1 second"));
        }

        // Validate consensus configuration
        if self.consensus.min_validators < 1 {
            return Err(Error::validation("Minimum validators must be greater than 0"));
        }

        if self.consensus.consensus_threshold <= 0.0 || self.consensus.consensus_threshold > 1.0 {
            return Err(Error::validation("Consensus threshold must be between 0 and 1"));
        }

        // Validate storage configuration
        if self.storage.max_connections < 1 {
            return Err(Error::validation("Max connections must be greater than 0"));
        }

        if self.storage.database_url.is_empty() {
            return Err(Error::validation("Database URL cannot be empty"));
        }

        // Validate runtime configuration
        if self.runtime.max_tasks < 1 {
            return Err(Error::validation("Max tasks must be greater than 0"));
        }

        if self.runtime.thread_pool_size < 1 {
            return Err(Error::validation("Thread pool size must be greater than 0"));
        }

        Ok(())
    }

    /// Get the log level as a tracing Level
    pub fn get_tracing_level(&self) -> tracing::Level {
        match self.telemetry.log_level {
            LogLevel::Error => tracing::Level::ERROR,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Trace => tracing::Level::TRACE,
        }
    }

    /// Check if debug logging is enabled
    pub fn is_debug_enabled(&self) -> bool {
        self.telemetry.enable_debug ||
        matches!(self.telemetry.log_level, LogLevel::Debug | LogLevel::Trace)
    }

    /// Get the data directory as a PathBuf
    pub fn get_data_dir(&self) -> PathBuf {
        self.node.data_dir.clone()
    }

    /// Get the database connection string with SSL configuration
    pub fn get_database_url(&self) -> String {
        if self.storage.enable_ssl {
            format!("{}?sslmode=require", self.storage.database_url)
        } else {
            self.storage.database_url.clone()
        }
    }

    /// Get the complete node address (host:port)
    pub fn get_node_address(&self) -> String {
        format!("{}:{}", self.network.listen_addr, self.network.listen_port)
    }

    /// Check if the node is a validator
    pub fn is_validator(&self) -> bool {
        matches!(self.node.role, NodeRole::Validator)
    }
}

// Default implementations

impl Default for Config {
    fn default() -> Self {
        Self {
            node: NodeConfig::default(),
            network: NetworkConfig::default(),
            consensus: ConsensusConfig::default(),
            storage: StorageConfig::default(),
            runtime: RuntimeConfig::default(),
            telemetry: TelemetryConfig::default(),
        }
    }
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            id: String::from("default-node"),
            role: NodeRole::Observer,
            data_dir: PathBuf::from("/tmp/icn"),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: String::from("127.0.0.1"),
            listen_port: 8000,
            max_peers: 50,
            bootstrap_nodes: Vec::new(),
            connection_timeout: Duration::from_secs(5),
        }
    }
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            min_validators: 4,
            round_timeout: Duration::from_secs(30),
            consensus_threshold: 0.66,
            max_consecutive_rounds: 3,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database_url: String::from("postgres://localhost/icn"),
            max_connections: 10,
            connection_timeout: Duration::from_secs(10),
            enable_ssl: false,
        }
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_tasks: 100,
            stack_size: 2 * 1024 * 1024, // 2MB
            thread_pool_size: num_cpus::get(),
        }
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            metrics_endpoint: String::from("127.0.0.1:9100"),
            log_level: LogLevel::Info,
            enable_debug: false,
        }
    }
}

// String conversion implementations for config types

impl From<&str> for NodeRole {
    fn from(role: &str) -> Self {
        match role.to_lowercase().as_str() {
            "validator" => NodeRole::Validator,
            "observer" => NodeRole::Observer,
            _ => NodeRole::Light,
        }
    }
}

impl From<&str> for LogLevel {
    fn from(level: &str) -> Self {
        match level.to_lowercase().as_str() {
            "error" => LogLevel::Error,
            "warn" => LogLevel::Warn,
            "debug" => LogLevel::Debug,
            "trace" => LogLevel::Trace,
            _ => LogLevel::Info,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_config_builder() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Create test configuration file
        let config_content = r#"
            node:
              id: "test-node-1"
              role: "validator"
              data_dir: "/tmp/icn"
            network:
              listen_addr: "127.0.0.1"
              listen_port: 8000
              max_peers: 50
              bootstrap_nodes: []
              connection_timeout: "5s"
        "#;
        fs::write(&config_path, config_content).unwrap();

        let config = Config::builder()
            .with_config(config_path)
            .with_env_prefix("ICN")
            .build()
            .unwrap();

        assert_eq!(config.node.id, "test-node-1");
        assert_eq!(config.network.listen_port, 8000);
        assert_eq!(config.network.max_peers, 50);
    }

    #[test]
    fn test_config_validation() {
        let config = Config::builder()
            .with_test_defaults()
            .build()
            .unwrap();

        // Test with invalid values
        let mut invalid_config = config.clone();
        invalid_config.node.id = String::new();
        assert!(invalid_config.validate().is_err());

        invalid_config = config.clone();
        invalid_config.network.max_peers = 0;
        assert!(invalid_config.validate().is_err());

        invalid_config = config.clone();
        invalid_config.consensus.consensus_threshold = 1.5;
        assert!(invalid_config.validate().is_err());

        invalid_config = config.clone();
        invalid_config.storage.database_url = String::new();
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_env_override() {
        std::env::set_var("ICN__NODE__ID", "env-node");
        std::env::set_var("ICN__NETWORK__LISTEN_PORT", "9000");

        let config = Config::builder()
            .with_test_defaults()
            .with_env_prefix("ICN")
            .build()
            .unwrap();

        assert_eq!(config.node.id, "env-node");
        assert_eq!(config.network.listen_port, 9000);

        std::env::remove_var("ICN__NODE__ID");
        std::env::remove_var("ICN__NETWORK__LISTEN_PORT");
    }

    #[test]
    fn test_test_defaults() {
        let config = Config::builder()
            .with_test_defaults()
            .build()
            .unwrap();

        assert_eq!(config.node.role, NodeRole::Observer);
        assert_eq!(config.network.max_peers, 10);
        assert_eq!(config.consensus.min_validators, 1);
        assert!((config.consensus.consensus_threshold - 0.51).abs() < f64::EPSILON);
    }

    #[test]
    fn test_duration_parsing() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        let config_content = r#"
            node:
              id: "test-node"
              role: "validator"
              data_dir: "/tmp/icn"
            network:
              listen_addr: "127.0.0.1"
              listen_port: 8000
              max_peers: 50
              bootstrap_nodes: []
              connection_timeout: "5s"
            consensus:
              min_validators: 4
              round_timeout: "30s"
              consensus_threshold: 0.66
              max_consecutive_rounds: 3
            storage:
              database_url: "postgres://localhost/test"
              max_connections: 10
              connection_timeout: "10s"
              enable_ssl: false
            runtime:
              max_tasks: 100
              stack_size: 2097152
              thread_pool_size: 4
            telemetry:
              enable_metrics: true
              metrics_endpoint: "127.0.0.1:9100"
              log_level: "info"
              enable_debug: false
        "#;

        fs::write(&config_path, config_content).unwrap();

        let config = Config::builder()
            .with_config(config_path)
            .build()
            .unwrap();

        assert_eq!(config.network.connection_timeout, Duration::from_secs(5));
        assert_eq!(config.consensus.round_timeout, Duration::from_secs(30));
        assert_eq!(config.storage.connection_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_node_role_conversion() {
        assert_eq!(NodeRole::from("validator"), NodeRole::Validator);
        assert_eq!(NodeRole::from("observer"), NodeRole::Observer);
        assert_eq!(NodeRole::from("light"), NodeRole::Light);
        assert_eq!(NodeRole::from("unknown"), NodeRole::Light); // Default to light node
    }

    #[test]
    fn test_log_level_conversion() {
        assert_eq!(LogLevel::from("error"), LogLevel::Error);
        assert_eq!(LogLevel::from("warn"), LogLevel::Warn);
        assert_eq!(LogLevel::from("info"), LogLevel::Info);
        assert_eq!(LogLevel::from("debug"), LogLevel::Debug);
        assert_eq!(LogLevel::from("trace"), LogLevel::Trace);
        assert_eq!(LogLevel::from("unknown"), LogLevel::Info); // Default to info
    }

    #[test]
    fn test_config_utility_functions() {
        let config = Config::builder()
            .with_test_defaults()
            .build()
            .unwrap();

        // Test tracing level conversion
        assert_eq!(config.get_tracing_level(), tracing::Level::INFO);

        // Test debug mode checking
        assert!(!config.is_debug_enabled());
        let mut debug_config = config.clone();
        debug_config.telemetry.enable_debug = true;
        assert!(debug_config.is_debug_enabled());

        // Test database URL formatting
        assert_eq!(config.get_database_url(), config.storage.database_url);
        let mut ssl_config = config.clone();
        ssl_config.storage.enable_ssl = true;
        assert!(ssl_config.get_database_url().contains("?sslmode=require"));

        // Test node address formatting
        assert_eq!(
            config.get_node_address(),
            format!("{}:{}", config.network.listen_addr, config.network.listen_port)
        );

        // Test validator role checking
        assert!(!config.is_validator());
        let mut validator_config = config.clone();
        validator_config.node.role = NodeRole::Validator;
        assert!(validator_config.is_validator());
    }

    #[test]
    fn test_missing_required_fields() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Create config with missing required fields
        let config_content = r#"
            node:
              role: "validator"
            network:
              listen_addr: "127.0.0.1"
        "#;

        fs::write(&config_path, config_content).unwrap();

        let result = Config::builder()
            .with_config(config_path)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_durations() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");

        // Create config with invalid duration
        let config_content = r#"
            node:
              id: "test-node"
              role: "validator"
              data_dir: "/tmp/icn"
            network:
              listen_addr: "127.0.0.1"
              listen_port: 8000
              max_peers: 50
              connection_timeout: "invalid"
        "#;

        fs::write(&config_path, config_content).unwrap();

        let result = Config::builder()
            .with_config(config_path)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_config_clone() {
        let original = Config::builder()
            .with_test_defaults()
            .build()
            .unwrap();

        let cloned = original.clone();

        assert_eq!(original.node.id, cloned.node.id);
        assert_eq!(original.network.listen_port, cloned.network.listen_port);
        assert_eq!(original.consensus.consensus_threshold, cloned.consensus.consensus_threshold);
    }

    #[test]
    fn test_multiple_config_sources() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create default config
        let default_config = r#"
            node:
              id: "default-node"
              role: "observer"
            network:
              listen_port: 8000
        "#;
        fs::write(temp_dir.path().join("default.yaml"), default_config).unwrap();

        // Create environment-specific config
        let env_config = r#"
            node:
              id: "env-node"
            network:
              listen_port: 9000
        "#;
        fs::write(temp_dir.path().join("env.yaml"), env_config).unwrap();

        // Create local config
        let local_config = r#"
            node:
              id: "local-node"
        "#;
        fs::write(temp_dir.path().join("local.yaml"), local_config).unwrap();

        std::env::set_var("ICN_ENV", "env");
        
        let config = Config::builder()
            .with_config(temp_dir.path().join("local.yaml"))
            .build()
            .unwrap();

        // Local config should override env config which overrides default
        assert_eq!(config.node.id, "local-node");
        assert_eq!(config.network.listen_port, 9000);

        std::env::remove_var("ICN_ENV");
    }
}