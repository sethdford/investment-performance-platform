use aws_lambda_events::event::apigw::ApiGatewayProxyResponse;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use std::collections::HashMap;
use crate::auth_service::{AuthService, User, Role, Claims, AuthError};

/// Request to create a new user
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    /// Username
    pub username: String,
    /// Email address
    pub email: String,
    /// Password
    pub password: String,
    /// Tenant ID
    pub tenant_id: String,
    /// User roles
    pub roles: Vec<String>,
    /// First name (optional)
    pub first_name: Option<String>,
    /// Last name (optional)
    pub last_name: Option<String>,
}

/// Request to update a user
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    /// Email address (optional)
    pub email: Option<String>,
    /// Roles (optional)
    pub roles: Option<Vec<String>>,
    /// First name (optional)
    pub first_name: Option<String>,
    /// Last name (optional)
    pub last_name: Option<String>,
    /// Status (optional)
    pub status: Option<String>,
}

/// Request to change a user's password
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    /// Current password
    pub current_password: String,
    /// New password
    pub new_password: String,
}

/// Response for user operations
#[derive(Debug, Serialize)]
pub struct UserResponse {
    /// User ID
    pub id: String,
    /// Username
    pub username: String,
    /// Email address
    pub email: String,
    /// Tenant ID
    pub tenant_id: String,
    /// User roles
    pub roles: Vec<String>,
    /// First name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    /// Last name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    /// User status
    pub status: String,
    /// Created timestamp
    pub created_at: String,
    /// Last updated timestamp
    pub updated_at: String,
}

/// Convert a User to a UserResponse
fn user_to_response(user: &User) -> UserResponse {
    UserResponse {
        id: user.id.clone(),
        username: user.username.clone(),
        email: user.email.clone(),
        tenant_id: user.tenant_id.clone(),
        roles: user.roles.iter().map(|r| r.as_str().to_string()).collect(),
        first_name: user.first_name.clone(),
        last_name: user.last_name.clone(),
        status: user.status.to_string(),
        created_at: user.created_at.to_rfc3339(),
        updated_at: user.updated_at.to_rfc3339(),
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

/// Handle listing users
pub async fn handle_list_users(
    dynamodb_client: &DynamoDbClient,
    table_name: &str,
    query_params: &HashMap<String, String>,
    claims: &Claims,
    request_id: String,
) -> Result<ApiGatewayProxyResponse, aws_lambda_events::error::Error> {
    info!("Listing users, request_id: {}", request_id);
    
    // Check if user has permission to list users
    if !claims.has_permission("user:list") {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(serde_json::json!({
                "error": "Forbidden",
                "message": "You do not have permission to list users"
            })),
            Some(request_id),
        ));
    }
    
    // Get auth service
    let auth_service = match crate::auth_service::get_auth_service().await {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to get auth service: {:?}", e);
            return Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Authentication service unavailable"
                })),
                Some(request_id),
            ));
        }
    };
    
    // Extract query parameters
    let tenant_id = query_params.get("tenantId");
    let limit = query_params.get("limit").and_then(|l| l.parse::<usize>().ok()).unwrap_or(50);
    let offset = query_params.get("offset").and_then(|o| o.parse::<usize>().ok()).unwrap_or(0);
    
    // List users
    match auth_service.list_users(tenant_id, limit, offset).await {
        Ok(users) => {
            let user_responses: Vec<UserResponse> = users.iter().map(user_to_response).collect();
            
            Ok(build_response(
                StatusCode::OK,
                Some(serde_json::json!({
                    "users": user_responses,
                    "count": user_responses.len(),
                    "total": users.len(),
                    "limit": limit,
                    "offset": offset
                })),
                Some(request_id),
            ))
        },
        Err(e) => {
            error!("Failed to list users: {:?}", e);
            Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Failed to list users"
                })),
                Some(request_id),
            ))
        }
    }
}

