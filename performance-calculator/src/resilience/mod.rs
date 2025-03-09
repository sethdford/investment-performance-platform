use std::future::Future;
use std::sync::Arc;
use anyhow::Result;
use thiserror::Error;
use circuit_breaker::CircuitBreaker;
use bulkhead::Bulkhead;
use std::fmt::{Debug, Display};
use std::error::Error as StdError;
use async_trait::async_trait;
use std::pin::Pin;
use futures::future::BoxFuture;

// Module declarations
pub mod circuit_breaker;
pub mod bulkhead;
pub mod retry;
pub mod health_check;
pub mod chaos;

// Re-exports
pub use circuit_breaker::{CircuitBreakerConfig, CircuitBreakerState, CircuitBreakerError, StandardCircuitBreaker};
pub use bulkhead::{BulkheadConfig, BulkheadError, StandardBulkhead};
pub use retry::{RetryConfig, RetryError, retry_with_exponential_backoff};

// Error types
#[derive(Debug)]
pub enum ResilienceError<E> {
    CircuitBreaker(CircuitBreakerError<E>),
    Bulkhead(BulkheadError<E>),
    Retry(RetryError<E>),
    Chaos(String),
    Operation(E),
    Other(anyhow::Error),
    Internal(String),
}

// Implement Clone for ResilienceError
impl<E> Clone for ResilienceError<E>
where
    E: Clone + StdError + 'static,
{
    fn clone(&self) -> Self {
        match self {
            Self::CircuitBreaker(e) => Self::CircuitBreaker(match e {
                CircuitBreakerError::Open => CircuitBreakerError::Open,
                CircuitBreakerError::ServiceError(e) => CircuitBreakerError::ServiceError(e.clone()),
                CircuitBreakerError::Internal(s) => CircuitBreakerError::Internal(s.clone()),
            }),
            Self::Bulkhead(e) => Self::Bulkhead(match e {
                BulkheadError::Full => BulkheadError::Full,
                BulkheadError::Timeout => BulkheadError::Timeout,
                BulkheadError::OperationError(e) => BulkheadError::OperationError(e.clone()),
            }),
            Self::Retry(e) => Self::Retry(match e {
                RetryError::MaxRetriesExceeded(e) => RetryError::MaxRetriesExceeded(e.clone()),
                RetryError::Aborted(e) => RetryError::Aborted(e.clone()),
                RetryError::Other(e) => RetryError::Other(e.clone()),
            }),
            Self::Chaos(s) => Self::Chaos(s.clone()),
            Self::Operation(e) => Self::Operation(e.clone()),
            Self::Other(e) => Self::Other(anyhow::Error::msg(format!("{}", e))),
            Self::Internal(s) => Self::Internal(s.clone()),
        }
    }
}

// Implement Clone for CircuitBreakerError
impl<E: Clone> Clone for CircuitBreakerError<E> {
    fn clone(&self) -> Self {
        match self {
            Self::Open => Self::Open,
            Self::ServiceError(e) => Self::ServiceError(e.clone()),
            Self::Internal(s) => Self::Internal(s.clone()),
        }
    }
}

// Implement Clone for BulkheadError
impl<E: Clone> Clone for BulkheadError<E> {
    fn clone(&self) -> Self {
        match self {
            Self::Full => Self::Full,
            Self::Timeout => Self::Timeout,
            Self::OperationError(e) => Self::OperationError(e.clone()),
        }
    }
}

// Implement Clone for RetryError
impl<E: Clone> Clone for RetryError<E> {
    fn clone(&self) -> Self {
        match self {
            Self::MaxRetriesExceeded(e) => Self::MaxRetriesExceeded(e.clone()),
            Self::Aborted(e) => Self::Aborted(e.clone()),
            Self::Other(e) => Self::Other(e.clone()),
        }
    }
}

