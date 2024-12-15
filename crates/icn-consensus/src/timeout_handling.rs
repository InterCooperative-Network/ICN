use tokio::time::{sleep, Duration};

pub struct TimeoutHandling {
    timeout: Duration,
}

impl TimeoutHandling {
    pub fn new(timeout: Duration) -> Self {
        TimeoutHandling { timeout }
    }

    pub async fn handle_timeout(&self) {
        sleep(self.timeout).await;
        // Add logic to handle timeout here
    }
}
