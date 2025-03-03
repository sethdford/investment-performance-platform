//! Resilience patterns for the application

pub mod circuit_breaker;
pub mod bulkhead;
pub mod retry;

use std::future::Future;
use std::time::Duration;
use tracing::{info, warn, error};

/// Execute a function with retry logic
pub async fn with_retry<F, Fut, T, E>(
    operation_name: &str,
    config: retry::RetryConfig,
    f: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    retry::with_retry(operation_name, config, f).await
}

/// Execute a function with circuit breaker
pub async fn with_circuit_breaker<F, Fut, T, E>(
    operation_name: &str,
    config: circuit_breaker::CircuitBreakerConfig,
    f: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    circuit_breaker::with_circuit_breaker(operation_name, config, f).await
}

/// Execute a function with bulkhead
pub async fn with_bulkhead<F, Fut, T, E>(
    operation_name: &str,
    config: bulkhead::BulkheadConfig,
    f: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    bulkhead::with_bulkhead(operation_name, config, f).await
}

/// Execute a function with retry, circuit breaker, and bulkhead
pub async fn with_resilience<F, Fut, T, E>(
    operation_name: &str,
    retry_config: retry::RetryConfig,
    circuit_breaker_config: circuit_breaker::CircuitBreakerConfig,
    bulkhead_config: bulkhead::BulkheadConfig,
    f: F,
) -> Result<T, E>
where
    F: Fn() -> Fut + Clone,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    with_bulkhead(
        operation_name,
        bulkhead_config,
        move || {
            let f_clone = f.clone();
            async move {
                with_circuit_breaker(
                    operation_name,
                    circuit_breaker_config.clone(),
                    move || {
                        let f_clone = f.clone();
                        async move {
                            with_retry(
                                operation_name,
                                retry_config.clone(),
                                f_clone,
                            ).await
                        }
                    },
                ).await
            }
        },
    ).await
} 