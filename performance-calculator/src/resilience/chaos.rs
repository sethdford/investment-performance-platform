use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use rand::{Rng, thread_rng};
use tokio::time::sleep;
use tracing::{info, warn, error};

/// Chaos testing configuration
#[derive(Debug, Clone)]
pub struct ChaosConfig {
    /// Whether chaos testing is enabled
    pub enabled: bool,
    
    /// Probability of injecting a failure (0.0 - 1.0)
    pub failure_probability: f64,
    
    /// Probability of injecting a delay (0.0 - 1.0)
    pub delay_probability: f64,
    
    /// Maximum delay to inject in milliseconds
    pub max_delay_ms: u64,
    
    /// Services to target for chaos testing
    pub target_services: Vec<String>,
    
    /// Tenant IDs to target for chaos testing (empty means all tenants)
    pub target_tenant_ids: Vec<String>,
}

impl Default for ChaosConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            failure_probability: 0.05,
            delay_probability: 0.1,
            max_delay_ms: 1000,
            target_services: vec![],
            target_tenant_ids: vec![],
        }
    }
}

/// Chaos testing metrics
#[derive(Debug, Clone)]
pub struct ChaosMetrics {
    /// Number of failures injected
    pub failures_injected: u64,
    
    /// Number of delays injected
    pub delays_injected: u64,
    
    /// Total delay injected in milliseconds
    pub total_delay_ms: u64,
    
    /// Last chaos event timestamp
    pub last_event: Option<Instant>,
    
    /// Failures injected per service
    pub failures_per_service: HashMap<String, u64>,
    
    /// Delays injected per service
    pub delays_per_service: HashMap<String, u64>,
}

impl Default for ChaosMetrics {
    fn default() -> Self {
        Self {
            failures_injected: 0,
            delays_injected: 0,
            total_delay_ms: 0,
            last_event: None,
            failures_per_service: HashMap::new(),
            delays_per_service: HashMap::new(),
        }
    }
}

/// Chaos testing service
pub struct ChaosService {
    config: Arc<Mutex<ChaosConfig>>,
    metrics: Arc<Mutex<ChaosMetrics>>,
}