impl<E> Display for ResilienceError<E>
where
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResilienceError::CircuitBreaker(e) => write!(f, "Circuit breaker error: {}", e),
            ResilienceError::Bulkhead(e) => write!(f, "Bulkhead error: {}", e),
            ResilienceError::Retry(e) => write!(f, "Retry error: {}", e),
            ResilienceError::Chaos(msg) => write!(f, "Chaos error: {}", msg),
            ResilienceError::Operation(e) => write!(f, "Operation error: {}", e),
            ResilienceError::Other(e) => write!(f, "Other error: {}", e),
            ResilienceError::Internal(s) => write!(f, "Internal error: {}", s),
        }
    }
}

impl<E> StdError for ResilienceError<E>
where
    E: StdError + 'static,
{
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ResilienceError::CircuitBreaker(e) => Some(e),
            ResilienceError::Bulkhead(e) => Some(e),
            ResilienceError::Retry(e) => Some(e),
            ResilienceError::Chaos(_) => None,
            ResilienceError::Operation(e) => Some(e),
            ResilienceError::Other(e) => Some(e.as_ref()),
            ResilienceError::Internal(_) => None,
        }
    }
}

/// Execute an operation with a circuit breaker
pub async fn with_circuit_breaker<T, E, F, Fut>(
    circuit_breaker: &impl CircuitBreaker<E>,
    operation: F,
) -> Result<T, ResilienceError<E>>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: StdError + Send + Sync + 'static,
{
    if let Err(e) = circuit_breaker.check().await {
        return Err(ResilienceError::CircuitBreaker(e));
    }

    match operation().await {
        Ok(result) => {
            circuit_breaker.on_success();
            Ok(result)
        }
        Err(error) => {
            circuit_breaker.on_error(&error);
            Err(ResilienceError::CircuitBreaker(CircuitBreakerError::ServiceError(error)))
        }
    }
}

/// Execute an operation with a bulkhead
pub async fn with_bulkhead<T, E, F, Fut>(
    bulkhead: &bulkhead::StandardBulkhead,
    operation: F,
) -> Result<T, ResilienceError<E>>
where
    F: FnOnce() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<T, E>> + Send,
    E: std::error::Error + Send + Sync + 'static,
    T: Send,
{
    match bulkhead.execute(operation).await {
        Ok(result) => Ok(result),
        Err(e) => Err(ResilienceError::Bulkhead(e)),
    }
}

/// Get a circuit breaker for a tenant
pub fn get_tenant_circuit_breaker(
    tenant_id: Option<&str>,
) -> Result<Arc<circuit_breaker::StandardCircuitBreaker>, String> {
    let tenant_id = tenant_id.unwrap_or("default");
    circuit_breaker::get_circuit_breaker(tenant_id)
}

/// Get a bulkhead for a tenant
pub fn get_tenant_bulkhead_wrapper(
    tenant_id: Option<&str>,
) -> Result<Arc<bulkhead::StandardBulkhead>, String> {
    let tenant_id = tenant_id.unwrap_or("default");
    Ok(bulkhead::get_tenant_bulkhead(tenant_id))
}

/// Execute an operation with resilience (circuit breaker + bulkhead + retry)
pub async fn with_resilience<T, E, F, Fut>(
    tenant_id: &str,
    service_name: &str,
    operation: F,
) -> Result<T, ResilienceError<E>>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<T, E>> + Send + 'static,
    E: std::error::Error + Send + Sync + 'static,
    T: Send + 'static,
{
    let circuit_breaker_name = format!("{}-{}", tenant_id, service_name);
    let bulkhead_name = format!("{}-{}", tenant_id, service_name);
    
    let circuit_breaker = circuit_breaker::get_circuit_breaker(&circuit_breaker_name)
        .map_err(|e| ResilienceError::Internal(format!("Failed to get circuit breaker: {}", e)))?;
    
    let bulkhead = bulkhead::get_tenant_bulkhead(tenant_id);
    
    // Create a new operation that returns a BoxFuture
    let boxed_operation = move || -> BoxFuture<'static, Result<T, E>> {
        let fut = operation();
        Box::pin(fut)
    };
    
    with_circuit_breaker_and_bulkhead(&*circuit_breaker, &*bulkhead, boxed_operation).await
}

