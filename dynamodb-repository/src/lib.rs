//! DynamoDB Repository
//!
//! This crate provides a repository implementation for DynamoDB.

use aws_sdk_dynamodb::Client;
use thiserror::Error;

/// Error type for the DynamoDB repository
#[derive(Error, Debug)]
pub enum DynamoDbError {
    /// AWS SDK error
    #[error("AWS SDK error: {0}")]
    AwsSdk(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Item not found
    #[error("Item not found")]
    NotFound,
    
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
}

/// DynamoDB Repository
pub struct DynamoDbRepository {
    /// DynamoDB client
    client: Client,
    
    /// Table name
    table_name: String,
}

impl DynamoDbRepository {
    /// Create a new DynamoDB repository
    pub fn new(client: Client, table_name: String) -> Self {
        Self {
            client,
            table_name,
        }
    }
    
    /// Get the DynamoDB client
    pub fn client(&self) -> &Client {
        &self.client
    }
    
    /// Get the table name
    pub fn table_name(&self) -> &str {
        &self.table_name
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
} 