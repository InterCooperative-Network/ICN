use serde::{Serialize, Deserialize};
use std::time::Duration;

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
