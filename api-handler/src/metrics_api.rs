use aws_lambda_events::event::apigw::ApiGatewayProxyResponse;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::auth_service::Claims;
use performance_calculator::calculations::tenant::{
    TenantUsageMetrics, 
    TenantBillingRecord, 
    BillingStatus, 
    get_tenant_metrics_manager
};

/// Response for tenant usage metrics
#[derive(Debug, Serialize)]
pub struct TenantUsageMetricsResponse {
    /// Tenant ID
    pub tenant_id: String,
    
    /// Current number of portfolios
    pub portfolio_count: usize,
    
    /// Current number of accounts
    pub account_count: usize,
    
    /// Current number of securities
    pub security_count: usize,
    
    /// Current number of transactions
    pub transaction_count: usize,
    
    /// Current storage usage in bytes
    pub storage_usage_bytes: u64,
    
    /// API requests in the current period
    pub api_requests: u32,
    
    /// Concurrent calculations currently running
    pub concurrent_calculations: usize,
    
    /// Resource usage percentages
    pub usage_percentages: HashMap<String, f64>,
    
    /// Timestamp when metrics were last updated
    pub updated_at: String,
}

/// Response for tenant billing record
#[derive(Debug, Serialize)]
pub struct TenantBillingRecordResponse {
    /// Record ID
    pub id: String,
    
    /// Tenant ID
    pub tenant_id: String,
    
    /// Billing period start
    pub period_start: String,
    
    /// Billing period end
    pub period_end: String,
    
    /// Subscription tier
    pub subscription_tier: String,
    
    /// Base amount
    pub base_amount: f64,
    
    /// Additional charges
    pub additional_charges: HashMap<String, f64>,
    
    /// Total amount
    pub total_amount: f64,
    
    /// Currency code (e.g., USD)
    pub currency: String,
    
    /// Payment status
    pub status: String,
    
    /// Created timestamp
    pub created_at: String,
    
    /// Updated timestamp
    pub updated_at: String,
}

/// Request to update a billing record
#[derive(Debug, Deserialize)]
pub struct UpdateBillingRecordRequest {
    /// Additional charges to add
    pub additional_charges: Option<HashMap<String, f64>>,
    
    /// Payment status
    pub status: Option<String>,
}

/// Convert a TenantUsageMetrics to a TenantUsageMetricsResponse
fn metrics_to_response(metrics: &TenantUsageMetrics, resource_limits: &performance_calculator::calculations::tenant::ResourceLimits) -> TenantUsageMetricsResponse {
    let mut usage_percentages = HashMap::new();
    
    usage_percentages.insert("portfolios".to_string(), metrics.usage_percentage("portfolios", resource_limits));
    usage_percentages.insert("accounts".to_string(), metrics.usage_percentage("accounts", resource_limits));
    usage_percentages.insert("securities".to_string(), metrics.usage_percentage("securities", resource_limits));
    usage_percentages.insert("transactions".to_string(), metrics.usage_percentage("transactions", resource_limits));
    usage_percentages.insert("storage".to_string(), metrics.usage_percentage("storage", resource_limits));
    usage_percentages.insert("api_requests".to_string(), metrics.usage_percentage("api_requests", resource_limits));
    usage_percentages.insert("calculations".to_string(), metrics.usage_percentage("calculations", resource_limits));
    
    TenantUsageMetricsResponse {
        tenant_id: metrics.tenant_id.clone(),
        portfolio_count: metrics.portfolio_count,
        account_count: metrics.account_count,
        security_count: metrics.security_count,
        transaction_count: metrics.transaction_count,
        storage_usage_bytes: metrics.storage_usage_bytes,
        api_requests: metrics.api_requests,
        concurrent_calculations: metrics.concurrent_calculations,
        usage_percentages,
        updated_at: metrics.updated_at.to_rfc3339(),
    }
}

