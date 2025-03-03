use lambda_http::{run, service_fn, Body, Error, Request, Response};
use tracing::{info, error};
use shared::{
    models::{Item, ApiResponse, ErrorResponse},
    repository::DynamoDbRepository,
    config::AppConfig,
    AppError,
    sanitization,
    encryption::EncryptionService,
};
use aws_sdk_sqs::Client as SqsClient;
use serde_json::json;
use std::env;
use uuid;
use md5;
use chrono::{Utc};
use aws_lambda_events::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
use aws_lambda_events::encodings::Body;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use lambda_runtime::{service_fn, LambdaEvent};
use serde::{Deserialize, Serialize};
use performance_calculator::resilience;
use std::collections::HashMap;
use std::time::Duration;
use shared::logging;
use shared::metrics::{MetricsCollector, MetricsMiddleware};
use crate::middleware::{TracingMiddleware, RequestContext};

/// Test module for unit testing the API handler
#[cfg(test)]
mod tests;

/// Authentication and authorization module
mod auth;

/// Tenant management API module
mod tenant_api;

/// User management API module
mod user_api;

/// Metrics and billing API module
mod metrics_api;

/// Middleware for request processing
mod middleware;

/// Validation module for API requests
mod validation;

/// Main entry point for the API handler Lambda function
///
/// This function initializes the AWS SDK, sets up logging, and starts the Lambda runtime.
/// It handles HTTP requests from API Gateway and routes them to the appropriate handler functions.
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize logging
    let _guard = logging::init();

    info!("API Handler Lambda starting up");
    
    // Load configuration from environment variables
    let config = AppConfig::from_env();
    
    // Initialize AWS SDK clients
    let aws_config = aws_config::load_from_env().await;
    let dynamodb_client = DynamoDbClient::new(&aws_config);
    let sqs_client = SqsClient::new(&aws_config);
    
    // Initialize repositories
    let repository = DynamoDbRepository::new(
        dynamodb_client,
        config.table_name.clone(),
    );
    let cached_repository = CachedDynamoDbRepository::new(
        repository,
        100, // capacity
        Duration::from_secs(300), // TTL (5 minutes)
    );
    
    // Initialize authentication middleware
    let auth_middleware = auth::AuthMiddleware::new(
        config.jwt_secret.as_bytes(),
    );
    
    // Initialize metrics collector
    let metrics_collector = MetricsCollector::new();
    
    // Initialize tracing middleware
    let tracing_middleware = TracingMiddleware::new(metrics_collector.clone());
    
    // Initialize API handlers
    let portfolio_api = api::PortfolioApi::new(
        cached_repository.clone(),
        auth_middleware.clone(),
    );
    
    let metrics_api = api::MetricsApi::new(metrics_collector.clone());
    
    // Run the Lambda function
    run(service_fn(move |event: Request| {
        let context = RequestContext::new(&event);
        
        // Check if this is a metrics request
        if event.uri().path() == "/metrics" {
            return metrics_api.handle_request(event);
        }
        
        tracing_middleware.process(event, context, |request, _context| {
            handle_request(request, &portfolio_api)
        })
    })).await
}