impl ChaosService {
    /// Create a new chaos testing service
    pub fn new(config: ChaosConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            metrics: Arc::new(Mutex::new(ChaosMetrics::default())),
        }
    }
    
    /// Create a new chaos testing service with default configuration
    pub fn default() -> Self {
        Self::new(ChaosConfig::default())
    }
    
    /// Get the current configuration
    pub fn get_config(&self) -> ChaosConfig {
        self.config.lock().unwrap().clone()
    }
    
    /// Set the configuration
    pub fn set_config(&self, config: ChaosConfig) {
        let mut cfg = self.config.lock().unwrap();
        *cfg = config;
    }
    
    /// Get the current metrics
    pub fn get_metrics(&self) -> ChaosMetrics {
        self.metrics.lock().unwrap().clone()
    }
    
    /// Reset the metrics
    pub fn reset_metrics(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        *metrics = ChaosMetrics::default();
        info!("Chaos testing metrics reset");
    }
    
    /// Check if chaos should be injected for a service and tenant
    fn should_inject_chaos(&self, service: &str, tenant_id: Option<&str>) -> bool {
        let config = self.config.lock().unwrap();
        
        // Check if chaos testing is enabled
        if !config.enabled {
            return false;
        }
        
        // Check if the service is targeted
        if !config.target_services.is_empty() && !config.target_services.contains(&service.to_string()) {
            return false;
        }
        
        // Check if the tenant is targeted
        if let Some(tenant_id) = tenant_id {
            if !config.target_tenant_ids.is_empty() && !config.target_tenant_ids.contains(&tenant_id.to_string()) {
                return false;
            }
        }
        
        true
    }
    
    /// Maybe inject a failure
    pub fn maybe_inject_failure(&self, service: &str, tenant_id: Option<&str>) -> bool {
        // Check if chaos should be injected
        if !self.should_inject_chaos(service, tenant_id) {
            return false;
        }
        
        // Get the failure probability
        let failure_probability = self.config.lock().unwrap().failure_probability;
        
        // Generate a random number
        let random = thread_rng().gen_range(0.0..1.0);
        
        // Check if a failure should be injected
        if random < failure_probability {
            // Update metrics
            let mut metrics = self.metrics.lock().unwrap();
            metrics.failures_injected += 1;
            metrics.last_event = Some(Instant::now());
            
            // Update service-specific metrics
            *metrics.failures_per_service.entry(service.to_string()).or_insert(0) += 1;
            
            info!("Injecting failure for service: {}, tenant: {:?}", service, tenant_id);
            
            true
        } else {
            false
        }
    }
    
    /// Maybe inject a delay
    pub async fn maybe_inject_delay(&self, service: &str, tenant_id: Option<&str>) {
        // Check if chaos should be injected
        if !self.should_inject_chaos(service, tenant_id) {
            return;
        }
        
        // Get the delay probability and maximum delay
        let config = self.config.lock().unwrap();
        let delay_probability = config.delay_probability;
        let max_delay_ms = config.max_delay_ms;
        
        // Generate a random number
        let random = thread_rng().gen_range(0.0..1.0);
        
        // Check if a delay should be injected
        if random < delay_probability {
            // Generate a random delay
            let delay_ms = thread_rng().gen_range(1..=max_delay_ms);
            
            // Update metrics
            let mut metrics = self.metrics.lock().unwrap();
            metrics.delays_injected += 1;
            metrics.total_delay_ms += delay_ms;
            metrics.last_event = Some(Instant::now());
            
            // Update service-specific metrics
            *metrics.delays_per_service.entry(service.to_string()).or_insert(0) += 1;
            
            info!("Injecting delay of {}ms for service: {}, tenant: {:?}", delay_ms, service, tenant_id);
            
            // Sleep for the specified delay
            sleep(Duration::from_millis(delay_ms)).await;
        }
    }
}

/// Global chaos testing service
static mut CHAOS_SERVICE: Option<ChaosService> = None;

/// Get the global chaos testing service
pub fn get_chaos_service() -> &'static ChaosService {
    unsafe {
        if CHAOS_SERVICE.is_none() {
            CHAOS_SERVICE = Some(ChaosService::default());
        }
        
        CHAOS_SERVICE.as_ref().unwrap()
    }
}

/// Enable chaos testing with the given configuration
pub fn enable_chaos(config: ChaosConfig) {
    let mut cfg = config;
    cfg.enabled = true;
    get_chaos_service().set_config(cfg.clone());
    info!("Chaos testing enabled with configuration: {:?}", cfg);
}

/// Disable chaos testing
pub fn disable_chaos_testing() {
    let mut config = get_chaos_service().get_config();
    config.enabled = false;
    get_chaos_service().set_config(config);
    info!("Chaos testing disabled");
}

/// Maybe inject a failure for a service and tenant
pub fn maybe_inject_failure(service: &str, tenant_id: Option<&str>) -> bool {
    get_chaos_service().maybe_inject_failure(service, tenant_id)
}

/// Maybe inject a delay for a service and tenant
pub async fn maybe_inject_delay(service: &str, tenant_id: Option<&str>) {
    get_chaos_service().maybe_inject_delay(service, tenant_id).await
}

/// Convenience function to execute an operation with chaos testing
pub async fn with_chaos<T, E, F, Fut>(
    service: &str,
    tenant_id: Option<&str>,
    f: F,
) -> Result<T, E>
where
    F: FnOnce() -> Fut + Send,
    Fut: std::future::Future<Output = Result<T, E>> + Send,
    E: std::error::Error + From<std::io::Error> + Send + Sync + 'static,
{
    // Maybe inject a delay
    maybe_inject_delay(service, tenant_id).await;
    
    // Maybe inject a failure
    if maybe_inject_failure(service, tenant_id) {
        return Err(E::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Chaos-induced failure for service: {}", service),
        )));
    }
    
    // Execute the operation
    f().await
} 