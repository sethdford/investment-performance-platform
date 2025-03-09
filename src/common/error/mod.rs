//! Error types for the Investment Management Platform
//!
//! This module defines the error types used throughout the platform.

use thiserror::Error;
use std::fmt;
use std::string::FromUtf8Error;
use serde_json;
use std::fmt::Debug;

/// Result type for the Investment Management Platform
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for the Investment Management Platform
#[derive(Error, Debug)]
pub enum Error {
    /// API error
    #[error("API error: {0}")]
    Api(String),
    
    /// Database error
    #[error("Database error: {0}")]
    Database(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Authentication error
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    /// Authorization error
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    /// External service error
    #[error("External service error: {0}")]
    ExternalService(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Model portfolio error
    #[error("Model portfolio error: {error_type}: {message}")]
    ModelPortfolioError {
        /// Type of model portfolio error
        error_type: ModelPortfolioErrorType,
        /// Error message
        message: String,
    },
    
    /// AWS SDK error
    #[error("AWS SDK error: {0}")]
    AwsSdk(String),
    
    /// Serde JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// UTF-8 conversion error
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] FromUtf8Error),
    
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

/// Types of errors specific to model portfolios
#[derive(Debug, Clone, PartialEq)]
pub enum ModelPortfolioErrorType {
    /// Error when creating sleeves from a model
    SleeveCreationError,
    /// Error when a child model is not found
    ChildModelNotFound,
    /// Error when model weights don't sum to 1.0
    InvalidWeights,
    /// Error when a model has an invalid structure
    InvalidModelStructure,
    /// Error when a security is not found
    SecurityNotFound,
    /// Error when a model has duplicate securities
    DuplicateSecurities,
    /// Error when a model has invalid allocation
    InvalidAllocation,
}

impl fmt::Display for ModelPortfolioErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SleeveCreationError => write!(f, "SleeveCreationError"),
            Self::ChildModelNotFound => write!(f, "ChildModelNotFound"),
            Self::InvalidWeights => write!(f, "InvalidWeights"),
            Self::InvalidModelStructure => write!(f, "InvalidModelStructure"),
            Self::SecurityNotFound => write!(f, "SecurityNotFound"),
            Self::DuplicateSecurities => write!(f, "DuplicateSecurities"),
            Self::InvalidAllocation => write!(f, "InvalidAllocation"),
        }
    }
}

// For backward compatibility
pub type ApiError = Error;
pub type ApiResult<T> = Result<T>;

impl Error {
    /// Create a new API error
    pub fn api<S: Into<String>>(msg: S) -> Self {
        Error::Api(msg.into())
    }
    
    /// Create a new database error
    pub fn database<S: Into<String>>(msg: S) -> Self {
        Error::Database(msg.into())
    }
    
    /// Create a new validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Error::Validation(msg.into())
    }
    
    /// Create a new not found error
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Error::NotFound(msg.into())
    }
    
    /// Create a new authentication error
    pub fn authentication<S: Into<String>>(msg: S) -> Self {
        Error::Authentication(msg.into())
    }
    
    /// Create a new authorization error
    pub fn authorization<S: Into<String>>(msg: S) -> Self {
        Error::Authorization(msg.into())
    }
    
    /// Create a new external service error
    pub fn external_service<S: Into<String>>(msg: S) -> Self {
        Error::ExternalService(msg.into())
    }
    
    /// Create a new internal error
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Error::Internal(msg.into())
    }
    
    /// Create a new serialization error
    pub fn serialization<S: Into<String>>(msg: S) -> Self {
        Error::Serialization(msg.into())
    }
    
    /// Create a new AWS SDK error
    pub fn aws_sdk<S: Into<String>>(msg: S) -> Self {
        Error::AwsSdk(msg.into())
    }
    
    /// Create a new other error
    pub fn other<S: Into<String>>(msg: S) -> Self {
        Error::Other(msg.into())
    }
}

impl From<String> for ApiError {
    fn from(message: String) -> Self {
        ApiError::Internal(message)
    }
}

