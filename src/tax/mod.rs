//! Tax optimization and management for the Investment Management Platform
//!
//! This module provides tax optimization and management capabilities for the platform.
//!
//! ## Submodules
//!
//! - **tlh**: Algorithmic tax-loss harvesting

pub mod tlh;

// Re-export common types
pub use tlh::{TaxOptimizationSettings, AlgorithmicTLHService, AlgorithmicTLHConfig, TLHPerformanceReport}; 