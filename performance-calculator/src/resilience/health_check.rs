use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use async_trait::async_trait;
use tokio::task::JoinHandle;
use tokio::time;
use tracing::{info, warn, error};

/// Health check status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    
    /// Service is degraded but still operational
    Degraded,
    
    /// Service is unhealthy and not operational
    Unhealthy,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "Healthy"),
            HealthStatus::Degraded => write!(f, "Degraded"),
            HealthStatus::Unhealthy => write!(f, "Unhealthy"),
        }
    }
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Service name
    pub service_name: String,
    
    /// Health status
    pub status: HealthStatus,
    
    /// Additional details
    pub details: Option<String>,
    
    /// Timestamp
    pub timestamp: Instant,
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Check interval in seconds
    pub check_interval_seconds: u64,
    
    /// Timeout in seconds
    pub timeout_seconds: u64,
    
    /// Number of consecutive failures before marking as unhealthy
    pub failure_threshold: u32,
    
    /// Number of consecutive successes before marking as healthy
    pub success_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 60,
            timeout_seconds: 5,
            failure_threshold: 3,
            success_threshold: 2,
        }
    }
}

/// Health check trait
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Get the service name
    fn service_name(&self) -> &str;
    
    /// Perform a health check
    async fn check_health(&self) -> HealthStatus;
    
    /// Get additional details
    fn details(&self) -> Option<String> {
        None
    }
}

/// Health check registry
pub struct HealthCheckRegistry {
    health_checks: Arc<Mutex<HashMap<String, Arc<dyn HealthCheck>>>>,
    results: Arc<Mutex<HashMap<String, HealthCheckResult>>>,
    config: HealthCheckConfig,
    running: Arc<Mutex<bool>>,
    task_handle: Mutex<Option<JoinHandle<()>>>,
}

impl HealthCheckRegistry {
    /// Create a new health check registry
    pub fn new(config: HealthCheckConfig) -> Self {
        Self {
            health_checks: Arc::new(Mutex::new(HashMap::new())),
            results: Arc::new(Mutex::new(HashMap::new())),
            config,
            running: Arc::new(Mutex::new(false)),
            task_handle: Mutex::new(None),
        }
    }
    
    /// Create a new health check registry with default configuration
    pub fn default() -> Self {
        Self::new(HealthCheckConfig::default())
    }
    
    /// Register a health check
    pub fn register<H>(&self, health_check: H)
    where
        H: HealthCheck + 'static,
    {
        let service_name = health_check.service_name().to_string();
        let health_check = Arc::new(health_check);
        
        let mut health_checks = self.health_checks.lock().unwrap();
        health_checks.insert(service_name, health_check);
    }
    
    /// Unregister a health check
    pub fn unregister(&self, service_name: &str) {
        let mut health_checks = self.health_checks.lock().unwrap();
        health_checks.remove(service_name);
    }
    
    /// Get all health check results
    pub fn get_all_results(&self) -> HashMap<String, HealthCheckResult> {
        let results = self.results.lock().unwrap();
        results.clone()
    }
    
    /// Get a specific health check result
    pub fn get_result(&self, service_name: &str) -> Option<HealthCheckResult> {
        let results = self.results.lock().unwrap();
        results.get(service_name).cloned()
    }
    
