use aws_lambda_events::event::apigw::ApiGatewayProxyResponse;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use http::StatusCode;
use lambda_runtime::Error;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tracing::{error, info};
use uuid::Uuid;
use chrono::Utc;

use crate::auth_service::{AuthService, Permission, Claims};
use performance_calculator::calculations::tenant::{
    Tenant, TenantManager, TenantStatus, SubscriptionTier, ResourceLimits, DynamoDbTenantManager
};

/// Request to create a new tenant
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTenantRequest {
    /// Tenant name
    pub name: String,
    /// Tenant description (optional)
    pub description: Option<String>,
    /// Subscription tier
    pub subscription_tier: String,
    /// Custom configuration (optional)
    pub custom_config: Option<HashMap<String, String>>,
    /// Resource limits (optional, will use defaults for tier if not provided)
    pub resource_limits: Option<ResourceLimitsRequest>,
}

/// Request to update an existing tenant
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTenantRequest {
    /// Tenant name (optional)
    pub name: Option<String>,
    /// Tenant description (optional)
    pub description: Option<String>,
    /// Subscription tier (optional)
    pub subscription_tier: Option<String>,
    /// Custom configuration (optional)
    pub custom_config: Option<HashMap<String, String>>,
    /// Resource limits (optional)
    pub resource_limits: Option<ResourceLimitsRequest>,
}

/// Resource limits in a request
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceLimitsRequest {
    /// Maximum number of portfolios
    pub max_portfolios: Option<usize>,
    /// Maximum number of accounts per portfolio
    pub max_accounts_per_portfolio: Option<usize>,
    /// Maximum number of securities
    pub max_securities: Option<usize>,
    /// Maximum number of transactions
    pub max_transactions: Option<usize>,
    /// Maximum number of concurrent calculations
    pub max_concurrent_calculations: Option<usize>,
    /// Maximum storage in bytes
    pub max_storage_bytes: Option<u64>,
    /// Maximum API requests per minute
    pub max_api_requests_per_minute: Option<u32>,
}

/// Response for a tenant
#[derive(Debug, Serialize, Deserialize)]
pub struct TenantResponse {
    /// Tenant ID
    pub id: String,
    /// Tenant name
    pub name: String,
    /// Tenant description
    pub description: Option<String>,
    /// Tenant status
    pub status: String,
    /// Subscription tier
    pub subscription_tier: String,
    /// Resource limits
    pub resource_limits: ResourceLimitsResponse,
    /// Custom configuration
    pub custom_config: HashMap<String, String>,
    /// Created timestamp
    pub created_at: String,
    /// Last updated timestamp
    pub updated_at: String,
}

/// Resource limits in a response
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceLimitsResponse {
    /// Maximum number of portfolios
    pub max_portfolios: usize,
    /// Maximum number of accounts per portfolio
    pub max_accounts_per_portfolio: usize,
    /// Maximum number of securities
    pub max_securities: usize,
    /// Maximum number of transactions
    pub max_transactions: usize,
    /// Maximum number of concurrent calculations
    pub max_concurrent_calculations: usize,
    /// Maximum storage in bytes
    pub max_storage_bytes: u64,
    /// Maximum API requests per minute
    pub max_api_requests_per_minute: u32,
}

/// Convert a Tenant to a TenantResponse
fn tenant_to_response(tenant: &Tenant) -> TenantResponse {
    let status = match tenant.status {
        TenantStatus::Active => "active",
        TenantStatus::Suspended => "suspended",
        TenantStatus::Deactivated => "deactivated",
    };
    
    let subscription_tier = match tenant.subscription_tier {
        SubscriptionTier::Free => "free",
        SubscriptionTier::Basic => "basic",
        SubscriptionTier::Professional => "professional",
        SubscriptionTier::Enterprise => "enterprise",
        SubscriptionTier::Custom => "custom",
    };
    
    let resource_limits = ResourceLimitsResponse {
        max_portfolios: tenant.resource_limits.max_portfolios,
        max_accounts_per_portfolio: tenant.resource_limits.max_accounts_per_portfolio,
        max_securities: tenant.resource_limits.max_securities,
        max_transactions: tenant.resource_limits.max_transactions,
        max_concurrent_calculations: tenant.resource_limits.max_concurrent_calculations,
        max_storage_bytes: tenant.resource_limits.max_storage_bytes,
        max_api_requests_per_minute: tenant.resource_limits.max_api_requests_per_minute,
    };
    
    TenantResponse {
        id: tenant.id.clone(),
        name: tenant.name.clone(),
        description: tenant.description.clone(),
        status: status.to_string(),
        subscription_tier: subscription_tier.to_string(),
        resource_limits,
        custom_config: tenant.custom_config.clone(),
        created_at: tenant.created_at.to_rfc3339(),
        updated_at: tenant.updated_at.to_rfc3339(),
    }
}

