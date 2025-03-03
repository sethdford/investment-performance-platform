use aws_sdk_dynamodb::{Client as DynamoDbClient, Error as DynamoDbError};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, errors::Error as JwtError};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::sync::Arc;
use thiserror::Error;
use tracing::{error, info};

/// Authentication and authorization errors
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Invalid token")]
    InvalidToken,
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("DynamoDB error: {0}")]
    DynamoDb(#[from] DynamoDbError),
    
    #[error("JWT error: {0}")]
    Jwt(#[from] JwtError),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for auth operations
pub type Result<T> = std::result::Result<T, AuthError>;

/// User role
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Admin,
    Manager,
    Analyst,
    Viewer,
}

impl Role {
    /// Convert a string to a Role
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Some(Role::Admin),
            "manager" => Some(Role::Manager),
            "analyst" => Some(Role::Analyst),
            "viewer" => Some(Role::Viewer),
            _ => None,
        }
    }
    
    /// Convert a Role to a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Manager => "manager",
            Role::Analyst => "analyst",
            Role::Viewer => "viewer",
        }
    }
}

/// Permission for a specific action
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    // Tenant management
    ManageTenants,
    ViewTenants,
    
    // User management
    ManageUsers,
    ViewUsers,
    
    // Portfolio management
    ManagePortfolios,
    ViewPortfolios,
    
    // Account management
    ManageAccounts,
    ViewAccounts,
    
    // Security management
    ManageSecurities,
    ViewSecurities,
    
    // Transaction management
    ManageTransactions,
    ViewTransactions,
    
    // Performance calculation
    RunCalculations,
    ViewPerformance,
    
    // API access
    AccessApi,
}

impl Permission {
    /// Convert a string to a Permission
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "manage_tenants" => Some(Permission::ManageTenants),
            "view_tenants" => Some(Permission::ViewTenants),
            "manage_users" => Some(Permission::ManageUsers),
            "view_users" => Some(Permission::ViewUsers),
            "manage_portfolios" => Some(Permission::ManagePortfolios),
            "view_portfolios" => Some(Permission::ViewPortfolios),
            "manage_accounts" => Some(Permission::ManageAccounts),
            "view_accounts" => Some(Permission::ViewAccounts),
            "manage_securities" => Some(Permission::ManageSecurities),
            "view_securities" => Some(Permission::ViewSecurities),
            "manage_transactions" => Some(Permission::ManageTransactions),
            "view_transactions" => Some(Permission::ViewTransactions),
            "run_calculations" => Some(Permission::RunCalculations),
            "view_performance" => Some(Permission::ViewPerformance),
            "access_api" => Some(Permission::AccessApi),
            _ => None,
        }
    }
    
    /// Convert a Permission to a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::ManageTenants => "manage_tenants",
            Permission::ViewTenants => "view_tenants",
            Permission::ManageUsers => "manage_users",
            Permission::ViewUsers => "view_users",
            Permission::ManagePortfolios => "manage_portfolios",
            Permission::ViewPortfolios => "view_portfolios",
            Permission::ManageAccounts => "manage_accounts",
            Permission::ViewAccounts => "view_accounts",
            Permission::ManageSecurities => "manage_securities",
            Permission::ViewSecurities => "view_securities",
            Permission::ManageTransactions => "manage_transactions",
            Permission::ViewTransactions => "view_transactions",
            Permission::RunCalculations => "run_calculations",
            Permission::ViewPerformance => "view_performance",
            Permission::AccessApi => "access_api",
        }
    }
}

/// User status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub tenant_id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub roles: Vec<Role>,
    pub status: UserStatus,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // User ID
    pub tenant_id: String,
    pub username: String,
    pub roles: Vec<String>,
    pub exp: usize,  // Expiration time (as UTC timestamp)
    pub iat: usize,  // Issued at (as UTC timestamp)
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
    pub jwt_secret: String,
    pub token_expiration_minutes: u64,
    pub enable_caching: bool,
    pub cache_ttl_seconds: u64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            jwt_secret: "default_secret_key_change_in_production".to_string(),
            token_expiration_minutes: 60,
            enable_caching: true,
            cache_ttl_seconds: 300,
        }
    }
}

/// Authentication service trait
pub trait AuthService: Send + Sync {
    /// Authenticate a user with username and password
    async fn authenticate(&self, username: &str, password: &str) -> Result<Option<User>>;
    