/// Handle getting a user
pub async fn handle_get_user(
    dynamodb_client: &DynamoDbClient,
    table_name: &str,
    user_id: &str,
    claims: &Claims,
    request_id: String,
) -> Result<ApiGatewayProxyResponse, aws_lambda_events::error::Error> {
    info!("Getting user {}, request_id: {}", user_id, request_id);
    
    // Check if user has permission to view users
    if !claims.has_permission("user:read") && claims.user_id != user_id {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(serde_json::json!({
                "error": "Forbidden",
                "message": "You do not have permission to view this user"
            })),
            Some(request_id),
        ));
    }
    
    // Get auth service
    let auth_service = match crate::auth_service::get_auth_service().await {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to get auth service: {:?}", e);
            return Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Authentication service unavailable"
                })),
                Some(request_id),
            ));
        }
    };
    
    // Get user
    match auth_service.get_user(user_id).await {
        Ok(Some(user)) => {
            // Check tenant access if not admin
            if !claims.has_role("Admin") && claims.tenant_id != user.tenant_id {
                return Ok(build_response(
                    StatusCode::FORBIDDEN,
                    Some(serde_json::json!({
                        "error": "Forbidden",
                        "message": "You do not have permission to view users from other tenants"
                    })),
                    Some(request_id),
                ));
            }
            
            Ok(build_response(
                StatusCode::OK,
                Some(serde_json::json!(user_to_response(&user))),
                Some(request_id),
            ))
        },
        Ok(None) => {
            Ok(build_response(
                StatusCode::NOT_FOUND,
                Some(serde_json::json!({
                    "error": "Not Found",
                    "message": "User not found"
                })),
                Some(request_id),
            ))
        },
        Err(e) => {
            error!("Failed to get user: {:?}", e);
            Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Failed to get user"
                })),
                Some(request_id),
            ))
        }
    }
}

/// Handle creating a user
pub async fn handle_create_user(
    dynamodb_client: &DynamoDbClient,
    table_name: &str,
    create_request: CreateUserRequest,
    claims: &Claims,
    request_id: String,
) -> Result<ApiGatewayProxyResponse, aws_lambda_events::error::Error> {
    info!("Creating user {}, request_id: {}", create_request.username, request_id);
    
    // Check if user has permission to create users
    if !claims.has_permission("user:create") {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(serde_json::json!({
                "error": "Forbidden",
                "message": "You do not have permission to create users"
            })),
            Some(request_id),
        ));
    }
    
    // Check tenant access if not admin
    if !claims.has_role("Admin") && claims.tenant_id != create_request.tenant_id {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(serde_json::json!({
                "error": "Forbidden",
                "message": "You do not have permission to create users for other tenants"
            })),
            Some(request_id),
        ));
    }
    
    // Get auth service
    let auth_service = match crate::auth_service::get_auth_service().await {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to get auth service: {:?}", e);
            return Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Authentication service unavailable"
                })),
                Some(request_id),
            ));
        }
    };
    
    // Parse roles
    let roles = match parse_roles(&create_request.roles) {
        Ok(roles) => roles,
        Err(e) => {
            return Ok(build_response(
                StatusCode::BAD_REQUEST,
                Some(serde_json::json!({
                    "error": "Bad Request",
                    "message": format!("Invalid roles: {}", e)
                })),
                Some(request_id),
            ));
        }
    };
    
    // Create user
    let user = User::new(
        &create_request.username,
        &create_request.email,
        &create_request.tenant_id,
        roles,
        create_request.first_name.as_deref(),
        create_request.last_name.as_deref(),
    );
    
    match auth_service.create_user(&user, &create_request.password).await {
        Ok(created_user) => {
            Ok(build_response(
                StatusCode::CREATED,
                Some(serde_json::json!(user_to_response(&created_user))),
                Some(request_id),
            ))
        },
        Err(AuthError::AlreadyExists) => {
            Ok(build_response(
                StatusCode::CONFLICT,
                Some(serde_json::json!({
                    "error": "Conflict",
                    "message": "A user with this username already exists"
                })),
                Some(request_id),
            ))
        },
        Err(e) => {
            error!("Failed to create user: {:?}", e);
            Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Failed to create user"
                })),
                Some(request_id),
            ))
        }
    }
}

