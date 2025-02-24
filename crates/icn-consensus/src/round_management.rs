use std::time::Duration;
use tokio::time::sleep;

pub struct RoundManager {
    round_duration: Duration,
    current_round: u64,
}

impl RoundManager {
    pub fn new(round_duration: Duration) -> Self {
        Self {
            round_duration,
            current_round: 0,
        }
    }

    pub async fn wait_for_next_round(&mut self) {
        sleep(self.round_duration).await;
        self.current_round += 1;
    }

    pub fn get_current_round(&self) -> u64 {
        self.current_round
    }
}
