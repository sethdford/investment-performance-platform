//! Portfolio management and analysis for the Investment Management Platform
//!
//! This module provides portfolio management and analysis capabilities for the platform.
//!
//! ## Submodules
//!
//! - **model**: Model portfolios and investment strategies
//! - **rebalancing**: Portfolio rebalancing and trade generation
//! - **factor**: Factor model analysis and risk decomposition

pub mod model;
pub mod rebalancing;
pub mod factor;

// Re-export common types
pub use model::ModelPortfolio as Portfolio;
pub use model::Sleeve as PortfolioSleeve;
pub use rebalancing::PortfolioHolding;
pub use model::UnifiedManagedAccount;
pub use rebalancing::{RebalanceTrade, TradeReason};
pub use factor::{FactorExposure, FactorReturn, FactorModelApi}; 