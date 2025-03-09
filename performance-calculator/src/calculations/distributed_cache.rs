use anyhow::{Result, anyhow};
use async_trait::async_trait;
use redis::{AsyncCommands, Client, aio::Connection};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{info, warn, error};
use crate::calculations::error_handling::{with_retry, RetryConfig};
use std::future::Future;
use std::marker::Send;
use std::collections::HashMap;
use uuid;
use serde_json::json;
use std::pin::Pin;
use crate::calculations::query_api::AsyncComputeCache;

/// Generic cache interface for storing and retrieving typed data
#[async_trait]
pub trait Cache<K = String, V = serde_json::Value>: Send + Sync 
where
    K: Send + Sync + serde::Serialize + 'static,
    V: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
{
    /// Get a value from the cache
    async fn get(&self, key: &K) -> Result<Option<V>>;
    
    /// Set a value in the cache
    /// If ttl_seconds is None, the value will not expire
    async fn set(&self, key: K, value: V, ttl_seconds: Option<u64>) -> Result<()>;
    
    /// Delete a value from the cache
    async fn delete(&self, key: &K) -> Result<()>;
}

/// Basic cache interface for storing and retrieving string data
/// This trait is object-safe and can be used with dynamic dispatch
#[async_trait]
pub trait StringCache: Send + Sync {
    /// Get a value from the cache
    async fn get_string(&self, key: &str) -> Result<Option<String>>;
    
    /// Set a value in the cache
    /// If ttl_seconds is None, the value will not expire
    async fn set_string(&self, key: String, value: String, ttl_seconds: Option<u64>) -> Result<()>;
    
    /// Delete a value from the cache
    async fn delete_string(&self, key: &str) -> Result<()>;
}

/// Basic cache interface for storing and retrieving binary data
/// This trait is object-safe and can be used with dynamic dispatch
#[async_trait]
pub trait BinaryCache: Send + Sync {
    /// Get a value from the cache
    async fn get_binary(&self, key: &str) -> Result<Option<Vec<u8>>>;
    
    /// Set a value in the cache
    /// If ttl_seconds is None, the value will not expire
    async fn set_binary(&self, key: String, value: Vec<u8>, ttl_seconds: Option<u64>) -> Result<()>;
    
    /// Delete a value from the cache
    async fn delete_binary(&self, key: &str) -> Result<()>;
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
        K: Serialize + ?Sized,
    {
        serde_json::to_string(key)
            .map_err(|e| anyhow!("Failed to serialize cache key: {}", e))
    }
}

#[async_trait]
impl StringCache for RedisCache {
    async fn get_string(&self, key: &str) -> Result<Option<String>> {
        let redis_key = self.generate_key(key)?;
        let mut conn = self.get_connection().await?;
        
        // Define retry configuration
        let retry_config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 50,
            backoff_factor: 2.0,
            max_delay_ms: 500,
        };
        
