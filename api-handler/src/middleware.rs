use aws_lambda_events::event::apigw::ApiGatewayProxyRequest;
use tracing::{info, warn, error};
use crate::auth_service::Claims;
use performance_calculator::calculations::tenant::get_tenant_metrics_manager;
use lambda_http::{Request, Response, Body, Error};
use shared::{logging, metrics::{MetricsCollector, MetricsMiddleware}};
use std::time::{Duration, Instant};
use tracing_attributes::instrument;
use uuid::Uuid;
use std::future::Future;
use std::pin::Pin;

/// Track an API request for a tenant
pub async fn track_api_request(tenant_id: &str) {
    // Get metrics manager
    let metrics_manager = match get_tenant_metrics_manager().await {
        Ok(manager) => manager,
        Err(e) => {
            warn!("Failed to get metrics manager: {:?}. API request will not be tracked.", e);
            return;
        }
    };
    
    // Track API request
    if let Err(e) = metrics_manager.track_api_request(tenant_id).await {
        warn!("Failed to track API request for tenant {}: {:?}", tenant_id, e);
    }
}

/// Check if a tenant has exceeded their API request limit
pub async fn check_api_request_limit(tenant_id: &str) -> bool {
    // Get tenant manager to retrieve tenant info
    let tenant_manager = match performance_calculator::calculations::tenant::get_tenant_manager(
        &performance_calculator::calculations::tenant::TenantConfig::default()
    ).await {
        Ok(manager) => manager,
        Err(e) => {
            warn!("Failed to get tenant manager: {:?}. Cannot check API request limit.", e);
            return false; // Allow the request if we can't check
        }
    };
    
    // Get tenant to check resource limits
    let tenant = match tenant_manager.get_tenant(tenant_id).await {
        Ok(Some(tenant)) => tenant,
        Ok(None) => {
            warn!("Tenant {} not found. Cannot check API request limit.", tenant_id);
            return false; // Allow the request if tenant not found
        },
        Err(e) => {
            warn!("Failed to get tenant {}: {:?}. Cannot check API request limit.", tenant_id, e);
            return false; // Allow the request if we can't check
        }
    };
    
    // Get metrics manager
    let metrics_manager = match get_tenant_metrics_manager().await {
        Ok(manager) => manager,
        Err(e) => {
            warn!("Failed to get metrics manager: {:?}. Cannot check API request limit.", e);
            return false; // Allow the request if we can't check
        }
    };
    
    // Get usage metrics
    let metrics = match metrics_manager.get_usage_metrics(tenant_id).await {
        Ok(metrics) => metrics,
        Err(e) => {
            warn!("Failed to get usage metrics for tenant {}: {:?}. Cannot check API request limit.", tenant_id, e);
            return false; // Allow the request if we can't check
        }
    };
    
    // Check if API request limit is exceeded
    metrics.api_requests >= tenant.resource_limits.max_api_requests_per_minute
}

/// Extract tenant ID from request
pub fn extract_tenant_id(request: &ApiGatewayProxyRequest, claims: Option<&Claims>) -> Option<String> {
    // First, try to get tenant ID from claims
    if let Some(claims) = claims {
        return Some(claims.tenant_id.clone());
    }
    
    // If no claims, try to get tenant ID from path
    if let Some(path) = &request.path {
        // Check if path contains tenant ID
        if path.starts_with("/admin/tenants/") {
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 3 {
                return Some(parts[2].to_string());
            }
        }
    }
    
    // If no tenant ID found, return None
    None
}

/// Process request middleware
pub async fn process_request(request: &ApiGatewayProxyRequest, claims: Option<&Claims>) -> bool {
    // Extract tenant ID
    let tenant_id = match extract_tenant_id(request, claims) {
        Some(id) => id,
        None => {
            // No tenant ID found, allow the request
            return true;
        }
    };
    
    // Check if tenant has exceeded API request limit
    if check_api_request_limit(&tenant_id).await {
        // Tenant has exceeded API request limit
        error!("Tenant {} has exceeded API request limit", tenant_id);
        return false;
    }
    
    // Track API request
    track_api_request(&tenant_id).await;
    
    // Allow the request
    true
}

/// Request context
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// Correlation ID
    pub correlation_id: String,
    /// Request ID
    pub request_id: String,
    /// Tenant ID
    pub tenant_id: Option<String>,
    /// User ID
    pub user_id: Option<String>,
    /// Start time
    pub start_time: Instant,
}

impl RequestContext {
    /// Create a new request context
    pub fn new(request: &Request) -> Self {
        // Extract or generate correlation ID
        let correlation_id = request.headers()
            .get("x-correlation-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        
        // Extract or generate request ID
        let request_id = request.headers()
            .get("x-amzn-requestid")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        
        // Extract tenant ID if present
        let tenant_id = request.headers()
            .get("x-tenant-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());
        
        // Extract user ID if present
        let user_id = request.headers()
            .get("x-user-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());
        
        Self {
            correlation_id,
            request_id,
            tenant_id,
            user_id,
            start_time: Instant::now(),
        }
    }
    
    /// Get the request duration
    pub fn duration(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Middleware for request tracing
pub struct TracingMiddleware {
    /// Metrics middleware
    metrics_middleware: MetricsMiddleware,
}

impl TracingMiddleware {
    /// Create a new tracing middleware
    pub fn new(metrics_collector: MetricsCollector) -> Self {
        Self {
            metrics_middleware: MetricsMiddleware::new(metrics_collector),
        }
    }
    
    /// Process a request with tracing
    #[instrument(skip(self, request, handler), fields(
        correlation_id = %context.correlation_id,
        request_id = %context.request_id,
        tenant_id = context.tenant_id.as_deref().unwrap_or("unknown"),
        user_id = context.user_id.as_deref().unwrap_or("unknown"),
    ))]
    pub async fn process<F, Fut>(&self, request: Request, context: RequestContext, handler: F) -> Result<Response<Body>, Error>
    where
        F: FnOnce(Request, RequestContext) -> Fut,
        Fut: Future<Output = Result<Response<Body>, Error>>,
    {
        let method = request.method().as_str();
        let path = request.uri().path();
        
        // Log the request
        logging::log_request(
            method,
            path,
            &context.correlation_id,
            context.tenant_id.as_deref(),
            context.user_id.as_deref(),
            Some(&context.request_id),
        );
        
        // Process the request with metrics tracking
        let result = self.metrics_middleware.track_request(method, path, || {
            handler(request, context.clone())
        }).await;
        
        // Log the response
        match &result {
            Ok(response) => {
                let status = response.status().as_u16();
                
                logging::log_response(
                    method,
                    path,
                    status,
                    context.duration(),
                    &context.correlation_id,
                    context.tenant_id.as_deref(),
                    context.user_id.as_deref(),
                    Some(&context.request_id),
                );
            },
            Err(e) => {
                logging::log_error(
                    &e.to_string(),
                    &context.correlation_id,
                    context.tenant_id.as_deref(),
                    context.user_id.as_deref(),
                    Some(&context.request_id),
                );
            }
        }
        
        result
    }
} 