//! Cached repository implementation with TTL-based caching

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{info, debug, warn};
use lru::LruCache;

use crate::models::{Portfolio, Transaction, Account, Security, Client, Benchmark, Price, Position};
use crate::error::AppError;
use super::{Repository, PaginationOptions, PaginatedResult};

/// Cache entry with TTL
struct CacheEntry<T> {
    /// The cached value
    value: T,
    /// When this entry was created
    created_at: Instant,
    /// Time-to-live for this entry
    ttl: Duration,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            ttl,
        }
    }
    
    /// Check if this entry has expired
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// Cached repository implementation with TTL-based caching
pub struct CachedDynamoDbRepository<R: Repository> {
    /// The underlying repository
    repository: R,
    /// Cache for portfolio objects
    portfolio_cache: Arc<Mutex<LruCache<String, CacheEntry<Portfolio>>>>,
    /// Cache for benchmark objects
    benchmark_cache: Arc<Mutex<LruCache<String, CacheEntry<Benchmark>>>>,
    /// Cache for security objects
    security_cache: Arc<Mutex<LruCache<String, CacheEntry<Security>>>>,
    /// Cache for client objects
    client_cache: Arc<Mutex<LruCache<String, CacheEntry<Client>>>>,
    /// Cache for account objects
    account_cache: Arc<Mutex<LruCache<String, CacheEntry<Account>>>>,
    /// Cache for price objects (key: security_id:date)
    price_cache: Arc<Mutex<LruCache<String, CacheEntry<Price>>>>,
    /// Default TTL for cache entries
    default_ttl: Duration,
}

impl<R: Repository> CachedDynamoDbRepository<R> {
    /// Create a new cached repository
    pub fn new(repository: R, capacity: usize, default_ttl: Duration) -> Self {
        Self {
            repository,
            portfolio_cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            benchmark_cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            security_cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            client_cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            account_cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            price_cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            default_ttl,
        }
    }
    
    /// Create a price cache key
    fn price_cache_key(security_id: &str, date: &str) -> String {
        format!("{}:{}", security_id, date)
    }
    
    /// Invalidate a portfolio in the cache
    pub fn invalidate_portfolio(&self, id: &str) {
        if let Ok(mut cache) = self.portfolio_cache.lock() {
            cache.pop(id);
            debug!("Invalidated portfolio cache for ID: {}", id);
        } else {
            warn!("Failed to acquire lock for portfolio cache");
        }
    }
    
    /// Invalidate a benchmark in the cache
    pub fn invalidate_benchmark(&self, id: &str) {
        if let Ok(mut cache) = self.benchmark_cache.lock() {
            cache.pop(id);
            debug!("Invalidated benchmark cache for ID: {}", id);
        } else {
            warn!("Failed to acquire lock for benchmark cache");
        }
    }
    
    /// Invalidate a security in the cache
    pub fn invalidate_security(&self, id: &str) {
        if let Ok(mut cache) = self.security_cache.lock() {
            cache.pop(id);
            debug!("Invalidated security cache for ID: {}", id);
        } else {
            warn!("Failed to acquire lock for security cache");
        }
    }
    
    /// Invalidate a client in the cache
    pub fn invalidate_client(&self, id: &str) {
        if let Ok(mut cache) = self.client_cache.lock() {
            cache.pop(id);
            debug!("Invalidated client cache for ID: {}", id);
        } else {
            warn!("Failed to acquire lock for client cache");
        }
    }
    
    /// Invalidate an account in the cache
    pub fn invalidate_account(&self, id: &str) {
        if let Ok(mut cache) = self.account_cache.lock() {
            cache.pop(id);
            debug!("Invalidated account cache for ID: {}", id);
        } else {
            warn!("Failed to acquire lock for account cache");
        }
    }
    
    /// Invalidate a price in the cache
    pub fn invalidate_price(&self, security_id: &str, date: &str) {
        let key = Self::price_cache_key(security_id, date);
        if let Ok(mut cache) = self.price_cache.lock() {
            cache.pop(&key);
            debug!("Invalidated price cache for security ID: {} and date: {}", security_id, date);
        } else {
            warn!("Failed to acquire lock for price cache");
        }
    }
    
