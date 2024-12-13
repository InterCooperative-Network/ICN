use serde::{Serialize, Deserialize};

/// Configuration for metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Whether to enable metrics collection
    pub enabled: bool,

    /// Prefix for metric names
    pub prefix: String,
}

impl MetricsConfig {
    /// Validate the metrics configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.prefix.is_empty() {
            return Err(ConfigError::InvalidValue("prefix cannot be empty".into()));
        }
        Ok(())
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prefix: "icn_consensus".into(),
        }
    }
}
