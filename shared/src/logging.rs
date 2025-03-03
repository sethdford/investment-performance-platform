//! Logging utilities

use tracing::{info, warn, error, debug, trace, Level, Subscriber};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    EnvFilter,
    layer::SubscriberExt,
    registry::Registry,
};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_appender::non_blocking::WorkerGuard;
use std::sync::Once;
use std::time::Duration;
use uuid::Uuid;
use std::collections::HashMap;

/// Correlation ID key
pub const CORRELATION_ID_KEY: &str = "correlation_id";

/// Tenant ID key
pub const TENANT_ID_KEY: &str = "tenant_id";

/// User ID key
pub const USER_ID_KEY: &str = "user_id";

/// Request ID key
pub const REQUEST_ID_KEY: &str = "request_id";

/// Initialize logging once
static INIT: Once = Once::new();

/// Initialize logging
pub fn init() -> Option<WorkerGuard> {
    let mut guard = None;
    
    INIT.call_once(|| {
        // Create a rolling file appender
        let file_appender = RollingFileAppender::new(
            Rotation::DAILY,
            "logs",
            "application.log",
        );
        
        // Create a non-blocking writer
        let (non_blocking, worker_guard) = tracing_appender::non_blocking(file_appender);
        guard = Some(worker_guard);
        
        // Create a JSON formatter
        let json_layer = fmt::layer()
            .json()
            .with_writer(non_blocking)
            .with_span_events(FmtSpan::CLOSE);
        
        // Create a console formatter
        let console_layer = fmt::layer()
            .with_span_events(FmtSpan::CLOSE);
        
        // Create an environment filter
        let filter_layer = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));
        
        // Combine layers
        let subscriber = Registry::default()
            .with(filter_layer)
            .with(json_layer)
            .with(console_layer);
        
        // Set the subscriber as the global default
        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set global default subscriber");
    });
    
    guard
}

/// Generate a new correlation ID
pub fn generate_correlation_id() -> String {
    Uuid::new_v4().to_string()
}

/// Log a request
pub fn log_request(
    method: &str,
    path: &str,
    correlation_id: &str,
    tenant_id: Option<&str>,
    user_id: Option<&str>,
    request_id: Option<&str>,
) {
    let mut fields = HashMap::new();
    fields.insert(CORRELATION_ID_KEY, correlation_id.to_string());
    
    if let Some(tenant_id) = tenant_id {
        fields.insert(TENANT_ID_KEY, tenant_id.to_string());
    }
    
    if let Some(user_id) = user_id {
        fields.insert(USER_ID_KEY, user_id.to_string());
    }
    
    if let Some(request_id) = request_id {
        fields.insert(REQUEST_ID_KEY, request_id.to_string());
    }
    
    info!(
        correlation_id = %correlation_id,
        method = %method,
        path = %path,
        tenant_id = tenant_id.unwrap_or("unknown"),
        user_id = user_id.unwrap_or("unknown"),
        request_id = request_id.unwrap_or("unknown"),
        "Received request"
    );
}

/// Log a response
pub fn log_response(
    method: &str,
    path: &str,
    status: u16,
    duration: Duration,
    correlation_id: &str,
    tenant_id: Option<&str>,
    user_id: Option<&str>,
    request_id: Option<&str>,
) {
    info!(
        correlation_id = %correlation_id,
        method = %method,
        path = %path,
        status = %status,
        duration_ms = %duration.as_millis(),
        tenant_id = tenant_id.unwrap_or("unknown"),
        user_id = user_id.unwrap_or("unknown"),
        request_id = request_id.unwrap_or("unknown"),
        "Sent response"
    );
}

/// Log an error
pub fn log_error(
    error: &str,
    correlation_id: &str,
    tenant_id: Option<&str>,
    user_id: Option<&str>,
    request_id: Option<&str>,
) {
    error!(
        correlation_id = %correlation_id,
        error = %error,
        tenant_id = tenant_id.unwrap_or("unknown"),
        user_id = user_id.unwrap_or("unknown"),
        request_id = request_id.unwrap_or("unknown"),
        "Error occurred"
    );
} 