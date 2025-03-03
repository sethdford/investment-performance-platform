//! Repository module for data access
//! 
//! This module provides repository implementations for accessing data from various sources.
//! It includes a DynamoDB repository and a cached version with TTL-based caching.

mod dynamodb;
mod cached;

pub use dynamodb::DynamoDbRepository;
pub use cached::CachedDynamoDbRepository;

use async_trait::async_trait;
use crate::models::{Portfolio, Transaction, Account, Security, Client, Benchmark, Price, Position};
use crate::error::AppError;
use std::collections::HashMap;

/// Pagination options for repository queries
#[derive(Debug, Clone)]
pub struct PaginationOptions {
    /// Maximum number of items to return
    pub limit: Option<u32>,
    /// Token for retrieving the next page of results
    pub next_token: Option<String>,
}

/// Result of a paginated query
#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    /// Items returned by the query
    pub items: Vec<T>,
    /// Token for retrieving the next page of results
    pub next_token: Option<String>,
}

/// Repository trait for data access
#[async_trait]
pub trait Repository {
    /// Get a portfolio by ID
    async fn get_portfolio(&self, id: &str) -> Result<Option<Portfolio>, AppError>;
    
    /// List portfolios with optional filtering and pagination
    async fn list_portfolios(
        &self, 
        client_id: Option<&str>,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Portfolio>, AppError>;
    
    /// Create or update a portfolio
    async fn put_portfolio(&self, portfolio: &Portfolio) -> Result<(), AppError>;
    
    /// Delete a portfolio
    async fn delete_portfolio(&self, id: &str) -> Result<(), AppError>;
    
    /// Get a transaction by ID
    async fn get_transaction(&self, id: &str) -> Result<Option<Transaction>, AppError>;
    
    /// List transactions with optional filtering and pagination
    async fn list_transactions(
        &self,
        account_id: Option<&str>,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Transaction>, AppError>;
    
    /// Create or update a transaction
    async fn put_transaction(&self, transaction: &Transaction) -> Result<(), AppError>;
    
    /// Delete a transaction
    async fn delete_transaction(&self, id: &str) -> Result<(), AppError>;
    
    /// Get an account by ID
    async fn get_account(&self, id: &str) -> Result<Option<Account>, AppError>;
    
    /// List accounts with optional filtering and pagination
    async fn list_accounts(
        &self,
        portfolio_id: Option<&str>,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Account>, AppError>;
    
    /// Create or update an account
    async fn put_account(&self, account: &Account) -> Result<(), AppError>;
    
    /// Delete an account
    async fn delete_account(&self, id: &str) -> Result<(), AppError>;
    
    /// Get a security by ID
    async fn get_security(&self, id: &str) -> Result<Option<Security>, AppError>;
    
    /// List securities with optional filtering and pagination
    async fn list_securities(
        &self,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Security>, AppError>;
    
    /// Create or update a security
    async fn put_security(&self, security: &Security) -> Result<(), AppError>;
    
    /// Delete a security
    async fn delete_security(&self, id: &str) -> Result<(), AppError>;
    
    /// Get a client by ID
    async fn get_client(&self, id: &str) -> Result<Option<Client>, AppError>;
    
    /// List clients with optional pagination
    async fn list_clients(
        &self,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Client>, AppError>;
    
    /// Create or update a client
    async fn put_client(&self, client: &Client) -> Result<(), AppError>;
    
    /// Delete a client
    async fn delete_client(&self, id: &str) -> Result<(), AppError>;
    
    /// Get a benchmark by ID
    async fn get_benchmark(&self, id: &str) -> Result<Option<Benchmark>, AppError>;
    
    /// List benchmarks with optional pagination
    async fn list_benchmarks(
        &self,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Benchmark>, AppError>;
    
    /// Create or update a benchmark
    async fn put_benchmark(&self, benchmark: &Benchmark) -> Result<(), AppError>;
    
    /// Delete a benchmark
    async fn delete_benchmark(&self, id: &str) -> Result<(), AppError>;
    
    /// Get a price by security ID and date
    async fn get_price(&self, security_id: &str, date: &str) -> Result<Option<Price>, AppError>;
    
    /// List prices for a security with optional date range and pagination
    async fn list_prices(
        &self,
        security_id: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
        pagination: Option<PaginationOptions>
    ) -> Result<PaginatedResult<Price>, AppError>;
    
    /// Create or update a price
    async fn put_price(&self, price: &Price) -> Result<(), AppError>;
    
    /// Get positions for an account on a specific date
    async fn get_positions(
        &self,
        account_id: &str,
        date: &str
    ) -> Result<Vec<Position>, AppError>;
    
    /// Create or update a position
    async fn put_position(&self, position: &Position) -> Result<(), AppError>;
} 