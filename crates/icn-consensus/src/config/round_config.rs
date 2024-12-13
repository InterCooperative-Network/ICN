use serde::{Serialize, Deserialize};
use std::time::Duration;

/// Configuration for consensus rounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundConfig {
    /// Timeout duration for consensus rounds
    #[serde(with = "duration_serde")]
    pub round_timeout: Duration,

    /// Required threshold of weighted votes to reach consensus (0.0-1.0)
    pub consensus_threshold: f64,

    /// Maximum time difference allowed for block timestamps
    #[serde(with = "duration_serde")]
    pub max_timestamp_diff: Duration,

    /// Maximum number of transactions per block
    pub max_transactions_per_block: usize,
}

impl RoundConfig {
    /// Validate the round configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.round_timeout.as_secs() == 0 {
            return Err(ConfigError::InvalidValue("round_timeout must be greater than 0".into()));
        }
        if self.consensus_threshold <= 0.0 || self.consensus_threshold > 1.0 {
            return Err(ConfigError::InvalidValue("consensus_threshold must be between 0 and 1".into()));
        }
        if self.max_timestamp_diff.as_secs() == 0 {
            return Err(ConfigError::InvalidValue("max_timestamp_diff must be greater than 0".into()));
        }
        if self.max_transactions_per_block == 0 {
            return Err(ConfigError::InvalidValue("max_transactions_per_block must be greater than 0".into()));
        }
        Ok(())
    }
}

impl Default for RoundConfig {
    fn default() -> Self {
        Self {
            round_timeout: Duration::from_secs(30),
            consensus_threshold: 0.66,
            max_timestamp_diff: Duration::from_secs(60),
            max_transactions_per_block: 1000,
        }
    }
}