/// Parse a subscription tier from a string
fn parse_subscription_tier(tier: &str) -> Result<SubscriptionTier, String> {
    match tier.to_lowercase().as_str() {
        "free" => Ok(SubscriptionTier::Free),
        "basic" => Ok(SubscriptionTier::Basic),
        "professional" => Ok(SubscriptionTier::Professional),
        "enterprise" => Ok(SubscriptionTier::Enterprise),
        "custom" => Ok(SubscriptionTier::Custom),
        _ => Err(format!("Invalid subscription tier: {}", tier)),
    }
}

/// Build a response with the given status code and body
fn build_response(
    status_code: StatusCode,
    body: Option<serde_json::Value>,
    request_id: Option<String>,
) -> ApiGatewayProxyResponse {
    let mut headers = HashMap::new();
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
        multi_value_headers: HashMap::new(),
        body: body_with_request_id.map(|b| aws_lambda_events::encodings::Body::Text(b)),
        is_base64_encoded: Some(false),
    }
}

/// Handle a request to list all tenants
pub async fn handle_list_tenants(
    dynamodb_client: &DynamoDbClient,
    table_name: &str,
    query_params: &HashMap<String, String>,
    claims: &Claims,
    request_id: String,
) -> Result<ApiGatewayProxyResponse, Error> {
    // Check if user has permission to view tenants
    if !claims.roles.iter().any(|r| r == "admin") {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(json!({
                "error": "Forbidden",
                "message": "You do not have permission to view tenants"
            })),
            Some(request_id),
        ));
    }
    
    // Parse query parameters
    let limit = query_params.get("limit").and_then(|l| l.parse::<usize>().ok());
    let offset = query_params.get("offset").and_then(|o| o.parse::<usize>().ok());
    
    // Get tenant manager
    let tenant_manager = DynamoDbTenantManager::new(
        dynamodb_client.clone(),
        table_name.to_string(),
    );
    
    // List tenants
    match tenant_manager.list_tenants(limit, offset).await {
        Ok(tenants) => {
            let tenant_responses: Vec<TenantResponse> = tenants.iter()
                .map(|t| tenant_to_response(t))
                .collect();
            
            Ok(build_response(
                StatusCode::OK,
                Some(json!({
                    "tenants": tenant_responses,
                    "count": tenant_responses.len(),
                })),
                Some(request_id),
            ))
        },
        Err(e) => {
            error!("Failed to list tenants: {:?}", e);
            Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(json!({
                    "error": "Internal Server Error",
                    "message": format!("Failed to list tenants: {}", e)
                })),
                Some(request_id),
            ))
        }
    }
}

/// Handle a request to get a tenant by ID
pub async fn handle_get_tenant(
    dynamodb_client: &DynamoDbClient,
    table_name: &str,
    tenant_id: &str,
    claims: &Claims,
    request_id: String,
) -> Result<ApiGatewayProxyResponse, Error> {
    // Check if user has permission to view tenants
    if !claims.roles.iter().any(|r| r == "admin") && claims.tenant_id != tenant_id {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(json!({
                "error": "Forbidden",
                "message": "You do not have permission to view this tenant"
            })),
            Some(request_id),
        ));
    }
    
    // Get tenant manager
    let tenant_manager = DynamoDbTenantManager::new(
        dynamodb_client.clone(),
        table_name.to_string(),
    );
    
    // Get tenant
    match tenant_manager.get_tenant(tenant_id).await {
        Ok(Some(tenant)) => {
            let tenant_response = tenant_to_response(&tenant);
            
            Ok(build_response(
                StatusCode::OK,
                Some(json!({
                    "tenant": tenant_response
                })),
                Some(request_id),
            ))
        },
        Ok(None) => {
            Ok(build_response(
                StatusCode::NOT_FOUND,
                Some(json!({
                    "error": "Not Found",
                    "message": format!("Tenant with ID {} not found", tenant_id)
                })),
                Some(request_id),
            ))
        },
        Err(e) => {
            error!("Failed to get tenant: {:?}", e);
            Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(json!({
                    "error": "Internal Server Error",
                    "message": format!("Failed to get tenant: {}", e)
                })),
                Some(request_id),
            ))
        }
    }
}