/// Handle updating a user
pub async fn handle_update_user(
    dynamodb_client: &DynamoDbClient,
    table_name: &str,
    user_id: &str,
    update_request: UpdateUserRequest,
    claims: &Claims,
    request_id: String,
) -> Result<ApiGatewayProxyResponse, aws_lambda_events::error::Error> {
    info!("Updating user {}, request_id: {}", user_id, request_id);
    
    // Check if user has permission to update users
    if !claims.has_permission("user:update") && claims.user_id != user_id {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(serde_json::json!({
                "error": "Forbidden",
                "message": "You do not have permission to update this user"
            })),
            Some(request_id),
        ));
    }
    
    // Get auth service
    let auth_service = match crate::auth_service::get_auth_service().await {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to get auth service: {:?}", e);
            return Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Authentication service unavailable"
                })),
                Some(request_id),
            ));
        }
    };
    
    // Get user
    let user = match auth_service.get_user(user_id).await {
        Ok(Some(user)) => {
            // Check tenant access if not admin
            if !claims.has_role("Admin") && claims.tenant_id != user.tenant_id {
                return Ok(build_response(
                    StatusCode::FORBIDDEN,
                    Some(serde_json::json!({
                        "error": "Forbidden",
                        "message": "You do not have permission to update users from other tenants"
                    })),
                    Some(request_id),
                ));
            }
            
            user
        },
        Ok(None) => {
            return Ok(build_response(
                StatusCode::NOT_FOUND,
                Some(serde_json::json!({
                    "error": "Not Found",
                    "message": "User not found"
                })),
                Some(request_id),
            ));
        },
        Err(e) => {
            error!("Failed to get user: {:?}", e);
            return Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Failed to get user"
                })),
                Some(request_id),
            ));
        }
    };
    
    // Parse roles if provided
    let roles = if let Some(role_strings) = &update_request.roles {
        // Only admins can update roles
        if !claims.has_role("Admin") && claims.user_id != user_id {
            return Ok(build_response(
                StatusCode::FORBIDDEN,
                Some(serde_json::json!({
                    "error": "Forbidden",
                    "message": "You do not have permission to update user roles"
                })),
                Some(request_id),
            ));
        }
        
        match parse_roles(role_strings) {
            Ok(roles) => Some(roles),
            Err(e) => {
                return Ok(build_response(
                    StatusCode::BAD_REQUEST,
                    Some(serde_json::json!({
                        "error": "Bad Request",
                        "message": format!("Invalid roles: {}", e)
                    })),
                    Some(request_id),
                ));
            }
        }
    } else {
        None
    };
    
    // Update user
    let mut updated_user = user.clone();
    
    if let Some(email) = &update_request.email {
        updated_user.email = email.clone();
    }
    
    if let Some(roles) = roles {
        updated_user.roles = roles;
    }
    
    if let Some(first_name) = &update_request.first_name {
        updated_user.first_name = Some(first_name.clone());
    }
    
    if let Some(last_name) = &update_request.last_name {
        updated_user.last_name = Some(last_name.clone());
    }
    
    if let Some(status) = &update_request.status {
        // Only admins can update status
        if !claims.has_role("Admin") {
            return Ok(build_response(
                StatusCode::FORBIDDEN,
                Some(serde_json::json!({
                    "error": "Forbidden",
                    "message": "You do not have permission to update user status"
                })),
                Some(request_id),
            ));
        }
        
        updated_user.status = match status.as_str() {
            "active" => crate::auth_service::UserStatus::Active,
            "inactive" => crate::auth_service::UserStatus::Inactive,
            "suspended" => crate::auth_service::UserStatus::Suspended,
            _ => {
                return Ok(build_response(
                    StatusCode::BAD_REQUEST,
                    Some(serde_json::json!({
                        "error": "Bad Request",
                        "message": "Invalid status. Must be one of: active, inactive, suspended"
                    })),
                    Some(request_id),
                ));
            }
        };
    }
    
    match auth_service.update_user(&updated_user).await {
        Ok(updated_user) => {
            Ok(build_response(
                StatusCode::OK,
                Some(serde_json::json!(user_to_response(&updated_user))),
                Some(request_id),
            ))
        },
        Err(e) => {
            error!("Failed to update user: {:?}", e);
            Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Failed to update user"
                })),
                Some(request_id),
            ))
        }
    }
}

