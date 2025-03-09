//! Error types for the factor model module

use thiserror::Error;

/// Errors that can occur in the factor model module
#[derive(Error, Debug)]
pub enum FactorModelError {
    /// Error when a factor is not found
    #[error("Factor not found: {0}")]
    FactorNotFound(String),

    /// Error when a security is not found
    #[error("Security not found: {0}")]
    SecurityNotFound(String),

    /// Error when a portfolio is not found
    #[error("Portfolio not found: {0}")]
    PortfolioNotFound(String),

    /// Error when a benchmark is not found
    #[error("Benchmark not found: {0}")]
    BenchmarkNotFound(String),

    /// Error when data is not available for a specific date
    #[error("Data not available for date: {0}")]
    DataNotAvailable(String),

    /// Error when a calculation fails
    #[error("Calculation error: {0}")]
    CalculationError(String),

    /// Error when a matrix operation fails
    #[error("Matrix operation error: {0}")]
    MatrixError(String),

    /// Error when a repository operation fails
    #[error("Repository error: {0}")]
    RepositoryError(String),

    /// Error when a validation fails
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Error when a serialization or deserialization fails
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Error when an I/O operation fails
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error when a database operation fails
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Error when an optimization fails
    #[error("Optimization error: {0}")]
    OptimizationError(String),

    /// Error when a parameter is invalid
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Error when a feature is not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),
} 