/// Handle a request to create a new tenant
pub async fn handle_create_tenant(
    dynamodb_client: &DynamoDbClient,
    table_name: &str,
    request: CreateTenantRequest,
    claims: &Claims,
    request_id: String,
) -> Result<ApiGatewayProxyResponse, Error> {
    // Check if user has permission to create tenants
    if !claims.roles.iter().any(|r| r == "admin") {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(json!({
                "error": "Forbidden",
                "message": "You do not have permission to create tenants"
            })),
            Some(request_id),
        ));
    }
    
    // Parse subscription tier
    let subscription_tier = match parse_subscription_tier(&request.subscription_tier) {
        Ok(tier) => tier,
        Err(e) => {
            return Ok(build_response(
                StatusCode::BAD_REQUEST,
                Some(json!({
                    "error": "Bad Request",
                    "message": e
                })),
                Some(request_id),
            ));
        }
    };
    
    // Create tenant
    let mut tenant = Tenant::new(&request.name, request.description.as_deref(), subscription_tier);
    
    // Set custom config if provided
    if let Some(custom_config) = request.custom_config {
        tenant.custom_config = custom_config;
    }
    
    // Set resource limits if provided
    if let Some(limits_req) = request.resource_limits {
        let mut limits = tenant.resource_limits.clone();
        
        if let Some(max_portfolios) = limits_req.max_portfolios {
            limits.max_portfolios = max_portfolios;
        }
        
        if let Some(max_accounts_per_portfolio) = limits_req.max_accounts_per_portfolio {
            limits.max_accounts_per_portfolio = max_accounts_per_portfolio;
        }
        
        if let Some(max_securities) = limits_req.max_securities {
            limits.max_securities = max_securities;
        }
        
        if let Some(max_transactions) = limits_req.max_transactions {
            limits.max_transactions = max_transactions;
        }
        
        if let Some(max_concurrent_calculations) = limits_req.max_concurrent_calculations {
            limits.max_concurrent_calculations = max_concurrent_calculations;
        }
        
        if let Some(max_storage_bytes) = limits_req.max_storage_bytes {
            limits.max_storage_bytes = max_storage_bytes;
        }
        
        if let Some(max_api_requests_per_minute) = limits_req.max_api_requests_per_minute {
            limits.max_api_requests_per_minute = max_api_requests_per_minute;
        }
        
        tenant.resource_limits = limits;
    }
    
    // Get tenant manager
    let tenant_manager = DynamoDbTenantManager::new(
        dynamodb_client.clone(),
        table_name.to_string(),
    );
    
    // Create tenant
    match tenant_manager.create_tenant(tenant).await {
        Ok(created_tenant) => {
            let tenant_response = tenant_to_response(&created_tenant);
            
            Ok(build_response(
                StatusCode::CREATED,
                Some(json!({
                    "tenant": tenant_response
                })),
                Some(request_id),
            ))
        },
        Err(e) => {
            error!("Failed to create tenant: {:?}", e);
            Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(json!({
                    "error": "Internal Server Error",
                    "message": format!("Failed to create tenant: {}", e)
                })),
                Some(request_id),
            ))
        }
    }
}