    /// Clear all caches
    pub fn clear_all_caches(&self) {
        if let Ok(mut cache) = self.portfolio_cache.lock() {
            cache.clear();
        }
        if let Ok(mut cache) = self.benchmark_cache.lock() {
            cache.clear();
        }
        if let Ok(mut cache) = self.security_cache.lock() {
            cache.clear();
        }
        if let Ok(mut cache) = self.client_cache.lock() {
            cache.clear();
        }
        if let Ok(mut cache) = self.account_cache.lock() {
            cache.clear();
        }
        if let Ok(mut cache) = self.price_cache.lock() {
            cache.clear();
        }
        info!("Cleared all caches");
    }
}

#[async_trait]
impl<R: Repository + Send + Sync> Repository for CachedDynamoDbRepository<R> {
    async fn get_portfolio(&self, id: &str) -> Result<Option<Portfolio>, AppError> {
        // Check cache first
        if let Ok(mut cache) = self.portfolio_cache.lock() {
            if let Some(entry) = cache.get(id) {
                if !entry.is_expired() {
                    debug!("Cache hit for portfolio ID: {}", id);
                    return Ok(Some(entry.value.clone()));
                } else {
                    // Remove expired entry
                    cache.pop(id);
                    debug!("Removed expired cache entry for portfolio ID: {}", id);
                }
            }
        } else {
            warn!("Failed to acquire lock for portfolio cache");
        }
        
        // Cache miss or expired, fetch from repository
        let result = self.repository.get_portfolio(id).await?;
        
        // Cache the result if found
        if let Some(ref portfolio) = result {
            if let Ok(mut cache) = self.portfolio_cache.lock() {
                cache.put(
                    id.to_string(), 
                    CacheEntry::new(portfolio.clone(), self.default_ttl)
                );
                debug!("Cached portfolio ID: {}", id);
            }
        }
        
        Ok(result)
    }
    
