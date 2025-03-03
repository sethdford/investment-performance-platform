use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use async_trait::async_trait;
use tokio::sync::{Semaphore, SemaphorePermit};
use tokio::time::timeout;
use tracing::{info, warn, error};
use thiserror::Error;

/// Bulkhead error
#[derive(Error, Debug)]
pub enum BulkheadError<E> {
    #[error("Bulkhead is full")]
    Full,
    
    #[error("Bulkhead operation timed out")]
    Timeout,
    
    #[error("Underlying operation error: {0}")]
    OperationError(E),
}

/// Bulkhead configuration
#[derive(Debug, Clone)]
pub struct BulkheadConfig {
    /// Maximum concurrent executions
    pub max_concurrent_calls: usize,
    
    /// Maximum queue size
    pub max_queue_size: usize,
    
    /// Execution timeout in seconds
    pub execution_timeout_seconds: u64,
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        Self {
            max_concurrent_calls: 10,
            max_queue_size: 20,
            execution_timeout_seconds: 30,
        }
    }
}

/// Bulkhead metrics
#[derive(Debug, Clone)]
pub struct BulkheadMetrics {
    /// Current number of concurrent executions
    pub concurrent_executions: usize,
    
    /// Current queue size
    pub queue_size: usize,
    
    /// Total number of successful executions
    pub successful_executions: u64,
    
    /// Total number of failed executions
    pub failed_executions: u64,
    
    /// Total number of rejected executions (due to full bulkhead)
    pub rejected_executions: u64,
    
    /// Total number of timed out executions
    pub timed_out_executions: u64,
    
    /// Last execution timestamp
    pub last_execution: Option<Instant>,
}

impl Default for BulkheadMetrics {
    fn default() -> Self {
        Self {
            concurrent_executions: 0,
            queue_size: 0,
            successful_executions: 0,
            failed_executions: 0,
            rejected_executions: 0,
            timed_out_executions: 0,
            last_execution: None,
        }
    }
}

/// Bulkhead trait
#[async_trait]
pub trait Bulkhead<T, E> {
    /// Execute a function with bulkhead protection
    async fn execute<F, Fut>(&self, f: F) -> Result<T, BulkheadError<E>>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, E>> + Send;
    
    /// Get current bulkhead metrics
    fn metrics(&self) -> BulkheadMetrics;
    
    /// Reset the bulkhead metrics
    fn reset_metrics(&self);
}

/// Standard bulkhead implementation
pub struct StandardBulkhead {
    config: BulkheadConfig,
    execution_semaphore: Semaphore,
    queue_semaphore: Semaphore,
    metrics: Arc<Mutex<BulkheadMetrics>>,
}

impl StandardBulkhead {
    /// Create a new bulkhead
    pub fn new(config: BulkheadConfig) -> Self {
        Self {
            execution_semaphore: Semaphore::new(config.max_concurrent_calls),
            queue_semaphore: Semaphore::new(config.max_concurrent_calls + config.max_queue_size),
            metrics: Arc::new(Mutex::new(BulkheadMetrics::default())),
            config,
        }
    }
    
    /// Create a new bulkhead with default configuration
    pub fn default() -> Self {
        Self::new(BulkheadConfig::default())
    }
    
