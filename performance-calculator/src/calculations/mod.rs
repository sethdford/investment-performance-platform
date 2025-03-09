//! Calculation modules for the Performance Calculator
//!
//! This module contains all the calculation logic for the Performance Calculator,
//! including performance metrics, risk metrics, and supporting functionality.

// Core modules
pub mod config;
pub mod factory;
pub mod portfolio;
pub mod events;
pub mod cache;

// Performance calculation modules
pub mod performance_metrics;
pub mod risk_metrics;
pub mod periodic_returns;

// Infrastructure modules
pub mod distributed_cache;
pub mod currency;
pub mod audit;
pub mod error_handling;
pub mod parallel;

// Phase 2 modules
pub mod streaming;
pub mod query_api;
pub mod scheduler;

// Phase 3 modules
pub mod analytics;
pub mod visualization;
pub mod integration;

// Benchmarking
pub mod benchmarks;
pub mod benchmark_comparison;

// Tests
#[cfg(test)]
pub mod tests;

// Re-export commonly used items
pub use performance_metrics::calculate_modified_dietz;
pub use performance_metrics::calculate_irr;
pub use risk_metrics::calculate_volatility;
pub use risk_metrics::calculate_sharpe_ratio;
pub use currency::CurrencyConverter;
pub use distributed_cache::{StringCache, BinaryCache, InMemoryCache};
pub use audit::AuditTrail;
pub use config::AppConfig as Config;
pub use factory::ComponentFactory;
pub use cache::{CalculationCache, create_performance_cache};
pub mod tenant;
pub mod twr;
pub mod factor_model; 