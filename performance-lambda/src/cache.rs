use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use aws_sdk_dynamodb::model::AttributeValue;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn};

/// Cache entry with expiration time
#[derive(Clone, Debug)]
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// Cache key for DynamoDB items
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ItemCacheKey {
    table_name: String,
    partition_key: String,
    sort_key: Option<String>,
}

/// Cache key for DynamoDB queries
#[derive(Clone, Debug, PartialEq, Eq)]
struct QueryCacheKey {
    table_name: String,
    index_name: Option<String>,
    key_condition_expression: String,
    filter_expression: Option<String>,
    expression_attribute_values: HashMap<String, AttributeValue>,
}

impl Hash for QueryCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.table_name.hash(state);
        self.index_name.hash(state);
        self.key_condition_expression.hash(state);
        self.filter_expression.hash(state);
        
        // Sort keys to ensure consistent hashing
        let mut keys: Vec<_> = self.expression_attribute_values.keys().collect();
        keys.sort();
        
        for key in keys {
            key.hash(state);
            // Hash the string representation of the attribute value
            format!("{:?}", self.expression_attribute_values.get(key)).hash(state);
        }
    }
}

/// Cache metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub insertions: u64,
    pub size: usize,
}

impl CacheMetrics {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub ttl: Duration,
    pub max_size: usize,
    pub enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs(60), // 1 minute default TTL
            max_size: 1000,               // Default max size
            enabled: true,                // Enabled by default
        }
    }
}

/// Generic cache implementation
#[derive(Debug)]
pub struct Cache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    entries: RwLock<HashMap<K, CacheEntry<V>>>,
    config: CacheConfig,
    metrics: RwLock<CacheMetrics>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            config,
            metrics: RwLock::new(CacheMetrics::default()),
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        if !self.config.enabled {
            return None;
        }

        let entries = self.entries.read().await;
        match entries.get(key) {
            Some(entry) if !entry.is_expired() => {
                // Cache hit
                let mut metrics = self.metrics.write().await;
                metrics.hits += 1;
                Some(entry.value.clone())
            }
            _ => {
                // Cache miss
                let mut metrics = self.metrics.write().await;
                metrics.misses += 1;
                None
            }
        }
    }

    pub async fn put(&self, key: K, value: V) {
        if !self.config.enabled {
            return;
        }

        let mut entries = self.entries.write().await;
        
        // Check if we need to evict entries
        if entries.len() >= self.config.max_size {
            self.evict_expired_entries(&mut entries).await;
            
            // If still at capacity, evict oldest entry
            if entries.len() >= self.config.max_size {
                if let Some(oldest_key) = self.find_oldest_entry(&entries) {
                    entries.remove(&oldest_key);
                    
                    let mut metrics = self.metrics.write().await;
                    metrics.evictions += 1;
                }
            }
        }
        
        // Insert new entry
        entries.insert(key, CacheEntry::new(value, self.config.ttl));
        
        let mut metrics = self.metrics.write().await;
        metrics.insertions += 1;
        metrics.size = entries.len();
    }

    pub async fn invalidate(&self, key: &K) {
        if !self.config.enabled {
            return;
        }

        let mut entries = self.entries.write().await;
        entries.remove(key);
        
        let mut metrics = self.metrics.write().await;
        metrics.size = entries.len();
    }

    pub async fn clear(&self) {
        if !self.config.enabled {
            return;
        }

        let mut entries = self.entries.write().await;
        entries.clear();
        
        let mut metrics = self.metrics.write().await;
        metrics.size = 0;
        metrics.evictions += 1; // Count as a bulk eviction
    }

    pub async fn get_metrics(&self) -> CacheMetrics {
        self.metrics.read().await.clone()
    }

    async fn evict_expired_entries(&self, entries: &mut HashMap<K, CacheEntry<V>>) {
        let expired_keys: Vec<K> = entries
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();
        
        let eviction_count = expired_keys.len();
        
        for key in expired_keys {
            entries.remove(&key);
        }
        
        if eviction_count > 0 {
            let mut metrics = self.metrics.write().await;
            metrics.evictions += eviction_count as u64;
        }
    }

    fn find_oldest_entry(&self, entries: &HashMap<K, CacheEntry<V>>) -> Option<K> {
        entries
            .iter()
            .min_by_key(|(_, entry)| entry.expires_at)
            .map(|(key, _)| key.clone())
    }
}

/// DynamoDB item cache
pub type ItemCache = Cache<ItemCacheKey, HashMap<String, AttributeValue>>;

