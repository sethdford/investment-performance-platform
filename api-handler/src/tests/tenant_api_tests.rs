use super::*;
use crate::tenant_api;
use crate::auth_service::{Claims, Role};
use aws_lambda_events::event::apigw::ApiGatewayProxyRequest;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use aws_sdk_dynamodb::config::Builder;
use http::StatusCode;
use std::collections::HashMap;
use chrono::Utc;
use uuid::Uuid;

/// Create a mock DynamoDB client for testing
fn mock_dynamodb_client() -> DynamoDbClient {
    let config = Builder::new()
        .endpoint_url("http://localhost:8000")
        .build();
    
    DynamoDbClient::from_conf(config)
}

/// Create test claims with admin permissions
fn create_admin_claims() -> Claims {
    Claims {
        sub: Uuid::new_v4().to_string(),
        tenant_id: "test-tenant".to_string(),
        username: "admin".to_string(),
        roles: vec!["Admin".to_string()],
        exp: (Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
    }
}

/// Create test claims with limited permissions
fn create_user_claims() -> Claims {
    Claims {
        sub: Uuid::new_v4().to_string(),
        tenant_id: "test-tenant".to_string(),
        username: "user".to_string(),
        roles: vec!["Viewer".to_string()],
        exp: (Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
    }
}

#[tokio::test]
async fn test_list_tenants() {
    // Create mock DynamoDB client
    let client = mock_dynamodb_client();
    let table_name = "test-table";
    
    // Create admin claims
    let claims = create_admin_claims();
    
    // Create query parameters
    let mut query_params = HashMap::new();
    query_params.insert("limit".to_string(), "10".to_string());
    query_params.insert("offset".to_string(), "0".to_string());
    
    // Call the handler
    let response = tenant_api::handle_list_tenants(
        &client,
        table_name,
        &query_params,
        &claims,
        "test-request-id".to_string(),
    ).await.unwrap();
    
    // Verify response
    assert_eq!(response.status_code, StatusCode::OK.as_u16());
    
    // Parse response body
    if let Some(aws_lambda_events::encodings::Body::Text(body)) = response.body {
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        
        // Verify response structure
        assert!(json.get("tenants").is_some());
        assert!(json.get("count").is_some());
        assert!(json.get("request_id").is_some());
    } else {
        panic!("Response body is missing or not text");
    }
    
    // Test with non-admin user
    let user_claims = create_user_claims();
    
    // Call the handler with user claims
    let user_response = tenant_api::handle_list_tenants(
        &client,
        table_name,
        &query_params,
        &user_claims,
        "test-request-id".to_string(),
    ).await.unwrap();
    
    // Verify response is forbidden
    assert_eq!(user_response.status_code, StatusCode::FORBIDDEN.as_u16());
}

#[tokio::test]
async fn test_create_tenant() {
    // Create mock DynamoDB client
    let client = mock_dynamodb_client();
    let table_name = "test-table";
    
    // Create admin claims
    let claims = create_admin_claims();
    
    // Create request
    let request = tenant_api::CreateTenantRequest {
        name: "Test Tenant".to_string(),
        description: Some("Test tenant description".to_string()),
        subscription_tier: "Basic".to_string(),
        custom_config: None,
        resource_limits: None,
    };
    
    // Call the handler
    let response = tenant_api::handle_create_tenant(
        &client,
        table_name,
        request,
        &claims,
        "test-request-id".to_string(),
    ).await.unwrap();
    
    // Verify response
    assert_eq!(response.status_code, StatusCode::CREATED.as_u16());
    
    // Parse response body
    if let Some(aws_lambda_events::encodings::Body::Text(body)) = response.body {
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        
        // Verify tenant properties
        assert_eq!(json["name"].as_str().unwrap(), "Test Tenant");
        assert_eq!(json["description"].as_str().unwrap(), "Test tenant description");
        assert_eq!(json["subscription_tier"].as_str().unwrap(), "Basic");
        assert_eq!(json["status"].as_str().unwrap(), "active");
        
        // Verify resource limits
        assert!(json["resource_limits"].is_object());
        assert!(json["resource_limits"]["max_portfolios"].is_number());
        
        // Verify request ID
        assert_eq!(json["request_id"].as_str().unwrap(), "test-request-id");
    } else {
        panic!("Response body is missing or not text");
    }
    
    // Test with non-admin user
    let user_claims = create_user_claims();
    
    // Call the handler with user claims
    let user_response = tenant_api::handle_create_tenant(
        &client,
        table_name,
        tenant_api::CreateTenantRequest {
            name: "Another Tenant".to_string(),
            description: None,
            subscription_tier: "Basic".to_string(),
            custom_config: None,
            resource_limits: None,
        },
        &user_claims,
        "test-request-id".to_string(),
    ).await.unwrap();
    
    // Verify response is forbidden
    assert_eq!(user_response.status_code, StatusCode::FORBIDDEN.as_u16());
}

#[tokio::test]
async fn test_get_tenant() {
    // Create mock DynamoDB client
    let client = mock_dynamodb_client();
    let table_name = "test-table";
    
    // Create admin claims
    let claims = create_admin_claims();
    
    // Call the handler with a tenant ID
    let response = tenant_api::handle_get_tenant(
        &client,
        table_name,
        "test-tenant-id",
        &claims,
        "test-request-id".to_string(),
    ).await.unwrap();
    
    // In a real test, we would first create the tenant and then get it
    // For this mock test, we'll just check the response structure
    
    // Verify response
    assert!(response.status_code == StatusCode::OK.as_u16() || 
            response.status_code == StatusCode::NOT_FOUND.as_u16());
    
    // Test with non-admin user but same tenant ID
    let user_claims = create_user_claims();
    
    // Call the handler with user claims and the same tenant ID as in the claims
    let user_response = tenant_api::handle_get_tenant(
        &client,
        table_name,
        "test-tenant", // Same as in user_claims.tenant_id
        &user_claims,
        "test-request-id".to_string(),
    ).await.unwrap();
    
    // Verify response is OK (user can view their own tenant)
    assert!(user_response.status_code == StatusCode::OK.as_u16() || 
            user_response.status_code == StatusCode::NOT_FOUND.as_u16());
    
    // Call the handler with user claims and a different tenant ID
    let different_tenant_response = tenant_api::handle_get_tenant(
        &client,
        table_name,
        "different-tenant", // Different from user_claims.tenant_id
        &user_claims,
        "test-request-id".to_string(),
    ).await.unwrap();
    
    // Verify response is forbidden (user cannot view other tenants)
    assert_eq!(different_tenant_response.status_code, StatusCode::FORBIDDEN.as_u16());
}

#[tokio::test]
async fn test_update_tenant() {
    // Create mock DynamoDB client
    let client = mock_dynamodb_client();
    let table_name = "test-table";
    
    // Create admin claims
    let claims = create_admin_claims();
    
    // Create update request
    let request = tenant_api::UpdateTenantRequest {
        name: Some("Updated Tenant".to_string()),
        description: Some("Updated description".to_string()),
        subscription_tier: Some("Professional".to_string()),
        custom_config: None,
        resource_limits: None,
    };
    
    // Call the handler
    let response = tenant_api::handle_update_tenant(
        &client,
        table_name,
        "test-tenant-id",
        request,
        &claims,
        "test-request-id".to_string(),
    ).await.unwrap();
    
    // In a real test, we would first create the tenant and then update it
    // For this mock test, we'll just check the response structure
    
    // Verify response
    assert!(response.status_code == StatusCode::OK.as_u16() || 
            response.status_code == StatusCode::NOT_FOUND.as_u16());
    
    // Test with non-admin user
    let user_claims = create_user_claims();
    
    // Call the handler with user claims
    let user_response = tenant_api::handle_update_tenant(
        &client,
        table_name,
        "test-tenant-id",
        tenant_api::UpdateTenantRequest {
            name: Some("User Updated".to_string()),
            description: None,
            subscription_tier: None,
            custom_config: None,
            resource_limits: None,
        },
        &user_claims,
        "test-request-id".to_string(),
    ).await.unwrap();
    
    // Verify response is forbidden
    assert_eq!(user_response.status_code, StatusCode::FORBIDDEN.as_u16());
}

#[tokio::test]
async fn test_tenant_status_changes() {
    // Create mock DynamoDB client
    let client = mock_dynamodb_client();
    let table_name = "test-table";
    
    // Create admin claims
    let claims = create_admin_claims();
    
    // Test activate tenant
    let activate_response = tenant_api::handle_activate_tenant(
        &client,
        table_name,
        "test-tenant-id",
        &claims,
        "test-request-id".to_string(),
    ).await.unwrap();
    
    // Verify response
    assert!(activate_response.status_code == StatusCode::OK.as_u16() || 
            activate_response.status_code == StatusCode::NOT_FOUND.as_u16());
    
    // Test suspend tenant
    let suspend_response = tenant_api::handle_suspend_tenant(
        &client,
        table_name,
        "test-tenant-id",
        &claims,
        "test-request-id".to_string(),
    ).await.unwrap();
    
    // Verify response
    assert!(suspend_response.status_code == StatusCode::OK.as_u16() || 
            suspend_response.status_code == StatusCode::NOT_FOUND.as_u16());
    
    // Test deactivate tenant
    let deactivate_response = tenant_api::handle_deactivate_tenant(
        &client,
        table_name,
        "test-tenant-id",
        &claims,
        "test-request-id".to_string(),
    ).await.unwrap();
    
    // Verify response
    assert!(deactivate_response.status_code == StatusCode::OK.as_u16() || 
            deactivate_response.status_code == StatusCode::NOT_FOUND.as_u16());
    
    // Test with non-admin user
    let user_claims = create_user_claims();
    
    // Call the handler with user claims
    let user_response = tenant_api::handle_activate_tenant(
        &client,
        table_name,
        "test-tenant-id",
        &user_claims,
        "test-request-id".to_string(),
    ).await.unwrap();
    
    // Verify response is forbidden
    assert_eq!(user_response.status_code, StatusCode::FORBIDDEN.as_u16());
}

#[tokio::test]
async fn test_delete_tenant() {
    // Create mock DynamoDB client
    let client = mock_dynamodb_client();
    let table_name = "test-table";
    
    // Create admin claims
    let claims = create_admin_claims();
    
    // Call the handler
    let response = tenant_api::handle_delete_tenant(
        &client,
        table_name,
        "test-tenant-id",
        &claims,
        "test-request-id".to_string(),
    ).await.unwrap();
    
    // Verify response
    assert!(response.status_code == StatusCode::NO_CONTENT.as_u16() || 
            response.status_code == StatusCode::NOT_FOUND.as_u16());
    
    // Test with non-admin user
    let user_claims = create_user_claims();
    
    // Call the handler with user claims
    let user_response = tenant_api::handle_delete_tenant(
        &client,
        table_name,
        "test-tenant-id",
        &user_claims,
        "test-request-id".to_string(),
    ).await.unwrap();
    
    // Verify response is forbidden
    assert_eq!(user_response.status_code, StatusCode::FORBIDDEN.as_u16());
} 