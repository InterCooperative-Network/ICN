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
