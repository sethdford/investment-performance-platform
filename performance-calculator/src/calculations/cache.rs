use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Serialize, de::DeserializeOwned};

/// A simple in-memory cache with TTL support
pub struct CalculationCache<K, V> 
where 
    K: Eq + Hash + Clone,
    V: Clone,
{
    data: Mutex<HashMap<K, (V, Instant)>>,
    ttl_seconds: u64,
}

impl<K, V> CalculationCache<K, V> 
where 
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Create a new cache with the specified TTL in seconds
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
            ttl_seconds,
        }
    }

    /// Get a value from the cache
    pub fn get(&self, key: &K) -> Option<V> {
        let now = Instant::now();
        let mut data = self.data.lock().unwrap();
        
        if let Some((value, timestamp)) = data.get(key) {
            // Check if the value has expired
            if now.duration_since(*timestamp) < Duration::from_secs(self.ttl_seconds) {
                return Some(value.clone());
            } else {
                // Remove expired value
                data.remove(key);
            }
        }
        
        None
    }

    /// Set a value in the cache
    pub fn set(&self, key: K, value: V) {
        let mut data = self.data.lock().unwrap();
        data.insert(key, (value, Instant::now()));
    }

    /// Remove a value from the cache
    pub fn remove(&self, key: &K) -> Option<V> {
        let mut data = self.data.lock().unwrap();
        data.remove(key).map(|(value, _)| value)
    }

    /// Clear the cache
    pub fn clear(&self) {
        let mut data = self.data.lock().unwrap();
        data.clear();
    }
}

/// Create a performance cache with default settings
pub fn create_performance_cache() -> CalculationCache<String, serde_json::Value> {
    CalculationCache::new(300) // 5 minutes TTL
} 