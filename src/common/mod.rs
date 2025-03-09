//! Common utilities and error handling for the Investment Management Platform
//!
//! This module contains common utilities and error handling used throughout the platform.

pub mod error;

// Re-export error types for convenience
pub use error::{Error, Result, ApiError, ApiResult};

// Re-export common error handling utilities
pub use error::utils as error_utils;
pub use error::{ResultExt, OptionExt}; 