    /// Validate a JWT token
    async fn validate_token(&self, token: &str) -> Result<Claims>;
    
    /// Generate a JWT token for a user
    async fn generate_token(&self, user: &User) -> Result<String>;
    
    /// Get a user by ID
    async fn get_user(&self, user_id: &str) -> Result<Option<User>>;
    
    /// Create a new user
    async fn create_user(&self, user: User) -> Result<User>;
    
    /// Update a user
    async fn update_user(&self, user: User) -> Result<User>;
    
    /// Delete a user
    async fn delete_user(&self, user_id: &str) -> Result<()>;
    
    /// Check if a user has a specific permission
    async fn has_permission(&self, user_id: &str, permission: Permission) -> Result<bool>;
}

/// DynamoDB implementation of the AuthService
pub struct DynamoDbAuthService {
    client: DynamoDbClient,
    table_name: String,
    config: AuthConfig,
}

impl DynamoDbAuthService {
    /// Create a new DynamoDbAuthService
    pub fn new(client: DynamoDbClient, table_name: String, config: AuthConfig) -> Self {
        Self {
            client,
            table_name,
            config,
        }
    }
    
    /// Create a new DynamoDbAuthService from environment variables
    pub async fn from_env() -> Result<Self> {
        let config = aws_config::load_from_env().await;
        let client = DynamoDbClient::new(&config);
        
        let table_name = env::var("DYNAMODB_TABLE")
            .unwrap_or_else(|_| "investment-performance".to_string());
        
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "default_secret_key_change_in_production".to_string());
        
        let token_expiration_minutes = env::var("TOKEN_EXPIRATION_MINUTES")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60);
        
        let auth_config = AuthConfig {
            enabled: true,
            jwt_secret,
            token_expiration_minutes,
            enable_caching: true,
            cache_ttl_seconds: 300,
        };
        
        Ok(Self {
            client,
            table_name,
            config: auth_config,
        })
    }
    
    /// Hash a password
    fn hash_password(password: &str) -> String {
        // In a real implementation, use a proper password hashing algorithm like bcrypt
        // This is just a placeholder for demonstration purposes
        format!("hashed_{}", password)
    }
    
    /// Verify a password against a hash
    fn verify_password(password: &str, hash: &str) -> bool {
        // In a real implementation, use a proper password verification
        // This is just a placeholder for demonstration purposes
        hash == format!("hashed_{}", password)
    }
}

