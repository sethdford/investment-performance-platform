//! Cloud integration for the Investment Management Platform
//!
//! This module provides cloud integration capabilities for the platform.
//!
//! ## Submodules
//!
//! - **aws**: AWS-specific integrations

pub mod aws {
    pub mod tlh_alerts;
    
    #[cfg(test)]
    pub mod mock;
    
    // Re-export common types
    pub use tlh_alerts::{AwsTlhAlertConfig, AwsTlhAlertService, TlhOpportunityAlert, MarketPrediction};
}

// Legacy module exports for backward compatibility
#[deprecated(since = "0.1.0", note = "Use aws::tlh_alerts instead")]
pub use aws::tlh_alerts as aws_tlh_alerts;
#[cfg(test)]
#[deprecated(since = "0.1.0", note = "Use aws::mock instead")]
pub use aws::mock as mock_aws; 