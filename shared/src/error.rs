//! Error handling

use std::fmt;
use thiserror::Error;

/// Application error
#[derive(Error, Debug)]
pub enum AppError {
    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// Unauthorized error
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    /// Forbidden error
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    /// Database error
    #[error("Database error: {0}")]
    Database(String),
    
    /// External service error
    #[error("External service error: {0}")]
    ExternalService(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    /// Timeout error
    #[error("Timeout: {0}")]
    Timeout(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// Unknown error
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl AppError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            AppError::NotFound(_) => 404,
            AppError::Validation(_) => 400,
            AppError::Unauthorized(_) => 401,
            AppError::Forbidden(_) => 403,
            AppError::Database(_) => 500,
            AppError::ExternalService(_) => 502,
            AppError::Configuration(_) => 500,
            AppError::RateLimitExceeded(_) => 429,
            AppError::Timeout(_) => 504,
            AppError::Internal(_) => 500,
            AppError::Unknown(_) => 500,
        }
    }
    
    /// Get the error code for this error
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::Unauthorized(_) => "UNAUTHORIZED",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::ExternalService(_) => "EXTERNAL_SERVICE_ERROR",
            AppError::Configuration(_) => "CONFIGURATION_ERROR",
            AppError::RateLimitExceeded(_) => "RATE_LIMIT_EXCEEDED",
            AppError::Timeout(_) => "TIMEOUT",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::Unknown(_) => "UNKNOWN_ERROR",
        }
    }
}

/// Error context
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Request ID
    pub request_id: Option<String>,
    /// Correlation ID
    pub correlation_id: Option<String>,
    /// Tenant ID
    pub tenant_id: Option<String>,
    /// User ID
    pub user_id: Option<String>,
    /// Additional context
    pub additional: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new() -> Self {
        Self {
            request_id: None,
            correlation_id: None,
            tenant_id: None,
            user_id: None,
            additional: std::collections::HashMap::new(),
        }
    }
    
    /// Set the request ID
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
    
    /// Set the correlation ID
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }
    
    /// Set the tenant ID
    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }
    
    /// Set the user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
    
    /// Add additional context
    pub fn with_additional(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.additional.insert(key.into(), value.into());
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Error with context
#[derive(Debug)]
pub struct ErrorWithContext {
    /// The error
    pub error: AppError,
    /// The error context
    pub context: ErrorContext,
}

impl ErrorWithContext {
    /// Create a new error with context
    pub fn new(error: AppError, context: ErrorContext) -> Self {
        Self { error, context }
    }
    
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        self.error.status_code()
    }
    
    /// Get the error code for this error
    pub fn error_code(&self) -> &'static str {
        self.error.error_code()
    }
}

impl fmt::Display for ErrorWithContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl std::error::Error for ErrorWithContext {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Convert from AppError to ErrorWithContext
impl From<AppError> for ErrorWithContext {
    fn from(error: AppError) -> Self {
        Self::new(error, ErrorContext::new())
    }
}

/// Convert from ErrorWithContext to AppError
impl From<ErrorWithContext> for AppError {
    fn from(error_with_context: ErrorWithContext) -> Self {
        error_with_context.error
    }
}

/// Convert from std::io::Error to AppError
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::Internal(format!("IO error: {}", error))
    }
}

/// Convert from serde_json::Error to AppError
impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::Validation(format!("JSON error: {}", error))
    }
}

/// Convert from aws_sdk_dynamodb::Error to AppError
impl From<aws_sdk_dynamodb::Error> for AppError {
    fn from(error: aws_sdk_dynamodb::Error) -> Self {
        AppError::Database(format!("DynamoDB error: {}", error))
    }
}

/// Convert from aws_sdk_sqs::Error to AppError
impl From<aws_sdk_sqs::Error> for AppError {
    fn from(error: aws_sdk_sqs::Error) -> Self {
        AppError::ExternalService(format!("SQS error: {}", error))
    }
}

/// Convert from jsonwebtoken::errors::Error to AppError
impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        AppError::Unauthorized(format!("JWT error: {}", error))
    }
}

/// Result type with AppError
pub type Result<T> = std::result::Result<T, AppError>;

/// Result type with ErrorWithContext
pub type ResultWithContext<T> = std::result::Result<T, ErrorWithContext>; 