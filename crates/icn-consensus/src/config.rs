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

impl ConsensusConfig {
    /// Validate the consensus configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.validator.validate()?;
        self.round.validate()?;
        self.events.validate()?;
        self.metrics.validate()?;
        Ok(())
    }
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

impl ValidatorConfig {
    /// Validate the validator configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.min_validators == 0 {
            return Err(ConfigError::InvalidValue("min_validators must be greater than 0".into()));
        }
        if self.min_reputation < 0 {
            return Err(ConfigError::InvalidValue("min_reputation cannot be negative".into()));
        }
        if self.max_validators < self.min_validators {
            return Err(ConfigError::InvalidValue("max_validators cannot be less than min_validators".into()));
        }
        if self.voting_power_multiplier <= 0.0 {
            return Err(ConfigError::InvalidValue("voting_power_multiplier must be greater than 0".into()));
        }
        Ok(())
    }
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

/// Configuration for the event system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventConfig {
    /// Maximum size of the event broadcast channel
    pub channel_size: usize,

    /// Whether to log events
    pub log_events: bool,
}

impl EventConfig {
    /// Validate the event configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.channel_size == 0 {
            return Err(ConfigError::InvalidValue("channel_size must be greater than 0".into()));
        }
        Ok(())
    }
}

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

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_default_config() {
        let config = ConsensusConfig::default();
        assert!(config.validator.min_validators >= 4);
        assert!(config.validator.validate().is_ok());
        assert!(config.round.validate().is_ok());
        assert!(config.events.validate().is_ok());
        assert!(config.metrics.validate().is_ok());
    }

    #[test]
    fn test_invalid_validator_config() {
        let mut config = ValidatorConfig::default();
        config.min_validators = 0;
        assert!(config.validate().is_err());

        config.min_validators = 4;
        config.min_reputation = -1;
        assert!(config.validate().is_err());

        config.min_reputation = 100;
        config.max_validators = 3;
        assert!(config.validate().is_err());

        config.max_validators = 100;
        config.voting_power_multiplier = 0.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_round_config() {
        let mut config = RoundConfig::default();
        config.round_timeout = Duration::from_secs(0);
        assert!(config.validate().is_err());

        config.round_timeout = Duration::from_secs(30);
        config.consensus_threshold = 0.0;
        assert!(config.validate().is_err());

        config.consensus_threshold = 0.66;
        config.max_timestamp_diff = Duration::from_secs(0);
        assert!(config.validate().is_err());

        config.max_timestamp_diff = Duration::from_secs(60);
        config.max_transactions_per_block = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_event_config() {
        let mut config = EventConfig::default();
        config.channel_size = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_metrics_config() {
        let mut config = MetricsConfig::default();
        config.prefix = String::new();
        assert!(config.validate().is_err());
    }
}
