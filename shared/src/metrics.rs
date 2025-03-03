//! Metrics collection

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{info, warn, error};

/// Metric type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetricType {
    /// Counter metric
    Counter,
    /// Gauge metric
    Gauge,
    /// Histogram metric
    Histogram,
}

/// Metric value
#[derive(Debug, Clone)]
pub enum MetricValue {
    /// Counter value
    Counter(u64),
    /// Gauge value
    Gauge(f64),
    /// Histogram value
    Histogram(Vec<f64>),
}

/// Metric
#[derive(Debug, Clone)]
pub struct Metric {
    /// Metric name
    pub name: String,
    /// Metric type
    pub metric_type: MetricType,
    /// Metric value
    pub value: MetricValue,
    /// Metric labels
    pub labels: HashMap<String, String>,
}

/// Metrics collector
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    /// Metrics
    metrics: Arc<Mutex<HashMap<String, Metric>>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Increment a counter
    pub fn increment_counter(&self, name: &str, labels: Option<HashMap<String, String>>, value: u64) {
        let mut metrics = match self.metrics.lock() {
            Ok(metrics) => metrics,
            Err(e) => {
                error!("Failed to acquire lock for metrics: {}", e);
                return;
            }
        };
        
        let key = self.build_key(name, labels.as_ref());
        
        match metrics.get_mut(&key) {
            Some(metric) => {
                match &mut metric.value {
                    MetricValue::Counter(count) => {
                        *count += value;
                    },
                    _ => {
                        warn!("Metric {} is not a counter", name);
                    }
                }
            },
            None => {
                let metric = Metric {
                    name: name.to_string(),
                    metric_type: MetricType::Counter,
                    value: MetricValue::Counter(value),
                    labels: labels.unwrap_or_default(),
                };
                
                metrics.insert(key, metric);
            }
        }
    }
    
    /// Set a gauge
    pub fn set_gauge(&self, name: &str, labels: Option<HashMap<String, String>>, value: f64) {
        let mut metrics = match self.metrics.lock() {
            Ok(metrics) => metrics,
            Err(e) => {
                error!("Failed to acquire lock for metrics: {}", e);
                return;
            }
        };
        
        let key = self.build_key(name, labels.as_ref());
        
        let metric = Metric {
            name: name.to_string(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(value),
            labels: labels.unwrap_or_default(),
        };
        
        metrics.insert(key, metric);
    }
    
    /// Record a histogram value
    pub fn record_histogram(&self, name: &str, labels: Option<HashMap<String, String>>, value: f64) {
        let mut metrics = match self.metrics.lock() {
            Ok(metrics) => metrics,
            Err(e) => {
                error!("Failed to acquire lock for metrics: {}", e);
                return;
            }
        };
        
        let key = self.build_key(name, labels.as_ref());
        
        match metrics.get_mut(&key) {
            Some(metric) => {
                match &mut metric.value {
                    MetricValue::Histogram(values) => {
                        values.push(value);
                    },
                    _ => {
                        warn!("Metric {} is not a histogram", name);
                    }
                }
            },
            None => {
                let metric = Metric {
                    name: name.to_string(),
                    metric_type: MetricType::Histogram,
                    value: MetricValue::Histogram(vec![value]),
                    labels: labels.unwrap_or_default(),
                };
                
                metrics.insert(key, metric);
            }
        }
    }
    
    /// Record request duration
    pub fn record_request_duration(&self, path: &str, method: &str, status_code: u16, duration: Duration) {
        let mut labels = HashMap::new();
        labels.insert("path".to_string(), path.to_string());
        labels.insert("method".to_string(), method.to_string());
        labels.insert("status_code".to_string(), status_code.to_string());
        
        self.record_histogram("request_duration_seconds", Some(labels), duration.as_secs_f64());
    }
    
    /// Increment request count
    pub fn increment_request_count(&self, path: &str, method: &str, status_code: u16) {
        let mut labels = HashMap::new();
        labels.insert("path".to_string(), path.to_string());
        labels.insert("method".to_string(), method.to_string());
        labels.insert("status_code".to_string(), status_code.to_string());
        
        self.increment_counter("request_count", Some(labels), 1);
    }
    
    /// Get all metrics
    pub fn get_metrics(&self) -> HashMap<String, Metric> {
        match self.metrics.lock() {
            Ok(metrics) => metrics.clone(),
            Err(e) => {
                error!("Failed to acquire lock for metrics: {}", e);
                HashMap::new()
            }
        }
    }
    
    /// Build a metric key
    fn build_key(&self, name: &str, labels: Option<&HashMap<String, String>>) -> String {
        match labels {
            Some(labels) if !labels.is_empty() => {
                let labels_str = labels.iter()
                    .map(|(k, v)| format!("{}=\"{}\"", k, v))
                    .collect::<Vec<_>>()
                    .join(",");
                
                format!("{}{{{}}}",  name, labels_str)
            },
            _ => name.to_string(),
        }
    }
}