/// Handle a request to update a tenant
pub async fn handle_update_tenant(
    dynamodb_client: &DynamoDbClient,
    table_name: &str,
    tenant_id: &str,
    request: UpdateTenantRequest,
    claims: &Claims,
    request_id: String,
) -> Result<ApiGatewayProxyResponse, Error> {
    // Check if user has permission to update tenants
    if !claims.roles.iter().any(|r| r == "admin") {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(json!({
                "error": "Forbidden",
                "message": "You do not have permission to update tenants"
            })),
            Some(request_id),
        ));
    }
    
    // Get tenant manager
    let tenant_manager = DynamoDbTenantManager::new(
        dynamodb_client.clone(),
        table_name.to_string(),
    );
    
    // Get existing tenant
    let existing_tenant = match tenant_manager.get_tenant(tenant_id).await {
        Ok(Some(tenant)) => tenant,
        Ok(None) => {
            return Ok(build_response(
                StatusCode::NOT_FOUND,
                Some(json!({
                    "error": "Not Found",
                    "message": format!("Tenant with ID {} not found", tenant_id)
                })),
                Some(request_id),
            ));
        },
        Err(e) => {
            error!("Failed to get tenant: {:?}", e);
            return Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(json!({
                    "error": "Internal Server Error",
                    "message": format!("Failed to get tenant: {}", e)
                })),
                Some(request_id),
            ));
        }
    };
    
    // Update tenant
    let mut updated_tenant = existing_tenant.clone();
    updated_tenant.updated_at = Utc::now();
    
    // Update name if provided
    if let Some(name) = request.name {
        updated_tenant.name = name;
    }
    
    // Update description if provided
    if let Some(description) = request.description {
        updated_tenant.description = Some(description);
    }
    
    // Update subscription tier if provided
    if let Some(tier_str) = request.subscription_tier {
        match parse_subscription_tier(&tier_str) {
            Ok(tier) => {
                updated_tenant.subscription_tier = tier;
            },
            Err(e) => {
                return Ok(build_response(
                    StatusCode::BAD_REQUEST,
                    Some(json!({
                        "error": "Bad Request",
                        "message": e
                    })),
                    Some(request_id),
                ));
            }
        }
    }
    
    // Update custom config if provided
    if let Some(custom_config) = request.custom_config {
        updated_tenant.custom_config = custom_config;
    }
    
    // Update resource limits if provided
    if let Some(limits_req) = request.resource_limits {
        let mut limits = updated_tenant.resource_limits.clone();
        
        if let Some(max_portfolios) = limits_req.max_portfolios {
            limits.max_portfolios = max_portfolios;
        }
        
        if let Some(max_accounts_per_portfolio) = limits_req.max_accounts_per_portfolio {
            limits.max_accounts_per_portfolio = max_accounts_per_portfolio;
        }
        
        if let Some(max_securities) = limits_req.max_securities {
            limits.max_securities = max_securities;
        }
        
        if let Some(max_transactions) = limits_req.max_transactions {
            limits.max_transactions = max_transactions;
        }
        
        if let Some(max_concurrent_calculations) = limits_req.max_concurrent_calculations {
            limits.max_concurrent_calculations = max_concurrent_calculations;
        }
        
        if let Some(max_storage_bytes) = limits_req.max_storage_bytes {
            limits.max_storage_bytes = max_storage_bytes;
        }
        
        if let Some(max_api_requests_per_minute) = limits_req.max_api_requests_per_minute {
            limits.max_api_requests_per_minute = max_api_requests_per_minute;
        }
        
        updated_tenant.resource_limits = limits;
    }
    
    // Update tenant
    match tenant_manager.update_tenant(updated_tenant).await {
        Ok(tenant) => {
            let tenant_response = tenant_to_response(&tenant);
            
            Ok(build_response(
                StatusCode::OK,
                Some(json!({
                    "tenant": tenant_response
                })),
                Some(request_id),
            ))
        },
        Err(e) => {
            error!("Failed to update tenant: {:?}", e);
            Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(json!({
                    "error": "Internal Server Error",
                    "message": format!("Failed to update tenant: {}", e)
                })),
                Some(request_id),
            ))
        }
    }
}

/// Handle a request to delete a tenant
pub async fn handle_delete_tenant(
    dynamodb_client: &DynamoDbClient,
    table_name: &str,
    tenant_id: &str,
    claims: &Claims,
    request_id: String,
) -> Result<ApiGatewayProxyResponse, Error> {
    // Check if user has permission to delete tenants
    if !claims.roles.iter().any(|r| r == "admin") {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(json!({
                "error": "Forbidden",
                "message": "You do not have permission to delete tenants"
            })),
            Some(request_id),
        ));
    }
    
    // Get tenant manager
    let tenant_manager = DynamoDbTenantManager::new(
        dynamodb_client.clone(),
        table_name.to_string(),
    );
    
    // Delete tenant
    match tenant_manager.delete_tenant(tenant_id).await {
        Ok(()) => {
            Ok(build_response(
                StatusCode::NO_CONTENT,
                None,
                Some(request_id),
            ))
        },
        Err(e) => {
            error!("Failed to delete tenant: {:?}", e);
            Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(json!({
                    "error": "Internal Server Error",
                    "message": format!("Failed to delete tenant: {}", e)
                })),
                Some(request_id),
            ))
        }
    }
}

