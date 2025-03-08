use std::time::Duration;
use tokio::time::sleep;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TimeoutError {
    #[error("Timeout waiting for consensus")]
    ConsensusTimeout,
}

pub struct TimeoutHandling {
    pub timeout: Duration,
    pub max_retries: usize,
    current_retries: usize,
    exponential_backoff: bool,
}

impl TimeoutHandling {
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            max_retries: 3,
            current_retries: 0,
            exponential_backoff: true,
        }
    }

    /// Creates a new TimeoutHandling instance with advanced options
    pub fn with_options(
        timeout: Duration, 
        max_retries: usize, 
        exponential_backoff: bool
    ) -> Self {
        Self {
            timeout,
            max_retries,
            current_retries: 0,
            exponential_backoff,
        }
    }

    /// Start timeout tracking, returns future that resolves on timeout
    pub async fn start_timeout(&self) -> Result<(), TimeoutError> {
        sleep(self.timeout).await;
        Err(TimeoutError::ConsensusTimeout)
    }

    /// Handle a timeout condition
    pub async fn handle_timeout(&self) -> Result<(), TimeoutError> {
        // Log the timeout
        println!("Timeout occurred in consensus round");

        if self.current_retries >= self.max_retries {
            return Err(TimeoutError::ConsensusTimeout);
        }

        Ok(())
    }

    /// Calculate next timeout duration with exponential backoff if enabled
    pub fn next_timeout(&mut self) -> Duration {
        self.current_retries += 1;
        
        if self.exponential_backoff {
            // Calculate exponential backoff: timeout * 2^retries
            let backoff_factor = 2u32.pow(self.current_retries as u32);
            self.timeout.mul_f32(backoff_factor as f32)
        } else {
            // Use constant timeout
            self.timeout
        }
    }

    /// Reset retry counter
    pub fn reset(&mut self) {
        self.current_retries = 0;
    }
}

/// A helper utility for handling consensus timeouts with different strategies
pub enum TimeoutStrategy {
    /// Simple fixed timeout
    Fixed(Duration),
    
    /// Timeout with exponential backoff
    ExponentialBackoff {
        initial: Duration,
        max_timeout: Duration,
        factor: f32,
    },
    
    /// Adaptive timeout based on network conditions
    Adaptive {
        min: Duration,
        max: Duration,
        current: Duration,
        network_latency: Duration,
    },
}

impl TimeoutStrategy {
    /// Calculate next timeout based on the strategy
    pub fn next_timeout(&mut self) -> Duration {
        match self {
            Self::Fixed(duration) => *duration,
            
            Self::ExponentialBackoff { initial, max_timeout, factor } => {
                let next = initial.mul_f32(*factor);
                *initial = if next < *max_timeout { next } else { *max_timeout };
                *initial
            },
            
            Self::Adaptive { min, max, current, network_latency } => {
                // Adjust timeout based on network latency
                let next = network_latency.mul_f32(2.0);
                *current = if next > *min {
                    if next < *max { next } else { *max }
                } else {
                    *min
                };
                *current
            }
        }
    }
    
    /// Reset the timeout strategy to its initial state
    pub fn reset(&mut self) {
        match self {
            Self::Fixed(_) => {}, // Nothing to reset
            
            Self::ExponentialBackoff { initial, .. } => {
                // Reset to initial value but keep other parameters
                *initial = Duration::from_secs(1); // Example initial value
            },
            
            Self::Adaptive { current, min, .. } => {
                // Reset to minimum value
                *current = *min;
            }
        }
    }
    
    /// Update network latency for adaptive strategy
    pub fn update_network_latency(&mut self, latency: Duration) {
        if let Self::Adaptive { network_latency, .. } = self {
            *network_latency = latency;
        }
    }
}