/// Create a metrics middleware
pub struct MetricsMiddleware {
    /// Metrics collector
    collector: MetricsCollector,
}

impl MetricsMiddleware {
    /// Create a new metrics middleware
    pub fn new(collector: MetricsCollector) -> Self {
        Self { collector }
    }
    
    /// Track a request
    pub fn track_request<F, Fut, T, E>(&self, method: &str, path: &str, f: F) -> impl std::future::Future<Output = Result<T, E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        let collector = self.collector.clone();
        let method = method.to_string();
        let path = path.to_string();
        let start = Instant::now();
        
        async move {
            let result = f().await;
            
            let duration = start.elapsed();
            let status_code = match &result {
                Ok(_) => 200,
                Err(_) => 500,
            };
            
            collector.record_request_duration(&path, &method, status_code, duration);
            collector.increment_request_count(&path, &method, status_code);
            
            result
        }
    }
    
    /// Get metrics in Prometheus format
    pub fn get_prometheus_metrics(&self) -> String {
        let metrics = self.collector.get_metrics();
        
        let mut output = String::new();
        
        for (key, metric) in metrics {
            match metric.metric_type {
                MetricType::Counter => {
                    if let MetricValue::Counter(value) = metric.value {
                        output.push_str(&format!("# TYPE {} counter\n", metric.name));
                        output.push_str(&format!("{} {}\n", key, value));
                    }
                },
                MetricType::Gauge => {
                    if let MetricValue::Gauge(value) = metric.value {
                        output.push_str(&format!("# TYPE {} gauge\n", metric.name));
                        output.push_str(&format!("{} {}\n", key, value));
                    }
                },
                MetricType::Histogram => {
                    if let MetricValue::Histogram(values) = metric.value {
                        output.push_str(&format!("# TYPE {} histogram\n", metric.name));
                        
                        // Calculate histogram buckets
                        let buckets = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];
                        let mut counts = vec![0; buckets.len() + 1];
                        let mut sum = 0.0;
                        
                        for value in &values {
                            sum += value;
                            
                            for (i, bucket) in buckets.iter().enumerate() {
                                if value <= bucket {
                                    counts[i] += 1;
                                }
                            }
                            
                            counts[buckets.len()] += 1; // +Inf bucket
                        }
                        
                        // Output buckets
                        for (i, bucket) in buckets.iter().enumerate() {
                            let labels = if let Some(pos) = key.find('{') {
                                let (name, labels) = key.split_at(pos);
                                let labels = &labels[1..labels.len() - 1];
                                format!("{}{{le=\"{}\",{}}}", name, bucket, labels)
                            } else {
                                format!("{}{{le=\"{}\"}}", key, bucket)
                            };
                            
                            output.push_str(&format!("{}_bucket {} {}\n", metric.name, labels, counts[i]));
                        }
                        
                        // Output +Inf bucket
                        let labels = if let Some(pos) = key.find('{') {
                            let (name, labels) = key.split_at(pos);
                            let labels = &labels[1..labels.len() - 1];
                            format!("{}{{le=\"+Inf\",{}}}", name, labels)
                        } else {
                            format!("{}{{le=\"+Inf\"}}", key)
                        };
                        
                        output.push_str(&format!("{}_bucket {} {}\n", metric.name, labels, counts[buckets.len()]));
                        
                        // Output sum
                        let labels = if let Some(pos) = key.find('{') {
                            let (name, labels) = key.split_at(pos);
                            format!("{}_sum{}", name, &key[pos..])
                        } else {
                            format!("{}_sum", key)
                        };
                        
                        output.push_str(&format!("{} {}\n", labels, sum));
                        
                        // Output count
                        let labels = if let Some(pos) = key.find('{') {
                            let (name, labels) = key.split_at(pos);
                            format!("{}_count{}", name, &key[pos..])
                        } else {
                            format!("{}_count", key)
                        };
                        
                        output.push_str(&format!("{} {}\n", labels, values.len()));
                    }
                },
            }
        }
        
        output
    }
} 