    async fn list_portfolios(
        &self, 
        client_id: Option<&str>,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Portfolio>, AppError> {
        // For list operations with pagination, we don't cache to ensure fresh results
        self.repository.list_portfolios(client_id, pagination).await
    }
    
    async fn put_portfolio(&self, portfolio: &Portfolio) -> Result<(), AppError> {
        // Update the repository
        let result = self.repository.put_portfolio(portfolio).await;
        
        // Invalidate cache on success
        if result.is_ok() {
            self.invalidate_portfolio(&portfolio.id);
        }
        
        result
    }
    
    async fn delete_portfolio(&self, id: &str) -> Result<(), AppError> {
        // Update the repository
        let result = self.repository.delete_portfolio(id).await;
        
        // Invalidate cache on success
        if result.is_ok() {
            self.invalidate_portfolio(id);
        }
        
        result
    }
    
    async fn get_transaction(&self, id: &str) -> Result<Option<Transaction>, AppError> {
        // Transactions are not cached as they're less frequently accessed
        self.repository.get_transaction(id).await
    }
    
    async fn list_transactions(
        &self,
        account_id: Option<&str>,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Transaction>, AppError> {
        // For list operations with pagination, we don't cache to ensure fresh results
        self.repository.list_transactions(account_id, pagination).await
    }
    
    async fn put_transaction(&self, transaction: &Transaction) -> Result<(), AppError> {
        // Update the repository
        self.repository.put_transaction(transaction).await
    }
    
    async fn delete_transaction(&self, id: &str) -> Result<(), AppError> {
        // Update the repository
        self.repository.delete_transaction(id).await
    }
    
    async fn get_account(&self, id: &str) -> Result<Option<Account>, AppError> {
        // Check cache first
        if let Ok(mut cache) = self.account_cache.lock() {
            if let Some(entry) = cache.get(id) {
                if !entry.is_expired() {
                    debug!("Cache hit for account ID: {}", id);
                    return Ok(Some(entry.value.clone()));
                } else {
                    // Remove expired entry
                    cache.pop(id);
                    debug!("Removed expired cache entry for account ID: {}", id);
                }
            }
        } else {
            warn!("Failed to acquire lock for account cache");
        }
        
        // Cache miss or expired, fetch from repository
        let result = self.repository.get_account(id).await?;
        
        // Cache the result if found
        if let Some(ref account) = result {
            if let Ok(mut cache) = self.account_cache.lock() {
                cache.put(
                    id.to_string(), 
                    CacheEntry::new(account.clone(), self.default_ttl)
                );
                debug!("Cached account ID: {}", id);
            }
        }
        
        Ok(result)
    }
    
    async fn list_accounts(
        &self,
        portfolio_id: Option<&str>,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Account>, AppError> {
        // For list operations, we don't cache to ensure fresh results
        self.repository.list_accounts(portfolio_id, pagination).await
    }
    
    async fn put_account(&self, account: &Account) -> Result<(), AppError> {
        // Update the repository
        let result = self.repository.put_account(account).await;
        
        // Invalidate cache on success
        if result.is_ok() {
            self.invalidate_account(&account.id);
        }
        
        result
    }
    
    async fn delete_account(&self, id: &str) -> Result<(), AppError> {
        // Update the repository
        let result = self.repository.delete_account(id).await;
        
        // Invalidate cache on success
        if result.is_ok() {
            self.invalidate_account(id);
        }
        
        result
    }
    
    async fn get_security(&self, id: &str) -> Result<Option<Security>, AppError> {
        // Check cache first
        if let Ok(mut cache) = self.security_cache.lock() {
            if let Some(entry) = cache.get(id) {
                if !entry.is_expired() {
                    debug!("Cache hit for security ID: {}", id);
                    return Ok(Some(entry.value.clone()));
                } else {
                    // Remove expired entry
                    cache.pop(id);
                    debug!("Removed expired cache entry for security ID: {}", id);
                }
            }
        } else {
            warn!("Failed to acquire lock for security cache");
        }
        
        // Cache miss or expired, fetch from repository
        let result = self.repository.get_security(id).await?;
        
        // Cache the result if found
        if let Some(ref security) = result {
            if let Ok(mut cache) = self.security_cache.lock() {
                cache.put(
                    id.to_string(), 
                    CacheEntry::new(security.clone(), self.default_ttl)
                );
                debug!("Cached security ID: {}", id);
            }
        }
        
        Ok(result)
    }
    
    async fn list_securities(
        &self,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Security>, AppError> {
        // For list operations, we don't cache to ensure fresh results
        self.repository.list_securities(pagination).await
    }
    
    async fn put_security(&self, security: &Security) -> Result<(), AppError> {
        // Update the repository
        let result = self.repository.put_security(security).await;
        
        // Invalidate cache on success
        if result.is_ok() {
            self.invalidate_security(&security.id);
        }
        
        result
    }
    
    async fn delete_security(&self, id: &str) -> Result<(), AppError> {
        // Update the repository
        let result = self.repository.delete_security(id).await;
        
        // Invalidate cache on success
        if result.is_ok() {
            self.invalidate_security(id);
        }
        
        result
    }
    
    async fn get_client(&self, id: &str) -> Result<Option<Client>, AppError> {
        // Check cache first
        if let Ok(mut cache) = self.client_cache.lock() {
            if let Some(entry) = cache.get(id) {
                if !entry.is_expired() {
                    debug!("Cache hit for client ID: {}", id);
                    return Ok(Some(entry.value.clone()));
                } else {
                    // Remove expired entry
                    cache.pop(id);
                    debug!("Removed expired cache entry for client ID: {}", id);
                }
            }
        } else {
            warn!("Failed to acquire lock for client cache");
        }
        
        // Cache miss or expired, fetch from repository
        let result = self.repository.get_client(id).await?;
        
        // Cache the result if found
        if let Some(ref client) = result {
            if let Ok(mut cache) = self.client_cache.lock() {
                cache.put(
                    id.to_string(), 
                    CacheEntry::new(client.clone(), self.default_ttl)
                );
                debug!("Cached client ID: {}", id);
            }
        }
        
        Ok(result)
    }
    
    async fn list_clients(
        &self,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Client>, AppError> {
        // For list operations, we don't cache to ensure fresh results
        self.repository.list_clients(pagination).await
    }
    
    async fn put_client(&self, client: &Client) -> Result<(), AppError> {
        // Update the repository
        let result = self.repository.put_client(client).await;
        
        // Invalidate cache on success
        if result.is_ok() {
            self.invalidate_client(&client.id);
        }
        
        result
    }
    
    async fn delete_client(&self, id: &str) -> Result<(), AppError> {
        // Update the repository
        let result = self.repository.delete_client(id).await;
        
        // Invalidate cache on success
        if result.is_ok() {
            self.invalidate_client(id);
        }
        
        result
    }
    
    async fn get_benchmark(&self, id: &str) -> Result<Option<Benchmark>, AppError> {
        // Check cache first
        if let Ok(mut cache) = self.benchmark_cache.lock() {
            if let Some(entry) = cache.get(id) {
                if !entry.is_expired() {
                    debug!("Cache hit for benchmark ID: {}", id);
                    return Ok(Some(entry.value.clone()));
                } else {
                    // Remove expired entry
                    cache.pop(id);
                    debug!("Removed expired cache entry for benchmark ID: {}", id);
                }
            }
        } else {
            warn!("Failed to acquire lock for benchmark cache");
        }
        
        // Cache miss or expired, fetch from repository
        let result = self.repository.get_benchmark(id).await?;
        
        // Cache the result if found
        if let Some(ref benchmark) = result {
            if let Ok(mut cache) = self.benchmark_cache.lock() {
                cache.put(
                    id.to_string(), 
                    CacheEntry::new(benchmark.clone(), self.default_ttl)
                );
                debug!("Cached benchmark ID: {}", id);
            }
        }
        
        Ok(result)
    }
    
    async fn list_benchmarks(
        &self,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Benchmark>, AppError> {
        // For list operations, we don't cache to ensure fresh results
        self.repository.list_benchmarks(pagination).await
    }
    
    async fn put_benchmark(&self, benchmark: &Benchmark) -> Result<(), AppError> {
        // Update the repository
        let result = self.repository.put_benchmark(benchmark).await;
        
        // Invalidate cache on success
        if result.is_ok() {
            self.invalidate_benchmark(&benchmark.id);
        }
        
        result
    }
    
    async fn delete_benchmark(&self, id: &str) -> Result<(), AppError> {
        // Update the repository
        let result = self.repository.delete_benchmark(id).await;
        
        // Invalidate cache on success
        if result.is_ok() {
            self.invalidate_benchmark(id);
        }
        
        result
    }
    
    async fn get_price(&self, security_id: &str, date: &str) -> Result<Option<Price>, AppError> {
        let cache_key = Self::price_cache_key(security_id, date);
        
        // Check cache first
        if let Ok(mut cache) = self.price_cache.lock() {
            if let Some(entry) = cache.get(&cache_key) {
                if !entry.is_expired() {
                    debug!("Cache hit for price: security_id={}, date={}", security_id, date);
                    return Ok(Some(entry.value.clone()));
                } else {
                    // Remove expired entry
                    cache.pop(&cache_key);
                    debug!("Removed expired cache entry for price: security_id={}, date={}", security_id, date);
                }
            }
        } else {
            warn!("Failed to acquire lock for price cache");
        }
        
        // Cache miss or expired, fetch from repository
        let result = self.repository.get_price(security_id, date).await?;
        
        // Cache the result if found
        if let Some(ref price) = result {
            if let Ok(mut cache) = self.price_cache.lock() {
                cache.put(
                    cache_key, 
                    CacheEntry::new(price.clone(), self.default_ttl)
                );
                debug!("Cached price: security_id={}, date={}", security_id, date);
            }
        }
        
        Ok(result)
    }
    
    async fn list_prices(
        &self,
        security_id: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Price>, AppError> {
        // For list operations, we don't cache to ensure fresh results
        self.repository.list_prices(security_id, start_date, end_date, pagination).await
    }
    
    async fn put_price(&self, price: &Price) -> Result<(), AppError> {
        // Update the repository
        let result = self.repository.put_price(price).await;
        
        // Invalidate cache on success
        if result.is_ok() {
            self.invalidate_price(&price.security_id, &price.date.to_string());
        }
        
        result
    }
    
    async fn get_positions(
        &self,
        account_id: &str,
        date: &str
    ) -> Result<Vec<Position>, AppError> {
        // Positions are not cached as they're calculated on-the-fly
        self.repository.get_positions(account_id, date).await
    }
    
    async fn put_position(&self, position: &Position) -> Result<(), AppError> {
        // Update the repository
        self.repository.put_position(position).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;
    use std::time::Duration;
    
    // Create a mock repository for testing
    mock! {
        Repository {}
        
        #[async_trait]
        impl Repository for Repository {
            async fn get_portfolio(&self, id: &str) -> Result<Option<Portfolio>, AppError>;
            async fn list_portfolios(
                &self, 
                client_id: Option<&str>,
                pagination: Option<PaginationOptions>
            ) -> Result<PaginatedResult<Portfolio>, AppError>;
            async fn put_portfolio(&self, portfolio: &Portfolio) -> Result<(), AppError>;
            async fn delete_portfolio(&self, id: &str) -> Result<(), AppError>;
            async fn get_transaction(&self, id: &str) -> Result<Option<Transaction>, AppError>;
            async fn list_transactions(
                &self,
                account_id: Option<&str>,
                pagination: Option<PaginationOptions>
            ) -> Result<PaginatedResult<Transaction>, AppError>;
            async fn put_transaction(&self, transaction: &Transaction) -> Result<(), AppError>;
            async fn delete_transaction(&self, id: &str) -> Result<(), AppError>;
            async fn get_account(&self, id: &str) -> Result<Option<Account>, AppError>;
            async fn list_accounts(
                &self,
                portfolio_id: Option<&str>,
                pagination: Option<PaginationOptions>
            ) -> Result<PaginatedResult<Account>, AppError>;
            async fn put_account(&self, account: &Account) -> Result<(), AppError>;
            async fn delete_account(&self, id: &str) -> Result<(), AppError>;
            async fn get_security(&self, id: &str) -> Result<Option<Security>, AppError>;
            async fn list_securities(
                &self,
                pagination: Option<PaginationOptions>
            ) -> Result<PaginatedResult<Security>, AppError>;
            async fn put_security(&self, security: &Security) -> Result<(), AppError>;
            async fn delete_security(&self, id: &str) -> Result<(), AppError>;
            async fn get_client(&self, id: &str) -> Result<Option<Client>, AppError>;
            async fn list_clients(
                &self,
                pagination: Option<PaginationOptions>
            ) -> Result<PaginatedResult<Client>, AppError>;
            async fn put_client(&self, client: &Client) -> Result<(), AppError>;
            async fn delete_client(&self, id: &str) -> Result<(), AppError>;
            async fn get_benchmark(&self, id: &str) -> Result<Option<Benchmark>, AppError>;
            async fn list_benchmarks(
                &self,
                pagination: Option<PaginationOptions>
            ) -> Result<PaginatedResult<Benchmark>, AppError>;
            async fn put_benchmark(&self, benchmark: &Benchmark) -> Result<(), AppError>;
            async fn delete_benchmark(&self, id: &str) -> Result<(), AppError>;
            async fn get_price(&self, security_id: &str, date: &str) -> Result<Option<Price>, AppError>;
            async fn list_prices(
                &self,
                security_id: &str,
                start_date: Option<&str>,
                end_date: Option<&str>,
                pagination: Option<PaginationOptions>
            ) -> Result<PaginatedResult<Price>, AppError>;
            async fn put_price(&self, price: &Price) -> Result<(), AppError>;
            async fn get_positions(
                &self,
                account_id: &str,
                date: &str
            ) -> Result<Vec<Position>, AppError>;
            async fn put_position(&self, position: &Position) -> Result<(), AppError>;
        }
    }
    
    #[tokio::test]
    async fn test_portfolio_caching() {
        // Create a mock repository
        let mut mock_repo = MockRepository::new();
        
        // Set up expectations
        let portfolio_id = "test-portfolio-id";
        let portfolio = Portfolio {
            id: portfolio_id.to_string(),
            name: "Test Portfolio".to_string(),
            client_id: "test-client-id".to_string(),
            inception_date: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            benchmark_id: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            status: crate::models::Status::Active,
            metadata: HashMap::new(),
        };
        
        // First call should hit the repository
        mock_repo.expect_get_portfolio()
            .with(eq(portfolio_id))
            .times(1)
            .returning(move |_| Ok(Some(portfolio.clone())));
        
        // Create the cached repository
        let cached_repo = CachedDynamoDbRepository::new(
            mock_repo,
            100, // capacity
            Duration::from_secs(60) // TTL
        );
        
        // First call should hit the repository and cache the result
        let result1 = cached_repo.get_portfolio(portfolio_id).await.unwrap();
        assert!(result1.is_some());
        assert_eq!(result1.unwrap().id, portfolio_id);
        
        // Second call should hit the cache
        let result2 = cached_repo.get_portfolio(portfolio_id).await.unwrap();
        assert!(result2.is_some());
        assert_eq!(result2.unwrap().id, portfolio_id);
    }
    
    #[tokio::test]
    async fn test_benchmark_caching() {
        // Create a mock repository
        let mut mock_repo = MockRepository::new();
        
        // Set up expectations
        let benchmark_id = "test-benchmark-id";
        let benchmark = Benchmark {
            id: benchmark_id.to_string(),
            name: "Test Benchmark".to_string(),
            symbol: Some("TEST".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        
        // First call should hit the repository
        mock_repo.expect_get_benchmark()
            .with(eq(benchmark_id))
            .times(1)
            .returning(move |_| Ok(Some(benchmark.clone())));
        
        // Create the cached repository
        let cached_repo = CachedDynamoDbRepository::new(
            mock_repo,
            100, // capacity
            Duration::from_secs(60) // TTL
        );
        
        // First call should hit the repository and cache the result
        let result1 = cached_repo.get_benchmark(benchmark_id).await.unwrap();
        assert!(result1.is_some());
        assert_eq!(result1.unwrap().id, benchmark_id);
        
        // Second call should hit the cache
        let result2 = cached_repo.get_benchmark(benchmark_id).await.unwrap();
        assert!(result2.is_some());
        assert_eq!(result2.unwrap().id, benchmark_id);
    }
    
    #[tokio::test]
    async fn test_cache_invalidation() {
        // Create a mock repository
        let mut mock_repo = MockRepository::new();
        
        // Set up expectations
        let portfolio_id = "test-portfolio-id";
        let portfolio = Portfolio {
            id: portfolio_id.to_string(),
            name: "Test Portfolio".to_string(),
            client_id: "test-client-id".to_string(),
            inception_date: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            benchmark_id: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            status: crate::models::Status::Active,
            metadata: HashMap::new(),
        };
        
        // First call should hit the repository
        mock_repo.expect_get_portfolio()
            .with(eq(portfolio_id))
            .times(1)
            .returning(move |_| Ok(Some(portfolio.clone())));
        
        // Update should succeed
        mock_repo.expect_put_portfolio()
            .times(1)
            .returning(|_| Ok(()));
        
        // After invalidation, next get should hit the repository again
        let portfolio_clone = portfolio.clone();
        mock_repo.expect_get_portfolio()
            .with(eq(portfolio_id))
            .times(1)
            .returning(move |_| Ok(Some(portfolio_clone.clone())));
        
        // Create the cached repository
        let cached_repo = CachedDynamoDbRepository::new(
            mock_repo,
            100, // capacity
            Duration::from_secs(60) // TTL
        );
        
        // First call should hit the repository and cache the result
        let result1 = cached_repo.get_portfolio(portfolio_id).await.unwrap();
        assert!(result1.is_some());
        
        // Update the portfolio (should invalidate cache)
        cached_repo.put_portfolio(&portfolio).await.unwrap();
        
        // Next call should hit the repository again
        let result2 = cached_repo.get_portfolio(portfolio_id).await.unwrap();
        assert!(result2.is_some());
    }
    
    #[tokio::test]
    async fn test_cache_expiration() {
        // Create a mock repository
        let mut mock_repo = MockRepository::new();
        
        // Set up expectations
        let portfolio_id = "test-portfolio-id";
        let portfolio = Portfolio {
            id: portfolio_id.to_string(),
            name: "Test Portfolio".to_string(),
            client_id: "test-client-id".to_string(),
            inception_date: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            benchmark_id: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            status: crate::models::Status::Active,
            metadata: HashMap::new(),
        };
        
        // Repository should be called twice due to expiration
        mock_repo.expect_get_portfolio()
            .with(eq(portfolio_id))
            .times(2)
            .returning(move |_| Ok(Some(portfolio.clone())));
        
        // Create the cached repository with a very short TTL
        let cached_repo = CachedDynamoDbRepository::new(
            mock_repo,
            100, // capacity
            Duration::from_millis(10) // Very short TTL for testing
        );
        
        // First call should hit the repository and cache the result
        let result1 = cached_repo.get_portfolio(portfolio_id).await.unwrap();
        assert!(result1.is_some());
        
        // Wait for cache to expire
        tokio::time::sleep(Duration::from_millis(20)).await;
        
        // Next call should hit the repository again due to expiration
        let result2 = cached_repo.get_portfolio(portfolio_id).await.unwrap();
        assert!(result2.is_some());
    }
} 