//! DynamoDB repository implementation

use async_trait::async_trait;
use aws_sdk_dynamodb::{Client as DynamoDbClient, Error as DynamoDbError};
use std::collections::HashMap;
use tracing::{info, error, warn};

use crate::models::{Portfolio, Transaction, Account, Security, Client, Benchmark, Price, Position};
use crate::error::AppError;
use super::{Repository, PaginationOptions, PaginatedResult};

/// DynamoDB repository implementation
pub struct DynamoDbRepository {
    client: DynamoDbClient,
    table_name: String,
}

impl DynamoDbRepository {
    /// Create a new DynamoDB repository
    pub fn new(client: DynamoDbClient, table_name: String) -> Self {
        Self { client, table_name }
    }

    // Helper method to build pagination parameters for DynamoDB
    fn build_pagination_params(
        &self,
        pagination: Option<&PaginationOptions>,
    ) -> (Option<u32>, Option<HashMap<String, AttributeValue>>) {
        match pagination {
            Some(options) => {
                let limit = options.limit;
                
                // Convert next_token to exclusive_start_key if provided
                let exclusive_start_key = match &options.next_token {
                    Some(token) => {
                        // The token is a base64-encoded JSON representation of the last evaluated key
                        match base64::decode(token) {
                            Ok(decoded) => {
                                match serde_json::from_slice::<HashMap<String, AttributeValue>>(&decoded) {
                                    Ok(key) => Some(key),
                                    Err(e) => {
                                        warn!("Failed to deserialize pagination token: {}", e);
                                        None
                                    }
                                }
                            },
                            Err(e) => {
                                warn!("Failed to decode pagination token: {}", e);
                                None
                            }
                        }
                    },
                    None => None,
                };
                
                (limit, exclusive_start_key)
            },
            None => (None, None),
        }
    }
    
    // Helper method to build next_token from last_evaluated_key
    fn build_next_token(
        &self,
        last_evaluated_key: Option<HashMap<String, AttributeValue>>,
    ) -> Option<String> {
        match last_evaluated_key {
            Some(key) => {
                // Convert the last evaluated key to a base64-encoded JSON string
                match serde_json::to_vec(&key) {
                    Ok(json) => {
                        Some(base64::encode(&json))
                    },
                    Err(e) => {
                        warn!("Failed to serialize last evaluated key: {}", e);
                        None
                    }
                }
            },
            None => None,
        }
    }

    async fn get_item_by_id(&self, id: &str, entity_type: &str) -> Result<Option<HashMap<String, AttributeValue>>, AppError> {
        let result = self.client.get_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_string()))
            .key("entity_type", AttributeValue::S(entity_type.to_string()))
            .send()
            .await
            .map_err(|e| self.handle_dynamodb_error(e, "get_item", Some(id)))?;
        
        Ok(result.item)
    }
}

#[async_trait]
impl Repository for DynamoDbRepository {
    async fn get_portfolio(&self, id: &str) -> Result<Option<Portfolio>, AppError> {
        let item = self.get_item_by_id(id, "portfolio").await?;
        
        match item {
            Some(item) => {
                match serde_dynamodb::from_hashmap::<Portfolio>(item) {
                    Ok(portfolio) => Ok(Some(portfolio)),
                    Err(e) => Err(AppError::Internal(format!("Failed to deserialize portfolio: {}", e)))
                }
            },
            None => Ok(None)
        }
    }
    
