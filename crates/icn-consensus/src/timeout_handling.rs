use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;
use log::{debug, error};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TimeoutError {
    #[error("Timeout waiting for consensus")]
    ConsensusTimeout,
}

pub struct TimeoutHandling {
    timeout_duration: Duration,
    last_activity: Instant,
    timeout_sender: Option<mpsc::Sender<TimeoutEvent>>,
}

#[derive(Debug)]
pub enum TimeoutEvent {
    RoundTimeout,
    ProposalTimeout,
    VoteTimeout,
    NetworkTimeout,
    ConsensusTimeout,
}

impl TimeoutHandling {
    pub fn new(timeout_duration: Duration) -> Self {
        Self {
            timeout_duration,
            last_activity: Instant::now(),
            timeout_sender: None,
        }
    }

    pub fn start_timeout(&mut self) -> impl std::future::Future<Output = ()> {
        let duration = self.timeout_duration;
        let (sender, mut receiver) = mpsc::channel(32);
        self.timeout_sender = Some(sender);

        async move {
            tokio::time::sleep(duration).await;
            if let Some(event) = receiver.recv().await {
                debug!("Timeout event received: {:?}", event);
            }
        }
    }

    pub async fn handle_timeout(&self) -> Result<(), String> {
        if let Some(sender) = &self.timeout_sender {
            if let Err(e) = sender.send(TimeoutEvent::RoundTimeout).await {
                error!("Failed to send timeout event: {}", e);
                return Err(format!("Failed to handle timeout: {}", e));
            }
        }
        Ok(())
    }

    pub fn reset_timeout(&mut self) {
        self.last_activity = Instant::now();
    }

    pub fn is_timed_out(&self) -> bool {
        self.last_activity.elapsed() > self.timeout_duration
    }

    pub fn get_remaining_time(&self) -> Duration {
        let elapsed = self.last_activity.elapsed();
        if elapsed >= self.timeout_duration {
            Duration::from_secs(0)
        } else {
            self.timeout_duration - elapsed
        }
    }

    pub fn update_timeout_duration(&mut self, new_duration: Duration) {
        self.timeout_duration = new_duration;
        self.reset_timeout();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timeout_handling() {
        let mut handler = TimeoutHandling::new(Duration::from_millis(100));
        assert!(!handler.is_timed_out());

        // Sleep longer than timeout duration
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(handler.is_timed_out());

        // Reset and check again
        handler.reset_timeout();
        assert!(!handler.is_timed_out());
    }

    #[tokio::test]
    async fn test_timeout_event() {
        let mut handler = TimeoutHandling::new(Duration::from_millis(50));
        let timeout_future = handler.start_timeout();

        // Handle timeout
        tokio::time::sleep(Duration::from_millis(75)).await;
        assert!(handler.handle_timeout().await.is_ok());

        // Wait for timeout future to complete
        timeout_future.await;
    }

    #[tokio::test]
    async fn test_remaining_time() {
        let mut handler = TimeoutHandling::new(Duration::from_secs(1));
        tokio::time::sleep(Duration::from_millis(500)).await;

        let remaining = handler.get_remaining_time();
        assert!(remaining <= Duration::from_millis(500));
        assert!(remaining > Duration::from_millis(0));
    }
}
