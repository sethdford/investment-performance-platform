//! Calculation modules for the Performance Calculator
//!
//! This module contains all the calculation logic for the Performance Calculator,
//! including performance metrics, risk metrics, and supporting functionality.

// Core modules
pub mod config;
pub mod factory;
pub mod portfolio;
pub mod events;

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
pub use performance_metrics::{calculate_twr, calculate_mwr};
pub use risk_metrics::{calculate_volatility, calculate_sharpe_ratio};
pub use currency::CurrencyConverter;
pub use distributed_cache::Cache;
pub use audit::AuditTrail;
pub use config::Config;
pub use factory::ComponentFactory; 