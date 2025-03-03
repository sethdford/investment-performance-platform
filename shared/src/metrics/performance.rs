//! Performance metrics collection

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{info, debug};

/// Performance metric
#[derive(Debug, Clone)]
pub struct PerformanceMetric {
    /// Operation name
    pub operation: String,
    /// Duration in milliseconds
    pub duration_ms: f64,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional labels
    pub labels: HashMap<String, String>,
}

/// Performance metrics collector
#[derive(Debug, Clone)]
pub struct PerformanceMetricsCollector {
    /// Metrics
    metrics: Arc<Mutex<Vec<PerformanceMetric>>>,
    /// Whether metrics collection is enabled
    enabled: bool,
}

impl PerformanceMetricsCollector {
    /// Create a new performance metrics collector
    pub fn new(enabled: bool) -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
            enabled,
        }
    }
    
    /// Record a performance metric
    pub fn record(&self, operation: &str, duration: Duration, labels: Option<HashMap<String, String>>) {
        if !self.enabled {
            return;
        }
        
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.push(PerformanceMetric {
                operation: operation.to_string(),
                duration_ms: duration.as_secs_f64() * 1000.0,
                timestamp: chrono::Utc::now(),
                labels: labels.unwrap_or_default(),
            });
        }
    }
    
    /// Measure the performance of a function
    pub fn measure<F, T>(&self, operation: &str, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        if !self.enabled {
            return f();
        }
        
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        
        self.record(operation, duration, None);
        
        result
    }
    
    /// Measure the performance of an async function
    pub async fn measure_async<F, Fut, T>(&self, operation: &str, f: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        if !self.enabled {
            return f().await;
        }
        
        let start = Instant::now();
        let result = f().await;
        let duration = start.elapsed();
        
        self.record(operation, duration, None);
        
        result
    }
    
    /// Get all metrics
    pub fn get_metrics(&self) -> Vec<PerformanceMetric> {
        if let Ok(metrics) = self.metrics.lock() {
            metrics.clone()
        } else {
            Vec::new()
        }
    }
    
    /// Clear all metrics
    pub fn clear(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.clear();
        }
    }
    
    /// Log metrics summary
    pub fn log_summary(&self) {
        if !self.enabled {
            return;
        }
        
        let metrics = self.get_metrics();
        
        if metrics.is_empty() {
            return;
        }
        
        info!("Performance metrics summary:");
        
        // Group metrics by operation
        let mut operation_metrics: HashMap<String, Vec<f64>> = HashMap::new();
        
        for metric in &metrics {
            operation_metrics
                .entry(metric.operation.clone())
                .or_insert_with(Vec::new)
                .push(metric.duration_ms);
        }
        
        // Calculate statistics for each operation
        for (operation, durations) in &operation_metrics {
            let count = durations.len();
            let total = durations.iter().sum::<f64>();
            let avg = total / count as f64;
            let min = durations.iter().min().copied().unwrap_or(0.0);
            let max = durations.iter().max().copied().unwrap_or(0.0);
            
            info!("Operation '{}':", operation);
            info!("  Count: {}", count);
            info!("  Total: {:.2} ms", total);
            info!("  Average: {:.2} ms", avg);
            info!("  Minimum: {:.2} ms", min);
            info!("  Maximum: {:.2} ms", max);
        }
    }
} 