/// Convert a TenantBillingRecord to a TenantBillingRecordResponse
fn billing_to_response(record: &TenantBillingRecord) -> TenantBillingRecordResponse {
    TenantBillingRecordResponse {
        id: record.id.clone(),
        tenant_id: record.tenant_id.clone(),
        period_start: record.period_start.to_rfc3339(),
        period_end: record.period_end.to_rfc3339(),
        subscription_tier: record.subscription_tier.clone(),
        base_amount: record.base_amount,
        additional_charges: record.additional_charges.clone(),
        total_amount: record.total_amount,
        currency: record.currency.clone(),
        status: match record.status {
            BillingStatus::Pending => "pending".to_string(),
            BillingStatus::Paid => "paid".to_string(),
            BillingStatus::Failed => "failed".to_string(),
            BillingStatus::Cancelled => "cancelled".to_string(),
        },
        created_at: record.created_at.to_rfc3339(),
        updated_at: record.updated_at.to_rfc3339(),
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
            Some(serde_json::json!({ "request_id": req_id }).to_string())
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

/// Handle getting tenant usage metrics
pub async fn handle_get_tenant_metrics(
    dynamodb_client: &DynamoDbClient,
    table_name: &str,
    tenant_id: &str,
    claims: &Claims,
    request_id: String,
) -> Result<ApiGatewayProxyResponse, aws_lambda_events::error::Error> {
    info!("Getting metrics for tenant {}, request_id: {}", tenant_id, request_id);
    
    // Check if user has permission to view tenant metrics
    if !claims.has_permission("tenant:metrics:read") && claims.tenant_id != tenant_id {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(serde_json::json!({
                "error": "Forbidden",
                "message": "You do not have permission to view metrics for this tenant"
            })),
            Some(request_id),
        ));
    }
    
    // Get tenant manager to retrieve tenant info
    let tenant_manager = match performance_calculator::calculations::tenant::get_tenant_manager(
        &performance_calculator::calculations::tenant::TenantConfig::default()
    ).await {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to get tenant manager: {:?}", e);
            return Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Tenant management service unavailable"
                })),
                Some(request_id),
            ));
        }
    };
    
    // Get tenant to check if it exists and get resource limits
    let tenant = match tenant_manager.get_tenant(tenant_id).await {
        Ok(Some(tenant)) => tenant,
        Ok(None) => {
            return Ok(build_response(
                StatusCode::NOT_FOUND,
                Some(serde_json::json!({
                    "error": "Not Found",
                    "message": "Tenant not found"
                })),
                Some(request_id),
            ));
        },
        Err(e) => {
            error!("Failed to get tenant: {:?}", e);
            return Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Failed to get tenant information"
                })),
                Some(request_id),
            ));
        }
    };
    
    // Get metrics manager
    let metrics_manager = match get_tenant_metrics_manager().await {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to get metrics manager: {:?}", e);
            return Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Metrics service unavailable"
                })),
                Some(request_id),
            ));
        }
    };
    
    // Get usage metrics
    match metrics_manager.get_usage_metrics(tenant_id).await {
        Ok(metrics) => {
            let response = metrics_to_response(&metrics, &tenant.resource_limits);
            
            Ok(build_response(
                StatusCode::OK,
                Some(serde_json::json!(response)),
                Some(request_id),
            ))
        },
        Err(e) => {
            error!("Failed to get usage metrics: {:?}", e);
            Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Failed to get usage metrics"
                })),
                Some(request_id),
            ))
        }
    }
}

