/// Shared library for the Rust SAM application
///
/// This library contains common code shared between the API handler and event processor
/// Lambda functions. It includes data models, repository access, error handling, and configuration.
///
/// # Modules
///
/// * `models` - Data models for items and events
/// * `repository` - DynamoDB repository for data access
/// * `error` - Error handling
/// * `config` - Configuration management
/// * `validation` - Data validation utilities and implementations
/// * `visualization` - Chart generation and visualization services

pub mod models;
pub mod repository;
pub mod error;
pub mod config;
pub mod validation;
pub mod visualization;

// Re-export common types
pub use error::AppError;
pub use validation::{Validate, ValidationError};
pub use repository::{Repository, PaginationOptions, PaginatedResult};

#[cfg(test)]
mod tests; 