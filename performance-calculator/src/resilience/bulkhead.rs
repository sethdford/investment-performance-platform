use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use async_trait::async_trait;
use tokio::sync::{Semaphore, SemaphorePermit};
use tokio::time::timeout;
use tracing::{info, warn, error};
use thiserror::Error;
use std::error::Error;
use std::fmt::Debug;
use std::pin::Pin;
use std::future::Future;

/// Bulkhead error types
#[derive(Debug)]
pub enum BulkheadError<E> {
    /// Bulkhead is full
    Full,
    /// Operation timed out
    Timeout,
    /// Operation error
    OperationError(E),
}

impl<E: std::error::Error + 'static> std::error::Error for BulkheadError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BulkheadError::OperationError(e) => Some(e),
            _ => None,
        }
    }
}

impl<E: std::fmt::Display> std::fmt::Display for BulkheadError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BulkheadError::Full => write!(f, "Bulkhead is full"),
            BulkheadError::Timeout => write!(f, "Operation timed out"),
            BulkheadError::OperationError(e) => write!(f, "Operation error: {}", e),
        }
    }
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

/// Bulkhead trait for limiting concurrent operations
pub trait Bulkhead: Send + Sync {
    type Permit: Send;
    type Future: Future<Output = Result<Self::Permit, BulkheadError<Box<dyn Error + Send + Sync>>>> + Send;

    fn acquire(&self) -> Self::Future;
    fn release(&self, permit: Self::Permit);
}

/// Standard bulkhead implementation
pub struct StandardBulkhead {
    name: String,
    config: BulkheadConfig,
    execution_semaphore: Arc<Semaphore>,
    queue_semaphore: Arc<Semaphore>,
    metrics: Arc<Mutex<BulkheadMetrics>>,
}

impl StandardBulkhead {
    /// Create a new bulkhead with custom configuration
    pub fn new(name: String, config: BulkheadConfig) -> Self {
        Self {
            name,
            execution_semaphore: Arc::new(Semaphore::new(config.max_concurrent_calls)),
            queue_semaphore: Arc::new(Semaphore::new(config.max_concurrent_calls + config.max_queue_size)),
            metrics: Arc::new(Mutex::new(BulkheadMetrics::default())),
            config,
        }
    }
    
    /// Create a new bulkhead with default configuration
    pub fn default(name: String) -> Self {
        Self::new(name, BulkheadConfig::default())
    }
    
    /// Reset metrics for this bulkhead
    pub fn reset_metrics(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        *metrics = BulkheadMetrics::default();
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

impl Bulkhead for StandardBulkhead {
    type Permit = SemaphorePermit<'static>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Permit, BulkheadError<Box<dyn Error + Send + Sync>>>> + Send>>;

    fn acquire(&self) -> Self::Future {
        let execution_semaphore = self.execution_semaphore.clone();
        let metrics = self.metrics.clone();
        let name = self.name.clone();
        
        Box::pin(async move {
            match execution_semaphore.try_acquire() {
                Ok(permit) => {
                    // Update metrics
                    let mut metrics_guard = metrics.lock().unwrap();
                    metrics_guard.concurrent_executions += 1;
                    metrics_guard.last_execution = Some(Instant::now());
                    
                    // Convert to 'static lifetime - this is safe because the permit will be dropped
                    // before the semaphore is dropped
                    let permit = unsafe { std::mem::transmute(permit) };
                    Ok(permit)
                },
                Err(_) => {
                    warn!("Bulkhead {} is full", name);
                    Err(BulkheadError::Full)
                }
            }
        })
    }

    fn release(&self, _permit: Self::Permit) {
        // The permit is automatically released when dropped
        let mut metrics = self.metrics.lock().unwrap();
        metrics.concurrent_executions -= 1;
    }
}

impl StandardBulkhead {
    pub async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, BulkheadError<E>>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, E>> + Send,
        T: Send,
        E: Error + Send + Sync + 'static,
    {
        // Try to acquire a queue permit
        let _queue_permit = match self.acquire_queue_permit().await {
            Some(permit) => permit,
            None => {
                warn!("Bulkhead {} queue is full", self.name);
                return Err(BulkheadError::Full);
            }
        };
        
        // Try to acquire an execution permit
        let _execution_permit = match self.acquire_execution_permit().await {
            Some(permit) => permit,
            None => {
                warn!("Bulkhead {} execution semaphore is full", self.name);
                return Err(BulkheadError::Full);
            }
        };
        
        info!("Acquired permits for bulkhead {}", self.name);
        
        // Execute the operation with timeout
        let timeout_duration = Duration::from_secs(self.config.execution_timeout_seconds);
        match timeout(timeout_duration, operation()).await {
            Ok(result) => {
                match result {
                    Ok(value) => {
                        self.record_success();
                        Ok(value)
                    }
                    Err(error) => {
                        self.record_failure();
                        Err(BulkheadError::OperationError(error))
                    }
                }
            }
            Err(_) => {
                self.record_timeout();
                warn!("Operation timed out in bulkhead {}", self.name);
                Err(BulkheadError::Timeout)
            }
        }
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
        tenant_configs.insert(tenant_id.to_string(), config.clone());
        
        // If the bulkhead already exists, recreate it with the new config
        let mut bulkheads = self.bulkheads.lock().unwrap();
        if bulkheads.contains_key(tenant_id) {
            let config_for_bulkhead = config.clone();
            bulkheads.insert(tenant_id.to_string(), Arc::new(StandardBulkhead::new(tenant_id.to_string(), config_for_bulkhead)));
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
            let bulkhead = Arc::new(StandardBulkhead::new(tenant_id.to_string(), config));
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
            bulkhead.as_ref().reset_metrics();
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

/// Get a bulkhead for a specific service
pub fn get_bulkhead(service_name: &str) -> Result<Arc<StandardBulkhead>, String> {
    // For now, just create a new bulkhead with default config
    Ok(Arc::new(StandardBulkhead::new(
        service_name.to_string(),
        BulkheadConfig::default()
    )))
}

/// Set the configuration for a specific tenant
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

/// Check if combined protection (circuit breaker + bulkhead) should be used
pub fn should_use_combined_protection() -> bool {
    // This could be based on configuration or environment variables
    // For now, just return true
    true
} 