/// Handle getting tenant billing records
pub async fn handle_get_tenant_billing_records(
    dynamodb_client: &DynamoDbClient,
    table_name: &str,
    tenant_id: &str,
    query_params: &HashMap<String, String>,
    claims: &Claims,
    request_id: String,
) -> Result<ApiGatewayProxyResponse, aws_lambda_events::error::Error> {
    info!("Getting billing records for tenant {}, request_id: {}", tenant_id, request_id);
    
    // Check if user has permission to view tenant billing
    if !claims.has_permission("tenant:billing:read") && claims.tenant_id != tenant_id {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(serde_json::json!({
                "error": "Forbidden",
                "message": "You do not have permission to view billing records for this tenant"
            })),
            Some(request_id),
        ));
    }
    
    // Get tenant manager to check if tenant exists
    let tenant_manager = match performance_calculator::calculations::tenant::get_tenant_manager(
        &performance_calculator::calculations::tenant::TenantConfig::default()
    ).await {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to get tenant manager: {:?}", e);
            return Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Tenant management service unavailable"
                })),
                Some(request_id),
            ));
        }
    };
    
    // Check if tenant exists
    match tenant_manager.tenant_exists(tenant_id).await {
        Ok(exists) => {
            if !exists {
                return Ok(build_response(
                    StatusCode::NOT_FOUND,
                    Some(serde_json::json!({
                        "error": "Not Found",
                        "message": "Tenant not found"
                    })),
                    Some(request_id),
                ));
            }
        },
        Err(e) => {
            error!("Failed to check if tenant exists: {:?}", e);
            return Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Failed to check if tenant exists"
                })),
                Some(request_id),
            ));
        }
    }
    
    // Get metrics manager
    let metrics_manager = match get_tenant_metrics_manager().await {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to get metrics manager: {:?}", e);
            return Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Metrics service unavailable"
                })),
                Some(request_id),
            ));
        }
    };
    
    // Parse query parameters
    let start_date = query_params.get("startDate").and_then(|s| {
        match chrono::DateTime::parse_from_rfc3339(s) {
            Ok(dt) => Some(dt.with_timezone(&Utc)),
            Err(_) => None,
        }
    });
    
    let end_date = query_params.get("endDate").and_then(|s| {
        match chrono::DateTime::parse_from_rfc3339(s) {
            Ok(dt) => Some(dt.with_timezone(&Utc)),
            Err(_) => None,
        }
    });
    
    let limit = query_params.get("limit").and_then(|l| l.parse::<usize>().ok()).unwrap_or(50);
    let offset = query_params.get("offset").and_then(|o| o.parse::<usize>().ok()).unwrap_or(0);
    
    // Get billing records
    match metrics_manager.get_billing_records(tenant_id, start_date, end_date, Some(limit), Some(offset)).await {
        Ok(records) => {
            let responses: Vec<TenantBillingRecordResponse> = records.iter()
                .map(|r| billing_to_response(r))
                .collect();
            
            Ok(build_response(
                StatusCode::OK,
                Some(serde_json::json!({
                    "records": responses,
                    "count": responses.len(),
                    "limit": limit,
                    "offset": offset
                })),
                Some(request_id),
            ))
        },
        Err(e) => {
            error!("Failed to get billing records: {:?}", e);
            Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Failed to get billing records"
                })),
                Some(request_id),
            ))
        }
    }
}

/// Handle getting a specific billing record
pub async fn handle_get_billing_record(
    dynamodb_client: &DynamoDbClient,
    table_name: &str,
    tenant_id: &str,
    record_id: &str,
    claims: &Claims,
    request_id: String,
) -> Result<ApiGatewayProxyResponse, aws_lambda_events::error::Error> {
    info!("Getting billing record {} for tenant {}, request_id: {}", record_id, tenant_id, request_id);
    
    // Check if user has permission to view tenant billing
    if !claims.has_permission("tenant:billing:read") && claims.tenant_id != tenant_id {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(serde_json::json!({
                "error": "Forbidden",
                "message": "You do not have permission to view billing records for this tenant"
            })),
            Some(request_id),
        ));
    }
    
    // Get metrics manager
    let metrics_manager = match get_tenant_metrics_manager().await {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to get metrics manager: {:?}", e);
            return Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Metrics service unavailable"
                })),
                Some(request_id),
            ));
        }
    };
    
    // Get all billing records for the tenant (we'll filter for the specific one)
    match metrics_manager.get_billing_records(tenant_id, None, None, None, None).await {
        Ok(records) => {
            // Find the specific record
            if let Some(record) = records.iter().find(|r| r.id == record_id) {
                Ok(build_response(
                    StatusCode::OK,
                    Some(serde_json::json!(billing_to_response(record))),
                    Some(request_id),
                ))
            } else {
                Ok(build_response(
                    StatusCode::NOT_FOUND,
                    Some(serde_json::json!({
                        "error": "Not Found",
                        "message": "Billing record not found"
                    })),
                    Some(request_id),
                ))
            }
        },
        Err(e) => {
            error!("Failed to get billing records: {:?}", e);
            Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Failed to get billing records"
                })),
                Some(request_id),
            ))
        }
    }
}