/// Execute an operation with circuit breaker and bulkhead protection
pub async fn with_circuit_breaker_and_bulkhead<F, T, E>(
    circuit_breaker: &StandardCircuitBreaker,
    bulkhead: &StandardBulkhead,
    operation: F,
) -> Result<T, ResilienceError<E>>
where
    F: FnOnce() -> BoxFuture<'static, Result<T, E>> + Send + 'static,
    T: Send + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    // Check if circuit breaker is open
    match circuit_breaker.check().await {
        Ok(_) => {}, // Circuit is closed, continue
        Err(e) => return Err(ResilienceError::CircuitBreaker(e)),
    }

    // Use the bulkhead's execute method directly
    match bulkhead.execute(operation).await {
        Ok(result) => {
            // Use fully qualified syntax to specify the type
            <StandardCircuitBreaker as CircuitBreaker<E>>::on_success(circuit_breaker);
            Ok(result)
        },
        Err(BulkheadError::Full) => {
            Err(ResilienceError::Bulkhead(BulkheadError::Full))
        },
        Err(BulkheadError::Timeout) => {
            Err(ResilienceError::Bulkhead(BulkheadError::Timeout))
        },
        Err(BulkheadError::OperationError(err)) => {
            circuit_breaker.on_error(&err);
            Err(ResilienceError::Operation(err))
        },
    }
}

/// Execute an operation with tenant-specific resilience (circuit breaker + bulkhead)
pub async fn with_tenant_resilience<T, E, F, Fut>(
    tenant_id: &str,
    service_name: &str,
    operation: F,
) -> Result<T, ResilienceError<E>>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<T, E>> + Send + 'static,
    E: std::error::Error + Send + Sync + 'static,
    T: Send + 'static,
{
    let circuit_breaker_name = format!("{}-{}", tenant_id, service_name);
    let bulkhead_name = format!("{}-{}", tenant_id, service_name);
    
    // Use owned strings to avoid lifetime issues
    with_resilience(
        &circuit_breaker_name,
        &bulkhead_name,
        move || operation()
    ).await
}

/// Convert a ResilienceError<std::io::Error> to a ResilienceError<E>
pub fn convert_io_error<E>(error: ResilienceError<std::io::Error>) -> ResilienceError<E>
where
    E: From<std::io::Error> + StdError + 'static + Send + Sync,
{
    match error {
        ResilienceError::CircuitBreaker(CircuitBreakerError::Open) => 
            ResilienceError::CircuitBreaker(CircuitBreakerError::Open),
        ResilienceError::CircuitBreaker(CircuitBreakerError::ServiceError(io_err)) => 
            ResilienceError::CircuitBreaker(CircuitBreakerError::ServiceError(E::from(io_err))),
        ResilienceError::CircuitBreaker(CircuitBreakerError::Internal(s)) => 
            ResilienceError::CircuitBreaker(CircuitBreakerError::Internal(s)),
        ResilienceError::Bulkhead(BulkheadError::Full) => 
            ResilienceError::Bulkhead(BulkheadError::Full),
        ResilienceError::Bulkhead(BulkheadError::Timeout) => 
            ResilienceError::Bulkhead(BulkheadError::Timeout),
        ResilienceError::Bulkhead(BulkheadError::OperationError(io_err)) => 
            ResilienceError::Bulkhead(BulkheadError::OperationError(E::from(io_err))),
        ResilienceError::Retry(RetryError::MaxRetriesExceeded(io_err)) => 
            ResilienceError::Retry(RetryError::MaxRetriesExceeded(E::from(io_err))),
        ResilienceError::Retry(RetryError::Aborted(io_err)) => 
            ResilienceError::Retry(RetryError::Aborted(E::from(io_err))),
        ResilienceError::Retry(RetryError::Other(io_err)) => 
            ResilienceError::Retry(RetryError::Other(E::from(io_err))),
        ResilienceError::Chaos(s) => 
            ResilienceError::Chaos(s),
        ResilienceError::Operation(io_err) => 
            ResilienceError::Operation(E::from(io_err)),
        ResilienceError::Other(e) => 
            ResilienceError::Other(anyhow::Error::msg(format!("{}", e))),
        ResilienceError::Internal(s) => 
            ResilienceError::Internal(s),
    }
} 