/// Handle deleting a user
pub async fn handle_delete_user(
    dynamodb_client: &DynamoDbClient,
    table_name: &str,
    user_id: &str,
    claims: &Claims,
    request_id: String,
) -> Result<ApiGatewayProxyResponse, aws_lambda_events::error::Error> {
    info!("Deleting user {}, request_id: {}", user_id, request_id);
    
    // Check if user has permission to delete users
    if !claims.has_permission("user:delete") {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(serde_json::json!({
                "error": "Forbidden",
                "message": "You do not have permission to delete users"
            })),
            Some(request_id),
        ));
    }
    
    // Get auth service
    let auth_service = match crate::auth_service::get_auth_service().await {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to get auth service: {:?}", e);
            return Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Authentication service unavailable"
                })),
                Some(request_id),
            ));
        }
    };
    
    // Get user
    let user = match auth_service.get_user(user_id).await {
        Ok(Some(user)) => {
            // Check tenant access if not admin
            if !claims.has_role("Admin") && claims.tenant_id != user.tenant_id {
                return Ok(build_response(
                    StatusCode::FORBIDDEN,
                    Some(serde_json::json!({
                        "error": "Forbidden",
                        "message": "You do not have permission to delete users from other tenants"
                    })),
                    Some(request_id),
                ));
            }
            
            user
        },
        Ok(None) => {
            return Ok(build_response(
                StatusCode::NOT_FOUND,
                Some(serde_json::json!({
                    "error": "Not Found",
                    "message": "User not found"
                })),
                Some(request_id),
            ));
        },
        Err(e) => {
            error!("Failed to get user: {:?}", e);
            return Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Failed to get user"
                })),
                Some(request_id),
            ));
        }
    };
    
    // Delete user
    match auth_service.delete_user(user_id).await {
        Ok(_) => {
            Ok(build_response(
                StatusCode::NO_CONTENT,
                None,
                Some(request_id),
            ))
        },
        Err(e) => {
            error!("Failed to delete user: {:?}", e);
            Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Failed to delete user"
                })),
                Some(request_id),
            ))
        }
    }
}

/// Handle changing a user's password
pub async fn handle_change_password(
    dynamodb_client: &DynamoDbClient,
    table_name: &str,
    user_id: &str,
    change_request: ChangePasswordRequest,
    claims: &Claims,
    request_id: String,
) -> Result<ApiGatewayProxyResponse, aws_lambda_events::error::Error> {
    info!("Changing password for user {}, request_id: {}", user_id, request_id);
    
    // Check if user has permission to change password
    if !claims.has_permission("user:update") && claims.user_id != user_id {
        return Ok(build_response(
            StatusCode::FORBIDDEN,
            Some(serde_json::json!({
                "error": "Forbidden",
                "message": "You do not have permission to change this user's password"
            })),
            Some(request_id),
        ));
    }
    
    // Get auth service
    let auth_service = match crate::auth_service::get_auth_service().await {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to get auth service: {:?}", e);
            return Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Authentication service unavailable"
                })),
                Some(request_id),
            ));
        }
    };
    
    // Get user
    let user = match auth_service.get_user(user_id).await {
        Ok(Some(user)) => {
            // Check tenant access if not admin
            if !claims.has_role("Admin") && claims.tenant_id != user.tenant_id && claims.user_id != user_id {
                return Ok(build_response(
                    StatusCode::FORBIDDEN,
                    Some(serde_json::json!({
                        "error": "Forbidden",
                        "message": "You do not have permission to change passwords for users from other tenants"
                    })),
                    Some(request_id),
                ));
            }
            
            user
        },
        Ok(None) => {
            return Ok(build_response(
                StatusCode::NOT_FOUND,
                Some(serde_json::json!({
                    "error": "Not Found",
                    "message": "User not found"
                })),
                Some(request_id),
            ));
        },
        Err(e) => {
            error!("Failed to get user: {:?}", e);
            return Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Failed to get user"
                })),
                Some(request_id),
            ));
        }
    };
    
    // Change password
    match auth_service.change_password(user_id, &change_request.current_password, &change_request.new_password).await {
        Ok(_) => {
            Ok(build_response(
                StatusCode::OK,
                Some(serde_json::json!({
                    "message": "Password changed successfully"
                })),
                Some(request_id),
            ))
        },
        Err(AuthError::InvalidCredentials) => {
            Ok(build_response(
                StatusCode::UNAUTHORIZED,
                Some(serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Current password is incorrect"
                })),
                Some(request_id),
            ))
        },
        Err(e) => {
            error!("Failed to change password: {:?}", e);
            Ok(build_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                Some(serde_json::json!({
                    "error": "Internal Server Error",
                    "message": "Failed to change password"
                })),
                Some(request_id),
            ))
        }
    }
}

/// Parse role strings into Role enum values
fn parse_roles(role_strings: &[String]) -> Result<Vec<Role>, String> {
    let mut roles = Vec::new();
    
    for role_str in role_strings {
        match role_str.as_str() {
            "Admin" => roles.push(Role::Admin),
            "Manager" => roles.push(Role::Manager),
            "Analyst" => roles.push(Role::Analyst),
            "Viewer" => roles.push(Role::Viewer),
            _ => return Err(format!("Invalid role: {}", role_str)),
        }
    }
    
    Ok(roles)
} 