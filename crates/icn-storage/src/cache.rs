use dashmap::DashMap;
use std::time::{Duration, Instant};
use tokio::time;

pub struct CacheEntry {
    data: Vec<u8>,
    created_at: Instant,
    ttl: Duration,
}

pub struct StorageCache {
    cache: DashMap<String, CacheEntry>,
    max_size: usize,
    default_ttl: Duration,
}

impl StorageCache {
    pub fn new(max_size: usize, default_ttl: Duration) -> Self {
        let cache = Self {
            cache: DashMap::new(),
            max_size,
            default_ttl,
        };
        
        // Start cleanup task
        tokio::spawn(cache.cleanup_task());
        
        cache
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.cache.get(key).and_then(|entry| {
            if entry.created_at.elapsed() > entry.ttl {
                self.cache.remove(key);
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }

    pub fn set(&self, key: String, value: Vec<u8>) {
        if self.cache.len() >= self.max_size {
            // Remove oldest entry if cache is full
            if let Some((key, _)) = self.cache.iter().next() {
                self.cache.remove(&key);
            }
        }

        self.cache.insert(key, CacheEntry {
            data: value,
            created_at: Instant::now(),
            ttl: self.default_ttl,
        });
    }

    async fn cleanup_task(self) {
        let mut interval = time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            self.remove_expired_entries();
        }
    }

    fn remove_expired_entries(&self) {
        self.cache.retain(|_, entry| {
            entry.created_at.elapsed() <= entry.ttl
        });
    }
}