    async fn list_portfolios(
        &self, 
        client_id: Option<&str>,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Portfolio>, AppError> {
        let (limit, exclusive_start_key) = self.build_pagination_params(pagination.as_ref());
        
        // Build the query
        let mut query = self.client.scan()
            .table_name(&self.table_name)
            .filter_expression("entity_type = :entity_type")
            .expression_attribute_values(":entity_type", AttributeValue::S("portfolio".to_string()));
        
        // Add client_id filter if provided
        if let Some(client_id) = client_id {
            query = query
                .filter_expression("#client_id = :client_id")
                .expression_attribute_names("#client_id", "client_id")
                .expression_attribute_values(":client_id", AttributeValue::S(client_id.to_string()));
        }
        
        // Add pagination parameters
        if let Some(limit) = limit {
            query = query.limit(limit);
        }
        
        if let Some(exclusive_start_key) = exclusive_start_key {
            query = query.exclusive_start_key(exclusive_start_key);
        }
        
        // Execute the query
        let result = query.send().await.map_err(|e| {
            error!("Failed to list portfolios: {}", e);
            AppError::Database(format!("Failed to list portfolios: {}", e))
        })?;
        
        // Parse the results
        let items = result.items().unwrap_or_default();
        let mut portfolios = Vec::with_capacity(items.len());
        
        for item in items {
            match serde_dynamodb::from_hashmap::<Portfolio>(item.clone()) {
                Ok(portfolio) => {
                    portfolios.push(portfolio);
                },
                Err(e) => {
                    warn!("Failed to deserialize portfolio: {}", e);
                }
            }
        }
        
        // Build the next token
        let next_token = self.build_next_token(result.last_evaluated_key);
        
        Ok(PaginatedResult {
            items: portfolios,
            next_token,
        })
    }
    
    async fn put_portfolio(&self, portfolio: &Portfolio) -> Result<(), AppError> {
        // Implementation will be added later
        todo!("Implement put_portfolio")
    }
    
    async fn delete_portfolio(&self, id: &str) -> Result<(), AppError> {
        // Implementation will be added later
        todo!("Implement delete_portfolio")
    }
    
    // Implement other methods...
    // For brevity, we'll skip the implementation of the remaining methods
    // They would follow a similar pattern to the portfolio methods

    async fn get_transaction(&self, id: &str) -> Result<Option<Transaction>, AppError> {
        todo!("Implement get_transaction")
    }
    
    async fn list_transactions(
        &self,
        account_id: Option<&str>,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Transaction>, AppError> {
        let (limit, exclusive_start_key) = self.build_pagination_params(pagination.as_ref());
        
        // Build the query
        let mut query = self.client.scan()
            .table_name(&self.table_name)
            .filter_expression("entity_type = :entity_type")
            .expression_attribute_values(":entity_type", AttributeValue::S("transaction".to_string()));
        
        // Add account_id filter if provided
        if let Some(account_id) = account_id {
            query = query
                .filter_expression("#account_id = :account_id")
                .expression_attribute_names("#account_id", "account_id")
                .expression_attribute_values(":account_id", AttributeValue::S(account_id.to_string()));
        }
        
        // Add pagination parameters
        if let Some(limit) = limit {
            query = query.limit(limit);
        }
        
        if let Some(exclusive_start_key) = exclusive_start_key {
            query = query.exclusive_start_key(exclusive_start_key);
        }
        
        // Execute the query
        let result = query.send().await.map_err(|e| {
            error!("Failed to list transactions: {}", e);
            AppError::Database(format!("Failed to list transactions: {}", e))
        })?;
        
        // Parse the results
        let items = result.items().unwrap_or_default();
        let mut transactions = Vec::with_capacity(items.len());
        
        for item in items {
            match serde_dynamodb::from_hashmap::<Transaction>(item.clone()) {
                Ok(transaction) => {
                    transactions.push(transaction);
                },
                Err(e) => {
                    warn!("Failed to deserialize transaction: {}", e);
                }
            }
        }
        
        // Build the next token
        let next_token = self.build_next_token(result.last_evaluated_key);
        
        Ok(PaginatedResult {
            items: transactions,
            next_token,
        })
    }
    
    async fn put_transaction(&self, transaction: &Transaction) -> Result<(), AppError> {
        todo!("Implement put_transaction")
    }
    
    async fn delete_transaction(&self, id: &str) -> Result<(), AppError> {
        todo!("Implement delete_transaction")
    }
    
    async fn get_account(&self, id: &str) -> Result<Option<Account>, AppError> {
        todo!("Implement get_account")
    }
    
    async fn list_accounts(
        &self,
        portfolio_id: Option<&str>,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Account>, AppError> {
        todo!("Implement list_accounts")
    }
    
    async fn put_account(&self, account: &Account) -> Result<(), AppError> {
        todo!("Implement put_account")
    }
    
    async fn delete_account(&self, id: &str) -> Result<(), AppError> {
        todo!("Implement delete_account")
    }
    
    async fn get_security(&self, id: &str) -> Result<Option<Security>, AppError> {
        todo!("Implement get_security")
    }
    
    async fn list_securities(
        &self,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Security>, AppError> {
        todo!("Implement list_securities")
    }
    
    async fn put_security(&self, security: &Security) -> Result<(), AppError> {
        todo!("Implement put_security")
    }
    
    async fn delete_security(&self, id: &str) -> Result<(), AppError> {
        todo!("Implement delete_security")
    }
    
    async fn get_client(&self, id: &str) -> Result<Option<Client>, AppError> {
        todo!("Implement get_client")
    }
    
    async fn list_clients(
        &self,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Client>, AppError> {
        todo!("Implement list_clients")
    }
    
    async fn put_client(&self, client: &Client) -> Result<(), AppError> {
        todo!("Implement put_client")
    }
    
    async fn delete_client(&self, id: &str) -> Result<(), AppError> {
        todo!("Implement delete_client")
    }
    
    async fn get_benchmark(&self, id: &str) -> Result<Option<Benchmark>, AppError> {
        todo!("Implement get_benchmark")
    }
    
    async fn list_benchmarks(
        &self,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Benchmark>, AppError> {
        todo!("Implement list_benchmarks")
    }
    
    async fn put_benchmark(&self, benchmark: &Benchmark) -> Result<(), AppError> {
        todo!("Implement put_benchmark")
    }
    
    async fn delete_benchmark(&self, id: &str) -> Result<(), AppError> {
        todo!("Implement delete_benchmark")
    }
    
    async fn get_price(&self, security_id: &str, date: &str) -> Result<Option<Price>, AppError> {
        todo!("Implement get_price")
    }
    
    async fn list_prices(
        &self,
        security_id: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Price>, AppError> {
        todo!("Implement list_prices")
    }
    
    async fn put_price(&self, price: &Price) -> Result<(), AppError> {
        todo!("Implement put_price")
    }
    
    async fn get_positions(
        &self,
        account_id: &str,
        date: &str
    ) -> Result<Vec<Position>, AppError> {
        todo!("Implement get_positions")
    }
    
    async fn put_position(&self, position: &Position) -> Result<(), AppError> {
        todo!("Implement put_position")
    }
} 