use redis::{Client as RedisClient, Commands};
use tokio::sync::broadcast;

pub struct DistributedWebSocketManager {
    redis: RedisClient,
    event_tx: broadcast::Sender<WebSocketEvent>,
}

impl DistributedWebSocketManager {
    pub fn new(redis_url: &str) -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            redis: RedisClient::open(redis_url).unwrap(),
            event_tx: tx,
        }
    }

    pub async fn broadcast_message(&self, message: WebSocketEvent) {
        // Local broadcast
        let _ = self.event_tx.send(message.clone());
        
        // Redis pub/sub broadcast
        let _ = self.redis.publish("ws_events", serde_json::to_string(&message).unwrap());
    }
}
