use serde::{Serialize, Deserialize};

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

impl Default for EventConfig {
    fn default() -> Self {
        Self {
            channel_size: 1000,
            log_events: true,
        }
    }
}
