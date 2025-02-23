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

pub struct WebSocketEvent {
    pub event_type: String,
    pub payload: String,
}

impl WebSocketEvent {
    pub fn new(event_type: &str, payload: &str) -> Self {
        Self {
            event_type: event_type.to_string(),
            payload: payload.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn test_broadcast_message() {
        let manager = DistributedWebSocketManager::new("redis://127.0.0.1/");
        let (tx, mut rx) = broadcast::channel(1000);

        let event = WebSocketEvent::new("test_event", "test_payload");
        manager.broadcast_message(event.clone()).await;

        let received_event = rx.recv().await.unwrap();
        assert_eq!(received_event.event_type, event.event_type);
        assert_eq!(received_event.payload, event.payload);
    }
}