/// Handle updating a billing record
pub async fn handle_update_billing_record(
    dynamodb_client: &DynamoDbClient,
    table_name: &str,
    tenant_id: &str,
    record_id: &str,
    update_request: UpdateBillingRecordRequest,
    claims: &Claims,
    request_id: String,
) -> Result<ApiGatewayProxyResponse, aws_lambda_events::error::Error> {
    info!("Updating billing record {} for tenant {}, request_id: {}", record_id, tenant_id, request_id);
    
    // Check if user has permission to update tenant billing
    if !claims.has_permission("tenant:billing:update") {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(serde_json::json!({
                "error": "Forbidden",
                "message": "You do not have permission to update billing records"
            })),
            Some(request_id),
        ));
    }
    
    // Get metrics manager
    let metrics_manager = match get_tenant_metrics_manager().await {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to get metrics manager: {:?}", e);
            return Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Metrics service unavailable"
                })),
                Some(request_id),
            ));
        }
    };
    
    // Get all billing records for the tenant (we'll find and update the specific one)
    match metrics_manager.get_billing_records(tenant_id, None, None, None, None).await {
        Ok(records) => {
            // Find the specific record
            if let Some(record) = records.iter().find(|r| r.id == record_id) {
                let mut updated_record = record.clone();
                
                // Update additional charges if provided
                if let Some(charges) = &update_request.additional_charges {
                    for (description, amount) in charges {
                        updated_record.add_charge(description, *amount);
                    }
                }
                
                // Update status if provided
                if let Some(status_str) = &update_request.status {
                    let status = match status_str.as_str() {
                        "pending" => BillingStatus::Pending,
                        "paid" => BillingStatus::Paid,
                        "failed" => BillingStatus::Failed,
                        "cancelled" => BillingStatus::Cancelled,
                        _ => {
                            return Ok(build_response(
                                StatusCode::BAD_REQUEST,
                                Some(serde_json::json!({
                                    "error": "Bad Request",
                                    "message": "Invalid status. Must be one of: pending, paid, failed, cancelled"
                                })),
                                Some(request_id),
                            ));
                        }
                    };
                    
                    updated_record.update_status(status);
                }
                
                // Update the record
                match metrics_manager.update_billing_record(updated_record).await {
                    Ok(updated) => {
                        Ok(build_response(
                            StatusCode::OK,
                            Some(serde_json::json!(billing_to_response(&updated))),
                            Some(request_id),
                        ))
                    },
                    Err(e) => {
                        error!("Failed to update billing record: {:?}", e);
                        Ok(build_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Some(serde_json::json!({
                                "error": "Internal Server Error",
                                "message": "Failed to update billing record"
                            })),
                            Some(request_id),
                        ))
                    }
                }
            } else {
                Ok(build_response(
                    StatusCode::NOT_FOUND,
                    Some(serde_json::json!({
                        "error": "Not Found",
                        "message": "Billing record not found"
                    })),
                    Some(request_id),
                ))
            }
        },
        Err(e) => {
            error!("Failed to get billing records: {:?}", e);
            Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Failed to get billing records"
                })),
                Some(request_id),
            ))
        }
    }
} 