impl From<&str> for ApiError {
    fn from(message: &str) -> Self {
        ApiError::Internal(message.to_string())
    }
}

impl<E, R> From<aws_smithy_runtime_api::client::result::SdkError<E, R>> for Error
where
    E: std::fmt::Debug,
    R: std::fmt::Debug,
{
    fn from(err: aws_smithy_runtime_api::client::result::SdkError<E, R>) -> Self {
        Error::AwsSdk(format!("{:?}", err))
    }
}

/// Module for error handling utilities
pub mod utils;

/// Extension trait for Result to provide consistent error handling patterns
pub trait ResultExt<T, E: Debug> {
    /// Convert the error to an ApiError with a custom message
    fn with_context<C, F>(self, context: F) -> ApiResult<T>
    where
        F: FnOnce() -> C,
        C: std::fmt::Display;

    /// Convert the error to an ApiError with the error type as the message
    fn with_error_type(self) -> ApiResult<T>;

    /// Convert the error to a NotFound ApiError
    fn not_found<S: AsRef<str>>(self, entity_type: S, id: S) -> ApiResult<T>;

    /// Convert the error to a ValidationError ApiError
    fn validation_error<S: AsRef<str>>(self, message: S) -> ApiResult<T>;

    /// Convert the error to an InvalidParameter ApiError
    fn invalid_parameter<S: AsRef<str>>(self, parameter: S, message: S) -> ApiResult<T>;
}

impl<T, E: Debug> ResultExt<T, E> for std::result::Result<T, E> {
    fn with_context<C, F>(self, context: F) -> ApiResult<T>
    where
        F: FnOnce() -> C,
        C: std::fmt::Display,
    {
        self.map_err(|e| {
            ApiError::Internal(format!("{}: {:?}", context(), e))
        })
    }

    fn with_error_type(self) -> ApiResult<T> {
        self.map_err(|e| {
            ApiError::Internal(format!("{:?}", e))
        })
    }

    fn not_found<S: AsRef<str>>(self, entity_type: S, id: S) -> ApiResult<T> {
        self.map_err(|_| {
            ApiError::NotFound(format!("{} with ID {} not found", entity_type.as_ref(), id.as_ref()))
        })
    }

    fn validation_error<S: AsRef<str>>(self, message: S) -> ApiResult<T> {
        self.map_err(|_| {
            ApiError::Validation(message.as_ref().to_string())
        })
    }

    fn invalid_parameter<S: AsRef<str>>(self, parameter: S, message: S) -> ApiResult<T> {
        self.map_err(|_| {
            ApiError::Validation(format!("Invalid parameter {}: {}", parameter.as_ref(), message.as_ref()))
        })
    }
}

/// Extension trait for Option to provide consistent error handling patterns
pub trait OptionExt<T> {
    /// Convert None to a NotFound ApiError
    fn ok_or_not_found<S: AsRef<str>>(self, entity_type: S, id: S) -> ApiResult<T>;

    /// Convert None to a ValidationError ApiError
    fn ok_or_validation_error<S: AsRef<str>>(self, message: S) -> ApiResult<T>;

    /// Convert None to an InvalidParameter ApiError
    fn ok_or_invalid_parameter<S: AsRef<str>>(self, parameter: S, message: S) -> ApiResult<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_not_found<S: AsRef<str>>(self, entity_type: S, id: S) -> ApiResult<T> {
        self.ok_or_else(|| {
            ApiError::NotFound(format!("{} with ID {} not found", entity_type.as_ref(), id.as_ref()))
        })
    }

    fn ok_or_validation_error<S: AsRef<str>>(self, message: S) -> ApiResult<T> {
        self.ok_or_else(|| {
            ApiError::Validation(message.as_ref().to_string())
        })
    }

    fn ok_or_invalid_parameter<S: AsRef<str>>(self, parameter: S, message: S) -> ApiResult<T> {
        self.ok_or_else(|| {
            ApiError::Validation(format!("Invalid parameter {}: {}", parameter.as_ref(), message.as_ref()))
        })
    }
} 