// src/config.rs

use serde::{Serialize, Deserialize};
use std::time::Duration;

/// Configuration for the consensus system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Validator configuration
    pub validator: ValidatorConfig,

    /// Round configuration
    pub round: RoundConfig,

    /// Event system configuration
    pub events: EventConfig,

    /// Metrics configuration
    pub metrics: MetricsConfig,
}

/// Configuration for validator management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// Minimum number of validators required for consensus
    pub min_validators: usize,

    /// Minimum reputation required to be an active validator
    pub min_reputation: i64,

    /// Maximum number of validators allowed
    pub max_validators: usize,

    /// Time after which a validator is considered inactive
    #[serde(with = "duration_serde")]
    pub inactivity_timeout: Duration,

    /// Base voting power multiplier
    pub voting_power_multiplier: f64,
}

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

/// Configuration for the event system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventConfig {
    /// Maximum size of the event broadcast channel
    pub channel_size: usize,

    /// Whether to log events
    pub log_events: bool,
}

/// Configuration for metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Whether to enable metrics collection
    pub enabled: bool,

    /// Prefix for metric names
    pub prefix: String,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            validator: ValidatorConfig::default(),
            round: RoundConfig::default(),
            events: EventConfig::default(),
            metrics: MetricsConfig::default(),
        }
    }
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            min_validators: 4,
            min_reputation: 100,
            max_validators: 100,
            inactivity_timeout: Duration::from_secs(300), // 5 minutes
            voting_power_multiplier: 0.01,
        }
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

impl Default for EventConfig {
    fn default() -> Self {
        Self {
            channel_size: 1000,
            log_events: true,
        }
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

mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_default_config() {
        let config = ConsensusConfig::default();
        assert!(config.validator.min_validators >= 4);
        assert!(config