/// DynamoDB query result cache
pub type QueryCache = Cache<QueryCacheKey, Vec<HashMap<String, AttributeValue>>>;

/// Cache manager for DynamoDB operations
#[derive(Debug)]
pub struct CacheManager {
    item_cache: Arc<ItemCache>,
    query_cache: Arc<QueryCache>,
    config: CacheConfig,
}

impl CacheManager {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            item_cache: Arc::new(ItemCache::new(config.clone())),
            query_cache: Arc::new(QueryCache::new(config.clone())),
            config,
        }
    }

    pub fn item_cache(&self) -> Arc<ItemCache> {
        self.item_cache.clone()
    }

    pub fn query_cache(&self) -> Arc<QueryCache> {
        self.query_cache.clone()
    }

    pub async fn get_item(
        &self,
        table_name: &str,
        partition_key: &str,
        sort_key: Option<&str>,
    ) -> Option<HashMap<String, AttributeValue>> {
        let key = ItemCacheKey {
            table_name: table_name.to_string(),
            partition_key: partition_key.to_string(),
            sort_key: sort_key.map(|s| s.to_string()),
        };
        
        self.item_cache.get(&key).await
    }

    pub async fn put_item(
        &self,
        table_name: &str,
        partition_key: &str,
        sort_key: Option<&str>,
        item: HashMap<String, AttributeValue>,
    ) {
        let key = ItemCacheKey {
            table_name: table_name.to_string(),
            partition_key: partition_key.to_string(),
            sort_key: sort_key.map(|s| s.to_string()),
        };
        
        self.item_cache.put(key, item).await;
    }

    pub async fn invalidate_item(
        &self,
        table_name: &str,
        partition_key: &str,
        sort_key: Option<&str>,
    ) {
        let key = ItemCacheKey {
            table_name: table_name.to_string(),
            partition_key: partition_key.to_string(),
            sort_key: sort_key.map(|s| s.to_string()),
        };
        
        self.item_cache.invalidate(&key).await;
    }

    pub async fn get_query_result(
        &self,
        table_name: &str,
        index_name: Option<&str>,
        key_condition_expression: &str,
        filter_expression: Option<&str>,
        expression_attribute_values: HashMap<String, AttributeValue>,
    ) -> Option<Vec<HashMap<String, AttributeValue>>> {
        let key = QueryCacheKey {
            table_name: table_name.to_string(),
            index_name: index_name.map(|s| s.to_string()),
            key_condition_expression: key_condition_expression.to_string(),
            filter_expression: filter_expression.map(|s| s.to_string()),
            expression_attribute_values,
        };
        
        self.query_cache.get(&key).await
    }

    pub async fn put_query_result(
        &self,
        table_name: &str,
        index_name: Option<&str>,
        key_condition_expression: &str,
        filter_expression: Option<&str>,
        expression_attribute_values: HashMap<String, AttributeValue>,
        items: Vec<HashMap<String, AttributeValue>>,
    ) {
        let key = QueryCacheKey {
            table_name: table_name.to_string(),
            index_name: index_name.map(|s| s.to_string()),
            key_condition_expression: key_condition_expression.to_string(),
            filter_expression: filter_expression.map(|s| s.to_string()),
            expression_attribute_values,
        };
        
        self.query_cache.put(key, items).await;
    }

    pub async fn invalidate_query(
        &self,
        table_name: &str,
        index_name: Option<&str>,
        key_condition_expression: &str,
        filter_expression: Option<&str>,
        expression_attribute_values: HashMap<String, AttributeValue>,
    ) {
        let key = QueryCacheKey {
            table_name: table_name.to_string(),
            index_name: index_name.map(|s| s.to_string()),
            key_condition_expression: key_condition_expression.to_string(),
            filter_expression: filter_expression.map(|s| s.to_string()),
            expression_attribute_values,
        };
        
        self.query_cache.invalidate(&key).await;
    }

    pub async fn clear_all(&self) {
        self.item_cache.clear().await;
        self.query_cache.clear().await;
        
        info!("Cleared all caches");
    }

    pub async fn get_metrics(&self) -> (CacheMetrics, CacheMetrics) {
        let item_metrics = self.item_cache.get_metrics().await;
        let query_metrics = self.query_cache.get_metrics().await;
        
        (item_metrics, query_metrics)
    }

    pub async fn log_metrics(&self) {
        let (item_metrics, query_metrics) = self.get_metrics().await;
        
        info!(
            "Item cache metrics: hits={}, misses={}, hit_rate={:.2}%, size={}, evictions={}",
            item_metrics.hits,
            item_metrics.misses,
            item_metrics.hit_rate() * 100.0,
            item_metrics.size,
            item_metrics.evictions
        );
        
        info!(
            "Query cache metrics: hits={}, misses={}, hit_rate={:.2}%, size={}, evictions={}",
            query_metrics.hits,
            query_metrics.misses,
            query_metrics.hit_rate() * 100.0,
            query_metrics.size,
            query_metrics.evictions
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[tokio::test]
    async fn test_cache_get_put() {
        let config = CacheConfig {
            ttl: Duration::from_secs(1),
            max_size: 10,
            enabled: true,
        };
        
        let cache = Cache::<String, String>::new(config);
        
        // Put and get
        cache.put("key1".to_string(), "value1".to_string()).await;
        let value = cache.get(&"key1".to_string()).await;
        assert_eq!(value, Some("value1".to_string()));
        
        // Get non-existent key
        let value = cache.get(&"key2".to_string()).await;
        assert_eq!(value, None);
        
        // Test expiration
        cache.put("key3".to_string(), "value3".to_string()).await;
        thread::sleep(Duration::from_secs(2));
        let value = cache.get(&"key3".to_string()).await;
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_cache_invalidate() {
        let config = CacheConfig::default();
        let cache = Cache::<String, String>::new(config);
        
        cache.put("key1".to_string(), "value1".to_string()).await;
        cache.invalidate(&"key1".to_string()).await;
        
        let value = cache.get(&"key1".to_string()).await;
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let config = CacheConfig::default();
        let cache = Cache::<String, String>::new(config);
        
        cache.put("key1".to_string(), "value1".to_string()).await;
        cache.put("key2".to_string(), "value2".to_string()).await;
        
        cache.clear().await;
        
        let value1 = cache.get(&"key1".to_string()).await;
        let value2 = cache.get(&"key2".to_string()).await;
        
        assert_eq!(value1, None);
        assert_eq!(value2, None);
    }

    #[tokio::test]
    async fn test_cache_metrics() {
        let config = CacheConfig::default();
        let cache = Cache::<String, String>::new(config);
        
        // Initial metrics
        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.hits, 0);
        assert_eq!(metrics.misses, 0);
        
        // Miss
        cache.get(&"key1".to_string()).await;
        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.hits, 0);
        assert_eq!(metrics.misses, 1);
        
        // Put and hit
        cache.put("key1".to_string(), "value1".to_string()).await;
        cache.get(&"key1".to_string()).await;
        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.misses, 1);
        assert_eq!(metrics.insertions, 1);
    }

    #[tokio::test]
    async fn test_cache_eviction() {
        let config = CacheConfig {
            ttl: Duration::from_secs(60),
            max_size: 2,
            enabled: true,
        };
        
        let cache = Cache::<String, String>::new(config);
        
        // Fill the cache
        cache.put("key1".to_string(), "value1".to_string()).await;
        cache.put("key2".to_string(), "value2".to_string()).await;
        
        // This should evict the oldest entry
        cache.put("key3".to_string(), "value3".to_string()).await;
        
        // key1 should be evicted
        let value1 = cache.get(&"key1".to_string()).await;
        let value2 = cache.get(&"key2".to_string()).await;
        let value3 = cache.get(&"key3".to_string()).await;
        
        assert_eq!(value1, None);
        assert_eq!(value2, Some("value2".to_string()));
        assert_eq!(value3, Some("value3".to_string()));
        
        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.evictions, 1);
    }

    #[tokio::test]
    async fn test_cache_manager() {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config);
        
        // Test item cache
        let mut item = HashMap::new();
        item.insert(
            "id".to_string(),
            AttributeValue::S("123".to_string()),
        );
        
        manager.put_item("table", "123", None, item.clone()).await;
        let cached_item = manager.get_item("table", "123", None).await;
        
        assert_eq!(cached_item, Some(item));
        
        // Test query cache
        let mut items = Vec::new();
        items.push(item.clone());
        
        let mut expr_attr_values = HashMap::new();
        expr_attr_values.insert(
            ":id".to_string(),
            AttributeValue::S("123".to_string()),
        );
        
        manager.put_query_result(
            "table",
            None,
            "id = :id",
            None,
            expr_attr_values.clone(),
            items.clone(),
        ).await;
        
        let cached_items = manager.get_query_result(
            "table",
            None,
            "id = :id",
            None,
            expr_attr_values,
        ).await;
        
        assert_eq!(cached_items, Some(items));
    }
} 