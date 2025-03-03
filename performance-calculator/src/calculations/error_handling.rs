use anyhow::{Result, anyhow};
use std::future::Future;
use std::time::Duration;
use tracing::{info, warn, error};

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries in milliseconds
    pub initial_delay_ms: u64,
    /// Backoff factor for exponential backoff
    pub backoff_factor: f64,
    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            backoff_factor: 2.0,
            max_delay_ms: 5000,
        }
    }
}

/// Execute an operation with retry logic
pub async fn with_retry<F, Fut, T>(
    operation: F,
    config: RetryConfig,
    operation_name: &str,
    request_id: &str,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempt = 1;
    let mut delay = config.initial_delay_ms;
    
    loop {
        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    info!(
                        request_id = %request_id,
                        operation = %operation_name,
                        attempt = attempt,
                        "Operation succeeded after retry"
                    );
                }
                return Ok(result);
            },
            Err(err) => {
                if attempt >= config.max_attempts {
                    error!(
                        request_id = %request_id,
                        operation = %operation_name,
                        attempt = attempt,
                        error = %err,
                        "Operation failed after maximum retry attempts"
                    );
                    return Err(anyhow!("Operation '{}' failed after {} attempts: {}", operation_name, attempt, err));
                }
                
                warn!(
                    request_id = %request_id,
                    operation = %operation_name,
                    attempt = attempt,
                    next_attempt = attempt + 1,
                    delay_ms = delay,
                    error = %err,
                    "Operation failed, retrying"
                );
                
                // Sleep before retrying
                tokio::time::sleep(Duration::from_millis(delay)).await;
                
                // Calculate next delay with exponential backoff
                delay = ((delay as f64) * config.backoff_factor) as u64;
                delay = delay.min(config.max_delay_ms);
                
                attempt += 1;
            }
        }
    }
}

/// Execute a database operation with retry logic
pub async fn with_db_retry<F, Fut, T>(
    operation: F,
    operation_name: &str,
    request_id: &str,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    // Use a more conservative retry config for database operations
    let config = RetryConfig {
        max_attempts: 5,
        initial_delay_ms: 200,
        backoff_factor: 2.0,
        max_delay_ms: 10000,
    };
    
    with_retry(operation, config, operation_name, request_id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    
    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let operation = || async { Ok::<_, anyhow::Error>(42) };
        
        let result = with_retry(
            operation,
            RetryConfig::default(),
            "test_operation",
            "test-request"
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }
    
    #[tokio::test]
    async fn test_retry_success_after_failure() {
        let attempt_counter = AtomicU32::new(0);
        
        let operation = || {
            let current_attempt = attempt_counter.fetch_add(1, Ordering::SeqCst);
            
            async move {
                if current_attempt < 2 {
                    Err(anyhow!("Simulated failure"))
                } else {
                    Ok(42)
                }
            }
        };
        
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 10, // Use small delays for testing
            backoff_factor: 1.0,
            max_delay_ms: 100,
        };
        
        let result = with_retry(
            operation,
            config,
            "test_operation",
            "test-request"
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);
    }
    
    #[tokio::test]
    async fn test_retry_max_attempts_exceeded() {
        let attempt_counter = AtomicU32::new(0);
        
        let operation = || {
            let current_attempt = attempt_counter.fetch_add(1, Ordering::SeqCst);
            
            async move {
                if current_attempt < 10 {
                    Err(anyhow!("Simulated failure"))
                } else {
                    Ok(42)
                }
            }
        };
        
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 10,
            backoff_factor: 1.0,
            max_delay_ms: 100,
        };
        
        let result = with_retry(
            operation,
            config,
            "test_operation",
            "test-request"
        ).await;
        
        assert!(result.is_err());
        assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);
    }
} 