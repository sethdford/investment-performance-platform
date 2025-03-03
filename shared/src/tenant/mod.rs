//! Tenant isolation module

use std::sync::Arc;
use tracing::{info, warn, error, debug};
use thiserror::Error;

/// Tenant error
#[derive(Error, Debug)]
pub enum TenantError {
    #[error("Tenant not found: {0}")]
    NotFound(String),
    
    #[error("Tenant access denied: {0}")]
    AccessDenied(String),
    
    #[error("Invalid tenant ID: {0}")]
    InvalidId(String),
    
    #[error("Tenant access denied: {0}")]
    AccessDenied(String),
} 