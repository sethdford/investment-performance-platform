// Resilience module for the performance calculator
// This module provides circuit breakers, retry mechanisms, health checks, bulkheads, and chaos testing
// to improve the reliability and fault tolerance of the application.

pub mod circuit_breaker;
pub mod retry;
pub mod health_check;
pub mod bulkhead;
pub mod chaos;

pub use circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError, CircuitBreakerMetrics,
    CircuitBreakerState, StandardCircuitBreaker, get_circuit_breaker, register_circuit_breaker,
};

pub use retry::{
    RetryConfig, RetryError, RetryPolicy, RetryableError, RetryableResult,
    with_retry, with_retry_and_circuit_breaker,
};

pub use health_check::{
    HealthCheck, HealthCheckConfig, HealthCheckError, HealthCheckMetrics, HealthCheckMonitor,
    HealthCheckRegistry, HealthCheckResult, HealthStatus, ServiceHealth, register_health_check,
    get_health_check_registry, get_health_status,
};

pub use bulkhead::{
    Bulkhead, BulkheadConfig, BulkheadError, BulkheadMetrics, StandardBulkhead,
    TenantBulkheadRegistry, get_tenant_bulkhead, get_tenant_bulkhead_registry,
    set_tenant_bulkhead_config, configure_tenant_bulkheads_from_tier,
};

pub use chaos::{
    ChaosConfig, ChaosMetrics, ChaosService, get_chaos_service,
    enable_chaos_testing, disable_chaos_testing, maybe_inject_failure, maybe_inject_delay,
    with_chaos,
};

// Convenience function to execute an operation with a tenant-specific bulkhead
pub async fn with_tenant_bulkhead<T, E, F, Fut>(
    tenant_id: &str,
    f: F,
) -> Result<T, bulkhead::BulkheadError<E>>
where
    F: FnOnce() -> Fut + Send,
    Fut: std::future::Future<Output = Result<T, E>> + Send,
    T: Send,
    E: std::error::Error + Send + Sync + 'static,
{
    let bulkhead = bulkhead::get_tenant_bulkhead(tenant_id);
    bulkhead.execute(f).await
}

// Convenience function to execute an operation with a tenant-specific bulkhead and retry
pub async fn with_tenant_bulkhead_and_retry<T, E, F, Fut>(
    tenant_id: &str,
    retry_config: retry::RetryConfig,
    f: F,
) -> Result<T, retry::RetryError<bulkhead::BulkheadError<E>>>
where
    F: Fn() -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = Result<T, E>> + Send,
    T: Send + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    let bulkhead = bulkhead::get_tenant_bulkhead(tenant_id);
    
    retry::with_retry(retry_config, || async {
        bulkhead.execute(|| f()).await
    }).await
}

// Convenience function to execute an operation with a tenant-specific bulkhead, retry, and circuit breaker
pub async fn with_tenant_bulkhead_retry_and_circuit_breaker<T, E, F, Fut>(
    tenant_id: &str,
    circuit_name: &str,
    retry_config: retry::RetryConfig,
    f: F,
) -> Result<T, retry::RetryError<circuit_breaker::CircuitBreakerError<bulkhead::BulkheadError<E>>>>
where
    F: Fn() -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = Result<T, E>> + Send,
    T: Send + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    let bulkhead = bulkhead::get_tenant_bulkhead(tenant_id);
    let circuit_breaker = circuit_breaker::get_circuit_breaker(circuit_name);
    
    retry::with_retry(retry_config, || async {
        circuit_breaker.execute(|| async {
            bulkhead.execute(|| f()).await
        }).await
    }).await
}

// Convenience function to execute an operation with chaos testing, tenant-specific bulkhead, retry, and circuit breaker
pub async fn with_chaos_bulkhead_retry_and_circuit_breaker<T, E, F, Fut>(
    service: &str,
    tenant_id: &str,
    circuit_name: &str,
    retry_config: retry::RetryConfig,
    f: F,
) -> Result<T, retry::RetryError<circuit_breaker::CircuitBreakerError<bulkhead::BulkheadError<E>>>>
where
    F: Fn() -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = Result<T, E>> + Send,
    T: Send + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    // Create a wrapper function that includes chaos testing
    let chaos_fn = move || async {
        chaos::with_chaos(service, Some(tenant_id), || f()).await
    };
    
    // Use the existing function with the chaos wrapper
    with_tenant_bulkhead_retry_and_circuit_breaker(
        tenant_id,
        circuit_name,
        retry_config,
        chaos_fn,
    ).await
} 