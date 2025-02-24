use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Entry in the storage cache
struct CacheEntry {
    data: Vec<u8>,
    expires_at: Instant,
}

/// Cache for the storage system
pub struct StorageCache {
    entries: Mutex<HashMap<String, CacheEntry>>,
    max_size: usize,
    ttl: Duration,
}

impl StorageCache {
    /// Create a new cache with the specified size limit and TTL
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            entries: Mutex::new(HashMap::with_capacity(max_size)),
            max_size,
            ttl,
        }
    }

    /// Store a value in the cache
    pub fn set(&self, key: String, value: Vec<u8>) {
        let mut entries = self.entries.lock().unwrap();
        
        // Clean expired entries if we're at capacity
        if entries.len() >= self.max_size {
            self.remove_expired(&mut entries);
        }
        
        // If still at capacity, remove oldest entry
        if entries.len() >= self.max_size {
            if let Some(oldest_key) = entries.keys()
                .min_by_key(|k| entries.get(*k).map(|e| e.expires_at))
                .cloned() {
                entries.remove(&oldest_key);
            }
        }
        
        entries.insert(key, CacheEntry {
            data: value,
            expires_at: Instant::now() + self.ttl,
        });
    }

    /// Retrieve a value from the cache if it exists and hasn't expired
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut entries = self.entries.lock().unwrap();
        
        match entries.get(key) {
            Some(entry) if entry.expires_at > Instant::now() => {
                Some(entry.data.clone())
            }
            _ => {
                // Remove expired entry if it exists
                entries.remove(key);
                None
            }
        }
    }

    /// Remove expired entries from the cache
    fn remove_expired(&self, entries: &mut HashMap<String, CacheEntry>) {
        let now = Instant::now();
        entries.retain(|_, entry| entry.expires_at > now);
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        let mut entries = self.entries.lock().unwrap();
        entries.clear();
    }

    /// Get the current number of entries in the cache
    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.lock().unwrap().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_basic_operations() {
        let cache = StorageCache::new(10, Duration::from_secs(1));
        
        // Test set and get
        cache.set("key1".to_string(), vec![1, 2, 3]);
        assert_eq!(cache.get("key1"), Some(vec![1, 2, 3]));
        
        // Test expiration
        thread::sleep(Duration::from_secs(2));
        assert_eq!(cache.get("key1"), None);
    }

    #[test]
    fn test_cache_capacity() {
        let cache = StorageCache::new(2, Duration::from_secs(10));
        
        // Fill cache
        cache.set("key1".to_string(), vec![1]);
        cache.set("key2".to_string(), vec![2]);
        
        // Add one more item
        cache.set("key3".to_string(), vec![3]);
        
        // Verify oldest item was removed
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get("key1"), None);
        assert_eq!(cache.get("key2"), Some(vec![2]));
        assert_eq!(cache.get("key3"), Some(vec![3]));
    }

    #[test]
    fn test_cache_clear() {
        let cache = StorageCache::new(10, Duration::from_secs(10));
        
        cache.set("key1".to_string(), vec![1]);
        cache.set("key2".to_string(), vec![2]);
        
        assert_eq!(cache.len(), 2);
        
        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.get("key1"), None);
        assert_eq!(cache.get("key2"), None);
    }
}