    /// Acquire a permit from the queue semaphore
    async fn acquire_queue_permit(&self) -> Option<SemaphorePermit<'_>> {
        match self.queue_semaphore.try_acquire() {
            Ok(permit) => {
                // Update metrics
                let mut metrics = self.metrics.lock().unwrap();
                metrics.queue_size += 1;
                Some(permit)
            },
            Err(_) => {
                // Update metrics
                let mut metrics = self.metrics.lock().unwrap();
                metrics.rejected_executions += 1;
                None
            },
        }
    }
    
    /// Acquire a permit from the execution semaphore
    async fn acquire_execution_permit(&self) -> Option<SemaphorePermit<'_>> {
        match self.execution_semaphore.acquire().await {
            Ok(permit) => {
                // Update metrics
                let mut metrics = self.metrics.lock().unwrap();
                metrics.queue_size -= 1;
                metrics.concurrent_executions += 1;
                metrics.last_execution = Some(Instant::now());
                Some(permit)
            },
            Err(_) => None,
        }
    }
    
    /// Record a successful execution
    fn record_success(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.concurrent_executions -= 1;
        metrics.successful_executions += 1;
    }
    
    /// Record a failed execution
    fn record_failure(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.concurrent_executions -= 1;
        metrics.failed_executions += 1;
    }
    
    /// Record a timed out execution
    fn record_timeout(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.concurrent_executions -= 1;
        metrics.timed_out_executions += 1;
    }
}

#[async_trait]
impl<T, E> Bulkhead<T, E> for StandardBulkhead
where
    T: Send,
    E: std::error::Error + Send + Sync + 'static,
{
    async fn execute<F, Fut>(&self, f: F) -> Result<T, BulkheadError<E>>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, E>> + Send,
    {
        // Try to acquire a queue permit
        let _queue_permit = match self.acquire_queue_permit().await {
            Some(permit) => permit,
            None => return Err(BulkheadError::Full),
        };
        
        // Try to acquire an execution permit
        let _execution_permit = match self.acquire_execution_permit().await {
            Some(permit) => permit,
            None => return Err(BulkheadError::Full),
        };
        
        // Execute the function with timeout
        match timeout(
            Duration::from_secs(self.config.execution_timeout_seconds),
            f(),
        ).await {
            Ok(Ok(result)) => {
                // Record success
                self.record_success();
                Ok(result)
            },
            Ok(Err(err)) => {
                // Record failure
                self.record_failure();
                Err(BulkheadError::OperationError(err))
            },
            Err(_) => {
                // Record timeout
                self.record_timeout();
                Err(BulkheadError::Timeout)
            },
        }
    }
    
    fn metrics(&self) -> BulkheadMetrics {
        self.metrics.lock().unwrap().clone()
    }
    
    fn reset_metrics(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        *metrics = BulkheadMetrics {
            concurrent_executions: metrics.concurrent_executions,
            queue_size: metrics.queue_size,
            ..BulkheadMetrics::default()
        };
        info!("Bulkhead metrics reset");
    }
}

/// Tenant-specific bulkhead registry
pub struct TenantBulkheadRegistry {
    bulkheads: Arc<Mutex<HashMap<String, Arc<StandardBulkhead>>>>,
    default_config: BulkheadConfig,
    tenant_configs: Arc<Mutex<HashMap<String, BulkheadConfig>>>,
}