#[async_trait::async_trait]
impl AuthService for DynamoDbAuthService {
    async fn authenticate(&self, username: &str, password: &str) -> Result<Option<User>> {
        // For demonstration purposes, we'll use a hardcoded user
        // In a real implementation, query DynamoDB for the user
        
        if username == "admin" && password == "password" {
            let user = User {
                id: "user_1".to_string(),
                tenant_id: "tenant_1".to_string(),
                username: "admin".to_string(),
                email: "admin@example.com".to_string(),
                password_hash: Self::hash_password("password"),
                roles: vec![Role::Admin],
                status: UserStatus::Active,
                first_name: Some("Admin".to_string()),
                last_name: Some("User".to_string()),
                created_at: Utc::now(),
                updated_at: None,
            };
            
            Ok(Some(user))
        } else if username == "analyst" && password == "password" {
            let user = User {
                id: "user_2".to_string(),
                tenant_id: "tenant_1".to_string(),
                username: "analyst".to_string(),
                email: "analyst@example.com".to_string(),
                password_hash: Self::hash_password("password"),
                roles: vec![Role::Analyst],
                status: UserStatus::Active,
                first_name: Some("Analyst".to_string()),
                last_name: Some("User".to_string()),
                created_at: Utc::now(),
                updated_at: None,
            };
            
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    async fn validate_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::default();
        let key = DecodingKey::from_secret(self.config.jwt_secret.as_bytes());
        
        let token_data = decode::<Claims>(token, &key, &validation)
            .map_err(|e| {
                error!("Token validation error: {:?}", e);
                AuthError::InvalidToken
            })?;
        
        let claims = token_data.claims;
        
        // Check if token is expired
        let now = Utc::now().timestamp() as usize;
        if claims.exp < now {
            return Err(AuthError::TokenExpired);
        }
        
        Ok(claims)
    }
    
    async fn generate_token(&self, user: &User) -> Result<String> {
        let expiration = Utc::now() + Duration::minutes(self.config.token_expiration_minutes as i64);
        
        let claims = Claims {
            sub: user.id.clone(),
            tenant_id: user.tenant_id.clone(),
            username: user.username.clone(),
            roles: user.roles.iter().map(|r| r.as_str().to_string()).collect(),
            exp: expiration.timestamp() as usize,
            iat: Utc::now().timestamp() as usize,
        };
        
        let header = Header::default();
        let key = EncodingKey::from_secret(self.config.jwt_secret.as_bytes());
        
        encode(&header, &claims, &key)
            .map_err(|e| {
                error!("Token generation error: {:?}", e);
                AuthError::Internal(format!("Failed to generate token: {}", e))
            })
    }
    
    async fn get_user(&self, user_id: &str) -> Result<Option<User>> {
        // For demonstration purposes, we'll use hardcoded users
        // In a real implementation, query DynamoDB for the user
        
        if user_id == "user_1" {
            let user = User {
                id: "user_1".to_string(),
                tenant_id: "tenant_1".to_string(),
                username: "admin".to_string(),
                email: "admin@example.com".to_string(),
                password_hash: Self::hash_password("password"),
                roles: vec![Role::Admin],
                status: UserStatus::Active,
                first_name: Some("Admin".to_string()),
                last_name: Some("User".to_string()),
                created_at: Utc::now(),
                updated_at: None,
            };
            
            Ok(Some(user))
        } else if user_id == "user_2" {
            let user = User {
                id: "user_2".to_string(),
                tenant_id: "tenant_1".to_string(),
                username: "analyst".to_string(),
                email: "analyst@example.com".to_string(),
                password_hash: Self::hash_password("password"),
                roles: vec![Role::Analyst],
                status: UserStatus::Active,
                first_name: Some("Analyst".to_string()),
                last_name: Some("User".to_string()),
                created_at: Utc::now(),
                updated_at: None,
            };
            
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
    
    async fn create_user(&self, user: User) -> Result<User> {
        // In a real implementation, store the user in DynamoDB
        // This is just a placeholder for demonstration purposes
        Ok(user)
    }
    
    async fn update_user(&self, user: User) -> Result<User> {
        // In a real implementation, update the user in DynamoDB
        // This is just a placeholder for demonstration purposes
        Ok(user)
    }
    
    async fn delete_user(&self, _user_id: &str) -> Result<()> {
        // In a real implementation, delete the user from DynamoDB
        // This is just a placeholder for demonstration purposes
        Ok(())
    }
    
    async fn has_permission(&self, user_id: &str, permission: Permission) -> Result<bool> {
        // Get the user
        let user = match self.get_user(user_id).await? {
            Some(u) => u,
            None => return Err(AuthError::UserNotFound),
        };
        
        // Check if user is active
        if user.status != UserStatus::Active {
            return Ok(false);
        }
        
        // Check permissions based on roles
        for role in &user.roles {
            match role {
                Role::Admin => {
                    // Admins have all permissions
                    return Ok(true);
                },
                Role::Manager => {
                    // Managers have most permissions except tenant management
                    match permission {
                        Permission::ManageTenants => {},
                        _ => return Ok(true),
                    }
                },
                Role::Analyst => {
                    // Analysts can view data and run calculations
                    match permission {
                        Permission::ViewPortfolios | 
                        Permission::ViewAccounts | 
                        Permission::ViewSecurities | 
                        Permission::ViewTransactions | 
                        Permission::RunCalculations | 
                        Permission::ViewPerformance | 
                        Permission::AccessApi => return Ok(true),
                        _ => {},
                    }
                },
                Role::Viewer => {
                    // Viewers can only view data
                    match permission {
                        Permission::ViewPortfolios | 
                        Permission::ViewAccounts | 
                        Permission::ViewSecurities | 
                        Permission::ViewTransactions | 
                        Permission::ViewPerformance | 
                        Permission::AccessApi => return Ok(true),
                        _ => {},
                    }
                },
            }
        }
        
        Ok(false)
    }
}

/// Get an instance of the auth service
pub async fn get_auth_service() -> Result<impl AuthService, AuthError> {
    // Create DynamoDB auth service from environment variables
    DynamoDbAuthService::from_env().await
} 