async fn handle_request(
    event: Request,
    api_handler: &impl api::ApiHandler,
) -> Result<Response<Body>, Error> {
    // Extract request ID for logging
    let request_id = event.headers()
        .get("x-amzn-requestid")
        .map(|h| h.to_str().unwrap_or("unknown"))
        .unwrap_or("unknown")
        .to_string();
    
    // Authenticate the request
    let claims = match auth::AuthMiddleware::authenticate(&event) {
        Ok(claims) => claims,
        Err(e) => {
            error!(
                request_id = %request_id,
                error = %e,
                "Authentication error"
            );
            return Ok(Response::builder()
                .status(e.status_code())
                .header("Content-Type", "application/json")
                .body(Body::from(e.to_string()))?);
        }
    };
    
    // Handle the request
    match api_handler.handle_request(event).await {
        Ok(response) => {
            info!(
                request_id = %request_id,
                correlation_id = %context.correlation_id,
                "Request handled successfully"
            );
            Ok(response)
        },
        Err(e) => {
            error!(
                request_id = %request_id,
                correlation_id = %context.correlation_id,
                error = %e,
                "Error handling request"
            );
            
            // Create error context
            let error_context = ErrorContext::new()
                .with_request_id(request_id.clone())
                .with_correlation_id(context.correlation_id.clone());
            
            if let Some(tenant_id) = &context.tenant_id {
                error_context.with_tenant_id(tenant_id.clone());
            }
            
            if let Some(user_id) = &context.user_id {
                error_context.with_user_id(user_id.clone());
            }
            
            // Create error with context
            let error_with_context = ErrorWithContext::new(e, error_context);
            
            // Build error response
            let error_response = json!({
                "error": {
                    "code": error_with_context.error_code(),
                    "message": error_with_context.error.to_string(),
                    "request_id": request_id,
                    "correlation_id": context.correlation_id,
                }
            });
            
            Ok(Response::builder()
                .status(error_with_context.status_code())
                .header("Content-Type", "application/json")
                .body(Body::from(error_response.to_string()))
                .unwrap_or_else(|_| {
                    Response::builder()
                        .status(500)
                        .body(Body::from("Failed to build error response"))
                        .unwrap()
                })
        }
    }
}

/// Initialize tenant bulkhead configurations based on tenant subscription tiers
async fn initialize_tenant_bulkheads() -> Result<(), Error> {
    // Get tenant manager
    let tenant_manager = performance_calculator::calculations::tenant::CachedTenantManager::new(
        performance_calculator::calculations::tenant::DynamoDbTenantManager::from_env().await?,
        std::time::Duration::from_secs(300), // 5 minutes TTL
    );

    // List all active tenants
    let tenants = tenant_manager.list_tenants(None, None).await?;

    // Configure bulkheads for each tenant based on subscription tier
    for tenant in tenants {
        if tenant.status == performance_calculator::calculations::tenant::TenantStatus::Active {
            performance_calculator::resilience::configure_tenant_bulkheads_from_tier(
                &tenant.id,
                &tenant.subscription_tier,
            );
            tracing::info!(
                "Configured bulkhead for tenant {} with tier {:?}",
                tenant.id,
                tenant.subscription_tier
            );
        }
    }

    Ok(())
}

/// Main request handler for the API Lambda
///
/// This function routes incoming HTTP requests to the appropriate handler function
/// based on the HTTP method and path.
///
/// # Arguments
///
/// * `event` - The HTTP request from API Gateway
/// * `repo` - The DynamoDB repository for data access
/// * `sqs_client` - The SQS client for sending events
/// * `queue_url` - The URL of the SQS queue for events
///
/// # Returns
///
/// * `Result<Response<Body>, AppError>` - The HTTP response or an error
async fn get_items(repo: &DynamoDbRepository) -> Result<Response<Body>, AppError> {
    // Retrieve all items from the database
    let items = repo.list_items().await?;
    
    // Create a successful response
    let response = ApiResponse {
        status_code: 200,
        body: items,
    };
    
    // Serialize the response body to JSON
    let body = serde_json::to_string(&response.body)?;
    
    // Build and return the HTTP response
    Ok(Response::builder()
        .status(response.status_code)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .map_err(|e| AppError::Internal(e.to_string()))?)
}

/// Handler for GET /items/{id} endpoint
///
/// Retrieves a specific item by ID from the database.
///
/// # Arguments
///
/// * `repo` - The DynamoDB repository for data access
/// * `id` - The ID of the item to retrieve
///
/// # Returns
///
/// * `Result<Response<Body>, AppError>` - A JSON response with the item or an error
async fn get_item(repo: &DynamoDbRepository, id: &str) -> Result<Response<Body>, AppError> {
    // Retrieve the item from the database
    let item = repo.get_item(id).await?;
    
    match item {
        Some(item) => {
            // Create a successful response
            let response = ApiResponse {
                status_code: 200,
                body: item,
            };
            
            // Serialize the response body to JSON
            let body = serde_json::to_string(&response.body)?;
            
            // Build and return the HTTP response
            Ok(Response::builder()
                .status(response.status_code)
                .header("Content-Type", "application/json")
                .body(Body::from(body))
                .map_err(|e| AppError::Internal(e.to_string()))?)
        },
        None => Err(AppError::NotFound(format!("Item with ID {} not found", id))),
    }
}

/// Validates an item before processing
///
/// This function performs comprehensive validation on an item to ensure it meets
/// security and business requirements. It checks for empty fields, malicious content,
/// and other validation rules.
///
/// # Arguments
///
/// * `item` - The item to validate
///
/// # Returns
///
/// * `Result<(), AppError>` - Ok if valid, Err with validation error otherwise
fn validate_item(item: &Item) -> Result<(), AppError> {
    // Check for empty or invalid fields
    if item.name.is_empty() {
        return Err(AppError::Validation("Item name cannot be empty".to_string()));
    }
    
    // Check name length
    if item.name.len() > 100 {
        return Err(AppError::Validation("Item name too long (max 100 characters)".to_string()));
    }
    
    // Check for malicious content in name
    if item.name.contains('<') || item.name.contains('>') || item.name.contains('&') {
        return Err(AppError::Validation("Item name contains invalid characters".to_string()));
    }
    
    // Check description if present
    if let Some(desc) = &item.description {
        if desc.len() > 1000 {
            return Err(AppError::Validation("Description too long (max 1000 characters)".to_string()));
        }
        
        // Check for malicious content in description
        if desc.contains('<') || desc.contains('>') || desc.contains('&') {
            return Err(AppError::Validation("Description contains invalid characters".to_string()));
        }
    }
    
    // Validate classification
    match item.classification.as_str() {
        "PUBLIC" | "INTERNAL" | "CONFIDENTIAL" | "RESTRICTED" => (),
        _ => return Err(AppError::Validation("Invalid classification level".to_string())),
    }
    
    Ok(())
}

/// Masks sensitive data for logging
///
/// This function masks sensitive data to prevent it from appearing in logs.
///
/// # Arguments
///
/// * `data` - The data to mask
///
/// # Returns
///
/// * `String` - The masked data
fn mask_sensitive_data(data: &str) -> String {
    if data.len() <= 4 {
        return "****".to_string();
    }
    let visible = &data[0..4];
    let masked = "*".repeat(data.len() - 4);
    format!("{}{}", visible, masked)
}

/// Creates an audit record for an action
///
/// This function creates an audit record for an action performed on an item.
///
/// # Arguments
///
/// * `action` - The action performed (create, update, delete)
/// * `item` - The item affected
/// * `previous_state` - The previous state of the item (for updates and deletes)
/// * `request_id` - The ID of the request that triggered the action
///
/// # Returns
///
/// * `AuditRecord` - The audit record
fn create_audit_record(
    action: &str,
    item: &Item,
    previous_state: Option<String>,
    request_id: &str,
) -> shared::models::AuditRecord {
    let new_state = if action != "delete" {
        Some(serde_json::to_string(item).unwrap_or_default())
    } else {
        None
    };
    
    // In a real application, you would get the user ID from authentication
    let user_id = "system".to_string();
    
    // Create a hash of the item for non-repudiation
    let item_json = serde_json::to_string(item).unwrap_or_default();
    let item_hash = format!("{:x}", md5::compute(item_json.as_bytes()));
    
    shared::models::AuditRecord {
        event_id: uuid::Uuid::new_v4().to_string(),
        user_id,
        action: action.to_string(),
        resource_id: item.id.clone(),
        resource_type: "item".to_string(),
        timestamp: Utc::now(),
        previous_state,
        new_state,
        request_id: request_id.to_string(),
        hash: Some(item_hash),
    }
}

/// Handler for POST /items endpoint
///
/// Creates a new item in the database and sends a creation event to SQS.
///
/// # Arguments
///
/// * `repo` - The DynamoDB repository for data access
/// * `sqs_client` - The SQS client for sending events
/// * `queue_url` - The URL of the SQS queue for events
/// * `item` - The item to create
///
/// # Returns
///
/// * `Result<Response<Body>, AppError>` - A JSON response with the created item or an error
async fn create_item(
    repo: &DynamoDbRepository,
    sqs_client: &SqsClient,
    queue_url: &str,
    item: Item,
) -> Result<Response<Body>, AppError> {
    // Validate item
    validate_item(&item)?;
    
    // Save item to DynamoDB
    repo.create_item(&item).await?;
    
    // Create an audit record
    let audit = create_audit_record("create", &item, None, "request-id");
    
    // In a real application, you would store the audit record
    // For now, we'll just log it
    info!(
        action = %audit.action,
        resource_id = %audit.resource_id,
        resource_type = %audit.resource_type,
        user_id = %audit.user_id,
        "Item created"
    );
    
    // Create an event for the item creation
    let event = shared::models::ItemEvent {
        event_type: shared::models::ItemEventType::Created,
        item: item.clone(),
        timestamp: chrono::Utc::now(),
    };
    
    // Serialize the event to JSON
    let event_json = serde_json::to_string(&event)?;
    
    // Send the event to SQS
    sqs_client.send_message()
        .queue_url(queue_url)
        .message_body(event_json)
        .send()
        .await
        .map_err(|e| AppError::Sqs(e.to_string()))?;
    
    // Create a successful response
    let response = ApiResponse {
        status_code: 201,
        body: item,
    };
    
    // Serialize the response body to JSON
    let body = serde_json::to_string(&response.body)?;
    
    // Build and return the HTTP response
    Ok(Response::builder()
        .status(response.status_code)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .map_err(|e| AppError::Internal(e.to_string()))?)
}

/// Handler for DELETE /items/{id} endpoint
///
/// Deletes an item from the database and sends a deletion event to SQS.
///
/// # Arguments
///
/// * `repo` - The DynamoDB repository for data access
/// * `sqs_client` - The SQS client for sending events
/// * `queue_url` - The URL of the SQS queue for events
/// * `id` - The ID of the item to delete
///
/// # Returns
///
/// * `Result<Response<Body>, AppError>` - A success response or an error
async fn delete_item(
    repo: &DynamoDbRepository,
    sqs_client: &SqsClient,
    queue_url: &str,
    id: &str,
) -> Result<Response<Body>, AppError> {
    // Check if item exists
    let item = repo.get_item(id).await?;
    
    match item {
        Some(item) => {
            // Create an audit record with the previous state
            let previous_state = serde_json::to_string(&item).ok();
            let audit = create_audit_record("delete", &item, previous_state, "request-id");
            
            // Delete item from DynamoDB
            repo.delete_item(id).await?;
            
            // In a real application, you would store the audit record
            // For now, we'll just log it
            info!(
                action = %audit.action,
                resource_id = %audit.resource_id,
                resource_type = %audit.resource_type,
                user_id = %audit.user_id,
                "Item deleted"
            );
            
            // Create an event for the item deletion
            let event = shared::models::ItemEvent {
                event_type: shared::models::ItemEventType::Deleted,
                item,
                timestamp: chrono::Utc::now(),
            };
            
            // Serialize the event to JSON
            let event_json = serde_json::to_string(&event)?;
            
            // Send the event to SQS
            sqs_client.send_message()
                .queue_url(queue_url)
                .message_body(event_json)
                .send()
                .await
                .map_err(|e| AppError::Sqs(e.to_string()))?;
            
            // Build and return a 204 No Content response
            Ok(Response::builder()
                .status(204)
                .body(Body::Empty)
                .map_err(|e| AppError::Internal(e.to_string()))?)
        },
        None => Err(AppError::NotFound(format!("Item with ID {} not found", id))),
    }
}

/// Helper function to create an error response
///
/// # Arguments
///
/// * `message` - The error message
/// * `status_code` - The HTTP status code
///
/// # Returns
///
/// * `Response<Body>` - An HTTP response with the error message
fn error_response(message: String, status_code: u16) -> Response<Body> {
    let error = ErrorResponse { message };
    let body = serde_json::to_string(&error).unwrap_or_else(|_| {
        json!({ "message": "Error serializing error response" }).to_string()
    });
    
    Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap_or_else(|_| {
            let fallback_body = json!({ "message": "Internal server error" }).to_string();
            Response::builder()
                .status(500)
                .header("Content-Type", "application/json")
                .body(Body::from(fallback_body))
                .unwrap()
        })
}

#[derive(Debug, Serialize, Deserialize)]
struct Item {
    id: String,
    name: String,
    description: Option<String>,
    created_at: String,
    updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Portfolio {
    id: String,
    name: String,
    client_id: String,
    created_at: String,
    updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Account {
    id: String,
    name: String,
    portfolio_id: String,
    created_at: String,
    updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Security {
    id: String,
    symbol: String,
    name: String,
    security_type: String,
    created_at: String,
    updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Transaction {
    id: String,
    account_id: String,
    security_id: Option<String>,
    transaction_type: String,
    amount: f64,
    quantity: Option<f64>,
    price: Option<f64>,
    transaction_date: String,
    created_at: String,
    updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PerformanceMetrics {
    portfolio_id: String,
    start_date: String,
    end_date: String,
    twr: f64,
    mwr: f64,
    volatility: Option<f64>,
    sharpe_ratio: Option<f64>,
    max_drawdown: Option<f64>,
    calculated_at: String,
}

struct ApiHandler {
    dynamodb_client: DynamoDbClient,
    table_name: String,
}

impl ApiHandler {
    /// Create a new API handler
    async fn new() -> Result<Self, Error> {
        let config = aws_config::load_from_env().await;
        let dynamodb_client = DynamoDbClient::new(&config);
        let table_name = env::var("DYNAMODB_TABLE").unwrap_or_else(|_| "investment-performance".to_string());
        
        Ok(Self {
            dynamodb_client,
            table_name,
        })
    }
    
    /// Handle an API Gateway request
    async fn handle_request(&self, event: ApiGatewayProxyRequest) -> Result<ApiGatewayProxyResponse, Error> {
        let request_id = uuid::Uuid::new_v4().to_string();
        info!("Processing request: {}", request_id);
        
        // Extract path and method
        let path = event.path.as_deref().unwrap_or("/");
        let method = event.http_method.as_deref().unwrap_or("GET");
        
        info!("Request: {} {}", method, path);
        
        // Handle health check
        if path == "/health" {
            return self.health_check().await;
        }
        
        // Extract and validate JWT token for protected endpoints
        if path.starts_with("/api/") || path.starts_with("/admin/") {
            let auth_header = event.headers.get("Authorization");
            
            match auth_header {
                Some(header) => {
                    let token = header.strip_prefix("Bearer ").unwrap_or(header);
                    
                    // Get auth service
                    let auth_service = match auth_service::get_auth_service().await {
                        Ok(service) => service,
                        Err(e) => {
                            error!("Failed to get auth service: {:?}", e);
                            return Ok(build_response(
                                http::StatusCode::INTERNAL_SERVER_ERROR,
                                Some(json!({
                                    "error": "Internal Server Error",
                                    "message": "Authentication service unavailable"
                                })),
                                Some(request_id),
                            ));
                        }
                    };
                    
                    // Validate token
                    match auth_service.validate_token(token).await {
                        Ok(claims) => {
                            // Process request through middleware
                            if !middleware::process_request(&event, Some(&claims)).await {
                                return Ok(build_response(
                                    http::StatusCode::TOO_MANY_REQUESTS,
                                    Some(json!({
                                        "error": "Too Many Requests",
                                        "message": "API request limit exceeded"
                                    })),
                                    Some(request_id),
                                ));
                            }
                            
                            // Process authenticated request
                            self.process_authenticated_request(event, &claims, request_id).await
                        },
                        Err(e) => {
                            error!("Authentication error: {:?}", e);
                            Ok(build_response(
                                http::StatusCode::UNAUTHORIZED,
                                Some(json!({
                                    "error": "Unauthorized",
                                    "message": "Invalid or expired token"
                                })),
                                Some(request_id),
                            ))
                        }
                    }
                },
                None => {
                    // Process request through middleware without claims
                    if !middleware::process_request(&event, None).await {
                        return Ok(build_response(
                            http::StatusCode::TOO_MANY_REQUESTS,
                            Some(json!({
                                "error": "Too Many Requests",
                                "message": "API request limit exceeded"
                            })),
                            Some(request_id),
                        ));
                    }
                    
                    // No Authorization header
                    Ok(build_response(
                        http::StatusCode::UNAUTHORIZED,
                        Some(json!({
                            "error": "Unauthorized",
                            "message": "Authentication required"
                        })),
                        Some(request_id),
                    ))
                }
            }
        } else if path == "/auth/login" {
            // Process request through middleware without claims
            if !middleware::process_request(&event, None).await {
                return Ok(build_response(
                    http::StatusCode::TOO_MANY_REQUESTS,
                    Some(json!({
                        "error": "Too Many Requests",
                        "message": "API request limit exceeded"
                    })),
                    Some(request_id),
                ));
            }
            
            // Handle login request
            self.process_login_request(event, request_id).await
        } else {
            // Handle unknown endpoint
            Ok(build_response(
                http::StatusCode::NOT_FOUND,
                Some(json!({
                    "error": "Not Found",
                    "message": "The requested endpoint does not exist"
                })),
                Some(request_id),
            ))
        }
    }
    
    /// Process an authenticated request
    async fn process_authenticated_request(
        &self,
        event: ApiGatewayProxyRequest,
        claims: &auth_service::Claims,
        request_id: String,
    ) -> Result<ApiGatewayProxyResponse, Error> {
        // Extract path and method
        let path = event.path.as_deref().unwrap_or("/");
        let method = event.http_method.as_deref().unwrap_or("GET");
        
        // Extract query parameters
        let query_params = event.query_string_parameters.clone().unwrap_or_default();
        
        // Route the request based on path and method
        match (method, path) {
            // Tenant management endpoints
            ("GET", "/admin/tenants") => {
                tenant_api::handle_list_tenants(
                    &self.dynamodb_client,
                    &self.table_name,
                    &query_params,
                    claims,
                    request_id,
                ).await
            },
            ("GET", p) if p.starts_with("/admin/tenants/") => {
                let tenant_id = p.trim_start_matches("/admin/tenants/");
                tenant_api::handle_get_tenant(
                    &self.dynamodb_client,
                    &self.table_name,
                    tenant_id,
                    claims,
                    request_id,
                ).await
            },
            ("POST", "/admin/tenants") => {
                // Parse request body
                let body = match event.body {
                    Some(aws_lambda_events::encodings::Body::Text(body)) => body,
                    _ => {
                        return Ok(build_response(
                            http::StatusCode::BAD_REQUEST,
                            Some(json!({
                                "error": "Bad Request",
                                "message": "Missing request body"
                            })),
                            Some(request_id),
                        ));
                    }
                };
                
                // Parse create tenant request
                let create_request: tenant_api::CreateTenantRequest = match serde_json::from_str(&body) {
                    Ok(req) => req,
                    Err(e) => {
                        return Ok(build_response(
                            http::StatusCode::BAD_REQUEST,
                            Some(json!({
                                "error": "Bad Request",
                                "message": format!("Invalid request format: {}", e)
                            })),
                            Some(request_id),
                        ));
                    }
                };
                
                tenant_api::handle_create_tenant(
                    &self.dynamodb_client,
                    &self.table_name,
                    create_request,
                    claims,
                    request_id,
                ).await
            },
            ("PUT", p) if p.starts_with("/admin/tenants/") => {
                let tenant_id = p.trim_start_matches("/admin/tenants/");
                
                // Parse request body
                let body = match event.body {
                    Some(aws_lambda_events::encodings::Body::Text(body)) => body,
                    _ => {
                        return Ok(build_response(
                            http::StatusCode::BAD_REQUEST,
                            Some(json!({
                                "error": "Bad Request",
                                "message": "Missing request body"
                            })),
                            Some(request_id),
                        ));
                    }
                };
                
                // Parse update tenant request
                let update_request: tenant_api::UpdateTenantRequest = match serde_json::from_str(&body) {
                    Ok(req) => req,
                    Err(e) => {
                        return Ok(build_response(
                            http::StatusCode::BAD_REQUEST,
                            Some(json!({
                                "error": "Bad Request",
                                "message": format!("Invalid request format: {}", e)
                            })),
                            Some(request_id),
                        ));
                    }
                };
                
                tenant_api::handle_update_tenant(
                    &self.dynamodb_client,
                    &self.table_name,
                    tenant_id,
                    update_request,
                    claims,
                    request_id,
                ).await
            },
            ("DELETE", p) if p.starts_with("/admin/tenants/") => {
                let tenant_id = p.trim_start_matches("/admin/tenants/");
                tenant_api::handle_delete_tenant(
                    &self.dynamodb_client,
                    &self.table_name,
                    tenant_id,
                    claims,
                    request_id,
                ).await
            },
            ("POST", p) if p.starts_with("/admin/tenants/") && p.ends_with("/activate") => {
                let tenant_id = p.trim_start_matches("/admin/tenants/").trim_end_matches("/activate");
                tenant_api::handle_activate_tenant(
                    &self.dynamodb_client,
                    &self.table_name,
                    tenant_id,
                    claims,
                    request_id,
                ).await
            },
            ("POST", p) if p.starts_with("/admin/tenants/") && p.ends_with("/suspend") => {
                let tenant_id = p.trim_start_matches("/admin/tenants/").trim_end_matches("/suspend");
                tenant_api::handle_suspend_tenant(
                    &self.dynamodb_client,
                    &self.table_name,
                    tenant_id,
                    claims,
                    request_id,
                ).await
            },
            ("POST", p) if p.starts_with("/admin/tenants/") && p.ends_with("/deactivate") => {
                let tenant_id = p.trim_start_matches("/admin/tenants/").trim_end_matches("/deactivate");
                tenant_api::handle_deactivate_tenant(
                    &self.dynamodb_client,
                    &self.table_name,
                    tenant_id,
                    claims,
                    request_id,
                ).await
            },
            
            // User management endpoints
            ("GET", "/admin/users") => {
                user_api::handle_list_users(
                    &self.dynamodb_client,
                    &self.table_name,
                    &query_params,
                    claims,
                    request_id,
                ).await
            },
            ("GET", p) if p.starts_with("/admin/users/") => {
                let user_id = p.trim_start_matches("/admin/users/");
                user_api::handle_get_user(
                    &self.dynamodb_client,
                    &self.table_name,
                    user_id,
                    claims,
                    request_id,
                ).await
            },
            ("POST", "/admin/users") => {
                // Parse request body
                let body = match event.body {
                    Some(aws_lambda_events::encodings::Body::Text(body)) => body,
                    _ => {
                        return Ok(build_response(
                            http::StatusCode::BAD_REQUEST,
                            Some(json!({
                                "error": "Bad Request",
                                "message": "Missing request body"
                            })),
                            Some(request_id),
                        ));
                    }
                };
                
                // Parse create user request
                let create_request: user_api::CreateUserRequest = match serde_json::from_str(&body) {
                    Ok(req) => req,
                    Err(e) => {
                        return Ok(build_response(
                            http::StatusCode::BAD_REQUEST,
                            Some(json!({
                                "error": "Bad Request",
                                "message": format!("Invalid request format: {}", e)
                            })),
                            Some(request_id),
                        ));
                    }
                };
                
                user_api::handle_create_user(
                    &self.dynamodb_client,
                    &self.table_name,
                    create_request,
                    claims,
                    request_id,
                ).await
            },
            ("PUT", p) if p.starts_with("/admin/users/") => {
                let user_id = p.trim_start_matches("/admin/users/");
                
                // Parse request body
                let body = match event.body {
                    Some(aws_lambda_events::encodings::Body::Text(body)) => body,
                    _ => {
                        return Ok(build_response(
                            http::StatusCode::BAD_REQUEST,
                            Some(json!({
                                "error": "Bad Request",
                                "message": "Missing request body"
                            })),
                            Some(request_id),
                        ));
                    }
                };
                
                // Parse update user request
                let update_request: user_api::UpdateUserRequest = match serde_json::from_str(&body) {
                    Ok(req) => req,
                    Err(e) => {
                        return Ok(build_response(
                            http::StatusCode::BAD_REQUEST,
                            Some(json!({
                                "error": "Bad Request",
                                "message": format!("Invalid request format: {}", e)
                            })),
                            Some(request_id),
                        ));
                    }
                };
                
                user_api::handle_update_user(
                    &self.dynamodb_client,
                    &self.table_name,
                    user_id,
                    update_request,
                    claims,
                    request_id,
                ).await
            },
            ("DELETE", p) if p.starts_with("/admin/users/") => {
                let user_id = p.trim_start_matches("/admin/users/");
                user_api::handle_delete_user(
                    &self.dynamodb_client,
                    &self.table_name,
                    user_id,
                    claims,
                    request_id,
                ).await
            },
            ("POST", p) if p.starts_with("/admin/users/") && p.ends_with("/change-password") => {
                let user_id = p.trim_start_matches("/admin/users/").trim_end_matches("/change-password");
                
                // Parse request body
                let body = match event.body {
                    Some(aws_lambda_events::encodings::Body::Text(body)) => body,
                    _ => {
                        return Ok(build_response(
                            http::StatusCode::BAD_REQUEST,
                            Some(json!({
                                "error": "Bad Request",
                                "message": "Missing request body"
                            })),
                            Some(request_id),
                        ));
                    }
                };
                
                // Parse change password request
                let change_request: user_api::ChangePasswordRequest = match serde_json::from_str(&body) {
                    Ok(req) => req,
                    Err(e) => {
                        return Ok(build_response(
                            http::StatusCode::BAD_REQUEST,
                            Some(json!({
                                "error": "Bad Request",
                                "message": format!("Invalid request format: {}", e)
                            })),
                            Some(request_id),
                        ));
                    }
                };
                
                user_api::handle_change_password(
                    &self.dynamodb_client,
                    &self.table_name,
                    user_id,
                    change_request,
                    claims,
                    request_id,
                ).await
            },
            
            // Metrics endpoints
            ("GET", p) if p.starts_with("/admin/tenants/") && p.ends_with("/metrics") => {
                let tenant_id = p.trim_start_matches("/admin/tenants/").trim_end_matches("/metrics");
                metrics_api::handle_get_tenant_metrics(
                    &self.dynamodb_client,
                    &self.table_name,
                    tenant_id,
                    claims,
                    request_id,
                ).await
            },
            ("GET", p) if p.starts_with("/admin/tenants/") && p.ends_with("/billing") => {
                let tenant_id = p.trim_start_matches("/admin/tenants/").trim_end_matches("/billing");
                metrics_api::handle_get_tenant_billing_records(
                    &self.dynamodb_client,
                    &self.table_name,
                    tenant_id,
                    &query_params,
                    claims,
                    request_id,
                ).await
            },
            ("GET", p) if p.starts_with("/admin/tenants/") && p.contains("/billing/") => {
                let path_parts: Vec<&str> = p.split('/').collect();
                if path_parts.len() >= 5 && path_parts[1] == "tenants" && path_parts[3] == "billing" {
                    let tenant_id = path_parts[2];
                    let record_id = path_parts[4];
                    metrics_api::handle_get_billing_record(
                        &self.dynamodb_client,
                        &self.table_name,
                        tenant_id,
                        record_id,
                        claims,
                        request_id,
                    ).await
                } else {
                    Ok(build_response(
                        http::StatusCode::NOT_FOUND,
                        Some(json!({
                            "error": "Not Found",
                            "message": "The requested endpoint does not exist"
                        })),
                        Some(request_id),
                    ))
                },
            ("PUT", p) if p.starts_with("/admin/tenants/") && p.contains("/billing/") => {
                let path_parts: Vec<&str> = p.split('/').collect();
                if path_parts.len() >= 5 && path_parts[1] == "tenants" && path_parts[3] == "billing" {
                    let tenant_id = path_parts[2];
                    let record_id = path_parts[4];
                    
                    // Parse request body
                    let body = match event.body {
                        Some(aws_lambda_events::encodings::Body::Text(body)) => body,
                        _ => {
                            return Ok(build_response(
                                http::StatusCode::BAD_REQUEST,
                                Some(json!({
                                    "error": "Bad Request",
                                    "message": "Missing request body"
                                })),
                                Some(request_id),
                            ));
                        }
                    };
                    
                    // Parse update billing record request
                    let update_request: metrics_api::UpdateBillingRecordRequest = match serde_json::from_str(&body) {
                        Ok(req) => req,
                        Err(e) => {
                            return Ok(build_response(
                                http::StatusCode::BAD_REQUEST,
                                Some(json!({
                                    "error": "Bad Request",
                                    "message": format!("Invalid request format: {}", e)
                                })),
                                Some(request_id),
                            ));
                        }
                    };
                    
                    metrics_api::handle_update_billing_record(
                        &self.dynamodb_client,
                        &self.table_name,
                        tenant_id,
                        record_id,
                        update_request,
                        claims,
                        request_id,
                    ).await
                } else {
                    Ok(build_response(
                        http::StatusCode::NOT_FOUND,
                        Some(json!({
                            "error": "Not Found",
                            "message": "The requested endpoint does not exist"
                        })),
                        Some(request_id),
                    ))
                },
            
            // Client endpoints
            ("GET", "/api/clients") => {
                self.list_clients(&query_params, &request_id).await
            },
            ("GET", p) if p.starts_with("/api/clients/") => {
                let client_id = p.trim_start_matches("/api/clients/");
                self.get_client(client_id, &request_id).await
            },
            
            // Portfolio endpoints
            ("GET", "/api/portfolios") => {
                self.list_portfolios(&query_params, &request_id).await
            },
            ("GET", p) if p.starts_with("/api/portfolios/") => {
                let portfolio_id = p.trim_start_matches("/api/portfolios/");
                self.get_portfolio(portfolio_id, &request_id).await
            },
            
            // Unknown endpoint
            _ => {
                Ok(build_response(
                    http::StatusCode::NOT_FOUND,
                    Some(json!({
                        "error": "Not Found",
                        "message": "The requested endpoint does not exist"
                    })),
                    Some(request_id),
                ))
            }
        }
    }
    
    /// Process a login request
    async fn process_login_request(
        &self,
        event: ApiGatewayProxyRequest,
        request_id: String,
    ) -> Result<ApiGatewayProxyResponse, Error> {
        // Parse login request
        let body = match event.body {
            Some(aws_lambda_events::encodings::Body::Text(body)) => body,
            _ => {
                return Ok(build_response(
                    http::StatusCode::BAD_REQUEST,
                    Some(json!({
                        "error": "Bad Request",
                        "message": "Missing request body"
                    })),
                    Some(request_id),
                ));
            }
        };
        
        #[derive(Deserialize)]
        struct LoginRequest {
            username: String,
            password: String,
        }
        
        let login_request: LoginRequest = match serde_json::from_str(&body) {
            Ok(req) => req,
            Err(e) => {
                return Ok(build_response(
                    http::StatusCode::BAD_REQUEST,
                    Some(json!({
                        "error": "Bad Request",
                        "message": format!("Invalid request format: {}", e)
                    })),
                    Some(request_id),
                ));
            }
        };
        
        // Get auth service
        let auth_service = match auth_service::get_auth_service().await {
            Ok(service) => service,
            Err(e) => {
                error!("Failed to get auth service: {:?}", e);
                return Ok(build_response(
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                    Some(json!({
                        "error": "Internal Server Error",
                        "message": "Authentication service unavailable"
                    })),
                    Some(request_id),
                ));
            }
        };
        
        // Authenticate user
        match auth_service.authenticate(&login_request.username, &login_request.password).await {
            Ok(Some(user)) => {
                // Generate JWT token
                match auth_service.generate_token(&user).await {
                    Ok(token) => {
                        // Create response
                        #[derive(Serialize)]
                        struct UserResponse {
                            id: String,
                            username: String,
                            email: String,
                            tenant_id: String,
                            roles: Vec<String>,
                            #[serde(skip_serializing_if = "Option::is_none")]
                            first_name: Option<String>,
                            #[serde(skip_serializing_if = "Option::is_none")]
                            last_name: Option<String>,
                        }
                        
                        #[derive(Serialize)]
                        struct LoginResponse {
                            token: String,
                            user: UserResponse,
                        }
                        
                        let user_response = UserResponse {
                            id: user.id,
                            username: user.username,
                            email: user.email,
                            tenant_id: user.tenant_id,
                            roles: user.roles.iter().map(|r| r.as_str().to_string()).collect(),
                            first_name: user.first_name,
                            last_name: user.last_name,
                        };
                        
                        let login_response = LoginResponse {
                            token,
                            user: user_response,
                        };
                        
                        Ok(build_response(
                            http::StatusCode::OK,
                            Some(json!(login_response)),
                            Some(request_id),
                        ))
                    },
                    Err(e) => {
                        error!("Failed to generate token: {:?}", e);
                        Ok(build_response(
                            http::StatusCode::INTERNAL_SERVER_ERROR,
                            Some(json!({
                                "error": "Internal Server Error",
                                "message": "Failed to generate authentication token"
                            })),
                            Some(request_id),
                        ))
                    }
                }
            },
            Ok(None) => {
                Ok(build_response(
                    http::StatusCode::UNAUTHORIZED,
                    Some(json!({
                        "error": "Unauthorized",
                        "message": "Invalid username or password"
                    })),
                    Some(request_id),
                ))
            },
            Err(e) => {
                error!("Authentication error: {:?}", e);
                Ok(build_response(
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                    Some(json!({
                        "error": "Internal Server Error",
                        "message": "Authentication service error"
                    })),
                    Some(request_id),
                ))
            }
        }
    }
    
    /// Health check method
    async fn health_check(&self) -> Result<ApiGatewayProxyResponse, Error> {
        info!("Health check");
        
        // Get overall health status
        let health_status = resilience::get_overall_health_status();
        
        // Get all health check results
        let health_results = resilience::get_health_check_registry().get_all_results();
        
        // Convert health check results to JSON
        let mut services = Vec::new();
        for (name, result) in health_results {
            services.push(serde_json::json!({
                "name": name,
                "status": match result.status {
                    resilience::HealthStatus::Healthy => "healthy",
                    resilience::HealthStatus::Degraded => "degraded",
                    resilience::HealthStatus::Unhealthy => "unhealthy",
                },
                "details": result.details,
                "timestamp": result.timestamp.elapsed().as_secs(),
            }));
        }
        
        // Create response
        let status_code = match health_status {
            resilience::HealthStatus::Healthy => http::StatusCode::OK,
            resilience::HealthStatus::Degraded => http::StatusCode::OK,
            resilience::HealthStatus::Unhealthy => http::StatusCode::SERVICE_UNAVAILABLE,
        };
        
        Ok(build_response(
            status_code,
            Some(json!({
                "status": match health_status {
                    resilience::HealthStatus::Healthy => "healthy",
                    resilience::HealthStatus::Degraded => "degraded",
                    resilience::HealthStatus::Unhealthy => "unhealthy",
                },
                "version": env!("CARGO_PKG_VERSION"),
                "timestamp": Utc::now().to_rfc3339(),
                "services": services,
            })),
            None,
        ))
    }

    // Add a method to get bulkhead metrics for a tenant
    async fn get_tenant_bulkhead_metrics(
        &self,
        event: ApiGatewayProxyRequest,
    ) -> Result<ApiGatewayProxyResponse, Error> {
        // Extract tenant ID from the request
        let tenant_id = self.extract_tenant_id(&event)?;
        
        // Get the bulkhead for the tenant
        let bulkhead = performance_calculator::resilience::get_tenant_bulkhead(&tenant_id);
        
        // Get the metrics
        let metrics = bulkhead.metrics();
        
        // Build the response
        Ok(build_response(
            http::StatusCode::OK,
            serde_json::to_string(&metrics)?,
        ))
    }

    /// Handle a request to calculate performance metrics
    async fn calculate_performance(
        &self,
        event: ApiGatewayProxyRequest,
    ) -> Result<ApiGatewayProxyResponse, Error> {
        // Extract tenant ID from the request
        let tenant_id = self.extract_tenant_id(&event)?;
        
        // Parse the request body
        let request_body = match event.body {
            Some(body) => body,
            None => {
                return Ok(build_response(
                    http::StatusCode::BAD_REQUEST,
                    Some(json!({ "error": "Missing request body" })),
                    None,
                ));
            }
        };
        
        // Parse and validate the request
        let request_json: serde_json::Value = match serde_json::from_str(&request_body) {
            Ok(json) => json,
            Err(e) => {
                return Ok(build_response(
                    http::StatusCode::BAD_REQUEST,
                    Some(json!({ "error": format!("Invalid JSON: {}", e) })),
                    None,
                ));
            }
        };
        
        // Validate calculation parameters
        if let Err(e) = validation::validate_calculation_params(&request_json) {
            return Ok(build_response(
                http::StatusCode::BAD_REQUEST,
                Some(json!({ "error": e.to_string() })),
                None,
            ));
        }
        
        // Parse the validated request
        let request: CalculatePerformanceRequest = match serde_json::from_value(request_json) {
            Ok(req) => req,
            Err(e) => {
                return Ok(build_response(
                    http::StatusCode::BAD_REQUEST,
                    Some(json!({ "error": format!("Invalid request format: {}", e) })),
                    None,
                ));
            }
        };

        // Use bulkhead to isolate tenant operations
        match performance_calculator::resilience::with_tenant_bulkhead_retry_and_circuit_breaker(
            &tenant_id,
            "calculate_performance",
            performance_calculator::resilience::retry::RetryConfig {
                max_retries: 3,
                initial_backoff_ms: 100,
                max_backoff_ms: 1000,
                backoff_multiplier: 2.0,
                jitter_factor: 0.1,
            },
            || async {
                // Perform the calculation
                let result = self.metrics_manager.calculate_performance(
                    &tenant_id,
                    &request.portfolio,
                    &request.calculation_options,
                ).await?;
                
                Ok::<_, Error>(result)
            },
        ).await {
            Ok(result) => {
                // Build the response
                let response = CalculatePerformanceResponse {
                    portfolio_id: request.portfolio.id.clone(),
                    performance_metrics: result,
                };
                
                Ok(build_response(
                    http::StatusCode::OK,
                    serde_json::to_string(&response)?,
                ))
            },
            Err(performance_calculator::resilience::retry::RetryError::OperationError(
                performance_calculator::resilience::circuit_breaker::CircuitBreakerError::OperationError(
                    performance_calculator::resilience::bulkhead::BulkheadError::Full
                )
            )) => {
                // Bulkhead is full - tenant has reached their concurrency limit
                tracing::warn!("Bulkhead full for tenant {}", tenant_id);
                Ok(build_response(
                    http::StatusCode::TOO_MANY_REQUESTS,
                    json!({
                        "error": "Too many concurrent requests",
                        "message": "Your account has reached its concurrency limit. Please try again later."
                    }).to_string(),
                ))
            },
            Err(performance_calculator::resilience::retry::RetryError::OperationError(
                performance_calculator::resilience::circuit_breaker::CircuitBreakerError::OperationError(
                    performance_calculator::resilience::bulkhead::BulkheadError::Timeout
                )
            )) => {
                // Operation timed out
                tracing::warn!("Operation timed out for tenant {}", tenant_id);
                Ok(build_response(
                    http::StatusCode::GATEWAY_TIMEOUT,
                    json!({
                        "error": "Operation timed out",
                        "message": "The operation took too long to complete. Please try again with a smaller dataset."
                    }).to_string(),
                ))
            },
            Err(performance_calculator::resilience::retry::RetryError::OperationError(
                performance_calculator::resilience::circuit_breaker::CircuitBreakerError::Open
            )) => {
                // Circuit breaker is open
                tracing::warn!("Circuit breaker open for calculate_performance");
                Ok(build_response(
                    http::StatusCode::SERVICE_UNAVAILABLE,
                    json!({
                        "error": "Service temporarily unavailable",
                        "message": "The service is currently experiencing issues. Please try again later."
                    }).to_string(),
                ))
            },
            Err(performance_calculator::resilience::retry::RetryError::MaxRetriesExceeded(_, _)) => {
                // Max retries exceeded
                tracing::error!("Max retries exceeded for tenant {}", tenant_id);
                Ok(build_response(
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                    json!({
                        "error": "Internal server error",
                        "message": "The service encountered an error. Please try again later."
                    }).to_string(),
                ))
            },
            Err(err) => {
                // Other errors
                tracing::error!("Error calculating performance for tenant {}: {:?}", tenant_id, err);
                Ok(build_response(
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                    json!({
                        "error": "Internal server error",
                        "message": "The service encountered an error. Please try again later."
                    }).to_string(),
                ))
            },
        }
    }

    /// Extract tenant ID from the request
    fn extract_tenant_id(&self, event: &ApiGatewayProxyRequest) -> Result<String, Error> {
        // First try to get tenant ID from the Authorization header (JWT token)
        if let Some(headers) = &event.headers {
            if let Some(auth_header) = headers.get("Authorization") {
                if auth_header.starts_with("Bearer ") {
                    let token = auth_header.trim_start_matches("Bearer ");
                    // In a real implementation, you would validate the JWT and extract the tenant ID
                    // For now, we'll use a simple placeholder
                    if let Some(tenant_id) = self.extract_tenant_id_from_token(token) {
                        return Ok(tenant_id);
                    }
                }
            }
        }
        
        // If not found in the Authorization header, try to get it from the path parameters
        if let Some(path_parameters) = &event.path_parameters {
            if let Some(tenant_id) = path_parameters.get("tenantId") {
                return Ok(tenant_id.clone());
            }
        }
        
        // If not found in the path parameters, try to get it from the query string parameters
        if let Some(query_string_parameters) = &event.query_string_parameters {
            if let Some(tenant_id) = query_string_parameters.get("tenantId") {
                return Ok(tenant_id.clone());
            }
        }
        
        // If tenant ID is not found, return an error
        Err(Error::InvalidRequest("Tenant ID not found in the request".to_string()))
    }
    
    /// Extract tenant ID from a JWT token
    fn extract_tenant_id_from_token(&self, token: &str) -> Option<String> {
        // In a real implementation, you would validate the JWT and extract the tenant ID
        // For now, we'll use a simple placeholder
        // This would be replaced with actual JWT validation and claims extraction
        
        // Simple placeholder implementation
        if token.contains('.') {
            let parts: Vec<&str> = token.split('.').collect();
            if parts.len() == 3 {
                // In a real implementation, you would base64 decode the payload and parse the JSON
                // For now, we'll just return a placeholder tenant ID
                return Some("default-tenant".to_string());
            }
        }
        
        None
    }

    /// Handle a request to configure chaos testing
    async fn configure_chaos_testing(
        &self,
        event: ApiGatewayProxyRequest,
    ) -> Result<ApiGatewayProxyResponse, Error> {
        // Check if the user has admin permissions
        // In a real implementation, you would validate the JWT and check permissions
        // For now, we'll use a simple placeholder
        let is_admin = self.check_admin_permissions(&event)?;
        
        if !is_admin {
            return Ok(build_response(
                http::StatusCode::FORBIDDEN,
                json!({
                    "error": "Forbidden",
                    "message": "You do not have permission to configure chaos testing"
                }).to_string(),
            ));
        }
        
        // Parse the request body
        let config: performance_calculator::resilience::chaos::ChaosConfig = match event.body {
            Some(body) => serde_json::from_str(&body)?,
            None => {
                return Ok(build_response(
                    http::StatusCode::BAD_REQUEST,
                    json!({ "error": "Missing request body" }).to_string(),
                ));
            }
        };
        
        // Enable or disable chaos testing
        if config.enabled {
            performance_calculator::resilience::enable_chaos_testing(config);
            tracing::info!("Chaos testing enabled with configuration: {:?}", config);
        } else {
            performance_calculator::resilience::disable_chaos_testing();
            tracing::info!("Chaos testing disabled");
        }
        
        // Get the current configuration
        let current_config = performance_calculator::resilience::get_chaos_service().get_config();
        
        // Build the response
        Ok(build_response(
            http::StatusCode::OK,
            serde_json::to_string(&current_config)?,
        ))
    }
    
    /// Handle a request to get chaos testing metrics
    async fn get_chaos_metrics(
        &self,
        event: ApiGatewayProxyRequest,
    ) -> Result<ApiGatewayProxyResponse, Error> {
        // Check if the user has admin permissions
        // In a real implementation, you would validate the JWT and check permissions
        // For now, we'll use a simple placeholder
        let is_admin = self.check_admin_permissions(&event)?;
        
        if !is_admin {
            return Ok(build_response(
                http::StatusCode::FORBIDDEN,
                json!({
                    "error": "Forbidden",
                    "message": "You do not have permission to view chaos metrics"
                }).to_string(),
            ));
        }
        
        // Get the current metrics
        let metrics = performance_calculator::resilience::get_chaos_service().get_metrics();
        
        // Build the response
        Ok(build_response(
            http::StatusCode::OK,
            serde_json::to_string(&metrics)?,
        ))
    }
    
    /// Handle a request to reset chaos testing metrics
    async fn reset_chaos_metrics(
        &self,
        event: ApiGatewayProxyRequest,
    ) -> Result<ApiGatewayProxyResponse, Error> {
        // Check if the user has admin permissions
        // In a real implementation, you would validate the JWT and check permissions
        // For now, we'll use a simple placeholder
        let is_admin = self.check_admin_permissions(&event)?;
        
        if !is_admin {
            return Ok(build_response(
                http::StatusCode::FORBIDDEN,
                json!({
                    "error": "Forbidden",
                    "message": "You do not have permission to reset chaos metrics"
                }).to_string(),
            ));
        }
        
        // Reset the metrics
        performance_calculator::resilience::get_chaos_service().reset_metrics();
        tracing::info!("Chaos testing metrics reset");
        
        // Build the response
        Ok(build_response(
            http::StatusCode::OK,
            json!({ "message": "Chaos testing metrics reset" }).to_string(),
        ))
    }
    
    /// Check if the user has admin permissions
    fn check_admin_permissions(&self, event: &ApiGatewayProxyRequest) -> Result<bool, Error> {
        // In a real implementation, you would validate the JWT and check permissions
        // For now, we'll use a simple placeholder
        
        // Check if the Authorization header is present
        if let Some(headers) = &event.headers {
            if let Some(auth_header) = headers.get("Authorization") {
                if auth_header.starts_with("Bearer ") {
                    let token = auth_header.trim_start_matches("Bearer ");
                    // In a real implementation, you would validate the JWT and check permissions
                    // For now, we'll just check if the token contains "admin"
                    return Ok(token.contains("admin"));
                }
            }
        }
        
        // If no Authorization header is present, check for a special query parameter
        // This is just for testing purposes and would not be used in a real implementation
        if let Some(query_string_parameters) = &event.query_string_parameters {
            if let Some(admin) = query_string_parameters.get("admin") {
                return Ok(admin == "true");
            }
        }
        
        // If no admin credentials are found, return false
        Ok(false)
    }

    async fn list_portfolios(
        &self,
        query_params: &HashMap<String, String>,
        request_id: &str,
    ) -> Result<Response<Body>, Error> {
        info!(request_id = %request_id, "Listing portfolios");
        
        // Extract pagination parameters
        let limit = query_params.get("limit").and_then(|v| v.parse::<u32>().ok());
        let next_token = query_params.get("next_token").cloned();
        
        // Extract filter parameters
        let client_id = query_params.get("client_id").map(|s| s.as_str());
        
        // Create pagination options
        let pagination = Some(PaginationOptions {
            limit,
            next_token,
        });
        
        // Query the repository
        match self.repository.list_portfolios(client_id, pagination).await {
            Ok(result) => {
                // Build response
                let response = json!({
                    "items": result.items,
                    "next_token": result.next_token,
                });
                
                Ok(build_response(
                    http::StatusCode::OK,
                    Some(response),
                    Some(request_id.to_string()),
                ))
            },
            Err(e) => {
                error!(request_id = %request_id, error = %e, "Failed to list portfolios");
                
                let status_code = match e {
                    AppError::NotFound(_) => http::StatusCode::NOT_FOUND,
                    AppError::Validation(_) => http::StatusCode::BAD_REQUEST,
                    _ => http::StatusCode::INTERNAL_SERVER_ERROR,
                };
                
                Ok(build_response(
                    status_code,
                    Some(json!({
                        "error": e.to_string(),
                    })),
                    Some(request_id.to_string()),
                ))
            }
        }
    }

    // Similarly update other list endpoints to support pagination
    async fn list_transactions(
        &self,
        query_params: &HashMap<String, String>,
        request_id: &str,
    ) -> Result<Response<Body>, Error> {
        info!(request_id = %request_id, "Listing transactions");
        
        // Extract pagination parameters
        let limit = query_params.get("limit").and_then(|v| v.parse::<u32>().ok());
        let next_token = query_params.get("next_token").cloned();
        
        // Extract filter parameters
        let account_id = query_params.get("account_id").map(|s| s.as_str());
        
        // Create pagination options
        let pagination = Some(PaginationOptions {
            limit,
            next_token,
        });
        
        // Query the repository
        match self.repository.list_transactions(account_id, pagination).await {
            Ok(result) => {
                // Build response
                let response = json!({
                    "items": result.items,
                    "next_token": result.next_token,
                });
                
                Ok(build_response(
                    http::StatusCode::OK,
                    Some(response),
                    Some(request_id.to_string()),
                ))
            },
            Err(e) => {
                error!(request_id = %request_id, error = %e, "Failed to list transactions");
                
                let status_code = match e {
                    AppError::NotFound(_) => http::StatusCode::NOT_FOUND,
                    AppError::Validation(_) => http::StatusCode::BAD_REQUEST,
                    _ => http::StatusCode::INTERNAL_SERVER_ERROR,
                };
                
                Ok(build_response(
                    status_code,
                    Some(json!({
                        "error": e.to_string(),
                    })),
                    Some(request_id.to_string()),
                ))
            }
        }
    }
}

/// Build a response with the given status code and body
fn build_response(
    status_code: http::StatusCode,
    body: Option<serde_json::Value>,
    request_id: Option<String>,
) -> ApiGatewayProxyResponse {
    let mut headers = std::collections::HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    
    let body_with_request_id = match (body, request_id) {
        (Some(mut body_value), Some(req_id)) => {
            if let serde_json::Value::Object(ref mut map) = body_value {
                map.insert("request_id".to_string(), serde_json::Value::String(req_id));
            }
            Some(body_value.to_string())
        },
        (Some(body_value), None) => Some(body_value.to_string()),
        (None, Some(req_id)) => {
            Some(json!({ "request_id": req_id }).to_string())
        },
        (None, None) => None,
    };
    
    ApiGatewayProxyResponse {
        status_code: status_code.as_u16(),
        headers,
        multi_value_headers: std::collections::HashMap::new(),
        body: body_with_request_id.map(|b| aws_lambda_events::encodings::Body::Text(b)),
        is_base64_encoded: Some(false),
    }
}

/// Main request handler for the API Lambda
async fn handler(
    api_handler: ApiHandler,
) -> impl Fn(ApiGatewayProxyRequest, Context) -> BoxFuture<'static, Result<ApiGatewayProxyResponse, Error>> + Clone {
    move |event: ApiGatewayProxyRequest, _ctx: Context| {
        let api_handler = api_handler.clone();
        Box::pin(async move {
            // Extract the HTTP method and path
            let method = event.http_method.as_deref().unwrap_or("GET");
            let path = event.path.as_deref().unwrap_or("/");
            
            // Route the request to the appropriate handler
            match (method, path) {
                // Health check endpoint
                ("GET", "/health") => api_handler.health_check(event).await,
                
                // Performance calculation endpoint
                ("POST", "/calculate") => api_handler.calculate_performance(event).await,
                
                // Tenant bulkhead metrics endpoint
                ("GET", "/tenant/bulkhead/metrics") => api_handler.get_tenant_bulkhead_metrics(event).await,
                
                // Chaos testing endpoints
                ("POST", "/chaos/configure") => api_handler.configure_chaos_testing(event).await,
                ("GET", "/chaos/metrics") => api_handler.get_chaos_metrics(event).await,
                ("POST", "/chaos/reset") => api_handler.reset_chaos_metrics(event).await,
                
                // Unknown endpoint
                _ => Ok(build_response(
                    http::StatusCode::NOT_FOUND,
                    json!({ "error": "Not found" }).to_string(),
                )),
            }
        })
    }
} 