    /// Get overall health status
    pub fn overall_status(&self) -> HealthStatus {
        let results = self.results.lock().unwrap();
        
        if results.is_empty() {
            return HealthStatus::Healthy;
        }
        
        let mut has_degraded = false;
        
        for result in results.values() {
            match result.status {
                HealthStatus::Unhealthy => return HealthStatus::Unhealthy,
                HealthStatus::Degraded => has_degraded = true,
                HealthStatus::Healthy => {},
            }
        }
        
        if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
    
    /// Start the health check monitor
    pub fn start_monitor(&self) {
        let mut running = self.running.lock().unwrap();
        
        if *running {
            return;
        }
        
        *running = true;
        
        let health_checks = self.health_checks.clone();
        let results = self.results.clone();
        let config = self.config.clone();
        let running_flag = self.running.clone();
        
        let handle = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(config.check_interval_seconds));
            
            loop {
                interval.tick().await;
                
                {
                    let is_running = *running_flag.lock().unwrap();
                    if !is_running {
                        break;
                    }
                }
                
                let health_checks_clone = {
                    let health_checks = health_checks.lock().unwrap();
                    health_checks.clone()
                };
                
                for (service_name, health_check) in health_checks_clone.iter() {
                    let status = match time::timeout(
                        Duration::from_secs(config.timeout_seconds),
                        health_check.check_health(),
                    ).await {
                        Ok(status) => status,
                        Err(_) => {
                            warn!("Health check for {} timed out", service_name);
                            HealthStatus::Unhealthy
                        },
                    };
                    
                    let details = health_check.details();
                    let timestamp = Instant::now();
                    
                    let mut results = results.lock().unwrap();
                    
                    let result = HealthCheckResult {
                        service_name: service_name.clone(),
                        status,
                        details,
                        timestamp,
                    };
                    
                    results.insert(service_name.clone(), result);
                    
                    info!("Health check for {}: {}", service_name, status);
                }
            }
        });
        
        let mut task_handle = self.task_handle.lock().unwrap();
        *task_handle = Some(handle);
    }
    
    /// Stop the health check monitor
    pub fn stop_monitor(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
        
        let mut task_handle = self.task_handle.lock().unwrap();
        if let Some(handle) = task_handle.take() {
            handle.abort();
        }
    }
}

/// Global health check registry
static mut HEALTH_CHECK_REGISTRY: Option<HealthCheckRegistry> = None;

/// Get the global health check registry
pub fn get_health_check_registry() -> &'static HealthCheckRegistry {
    unsafe {
        if HEALTH_CHECK_REGISTRY.is_none() {
            HEALTH_CHECK_REGISTRY = Some(HealthCheckRegistry::default());
        }
        
        HEALTH_CHECK_REGISTRY.as_ref().unwrap()
    }
}

/// Register a health check with the global registry
pub fn register_health_check<H>(health_check: H)
where
    H: HealthCheck + 'static,
{
    get_health_check_registry().register(health_check);
}

/// Start the global health check monitor
pub fn start_health_check_monitor() {
    get_health_check_registry().start_monitor();
}

/// Stop the global health check monitor
pub fn stop_health_check_monitor() {
    get_health_check_registry().stop_monitor();
}

/// Get overall health status from the global registry
pub fn get_overall_health_status() -> HealthStatus {
    get_health_check_registry().overall_status()
}

/// DynamoDB health check
pub struct DynamoDbHealthCheck {
    client: aws_sdk_dynamodb::Client,
    table_name: String,
}

impl DynamoDbHealthCheck {
    /// Create a new DynamoDB health check
    pub fn new(client: aws_sdk_dynamodb::Client, table_name: String) -> Self {
        Self {
            client,
            table_name,
        }
    }
}

#[async_trait]
impl HealthCheck for DynamoDbHealthCheck {
    fn service_name(&self) -> &str {
        "DynamoDB"
    }
    
    async fn check_health(&self) -> HealthStatus {
        match self.client.describe_table()
            .table_name(&self.table_name)
            .send()
            .await {
                Ok(_) => HealthStatus::Healthy,
                Err(e) => {
                    error!("DynamoDB health check failed: {}", e);
                    HealthStatus::Unhealthy
                },
            }
    }
    
    fn details(&self) -> Option<String> {
        Some(format!("Table: {}", self.table_name))
    }
}

/// SQS health check
pub struct SqsHealthCheck {
    client: aws_sdk_sqs::Client,
    queue_url: String,
}

impl SqsHealthCheck {
    /// Create a new SQS health check
    pub fn new(client: aws_sdk_sqs::Client, queue_url: String) -> Self {
        Self {
            client,
            queue_url,
        }
    }
}

#[async_trait]
impl HealthCheck for SqsHealthCheck {
    fn service_name(&self) -> &str {
        "SQS"
    }
    
    async fn check_health(&self) -> HealthStatus {
        match self.client.get_queue_attributes()
            .queue_url(&self.queue_url)
            .attribute_names(aws_sdk_sqs::model::QueueAttributeName::All)
            .send()
            .await {
                Ok(_) => HealthStatus::Healthy,
                Err(e) => {
                    error!("SQS health check failed: {}", e);
                    HealthStatus::Unhealthy
                },
            }
    }
    
    fn details(&self) -> Option<String> {
        Some(format!("Queue: {}", self.queue_url))
    }
} 