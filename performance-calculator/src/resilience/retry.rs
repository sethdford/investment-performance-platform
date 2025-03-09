use std::error::Error;
use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::time::Duration;
use anyhow::{Result, anyhow};
use tracing::{info, warn, error};
use rand::Rng;
use thiserror::Error;

/// Retry error types
#[derive(Debug)]
pub enum RetryError<E> {
    MaxRetriesExceeded(E),
    Aborted(E),
    Other(E),
}

impl<E: std::error::Error + 'static> std::error::Error for RetryError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RetryError::MaxRetriesExceeded(e) => Some(e),
            RetryError::Aborted(e) => Some(e),
            RetryError::Other(e) => Some(e),
        }
    }
}

impl<E: std::fmt::Display> std::fmt::Display for RetryError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetryError::MaxRetriesExceeded(e) => write!(f, "Maximum retries exceeded: {}", e),
            RetryError::Aborted(e) => write!(f, "Operation aborted: {}", e),
            RetryError::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

/// Retry policy trait
#[async_trait::async_trait]
pub trait RetryPolicy<E>: Send + Sync
where
    E: Error + Send + Sync + 'static,
{
    /// Should retry the operation
    async fn should_retry(&self, error: &E, attempt: u32) -> bool;
    
    /// Get delay before next retry
    async fn get_delay(&self, attempt: u32) -> Duration;
}

/// Standard retry policy implementation
pub struct StandardRetryPolicy {
    name: String,
    max_attempts: u32,
    initial_delay_ms: u64,
    max_delay_ms: u64,
    multiplier: f64,
    jitter: bool,
}

/// Configuration for retry operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    
    /// Initial delay in milliseconds
    pub initial_delay_ms: u64,
    
    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,
    
    /// Multiplier for exponential backoff
    pub multiplier: f64,
    
    /// Whether to add jitter to the delay
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 1000,
            multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    pub fn new(
        max_attempts: u32,
        initial_delay_ms: u64,
        max_delay_ms: u64,
    ) -> Self {
        Self {
            max_attempts,
            initial_delay_ms,
            max_delay_ms,
            multiplier: 2.0,
            jitter: true,
        }
    }
}

impl StandardRetryPolicy {
    pub fn new(config: RetryConfig) -> Self {
        Self {
            name: "default".to_string(),
            max_attempts: config.max_attempts,
            initial_delay_ms: config.initial_delay_ms,
            max_delay_ms: config.max_delay_ms,
            multiplier: config.multiplier,
            jitter: config.jitter,
        }
    }
}

#[async_trait::async_trait]
impl<E> RetryPolicy<E> for StandardRetryPolicy
where
    E: Error + Send + Sync + 'static,
{
    async fn should_retry(&self, error: &E, attempt: u32) -> bool {
        if attempt >= self.max_attempts {
            warn!(
                "Maximum retries ({}) exceeded for {}: {}",
                self.max_attempts, self.name, error
            );
            return false;
        }
        
        // Check if error is retryable
        // For now, retry all errors
        true
    }
    
    async fn get_delay(&self, attempt: u32) -> Duration {
        let delay = self.initial_delay_ms as f64 * self.multiplier.powi(attempt as i32);
        let delay = delay.min(self.max_delay_ms as f64);
        Duration::from_millis(delay as u64)
    }
}

/// Retry an operation with the given policy
pub async fn retry_with_policy<T, E, F, Fut, P>(
    operation: F,
    retry_policy: &P,
) -> Result<T, RetryError<E>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    P: RetryPolicy<E>,
    E: Error + Send + Sync + Debug + 'static,
{
    let mut attempt = 0;
    
    loop {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(error) => {
                if retry_policy.should_retry(&error, attempt).await {
                    let backoff = retry_policy.get_delay(attempt).await;
                    warn!("Retry attempt {} failed: {:?}, retrying in {:?}", attempt + 1, error, backoff);
                    tokio::time::sleep(backoff).await;
                    attempt += 1;
                } else {
                    return Err(RetryError::Other(error));
                }
            }
        }
    }
}

/// Retry a function with exponential backoff
pub async fn retry_with_exponential_backoff<T, E, F, Fut>(
    mut f: F,
    config: RetryConfig,
) -> Result<T, RetryError<E>>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug + std::error::Error + Send + Sync + 'static,
{
    let retry_policy = StandardRetryPolicy::new(config);
    retry(f, &retry_policy).await
}

/// Retry a function with a custom retry predicate
pub async fn retry_if<T, E, F, Fut, P>(
    mut f: F,
    config: RetryConfig,
    predicate: P,
) -> Result<T, RetryError<E>>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    P: Fn(&E, u32) -> bool,
    E: std::fmt::Debug + std::error::Error + Send + Sync + 'static,
{
    let retry_policy = StandardRetryPolicy::new(config);
    retry(f, &retry_policy).await
}

pub async fn retry<F, Fut, T, E>(
    mut operation: F,
    policy: &StandardRetryPolicy,
) -> Result<T, RetryError<E>>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut attempt = 0;
    let mut delay_ms = policy.initial_delay_ms;
    let mut last_error = None;

    loop {
        attempt += 1;
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if attempt >= policy.max_attempts {
                    return Err(RetryError::MaxRetriesExceeded(error));
                }

                last_error = Some(error);
                let jitter = if policy.jitter {
                    let mut rng = rand::thread_rng();
                    rng.gen_range(0..=(delay_ms / 4))
                } else {
                    0
                };

                tokio::time::sleep(Duration::from_millis(delay_ms + jitter)).await;
                delay_ms = std::cmp::min(
                    (delay_ms as f64 * policy.multiplier) as u64,
                    policy.max_delay_ms
                );
            }
        }
    }
} 