/// Handle a request to activate a tenant
pub async fn handle_activate_tenant(
    dynamodb_client: &DynamoDbClient,
    table_name: &str,
    tenant_id: &str,
    claims: &Claims,
    request_id: String,
) -> Result<ApiGatewayProxyResponse, Error> {
    // Check if user has permission to manage tenants
    if !claims.roles.iter().any(|r| r == "admin") {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(json!({
                "error": "Forbidden",
                "message": "You do not have permission to manage tenants"
            })),
            Some(request_id),
        ));
    }
    
    // Get tenant manager
    let tenant_manager = DynamoDbTenantManager::new(
        dynamodb_client.clone(),
        table_name.to_string(),
    );
    
    // Activate tenant
    match tenant_manager.activate_tenant(tenant_id).await {
        Ok(tenant) => {
            let tenant_response = tenant_to_response(&tenant);
            
            Ok(build_response(
                StatusCode::OK,
                Some(json!({
                    "tenant": tenant_response
                })),
                Some(request_id),
            ))
        },
        Err(e) => {
            error!("Failed to activate tenant: {:?}", e);
            Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(json!({
                    "error": "Internal Server Error",
                    "message": format!("Failed to activate tenant: {}", e)
                })),
                Some(request_id),
            ))
        }
    }
}

/// Handle a request to suspend a tenant
pub async fn handle_suspend_tenant(
    dynamodb_client: &DynamoDbClient,
    table_name: &str,
    tenant_id: &str,
    claims: &Claims,
    request_id: String,
) -> Result<ApiGatewayProxyResponse, Error> {
    // Check if user has permission to manage tenants
    if !claims.roles.iter().any(|r| r == "admin") {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(json!({
                "error": "Forbidden",
                "message": "You do not have permission to manage tenants"
            })),
            Some(request_id),
        ));
    }
    
    // Get tenant manager
    let tenant_manager = DynamoDbTenantManager::new(
        dynamodb_client.clone(),
        table_name.to_string(),
    );
    
    // Suspend tenant
    match tenant_manager.suspend_tenant(tenant_id).await {
        Ok(tenant) => {
            let tenant_response = tenant_to_response(&tenant);
            
            Ok(build_response(
                StatusCode::OK,
                Some(json!({
                    "tenant": tenant_response
                })),
                Some(request_id),
            ))
        },
        Err(e) => {
            error!("Failed to suspend tenant: {:?}", e);
            Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(json!({
                    "error": "Internal Server Error",
                    "message": format!("Failed to suspend tenant: {}", e)
                })),
                Some(request_id),
            ))
        }
    }
}

/// Handle a request to deactivate a tenant
pub async fn handle_deactivate_tenant(
    dynamodb_client: &DynamoDbClient,
    table_name: &str,
    tenant_id: &str,
    claims: &Claims,
    request_id: String,
) -> Result<ApiGatewayProxyResponse, Error> {
    // Check if user has permission to manage tenants
    if !claims.roles.iter().any(|r| r == "admin") {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(json!({
                "error": "Forbidden",
                "message": "You do not have permission to manage tenants"
            })),
            Some(request_id),
        ));
    }
    
    // Get tenant manager
    let tenant_manager = DynamoDbTenantManager::new(
        dynamodb_client.clone(),
        table_name.to_string(),
    );
    
    // Deactivate tenant
    match tenant_manager.deactivate_tenant(tenant_id).await {
        Ok(tenant) => {
            let tenant_response = tenant_to_response(&tenant);
            
            Ok(build_response(
                StatusCode::OK,
                Some(json!({
                    "tenant": tenant_response
                })),
                Some(request_id),
            ))
        },
        Err(e) => {
            error!("Failed to deactivate tenant: {:?}", e);
            Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(json!({
                    "error": "Internal Server Error",
                    "message": format!("Failed to deactivate tenant: {}", e)
                })),
                Some(request_id),
            ))
        }
    }
} 