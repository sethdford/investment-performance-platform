//! Factor model implementation for risk analysis, performance attribution, and portfolio optimization.
//!
//! This module implements factor model capabilities similar to MSCI Barra, including:
//! - Factor-based risk analysis
//! - Performance attribution
//! - Portfolio optimization
//!
//! The implementation includes support for different types of factors (market, style, industry, etc.),
//! factor exposures, factor returns, and covariance matrices.

pub mod types;
pub mod repository;
pub mod calculator;
pub mod covariance;
pub mod error;
pub mod api;
pub mod visualization;

// Re-export key types and traits
pub use error::FactorModelError;
pub use types::{
    Factor, FactorCategory, FactorCovariance, FactorExposure, FactorReturn,
    FactorModelVersion, FactorModelStatus,
};
pub use repository::FactorRepository;
pub use calculator::FactorModelCalculator;
pub use covariance::{
    CovarianceEstimator, SampleCovarianceEstimator, LedoitWolfEstimator,
    ShrinkageTarget, CovarianceEstimatorFactory,
};
pub use api::FactorModelApiService;
pub use visualization::FactorModelVisualizationService;

/// Result type for factor model operations
pub type Result<T> = std::result::Result<T, FactorModelError>; 