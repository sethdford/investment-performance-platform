use anyhow::{Result, anyhow};
use async_trait::async_trait;
use redis::{AsyncCommands, Client, aio::Connection};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{info, warn, error};
use crate::calculations::error_handling::{with_retry, RetryConfig};

/// Cache interface for storing and retrieving calculation results
#[async_trait]
pub trait Cache<K, V>: Send + Sync
where
    K: Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    /// Get a value from the cache
    async fn get(&self, key: &K) -> Result<Option<V>>;
    
    /// Set a value in the cache with a TTL
    async fn set(&self, key: K, value: V, ttl_seconds: u64) -> Result<()>;
    
    /// Delete a value from the cache
    async fn delete(&self, key: &K) -> Result<()>;
    
    /// Get a value from the cache, or compute it if not present
    async fn get_or_compute<F, Fut>(&self, key: K, ttl_seconds: u64, compute_fn: F) -> Result<V>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<V>> + Send;
}

/// Redis-based distributed cache
pub struct RedisCache {
    client: Client,
    connection_pool: Arc<Mutex<Vec<Connection>>>,
    max_connections: usize,
}

impl RedisCache {
    /// Create a new Redis cache
    pub async fn new(redis_url: &str, max_connections: usize) -> Result<Self> {
        let client = Client::open(redis_url)
            .map_err(|e| anyhow!("Failed to create Redis client: {}", e))?;
        
        // Initialize connection pool
        let mut connections = Vec::with_capacity(max_connections);
        for _ in 0..max_connections {
            let conn = client.get_async_connection().await
                .map_err(|e| anyhow!("Failed to connect to Redis: {}", e))?;
            connections.push(conn);
        }
        
        Ok(Self {
            client,
            connection_pool: Arc::new(Mutex::new(connections)),
            max_connections,
        })
    }
    
    /// Get a connection from the pool
    async fn get_connection(&self) -> Result<Connection> {
        let mut pool = self.connection_pool.lock().await;
        
        if let Some(conn) = pool.pop() {
            Ok(conn)
        } else {
            // If pool is empty, create a new connection
            self.client.get_async_connection().await
                .map_err(|e| anyhow!("Failed to get Redis connection: {}", e))
        }
    }
    
    /// Return a connection to the pool
    async fn return_connection(&self, conn: Connection) {
        let mut pool = self.connection_pool.lock().await;
        
        if pool.len() < self.max_connections {
            pool.push(conn);
        }
        // If pool is full, connection will be dropped
    }
    
    /// Generate a cache key for a value
    fn generate_key<K>(&self, key: &K) -> Result<String>
    where
        K: Serialize,
    {
        serde_json::to_string(key)
            .map_err(|e| anyhow!("Failed to serialize cache key: {}", e))
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for RedisCache
where
    K: Serialize + Send + Sync + 'static,
    V: Serialize + for<'de> Deserialize<'de> + Send + Sync + Clone + 'static,
{
    async fn get(&self, key: &K) -> Result<Option<V>> {
        let redis_key = self.generate_key(key)?;
        let mut conn = self.get_connection().await?;
        
        // Define retry config for Redis operations
        let retry_config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 50,
            backoff_factor: 2.0,
            max_delay_ms: 500,
        };
        
        // Define the operation to retry
        let operation = || {
            let mut conn_clone = conn.clone();
            let redis_key_clone = redis_key.clone();
            
            async move {
                let result: Option<String> = conn_clone.get(&redis_key_clone).await
                    .map_err(|e| anyhow!("Redis get error: {}", e))?;
                
                match result {
                    Some(data) => {
                        let value = serde_json::from_str(&data)
                            .map_err(|e| anyhow!("Failed to deserialize cache value: {}", e))?;
                        Ok(Some(value))
                    },
                    None => Ok(None),
                }
            }
        };
        
        // Execute with retry
        let result = with_retry(
            operation,
            retry_config,
            "redis_get",
            "cache",
        ).await?;
        
        // Return connection to pool
        self.return_connection(conn).await;
        
        Ok(result)
    }
    
    async fn set(&self, key: K, value: V, ttl_seconds: u64) -> Result<()> {
        let redis_key = self.generate_key(&key)?;
        let mut conn = self.get_connection().await?;
        
        // Serialize value
        let serialized = serde_json::to_string(&value)
            .map_err(|e| anyhow!("Failed to serialize cache value: {}", e))?;
        
        // Define retry config for Redis operations
        let retry_config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 50,
            backoff_factor: 2.0,
            max_delay_ms: 500,
        };
        
        // Define the operation to retry
        let operation = || {
            let mut conn_clone = conn.clone();
            let redis_key_clone = redis_key.clone();
            let serialized_clone = serialized.clone();
            let ttl = ttl_seconds;
            
            async move {
                // Set with expiration
                conn_clone.set_ex(&redis_key_clone, serialized_clone, ttl as usize).await
                    .map_err(|e| anyhow!("Redis set error: {}", e))?;
                
                Ok(())
            }
        };
        
        // Execute with retry
        let result = with_retry(
            operation,
            retry_config,
            "redis_set",
            "cache",
        ).await;
        
        // Return connection to pool
        self.return_connection(conn).await;
        
        result
    }
    
    async fn delete(&self, key: &K) -> Result<()> {
        let redis_key = self.generate_key(key)?;
        let mut conn = self.get_connection().await?;
        
        // Define retry config for Redis operations
        let retry_config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 50,
            backoff_factor: 2.0,
            max_delay_ms: 500,
        };
        
        // Define the operation to retry
        let operation = || {
            let mut conn_clone = conn.clone();
            let redis_key_clone = redis_key.clone();
            
            async move {
                conn_clone.del(&redis_key_clone).await
                    .map_err(|e| anyhow!("Redis delete error: {}", e))?;
                
                Ok(())
            }
        };
        