        // Define the operation to retry
        let client = self.client.clone();
        let operation = || {
            let redis_key_clone = redis_key.clone();
            let client = client.clone();
            
            async move {
                // Create a new connection for each retry attempt
                let mut conn = client.get_async_connection().await
                    .map_err(|e| anyhow!("Redis connection error: {}", e))?;
                
                let result: Option<String> = conn.get(&redis_key_clone).await
                    .map_err(|e| anyhow!("Redis get error: {}", e))?;
                
                match result {
                    Some(data) => {
                        Ok(Some(data))
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
    
    async fn set_string(&self, key: String, value: String, ttl_seconds: Option<u64>) -> Result<()> {
        let redis_key = self.generate_key(&key)?;
        let mut conn = self.get_connection().await?;
        
        // Define retry configuration
        let retry_config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 50,
            backoff_factor: 2.0,
            max_delay_ms: 500,
        };
        
        // Define the operation to retry
        let client = self.client.clone();
        let operation = || {
            let redis_key_clone = redis_key.clone();
            let value_clone = value.clone();
            let ttl_seconds = ttl_seconds;
            let client = client.clone();
            
            async move {
                // Create a new connection for each retry attempt
                let mut conn = client.get_async_connection().await
                    .map_err(|e| anyhow!("Redis connection error: {}", e))?;
                
                // Set the value in Redis
                conn.set(&redis_key_clone, value_clone).await
                    .map_err(|e| anyhow!("Redis set error: {}", e))?;
                
                // Set expiration if provided
                if let Some(ttl) = ttl_seconds {
                    conn.expire(&redis_key_clone, ttl as usize).await
                        .map_err(|e| anyhow!("Redis expire error: {}", e))?;
                }
                
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
    
    async fn delete_string(&self, key: &str) -> Result<()> {
        let redis_key = self.generate_key(key)?;
        let mut conn = self.get_connection().await?;
        
        // Define retry configuration
        let retry_config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 50,
            backoff_factor: 2.0,
            max_delay_ms: 500,
        };
        
        // Define the operation to retry
        let client = self.client.clone();
        let operation = || {
            let redis_key_clone = redis_key.clone();
            let client = client.clone();
            
            async move {
                // Create a new connection for each retry attempt
                let mut conn = client.get_async_connection().await
                    .map_err(|e| anyhow!("Redis connection error: {}", e))?;
                
                conn.del(&redis_key_clone).await
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
}

#[async_trait]
impl BinaryCache for RedisCache {
    async fn get_binary(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let redis_key = self.generate_key(key)?;
        let mut conn = self.get_connection().await?;
        
        // Define retry configuration
        let retry_config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 50,
            backoff_factor: 2.0,
            max_delay_ms: 500,
        };
        
        // Define the operation to retry
        let client = self.client.clone();
        let operation = || {
            let redis_key_clone = redis_key.clone();
            let client = client.clone();
            
            async move {
                // Create a new connection for each retry attempt
                let mut conn = client.get_async_connection().await
                    .map_err(|e| anyhow!("Redis connection error: {}", e))?;
                
                let result: Option<Vec<u8>> = conn.get(&redis_key_clone).await
                    .map_err(|e| anyhow!("Redis get error: {}", e))?;
                
                match result {
                    Some(data) => {
                        Ok(Some(data))
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
    
    async fn set_binary(&self, key: String, value: Vec<u8>, ttl_seconds: Option<u64>) -> Result<()> {
        let redis_key = self.generate_key(&key)?;
        let mut conn = self.get_connection().await?;
        
        // Serialize value
        let serialized = serde_json::to_string(&value)
            .map_err(|e| anyhow!("Failed to serialize cache value: {}", e))?;
        
        // Define retry configuration
        let retry_config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 50,
            backoff_factor: 2.0,
            max_delay_ms: 500,
        };
        
        // Define the operation to retry
        let client = self.client.clone();
        let operation = || {
            let redis_key_clone = redis_key.clone();
            let serialized_clone = serialized.clone();
            let ttl_seconds = ttl_seconds;
            let client = client.clone();
            
            async move {
                // Create a new connection for each retry attempt
                let mut conn = client.get_async_connection().await
                    .map_err(|e| anyhow!("Redis connection error: {}", e))?;
                
                // Set the value in Redis
                conn.set(&redis_key_clone, serialized_clone).await
                    .map_err(|e| anyhow!("Redis set error: {}", e))?;
                
                // Set expiration if provided
                if let Some(ttl) = ttl_seconds {
                    conn.expire(&redis_key_clone, ttl as usize).await
                        .map_err(|e| anyhow!("Redis expire error: {}", e))?;
                }
                
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
    
    async fn delete_binary(&self, key: &str) -> Result<()> {
        // We can reuse the string delete implementation
        self.delete_string(key).await
    }
}

/// Wrapper type that implements AsyncComputeCache
pub struct AsyncCache<C>(C);

impl<C> AsyncCache<C> {
    pub fn new(cache: C) -> Self {
        Self(cache)
    }
}

#[async_trait]
impl<C, K, V> Cache<K, V> for AsyncCache<C>
where
    C: Cache<K, V> + Send + Sync,
    K: Send + Sync + serde::Serialize + 'static,
    V: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
{
    async fn get(&self, key: &K) -> Result<Option<V>> {
        self.0.get(key).await
    }
    
    async fn set(&self, key: K, value: V, ttl_seconds: Option<u64>) -> Result<()> {
        self.0.set(key, value, ttl_seconds).await
    }
    
    async fn delete(&self, key: &K) -> Result<()> {
        self.0.delete(key).await
    }
}

#[async_trait]
impl<C> StringCache for AsyncCache<C>
where
    C: StringCache + Send + Sync,
{
    async fn get_string(&self, key: &str) -> Result<Option<String>> {
        self.0.get_string(key).await
    }
    
    async fn set_string(&self, key: String, value: String, ttl_seconds: Option<u64>) -> Result<()> {
        self.0.set_string(key, value, ttl_seconds).await
    }
    
    async fn delete_string(&self, key: &str) -> Result<()> {
        self.0.delete_string(key).await
    }
}

#[async_trait]
impl<C> BinaryCache for AsyncCache<C>
where
    C: BinaryCache + Send + Sync,
{
    async fn get_binary(&self, key: &str) -> Result<Option<Vec<u8>>> {
        self.0.get_binary(key).await
    }
    
    async fn set_binary(&self, key: String, value: Vec<u8>, ttl_seconds: Option<u64>) -> Result<()> {
        self.0.set_binary(key, value, ttl_seconds).await
    }
    
    async fn delete_binary(&self, key: &str) -> Result<()> {
        self.0.delete_binary(key).await
    }
}

#[async_trait]
impl<C> AsyncComputeCache for AsyncCache<C>
where
    C: Cache<String, serde_json::Value> + Send + Sync,
{
    async fn compute_if_missing(
        &self,
        key: String,
        ttl_seconds: u64,
        compute: Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>> + Send + Sync>,
    ) -> Result<serde_json::Value> {
        // Check if value exists in cache
        if let Some(value) = self.0.get(&key).await? {
            return Ok(value);
        }
        
        // Compute new value
        let value = compute().await?;
        
        // Store in cache
        self.0.set(key, value.clone(), Some(ttl_seconds)).await?;
        
        Ok(value)
    }
}

/// In-memory implementation of the Cache trait for testing
pub struct InMemoryCache {
    string_data: Mutex<HashMap<String, (String, std::time::Instant, u64)>>,
    binary_data: Mutex<HashMap<String, (Vec<u8>, std::time::Instant, u64)>>,
}

impl InMemoryCache {
    /// Create a new in-memory cache
    pub fn new() -> Self {
        Self {
            string_data: Mutex::new(HashMap::new()),
            binary_data: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl StringCache for InMemoryCache {
    async fn get_string(&self, key: &str) -> Result<Option<String>> {
        let data = self.string_data.lock().await;
        if let Some((value, instant, ttl)) = data.get(key) {
            if instant.elapsed().as_secs() < *ttl {
                return Ok(Some(value.clone()));
            }
        }
        Ok(None)
    }
    
    async fn set_string(&self, key: String, value: String, ttl_seconds: Option<u64>) -> Result<()> {
        let mut data = self.string_data.lock().await;
        data.insert(key, (value, std::time::Instant::now(), ttl_seconds.unwrap_or(0)));
        Ok(())
    }
    
    async fn delete_string(&self, key: &str) -> Result<()> {
        let mut data = self.string_data.lock().await;
        data.remove(key);
        Ok(())
    }
}

#[async_trait]
impl BinaryCache for InMemoryCache {
    async fn get_binary(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let data = self.binary_data.lock().await;
        if let Some((value, instant, ttl)) = data.get(key) {
            if instant.elapsed().as_secs() < *ttl {
                return Ok(Some(value.clone()));
            }
        }
        Ok(None)
    }
    
    async fn set_binary(&self, key: String, value: Vec<u8>, ttl_seconds: Option<u64>) -> Result<()> {
        let mut data = self.binary_data.lock().await;
        data.insert(key, (value, std::time::Instant::now(), ttl_seconds.unwrap_or(0)));
        Ok(())
    }
    
    async fn delete_binary(&self, key: &str) -> Result<()> {
        let mut data = self.binary_data.lock().await;
        data.remove(key);
        Ok(())
    }
}

/// Factory for creating caches
pub struct CacheFactory;

impl CacheFactory {
    /// Create an in-memory cache for testing
    pub fn create_in_memory_cache() -> impl AsyncComputeCache + Send + Sync {
        AsyncCache::new(InMemoryCache::new())
    }
    
    /// Create a Redis cache
    pub async fn create_redis_cache(redis_url: &str, max_connections: usize) -> Result<impl AsyncComputeCache + Send + Sync> {
        Ok(AsyncCache::new(RedisCache::new(redis_url, max_connections).await?))
    }
}

/// Type-erased cache implementation that can be used with dynamic dispatch
pub struct TypeErasedCache<K, V>
where
    K: Send + Sync + serde::Serialize + 'static,
    V: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
{
    inner: Arc<dyn Cache<K, V> + Send + Sync>,
}

impl<K, V> TypeErasedCache<K, V>
where
    K: Send + Sync + serde::Serialize + 'static,
    V: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
{
    /// Create a new type-erased cache
    pub fn new<C: Cache<K, V> + Send + Sync + 'static>(cache: C) -> Self {
        Self {
            inner: Arc::new(cache),
        }
    }
    
    /// Get the inner cache implementation
    pub fn inner(&self) -> Arc<dyn Cache<K, V> + Send + Sync> {
        self.inner.clone()
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for TypeErasedCache<K, V>
where 
    K: Send + Sync + serde::Serialize + 'static,
    V: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static,
{
    async fn get(&self, key: &K) -> Result<Option<V>> {
        self.inner.get(key).await
    }
    
    async fn set(&self, key: K, value: V, ttl_seconds: Option<u64>) -> Result<()> {
        self.inner.set(key, value, ttl_seconds).await
    }
    
    async fn delete(&self, key: &K) -> Result<()> {
        self.inner.delete(key).await
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for RedisCache
where
    K: Send + Sync + serde::Serialize + std::fmt::Debug + 'static,
    V: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + Clone + 'static,
{
    async fn get(&self, key: &K) -> Result<Option<V>> {
        let redis_key = self.generate_key(key)?;
        let mut conn = self.get_connection().await?;
        
        // Define retry configuration
        let retry_config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 50,
            backoff_factor: 2.0,
            max_delay_ms: 500,
        };
        
        // Define the operation to retry
        let client = self.client.clone();
        let operation = || {
            let redis_key_clone = redis_key.clone();
            let client = client.clone();
            
            async move {
                // Create a new connection for each retry attempt
                let mut new_conn = client.get_async_connection().await
                    .map_err(|e| anyhow!("Redis connection error: {}", e))?;
                
                let result: Option<String> = new_conn.get(&redis_key_clone).await
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
    
    async fn set(&self, key: K, value: V, ttl_seconds: Option<u64>) -> Result<()> {
        let redis_key = self.generate_key(&key)?;
        let mut conn = self.get_connection().await?;
        
        // Serialize value
        let serialized = serde_json::to_string(&value)
            .map_err(|e| anyhow!("Failed to serialize cache value: {}", e))?;
        
        // Define retry configuration
        let retry_config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 50,
            backoff_factor: 2.0,
            max_delay_ms: 500,
        };
        
        // Define the operation to retry
        let client = self.client.clone();
        let operation = || {
            let redis_key_clone = redis_key.clone();
            let serialized_clone = serialized.clone();
            let ttl_seconds = ttl_seconds;
            let client = client.clone();
            
            async move {
                // Create a new connection for each retry attempt
                let mut new_conn = client.get_async_connection().await
                    .map_err(|e| anyhow!("Redis connection error: {}", e))?;
                
                // Set the value in Redis
                new_conn.set(&redis_key_clone, serialized_clone).await
                    .map_err(|e| anyhow!("Redis set error: {}", e))?;
                
                // Set expiration if provided
                if let Some(ttl) = ttl_seconds {
                    new_conn.expire(&redis_key_clone, ttl as usize).await
                        .map_err(|e| anyhow!("Redis expire error: {}", e))?;
                }
                
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
        let client = self.client.clone();
        let operation = || {
            let redis_key_clone = redis_key.clone();
            let client = client.clone();
            
            async move {
                // Create a new connection for each retry attempt
                let mut new_conn = client.get_async_connection().await
                    .map_err(|e| anyhow!("Redis connection error: {}", e))?;
                
                new_conn.del(&redis_key_clone).await
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
}

#[async_trait]
impl<K, V> Cache<K, V> for InMemoryCache
where
    K: Send + Sync + serde::Serialize + std::fmt::Debug + 'static,
    V: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + Clone + 'static,
{
    async fn get(&self, key: &K) -> Result<Option<V>> {
        let key_str = serde_json::to_string(key)
            .map_err(|e| anyhow!("Failed to serialize key: {}", e))?;
        
        let string_data = self.string_data.lock().await;
        
        if let Some((value_str, timestamp, ttl)) = string_data.get(&key_str) {
            // Check if expired
            let elapsed = timestamp.elapsed().as_secs();
            if elapsed > *ttl {
                return Ok(None);
            }
            
            // Deserialize value
            let value = serde_json::from_str(value_str)
                .map_err(|e| anyhow!("Failed to deserialize value: {}", e))?;
            
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
    
    async fn set(&self, key: K, value: V, ttl_seconds: Option<u64>) -> Result<()> {
        let key_str = serde_json::to_string(&key)
            .map_err(|e| anyhow!("Failed to serialize key: {}", e))?;
        
        let value_str = serde_json::to_string(&value)
            .map_err(|e| anyhow!("Failed to serialize value: {}", e))?;
        
        let mut string_data = self.string_data.lock().await;
        string_data.insert(key_str, (value_str, std::time::Instant::now(), ttl_seconds.unwrap_or(0)));
        
        Ok(())
    }
    
    async fn delete(&self, key: &K) -> Result<()> {
        let key_str = serde_json::to_string(key)
            .map_err(|e| anyhow!("Failed to serialize key: {}", e))?;
        
        let mut string_data = self.string_data.lock().await;
        string_data.remove(&key_str);
        
        Ok(())
    }
}

/// Extension trait for Cache that provides typed access
#[async_trait]
pub trait TypedCache {
    /// Get a typed value from the cache
    async fn get_typed<T>(&self, key: &str) -> Result<Option<T>> 
    where 
        T: serde::de::DeserializeOwned + Send + Sync;
    
    /// Set a typed value in the cache
    async fn set_typed<T>(&self, key: String, value: &T, ttl_seconds: Option<u64>) -> Result<()> 
    where 
        T: serde::Serialize + Send + Sync;
}

#[async_trait]
impl<C> TypedCache for C 
where 
    C: Cache<String, serde_json::Value> + Send + Sync + ?Sized
{
    async fn get_typed<T>(&self, key: &str) -> Result<Option<T>> 
    where 
        T: serde::de::DeserializeOwned + Send + Sync
    {
        let key_string = key.to_string();
        if let Some(value) = self.get(&key_string).await? {
            let typed_value = serde_json::from_value(value)?;
            Ok(Some(typed_value))
        } else {
            Ok(None)
        }
    }
    
    async fn set_typed<T>(&self, key: String, value: &T, ttl_seconds: Option<u64>) -> Result<()> 
    where 
        T: serde::Serialize + Send + Sync
    {
        let json_value = serde_json::to_value(value)?;
        self.set(key, json_value, ttl_seconds).await
    }
}

/// Extension trait for Cache that provides additional functionality
#[async_trait]
pub trait CacheExt<K, V>: Cache<K, V>
where
    K: Send + Sync + serde::Serialize + 'static,
    V: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + Clone + 'static,
{
    /// Get a value from the cache, or compute it if not present
    async fn get_or_compute<F, Fut>(&self, key: K, ttl_seconds: u64, compute: F) -> Result<V>
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<V>> + Send;
}

#[async_trait]
impl<T, K, V> CacheExt<K, V> for T
where
    T: Cache<K, V> + Send + Sync,
    K: Send + Sync + serde::Serialize + 'static,
    V: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + Clone + 'static,
{
    async fn get_or_compute<F, Fut>(&self, key: K, ttl_seconds: u64, compute: F) -> Result<V>
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<V>> + Send,
    {
        if let Some(cached) = self.get(&key).await? {
            return Ok(cached);
        }
        
        let value = compute().await?;
        self.set(key, value.clone(), Some(ttl_seconds)).await?;
        Ok(value)
    }
}

/// No-op cache implementation
pub struct NoOpCache;

impl NoOpCache {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for NoOpCache
where
    K: Send + Sync + serde::Serialize + std::fmt::Debug + 'static,
    V: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + Clone + 'static,
{
    async fn get(&self, _key: &K) -> Result<Option<V>> {
        Ok(None)
    }

    async fn set(&self, _key: K, _value: V, _ttl_seconds: Option<u64>) -> Result<()> {
        Ok(())
    }

    async fn delete(&self, _key: &K) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl StringCache for NoOpCache {
    async fn get_string(&self, _key: &str) -> anyhow::Result<Option<String>> {
        Ok(None)
    }

    async fn set_string(&self, _key: String, _value: String, _ttl_seconds: Option<u64>) -> anyhow::Result<()> {
        Ok(())
    }

    async fn delete_string(&self, _key: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

#[async_trait]
impl BinaryCache for NoOpCache {
    async fn get_binary(&self, _key: &str) -> anyhow::Result<Option<Vec<u8>>> {
        Ok(None)
    }

    async fn set_binary(&self, _key: String, _value: Vec<u8>, _ttl_seconds: Option<u64>) -> anyhow::Result<()> {
        Ok(())
    }

    async fn delete_binary(&self, _key: &str) -> anyhow::Result<()> {
        Ok(())
    }
} 