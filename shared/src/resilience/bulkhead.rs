//! Bulkhead pattern implementation

use std::future::Future;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::sync::Semaphore;
use tracing::{info, warn, error};

/// Bulkhead configuration
#[derive(Debug, Clone)]
pub struct BulkheadConfig {
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    /// Maximum queue size
    pub max_queue_size: usize,
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 10,
            max_queue_size: 10,
        }
    }
}

/// Bulkhead
#[derive(Debug, Clone)]
pub struct Bulkhead {
    /// Semaphore for limiting concurrent requests
    semaphore: Arc<Semaphore>,
    /// Configuration
    config: BulkheadConfig,
}

impl Bulkhead {
    /// Create a new bulkhead
    pub fn new(config: BulkheadConfig) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_requests)),
            config,
        }
    }
    
    /// Acquire a permit
    pub async fn acquire(&self) -> Result<tokio::sync::SemaphorePermit, &'static str> {
        match self.semaphore.try_acquire() {
            Ok(permit) => Ok(permit),
            Err(_) => {
                // Queue is full
                if self.semaphore.available_permits() == 0 {
                    Err("Bulkhead is full")
                } else {
                    // Try to acquire with timeout
                    match tokio::time::timeout(
                        tokio::time::Duration::from_millis(100),
                        self.semaphore.acquire(),
                    ).await {
                        Ok(Ok(permit)) => Ok(permit),
                        _ => Err("Bulkhead queue is full or timeout occurred"),
                    }
                }
            }
        }
    }
}

// Global bulkheads
lazy_static::lazy_static! {
    static ref BULKHEADS: Mutex<HashMap<String, Bulkhead>> = Mutex::new(HashMap::new());
}

/// Get or create a bulkhead
pub fn get_bulkhead(name: &str, config: BulkheadConfig) -> Bulkhead {
    let mut bulkheads = BULKHEADS.lock().unwrap();
    
    if let Some(bulkhead) = bulkheads.get(name) {
        bulkhead.clone()
    } else {
        let bulkhead = Bulkhead::new(config);
        bulkheads.insert(name.to_string(), bulkhead.clone());
        bulkhead
    }
}

/// Execute a function with bulkhead
pub async fn with_bulkhead<F, Fut, T, E>(
    operation_name: &str,
    config: BulkheadConfig,
    f: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let bulkhead = get_bulkhead(operation_name, config);
    
    let permit = match bulkhead.acquire().await {
        Ok(permit) => permit,
        Err(e) => {
            error!("Failed to acquire bulkhead permit for '{}': {}", operation_name, e);
            return Err(format!("Bulkhead rejected request for operation '{}': {}", operation_name, e).into());
        }
    };
    
    // Execute the function with the permit
    let result = f().await;
    
    // Permit is automatically released when dropped
    drop(permit);
    
    result
} 