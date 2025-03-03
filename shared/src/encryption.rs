//! Encryption utilities

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{Engine as _, engine::general_purpose};
use std::fmt;
use tracing::{info, error};

/// Encryption error
#[derive(Debug)]
pub enum EncryptionError {
    /// Key error
    KeyError(String),
    /// Encryption error
    EncryptionError(String),
    /// Decryption error
    DecryptionError(String),
}

impl fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncryptionError::KeyError(msg) => write!(f, "Key error: {}", msg),
            EncryptionError::EncryptionError(msg) => write!(f, "Encryption error: {}", msg),
            EncryptionError::DecryptionError(msg) => write!(f, "Decryption error: {}", msg),
        }
    }
} 