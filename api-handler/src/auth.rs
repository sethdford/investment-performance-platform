//! Authentication and authorization middleware

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use lambda_http::{Request, Body};
use serde::{Deserialize, Serialize};
use shared::{AppError, encryption::EncryptionService};
use std::collections::{HashMap, HashSet};
use tracing::{info, error, warn};
use chrono::{Utc, Duration};
use uuid::Uuid;

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// Expiration time
    pub exp: u64,
    /// Issued at
    pub iat: u64,
    /// JWT ID
    pub jti: String,
    /// Tenant ID
    pub tenant_id: String,
    /// Roles
    pub roles: Vec<String>,
    /// Permissions
    pub permissions: Vec<String>,
}

/// Role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Role name
    pub name: String,
    /// Role description
    pub description: String,
    /// Role permissions
    pub permissions: Vec<String>,
}

/// Permission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// Permission name
    pub name: String,
    /// Permission description
    pub description: String,
    /// Resource type
    pub resource_type: String,
    /// Action
    pub action: String,
}

/// Authentication service
pub struct AuthService {
    /// JWT encoding key
    jwt_encoding_key: EncodingKey,
    /// JWT decoding key
    jwt_decoding_key: DecodingKey,
    /// JWT validation
    jwt_validation: Validation,
    /// Encryption service
    encryption_service: EncryptionService,
    /// Roles
    roles: HashMap<String, Role>,
    /// Permissions
    permissions: HashMap<String, Permission>,
    /// Token blacklist
    token_blacklist: HashSet<String>,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(
        jwt_secret: &[u8],
        encryption_key: &[u8],
    ) -> Result<Self, AppError> {
        let encryption_service = EncryptionService::new(encryption_key)
            .map_err(|e| AppError::Configuration(format!("Failed to create encryption service: {}", e)))?;
        
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_audience(&["investment-performance-api"]);
        validation.set_issuer(&["investment-performance-auth"]);
        
        Ok(Self {
            jwt_encoding_key: EncodingKey::from_secret(jwt_secret),
            jwt_decoding_key: DecodingKey::from_secret(jwt_secret),
            jwt_validation: validation,
            encryption_service,
            roles: HashMap::new(),
            permissions: HashMap::new(),
            token_blacklist: HashSet::new(),
        })
    }
    
    /// Add a role
    pub fn add_role(&mut self, role: Role) {
        self.roles.insert(role.name.clone(), role);
    }
    
    /// Add a permission
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission.name.clone(), permission);
    }
    
    /// Create a JWT token
    pub fn create_token(
        &self,
        user_id: &str,
        tenant_id: &str,
        roles: &[String],
        expiration_minutes: i64,
    ) -> Result<String, AppError> {
        // Get permissions for the roles
        let mut permissions = HashSet::new();
        
        for role_name in roles {
            if let Some(role) = self.roles.get(role_name) {
                for permission in &role.permissions {
                    permissions.insert(permission.clone());
                }
            }
        }
        
        // Create claims
        let now = Utc::now();
        let expiration = now + Duration::minutes(expiration_minutes);
        
        let claims = Claims {
            sub: user_id.to_string(),
            iss: "investment-performance-auth".to_string(),
            aud: "investment-performance-api".to_string(),
            exp: expiration.timestamp() as u64,
            iat: now.timestamp() as u64,
            jti: Uuid::new_v4().to_string(),
            tenant_id: tenant_id.to_string(),
            roles: roles.to_vec(),
            permissions: permissions.into_iter().collect(),
        };
        
        // Encode the token
        encode(&Header::default(), &claims, &self.jwt_encoding_key)
            .map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))
    }
    
    /// Validate a JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        // Check if token is blacklisted
        if self.token_blacklist.contains(token) {
            return Err(AppError::Unauthorized("Token has been revoked".to_string()));
        }
        
        // Decode and validate the token
        let token_data = decode::<Claims>(token, &self.jwt_decoding_key, &self.jwt_validation)
            .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;
        
        Ok(token_data.claims)
    }
    
    /// Revoke a token
    pub fn revoke_token(&mut self, token: &str) {
        self.token_blacklist.insert(token.to_string());
    }
    
    /// Check if a user has a role
    pub fn has_role(&self, claims: &Claims, role: &str) -> bool {
        claims.roles.contains(&role.to_string())
    }
    
    /// Check if a user has a permission
    pub fn has_permission(&self, claims: &Claims, permission: &str) -> bool {
        claims.permissions.contains(&permission.to_string())
    }
    
    /// Check if a user has access to a resource
    pub fn has_resource_access(
        &self,
        claims: &Claims,
        resource_type: &str,
        resource_id: &str,
        action: &str,
    ) -> bool {
        // Check if user has admin role
        if self.has_role(claims, "admin") {
            return true;
        }
        
        // Check if user has specific permission
        let permission_name = format!("{}:{}:{}", resource_type, action, resource_id);
        if self.has_permission(claims, &permission_name) {
            return true;
        }
        
        // Check if user has wildcard permission
        let wildcard_permission = format!("{}:{}:*", resource_type, action);
        if self.has_permission(claims, &wildcard_permission) {
            return true;
        }
        
        // Check if resource belongs to user's tenant
        if resource_id.starts_with(&format!("{}-", claims.tenant_id)) {
            // Check if user has tenant-level permission
            let tenant_permission = format!("{}:{}:tenant", resource_type, action);
            if self.has_permission(claims, &tenant_permission) {
                return true;
            }
        }
        
        false
    }
    
    /// Encrypt sensitive data
    pub fn encrypt(&self, data: &str) -> Result<String, AppError> {
        self.encryption_service.encrypt_string(data)
            .map_err(|e| AppError::Internal(format!("Encryption failed: {}", e)))
    }
    
    /// Decrypt sensitive data
    pub fn decrypt(&self, data: &str) -> Result<String, AppError> {
        self.encryption_service.decrypt_string(data)
            .map_err(|e| AppError::Internal(format!("Decryption failed: {}", e)))
    }
}

/// Authentication middleware
pub struct AuthMiddleware {
    /// Authentication service
    auth_service: AuthService,
}

impl AuthMiddleware {
    /// Create a new authentication middleware
    pub fn new(auth_service: AuthService) -> Self {
        Self { auth_service }
    }
    
    /// Authenticate a request
    pub fn authenticate(&self, request: &lambda_http::Request) -> Result<Claims, AppError> {
        // Extract the Authorization header
        let auth_header = request.headers()
            .get("Authorization")
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?
            .to_str()
            .map_err(|_| AppError::Unauthorized("Invalid Authorization header".to_string()))?;
        
        // Check if it's a Bearer token
        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized("Invalid Authorization header format".to_string()));
        }
        
        // Extract the token
        let token = auth_header.trim_start_matches("Bearer ");
        
        // Validate the token
        self.auth_service.validate_token(token)
    }
    
    /// Authorize a request
    pub fn authorize(
        &self,
        claims: &Claims,
        resource_type: &str,
        resource_id: &str,
        action: &str,
    ) -> Result<(), AppError> {
        if !self.auth_service.has_resource_access(claims, resource_type, resource_id, action) {
            return Err(AppError::Forbidden(format!(
                "Access denied to {} {} for action {}",
                resource_type, resource_id, action
            )));
        }
        
        Ok(())
    }
} 