impl TenantBulkheadRegistry {
    /// Create a new tenant bulkhead registry
    pub fn new(default_config: BulkheadConfig) -> Self {
        Self {
            bulkheads: Arc::new(Mutex::new(HashMap::new())),
            default_config,
            tenant_configs: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Create a new tenant bulkhead registry with default configuration
    pub fn default() -> Self {
        Self::new(BulkheadConfig::default())
    }
    
    /// Set the configuration for a specific tenant
    pub fn set_tenant_config(&self, tenant_id: &str, config: BulkheadConfig) {
        let mut tenant_configs = self.tenant_configs.lock().unwrap();
        tenant_configs.insert(tenant_id.to_string(), config);
        
        // If the bulkhead already exists, recreate it with the new config
        let mut bulkheads = self.bulkheads.lock().unwrap();
        if bulkheads.contains_key(tenant_id) {
            bulkheads.insert(tenant_id.to_string(), Arc::new(StandardBulkhead::new(config)));
        }
    }
    
    /// Get the configuration for a specific tenant
    pub fn get_tenant_config(&self, tenant_id: &str) -> BulkheadConfig {
        let tenant_configs = self.tenant_configs.lock().unwrap();
        tenant_configs.get(tenant_id)
            .cloned()
            .unwrap_or_else(|| self.default_config.clone())
    }
    
    /// Get or create a bulkhead for a specific tenant
    pub fn get_or_create(&self, tenant_id: &str) -> Arc<StandardBulkhead> {
        let mut bulkheads = self.bulkheads.lock().unwrap();
        
        if let Some(bulkhead) = bulkheads.get(tenant_id) {
            bulkhead.clone()
        } else {
            let config = self.get_tenant_config(tenant_id);
            let bulkhead = Arc::new(StandardBulkhead::new(config));
            bulkheads.insert(tenant_id.to_string(), bulkhead.clone());
            bulkhead
        }
    }
    
    /// Get all bulkheads
    pub fn get_all(&self) -> HashMap<String, Arc<StandardBulkhead>> {
        let bulkheads = self.bulkheads.lock().unwrap();
        bulkheads.clone()
    }
    
    /// Reset metrics for all bulkheads
    pub fn reset_all_metrics(&self) {
        let bulkheads = self.bulkheads.lock().unwrap();
        
        for (tenant_id, bulkhead) in bulkheads.iter() {
            info!("Resetting bulkhead metrics for tenant: {}", tenant_id);
            bulkhead.reset_metrics();
        }
    }
    
    /// Remove a bulkhead for a specific tenant
    pub fn remove(&self, tenant_id: &str) {
        let mut bulkheads = self.bulkheads.lock().unwrap();
        bulkheads.remove(tenant_id);
        
        let mut tenant_configs = self.tenant_configs.lock().unwrap();
        tenant_configs.remove(tenant_id);
    }
}

/// Global tenant bulkhead registry
static mut TENANT_BULKHEAD_REGISTRY: Option<TenantBulkheadRegistry> = None;

/// Get the global tenant bulkhead registry
pub fn get_tenant_bulkhead_registry() -> &'static TenantBulkheadRegistry {
    unsafe {
        if TENANT_BULKHEAD_REGISTRY.is_none() {
            TENANT_BULKHEAD_REGISTRY = Some(TenantBulkheadRegistry::default());
        }
        
        TENANT_BULKHEAD_REGISTRY.as_ref().unwrap()
    }
}

/// Get a bulkhead for a specific tenant from the global registry
pub fn get_tenant_bulkhead(tenant_id: &str) -> Arc<StandardBulkhead> {
    get_tenant_bulkhead_registry().get_or_create(tenant_id)
}

/// Set the configuration for a specific tenant in the global registry
pub fn set_tenant_bulkhead_config(tenant_id: &str, config: BulkheadConfig) {
    get_tenant_bulkhead_registry().set_tenant_config(tenant_id, config);
}

/// Configure tenant bulkheads based on subscription tier
pub fn configure_tenant_bulkheads_from_tier(
    tenant_id: &str,
    tier: &crate::calculations::tenant::SubscriptionTier,
) {
    let config = match tier {
        crate::calculations::tenant::SubscriptionTier::Free => BulkheadConfig {
            max_concurrent_calls: 2,
            max_queue_size: 5,
            execution_timeout_seconds: 10,
        },
        crate::calculations::tenant::SubscriptionTier::Basic => BulkheadConfig {
            max_concurrent_calls: 5,
            max_queue_size: 10,
            execution_timeout_seconds: 20,
        },
        crate::calculations::tenant::SubscriptionTier::Professional => BulkheadConfig {
            max_concurrent_calls: 10,
            max_queue_size: 20,
            execution_timeout_seconds: 30,
        },
        crate::calculations::tenant::SubscriptionTier::Enterprise => BulkheadConfig {
            max_concurrent_calls: 20,
            max_queue_size: 40,
            execution_timeout_seconds: 60,
        },
        crate::calculations::tenant::SubscriptionTier::Custom => BulkheadConfig::default(),
    };
    
    set_tenant_bulkhead_config(tenant_id, config);
} 