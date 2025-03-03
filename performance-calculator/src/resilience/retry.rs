use std::future::Future;
use std::time::Duration;
use rand::Rng;
use tracing::{info, warn};
use thiserror::Error;

/// Retry error
#[derive(Error, Debug)]
pub enum RetryError<E> {
    #[error("Max retries exceeded: {0}")]
    MaxRetriesExceeded(E),
    
    #[error("Retry aborted: {0}")]
    Aborted(E),
}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    
    /// Initial backoff duration in milliseconds
    pub initial_backoff_ms: u64,
    
    /// Maximum backoff duration in milliseconds
    pub max_backoff_ms: u64,
    
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    
    /// Jitter factor (0.0 to 1.0)
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 10000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

/// Retry policy trait
pub trait RetryPolicy<E> {
    /// Determine if a retry should be attempted based on the error
    fn should_retry(&self, error: &E, attempt: u32) -> bool;
    
    /// Calculate the backoff duration for a retry attempt
    fn backoff_duration(&self, attempt: u32) -> Duration;
}

/// Standard retry policy
pub struct StandardRetryPolicy<F> {
    config: RetryConfig,
    should_retry_fn: F,
}

impl<F, E> StandardRetryPolicy<F>
where
    F: Fn(&E, u32) -> bool,
{
    /// Create a new retry policy
    pub fn new(config: RetryConfig, should_retry_fn: F) -> Self {
        Self {
            config,
            should_retry_fn,
        }
    }
    
    /// Create a new retry policy with default configuration
    pub fn with_default_config(should_retry_fn: F) -> Self {
        Self::new(RetryConfig::default(), should_retry_fn)
    }
}

impl<F, E> RetryPolicy<E> for StandardRetryPolicy<F>
where
    F: Fn(&E, u32) -> bool,
{
    fn should_retry(&self, error: &E, attempt: u32) -> bool {
        if attempt >= self.config.max_retries {
            return false;
        }
        
        (self.should_retry_fn)(error, attempt)
    }
    
    fn backoff_duration(&self, attempt: u32) -> Duration {
        let base_ms = (self.config.initial_backoff_ms as f64 * self.config.backoff_multiplier.powi(attempt as i32)) as u64;
        let capped_ms = base_ms.min(self.config.max_backoff_ms);
        
        if self.config.jitter_factor > 0.0 {
            let jitter_ms = (capped_ms as f64 * self.config.jitter_factor) as u64;
            let jitter = rand::thread_rng().gen_range(0..=jitter_ms);
            Duration::from_millis(capped_ms.saturating_add(jitter))
        } else {
            Duration::from_millis(capped_ms)
        }
    }
}

/// Retry a function with the given retry policy
pub async fn retry<T, E, F, Fut, P>(
    f: F,
    retry_policy: &P,
) -> Result<T, RetryError<E>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    P: RetryPolicy<E>,
    E: std::fmt::Debug,
{
    let mut attempt = 0;
    
    loop {
        match f().await {
            Ok(result) => {
                if attempt > 0 {
                    info!("Succeeded after {} retries", attempt);
                }
                return Ok(result);
            },
            Err(error) => {
                if retry_policy.should_retry(&error, attempt) {
                    let backoff = retry_policy.backoff_duration(attempt);
                    warn!("Retry attempt {} failed: {:?}, retrying in {:?}", attempt + 1, error, backoff);
                    tokio::time::sleep(backoff).await;
                    attempt += 1;
                } else if attempt == 0 {
                    return Err(RetryError::Aborted(error));
                } else {
                    warn!("Max retries exceeded after {} attempts", attempt + 1);
                    return Err(RetryError::MaxRetriesExceeded(error));
                }
            }
        }
    }
}

/// Retry a function with exponential backoff
pub async fn retry_with_exponential_backoff<T, E, F, Fut>(
    f: F,
    max_retries: u32,
    initial_backoff_ms: u64,
) -> Result<T, RetryError<E>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let config = RetryConfig {
        max_retries,
        initial_backoff_ms,
        ..RetryConfig::default()
    };
    
    let retry_policy = StandardRetryPolicy::new(config, |_, _| true);
    
    retry(f, &retry_policy).await
}

/// Retry a function with a custom retry predicate
pub async fn retry_if<T, E, F, Fut, P>(
    f: F,
    max_retries: u32,
    initial_backoff_ms: u64,
    predicate: P,
) -> Result<T, RetryError<E>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    P: Fn(&E, u32) -> bool,
    E: std::fmt::Debug,
{
    let config = RetryConfig {
        max_retries,
        initial_backoff_ms,
        ..RetryConfig::default()
    };
    
    let retry_policy = StandardRetryPolicy::new(config, predicate);
    
    retry(f, &retry_policy).await
} 