        // Execute with retry
        let result = with_retry(
            operation,
            retry_config,
            "redis_delete",
            "cache",
        ).await;
        
        // Return connection to pool
        self.return_connection(conn).await;
        
        result
    }
    
    async fn get_or_compute<F, Fut>(&self, key: K, ttl_seconds: u64, compute_fn: F) -> Result<V>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<V>> + Send,
    {
        // Try to get from cache
        if let Some(value) = self.get(&key).await? {
            return Ok(value);
        }
        
        // If not in cache, compute the value
        let value = compute_fn().await?;
        
        // Store in cache
        let value_clone = value.clone();
        let key_clone = key;
        
        // Spawn a task to set the cache asynchronously
        // This way we don't block the caller waiting for the cache to be updated
        tokio::spawn(async move {
            if let Err(e) = self.set(key_clone, value_clone, ttl_seconds).await {
                warn!("Failed to set cache value: {}", e);
            }
        });
        
        Ok(value)
    }
}

/// Cache factory for creating different types of caches
pub struct CacheFactory;

impl CacheFactory {
    /// Create a Redis cache
    pub async fn create_redis_cache(redis_url: &str, max_connections: usize) -> Result<Arc<dyn Cache<String, serde_json::Value> + Send + Sync>> {
        let cache = RedisCache::new(redis_url, max_connections).await?;
        Ok(Arc::new(cache))
    }
    
    /// Create a mock cache for testing
    #[cfg(test)]
    pub fn create_mock_cache() -> Arc<dyn Cache<String, serde_json::Value> + Send + Sync> {
        use std::collections::HashMap;
        use std::sync::Mutex as StdMutex;
        
        struct MockCache {
            data: StdMutex<HashMap<String, (serde_json::Value, std::time::Instant, u64)>>,
        }
        
        #[async_trait]
        impl Cache<String, serde_json::Value> for MockCache {
            async fn get(&self, key: &String) -> Result<Option<serde_json::Value>> {
                let data = self.data.lock().unwrap();
                
                if let Some((value, inserted_at, ttl)) = data.get(key) {
                    let elapsed = inserted_at.elapsed().as_secs();
                    
                    if elapsed < *ttl {
                        return Ok(Some(value.clone()));
                    }
                }
                
                Ok(None)
            }
            
            async fn set(&self, key: String, value: serde_json::Value, ttl_seconds: u64) -> Result<()> {
                let mut data = self.data.lock().unwrap();
                data.insert(key, (value, std::time::Instant::now(), ttl_seconds));
                Ok(())
            }
            
            async fn delete(&self, key: &String) -> Result<()> {
                let mut data = self.data.lock().unwrap();
                data.remove(key);
                Ok(())
            }
            
            async fn get_or_compute<F, Fut>(&self, key: String, ttl_seconds: u64, compute_fn: F) -> Result<serde_json::Value>
            where
                F: FnOnce() -> Fut + Send + 'static,
                Fut: std::future::Future<Output = Result<serde_json::Value>> + Send,
            {
                if let Some(value) = self.get(&key).await? {
                    return Ok(value);
                }
                
                let value = compute_fn().await?;
                self.set(key, value.clone(), ttl_seconds).await?;
                
                Ok(value)
            }
        }
        
        Arc::new(MockCache {
            data: StdMutex::new(HashMap::new()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[tokio::test]
    async fn test_mock_cache() {
        let cache = CacheFactory::create_mock_cache();
        
        // Test set and get
        let key = "test_key".to_string();
        let value = json!({ "data": "test_value" });
        
        cache.set(key.clone(), value.clone(), 60).await.unwrap();
        
        let result = cache.get(&key).await.unwrap();
        assert_eq!(result, Some(value.clone()));
        
        // Test delete
        cache.delete(&key).await.unwrap();
        
        let result = cache.get(&key).await.unwrap();
        assert_eq!(result, None);
        
        // Test get_or_compute
        let computed_value = cache.get_or_compute(
            key.clone(),
            60,
            || async { Ok(json!({ "computed": true })) }
        ).await.unwrap();
        
        assert_eq!(computed_value, json!({ "computed": true }));
        
        // Should get from cache now
        let cached_value = cache.get(&key).await.unwrap();
        assert_eq!(cached_value, Some(json!({ "computed": true })));
    }
    
    // Integration test with real Redis - only run when Redis is available
    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_integration() {
        let redis_url = "redis://localhost:6379";
        
        // Try to connect to Redis
        let client = redis::Client::open(redis_url);
        if client.is_err() {
            println!("Skipping Redis integration test - Redis not available");
            return;
        }
        
        let cache = CacheFactory::create_redis_cache(redis_url, 5).await.unwrap();
        
        // Test set and get
        let key = "test_key".to_string();
        let value = json!({ "data": "test_value" });
        
        cache.set(key.clone(), value.clone(), 60).await.unwrap();
        
        let result = cache.get(&key).await.unwrap();
        assert_eq!(result, Some(value.clone()));
        
        // Test delete
        cache.delete(&key).await.unwrap();
        
        let result = cache.get(&key).await.unwrap();
        assert_eq!(result, None);
        
        // Test get_or_compute
        let computed_value = cache.get_or_compute(
            key.clone(),
            60,
            || async { Ok(json!({ "computed": true })) }
        ).await.unwrap();
        
        assert_eq!(computed_value, json!({ "computed": true }));
        
        // Should get from cache now
        let cached_value = cache.get(&key).await.unwrap();
        assert_eq!(cached_value, Some(json!({ "computed": true })));
        
        // Clean up
        cache.delete(&key).await.unwrap();
    }
} 