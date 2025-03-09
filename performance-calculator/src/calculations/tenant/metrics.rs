use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use tracing::{info, warn};
use uuid::Uuid;
use aws_sdk_dynamodb::types::AttributeValue;
use crate::calculations::tenant::{Tenant, ResourceLimits};
use crate::calculations::error_handling::{CalculationError, RetryConfig, with_retry};
use aws_sdk_dynamodb::Client as DynamoDbClient;
use aws_sdk_dynamodb::operation::{
    put_item::PutItemInput,
    get_item::GetItemInput,
    query::QueryInput,
    delete_item::DeleteItemInput,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use aws_config;
use rust_decimal::Decimal;
use std::sync::{Arc, RwLock};
use shared::models::Portfolio;

/// Tenant usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantUsageMetrics {
    /// Tenant ID
    pub tenant_id: String,
    
    /// Current number of portfolios
    pub portfolio_count: usize,
    
    /// Current number of accounts
    pub account_count: usize,
    
    /// Current number of securities
    pub security_count: usize,
    
    /// Current number of transactions
    pub transaction_count: usize,
    
    /// Current storage usage in bytes
    pub storage_usage_bytes: u64,
    
    /// API requests in the current period
    pub api_requests: u32,
    
    /// Concurrent calculations currently running
    pub concurrent_calculations: usize,
    
    /// Timestamp when metrics were last updated
    pub updated_at: DateTime<Utc>,
}

impl TenantUsageMetrics {
    /// Create new tenant usage metrics
    pub fn new(tenant_id: &str) -> Self {
        Self {
            tenant_id: tenant_id.to_string(),
            portfolio_count: 0,
            account_count: 0,
            security_count: 0,
            transaction_count: 0,
            storage_usage_bytes: 0,
            api_requests: 0,
            concurrent_calculations: 0,
            updated_at: Utc::now(),
        }
    }
    
    /// Check if any resource limits are exceeded
    pub fn check_limits(&self, limits: &ResourceLimits) -> Vec<String> {
        let mut exceeded = Vec::new();
        
        if self.portfolio_count >= limits.max_portfolios {
            exceeded.push(format!("Portfolio limit exceeded: {}/{}", 
                self.portfolio_count, limits.max_portfolios));
        }
        
        if self.account_count >= limits.max_accounts_per_portfolio {
            exceeded.push(format!("Account limit exceeded: {}/{}", 
                self.account_count, limits.max_accounts_per_portfolio));
        }
        
        if self.security_count >= limits.max_securities {
            exceeded.push(format!("Security limit exceeded: {}/{}", 
                self.security_count, limits.max_securities));
        }
        
        if self.transaction_count >= limits.max_transactions {
            exceeded.push(format!("Transaction limit exceeded: {}/{}", 
                self.transaction_count, limits.max_transactions));
        }
        
        if self.storage_usage_bytes >= limits.max_storage_bytes {
            exceeded.push(format!("Storage limit exceeded: {}/{} bytes", 
                self.storage_usage_bytes, limits.max_storage_bytes));
        }
        
        if self.api_requests >= limits.max_api_requests_per_minute {
            exceeded.push(format!("API request limit exceeded: {}/{} per minute", 
                self.api_requests, limits.max_api_requests_per_minute));
        }
        
        if self.concurrent_calculations >= limits.max_concurrent_calculations {
            exceeded.push(format!("Concurrent calculation limit exceeded: {}/{}", 
                self.concurrent_calculations, limits.max_concurrent_calculations));
        }
        
        exceeded
    }
    
    /// Calculate usage percentage for a specific resource
    pub fn usage_percentage(&self, resource: &str, limits: &ResourceLimits) -> f64 {
        match resource {
            "portfolios" => {
                if limits.max_portfolios == 0 { 0.0 } 
                else { (self.portfolio_count as f64 / limits.max_portfolios as f64) * 100.0 }
            },
            "accounts" => {
                if limits.max_accounts_per_portfolio == 0 { 0.0 } 
                else { (self.account_count as f64 / limits.max_accounts_per_portfolio as f64) * 100.0 }
            },
            "securities" => {
                if limits.max_securities == 0 { 0.0 } 
                else { (self.security_count as f64 / limits.max_securities as f64) * 100.0 }
            },
            "transactions" => {
                if limits.max_transactions == 0 { 0.0 } 
                else { (self.transaction_count as f64 / limits.max_transactions as f64) * 100.0 }
            },
            "storage" => {
                if limits.max_storage_bytes == 0 { 0.0 } 
                else { (self.storage_usage_bytes as f64 / limits.max_storage_bytes as f64) * 100.0 }
            },
            "api_requests" => {
                if limits.max_api_requests_per_minute == 0 { 0.0 } 
                else { (self.api_requests as f64 / limits.max_api_requests_per_minute as f64) * 100.0 }
            },
            "calculations" => {
                if limits.max_concurrent_calculations == 0 { 0.0 } 
                else { (self.concurrent_calculations as f64 / limits.max_concurrent_calculations as f64) * 100.0 }
            },
            _ => 0.0,
        }
    }
}

/// Tenant billing record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBillingRecord {
    /// Record ID
    pub id: String,
    
    /// Tenant ID
    pub tenant_id: String,
    
    /// Billing period start
    pub period_start: DateTime<Utc>,
    
    /// Billing period end
    pub period_end: DateTime<Utc>,
    
    /// Subscription tier
    pub subscription_tier: String,
    
    /// Base amount
    pub base_amount: f64,
    
    /// Additional charges
    pub additional_charges: HashMap<String, f64>,
    
    /// Total amount
    pub total_amount: f64,
    
    /// Currency code (e.g., USD)
    pub currency: String,
    
    /// Payment status
    pub status: BillingStatus,
    
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Billing status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BillingStatus {
    Pending,
    Paid,
    Failed,
    Cancelled,
}

impl TenantBillingRecord {
    /// Create a new billing record
    pub fn new(
        tenant_id: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        subscription_tier: &str,
        base_amount: f64,
        currency: &str,
    ) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4().to_string(),
            tenant_id: tenant_id.to_string(),
            period_start,
            period_end,
            subscription_tier: subscription_tier.to_string(),
            base_amount,
            additional_charges: HashMap::new(),
            total_amount: base_amount,
            currency: currency.to_string(),
            status: BillingStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Add an additional charge
    pub fn add_charge(&mut self, description: &str, amount: f64) {
        self.additional_charges.insert(description.to_string(), amount);
        self.total_amount += amount;
        self.updated_at = Utc::now();
    }
    
    /// Update the billing status
    pub fn update_status(&mut self, status: BillingStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}

/// Tenant metrics manager
#[async_trait::async_trait]
pub trait TenantMetricsManager: Send + Sync {
    /// Get current usage metrics for a tenant
    async fn get_usage_metrics(&self, tenant_id: &str) -> Result<TenantUsageMetrics, CalculationError>;
    
    /// Update usage metrics for a tenant
    async fn update_usage_metrics(&self, metrics: TenantUsageMetrics) -> Result<TenantUsageMetrics, CalculationError>;
    
    /// Increment a specific metric
    async fn increment_metric(&self, tenant_id: &str, metric: &str, amount: usize) -> Result<TenantUsageMetrics, CalculationError>;
    
    /// Decrement a specific metric
    async fn decrement_metric(&self, tenant_id: &str, metric: &str, amount: usize) -> Result<TenantUsageMetrics, CalculationError>;
    
    /// Track API request
    async fn track_api_request(&self, tenant_id: &str) -> Result<(), CalculationError>;
    
    /// Reset API request counter (typically called every minute)
    async fn reset_api_requests(&self, tenant_id: &str) -> Result<(), CalculationError>;
    
    /// Get billing records for a tenant
    async fn get_billing_records(
        &self, 
        tenant_id: &str,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<TenantBillingRecord>, CalculationError>;
    
    /// Create a billing record
    async fn create_billing_record(&self, record: TenantBillingRecord) -> Result<TenantBillingRecord, CalculationError>;
    
    /// Update a billing record
    async fn update_billing_record(&self, record: TenantBillingRecord) -> Result<TenantBillingRecord, CalculationError>;
}

/// In-memory implementation of TenantMetricsManager
pub struct InMemoryTenantMetricsManager {
    metrics: Arc<RwLock<HashMap<String, TenantUsageMetrics>>>,
    billing_records: Arc<RwLock<Vec<TenantBillingRecord>>>,
}

impl InMemoryTenantMetricsManager {
    /// Create a new in-memory tenant metrics manager
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            billing_records: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl TenantMetricsManager for InMemoryTenantMetricsManager {
    async fn get_usage_metrics(&self, tenant_id: &str) -> Result<TenantUsageMetrics, CalculationError> {
        let metrics = self.metrics.read().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire read lock: {}", e))
        })?;
        
        match metrics.get(tenant_id) {
            Some(m) => Ok(m.clone()),
            None => {
                // Return empty metrics if not found
                Ok(TenantUsageMetrics::new(tenant_id))
            }
        }
    }
    
    async fn update_usage_metrics(&self, metrics: TenantUsageMetrics) -> Result<TenantUsageMetrics, CalculationError> {
        let mut metrics_map = self.metrics.write().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire write lock: {}", e))
        })?;
        
        let tenant_id = metrics.tenant_id.clone();
        metrics_map.insert(tenant_id, metrics.clone());
        
        Ok(metrics)
    }
    
    async fn increment_metric(&self, tenant_id: &str, metric: &str, amount: usize) -> Result<TenantUsageMetrics, CalculationError> {
        let mut metrics_map = self.metrics.write().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire write lock: {}", e))
        })?;
        
        let metrics = metrics_map.entry(tenant_id.to_string())
            .or_insert_with(|| TenantUsageMetrics::new(tenant_id));
        
        match metric {
            "portfolios" => metrics.portfolio_count += amount,
            "accounts" => metrics.account_count += amount,
            "securities" => metrics.security_count += amount,
            "transactions" => metrics.transaction_count += amount,
            "storage" => metrics.storage_usage_bytes += amount as u64,
            "calculations" => metrics.concurrent_calculations += amount,
            _ => return Err(CalculationError::InvalidInput(format!("Invalid metric: {}", metric))),
        }
        
        metrics.updated_at = Utc::now();
        Ok(metrics.clone())
    }
    
    async fn decrement_metric(&self, tenant_id: &str, metric: &str, amount: usize) -> Result<TenantUsageMetrics, CalculationError> {
        let mut metrics_map = self.metrics.write().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire write lock: {}", e))
        })?;
        
        let metrics = metrics_map.entry(tenant_id.to_string())
            .or_insert_with(|| TenantUsageMetrics::new(tenant_id));
        
        match metric {
            "portfolios" => {
                if metrics.portfolio_count >= amount {
                    metrics.portfolio_count -= amount;
                } else {
                    metrics.portfolio_count = 0;
                }
            },
            "accounts" => {
                if metrics.account_count >= amount {
                    metrics.account_count -= amount;
                } else {
                    metrics.account_count = 0;
                }
            },
            "securities" => {
                if metrics.security_count >= amount {
                    metrics.security_count -= amount;
                } else {
                    metrics.security_count = 0;
                }
            },
            "transactions" => {
                if metrics.transaction_count >= amount {
                    metrics.transaction_count -= amount;
                } else {
                    metrics.transaction_count = 0;
                }
            },
            "storage" => {
                if metrics.storage_usage_bytes >= amount as u64 {
                    metrics.storage_usage_bytes -= amount as u64;
                } else {
                    metrics.storage_usage_bytes = 0;
                }
            },
            "calculations" => {
                if metrics.concurrent_calculations >= amount {
                    metrics.concurrent_calculations -= amount;
                } else {
                    metrics.concurrent_calculations = 0;
                }
            },
            _ => return Err(CalculationError::InvalidInput(format!("Invalid metric: {}", metric))),
        }
        
        metrics.updated_at = Utc::now();
        Ok(metrics.clone())
    }
    
    async fn track_api_request(&self, tenant_id: &str) -> Result<(), CalculationError> {
        let mut metrics_map = self.metrics.write().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire write lock: {}", e))
        })?;
        
        let metrics = metrics_map.entry(tenant_id.to_string())
            .or_insert_with(|| TenantUsageMetrics::new(tenant_id));
        
        metrics.api_requests += 1;
        metrics.updated_at = Utc::now();
        
        Ok(())
    }
    
    async fn reset_api_requests(&self, tenant_id: &str) -> Result<(), CalculationError> {
        let mut metrics_map = self.metrics.write().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire write lock: {}", e))
        })?;
        
        if let Some(metrics) = metrics_map.get_mut(tenant_id) {
            metrics.api_requests = 0;
            metrics.updated_at = Utc::now();
        }
        
        Ok(())
    }
    
    async fn get_billing_records(
        &self, 
        tenant_id: &str,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<TenantBillingRecord>, CalculationError> {
        let records = self.billing_records.read().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire read lock: {}", e))
        })?;
        
        let mut filtered_records: Vec<TenantBillingRecord> = records.iter()
            .filter(|r| r.tenant_id == tenant_id)
            .filter(|r| {
                if let Some(start) = start_date {
                    r.period_end >= start
                } else {
                    true
                }
            })
            .filter(|r| {
                if let Some(end) = end_date {
                    r.period_start <= end
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        
        // Sort by period_start (newest first)
        filtered_records.sort_by(|a, b| b.period_start.cmp(&a.period_start));
        
        // Apply pagination
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(usize::MAX);
        
        if offset >= filtered_records.len() {
            return Ok(Vec::new());
        }
        
        let end = std::cmp::min(offset + limit, filtered_records.len());
        Ok(filtered_records[offset..end].to_vec())
    }
    
    async fn create_billing_record(&self, record: TenantBillingRecord) -> Result<TenantBillingRecord, CalculationError> {
        let mut records = self.billing_records.write().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire write lock: {}", e))
        })?;
        
        records.push(record.clone());
        
        Ok(record)
    }
    
    async fn update_billing_record(&self, record: TenantBillingRecord) -> Result<TenantBillingRecord, CalculationError> {
        let mut records = self.billing_records.write().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire write lock: {}", e))
        })?;
        
        if let Some(index) = records.iter().position(|r| r.id == record.id) {
            records[index] = record.clone();
            Ok(record)
        } else {
            Err(CalculationError::NotFound(format!("Billing record with ID {} not found", record.id)))
        }
    }
}

/// DynamoDB implementation of TenantMetricsManager
pub struct DynamoDbTenantMetricsManager {
    client: DynamoDbClient,
    metrics_table_name: String,
    billing_table_name: String,
}

impl DynamoDbTenantMetricsManager {
    /// Create a new DynamoDB tenant metrics manager
    pub fn new(
        client: DynamoDbClient,
        metrics_table_name: String,
        billing_table_name: String,
    ) -> Self {
        Self {
            client,
            metrics_table_name,
            billing_table_name,
        }
    }
    
    /// Create a DynamoDB tenant metrics manager from environment variables
    pub async fn from_env() -> Result<Self, CalculationError> {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = DynamoDbClient::new(&config);
        
        let metrics_table_name = std::env::var("TENANT_METRICS_TABLE")
            .unwrap_or_else(|_| "tenant-metrics".to_string());
            
        let billing_table_name = std::env::var("TENANT_BILLING_TABLE")
            .unwrap_or_else(|_| "tenant-billing".to_string());
        
        Ok(Self::new(client, metrics_table_name, billing_table_name))
    }
}

#[async_trait::async_trait]
impl TenantMetricsManager for DynamoDbTenantMetricsManager {
    async fn get_usage_metrics(&self, tenant_id: &str) -> Result<TenantUsageMetrics, CalculationError> {
        // Implement a simple retry mechanism
        let table_name = self.metrics_table_name.clone();
        let tenant_id_value = tenant_id.to_string();
        
        let max_attempts = 3;
        let mut attempt = 0;
        let mut last_error = None;
        
        loop {
            attempt += 1;
            
            match self.client.get_item()
                .table_name(&table_name)
                .key("tenant_id", AttributeValue::S(tenant_id_value.clone()))
                .send()
                .await
            {
                Ok(result) => {
                    // Parse the DynamoDB response into TenantUsageMetrics
                    match result.item() {
                        Some(item) => {
                            let metrics = TenantUsageMetrics {
                                tenant_id: tenant_id.to_string(),
                                api_requests: item.get("api_requests")
                                    .and_then(|v| v.as_n().ok())
                                    .and_then(|n| n.parse::<u32>().ok())
                                    .unwrap_or(0),
                                portfolio_count: item.get("portfolio_count")
                                    .and_then(|v| v.as_n().ok())
                                    .and_then(|n| n.parse::<usize>().ok())
                                    .unwrap_or(0),
                                account_count: item.get("account_count")
                                    .and_then(|v| v.as_n().ok())
                                    .and_then(|n| n.parse::<usize>().ok())
                                    .unwrap_or(0),
                                security_count: item.get("security_count")
                                    .and_then(|v| v.as_n().ok())
                                    .and_then(|n| n.parse::<usize>().ok())
                                    .unwrap_or(0),
                                transaction_count: item.get("transaction_count")
                                    .and_then(|v| v.as_n().ok())
                                    .and_then(|n| n.parse::<usize>().ok())
                                    .unwrap_or(0),
                                storage_usage_bytes: item.get("storage_usage_bytes")
                                    .and_then(|v| v.as_n().ok())
                                    .and_then(|n| n.parse::<u64>().ok())
                                    .unwrap_or(0),
                                concurrent_calculations: item.get("concurrent_calculations")
                                    .and_then(|v| v.as_n().ok())
                                    .and_then(|n| n.parse::<usize>().ok())
                                    .unwrap_or(0),
                                updated_at: item.get("updated_at")
                                    .and_then(|v| v.as_s().ok())
                                    .and_then(|s| s.parse::<i64>().ok())
                                    .map(|ts| DateTime::<Utc>::from_timestamp(ts, 0).unwrap_or_default())
                                    .unwrap_or_default(),
                            };
                            return Ok(metrics);
                        },
                        None => return Err(CalculationError::NotFound(format!("Tenant metrics not found for tenant {}", tenant_id))),
                    }
                },
                Err(e) => {
                    last_error = Some(CalculationError::Database(format!("Failed to get tenant metrics: {}", e)));
                    
                    if attempt >= max_attempts {
                        return Err(last_error.unwrap());
                    }
                    
                    // Wait before retrying
                    tokio::time::sleep(std::time::Duration::from_millis(100 * attempt)).await;
                }
            }
        }
    }
    
    async fn update_usage_metrics(&self, metrics: TenantUsageMetrics) -> Result<TenantUsageMetrics, CalculationError> {
        // Convert metrics to DynamoDB item
        let mut item = HashMap::new();
        
        item.insert("tenant_id".to_string(), AttributeValue::S(metrics.tenant_id.clone()));
        item.insert("portfolio_count".to_string(), AttributeValue::N(metrics.portfolio_count.to_string()));
        item.insert("account_count".to_string(), AttributeValue::N(metrics.account_count.to_string()));
        item.insert("security_count".to_string(), AttributeValue::N(metrics.security_count.to_string()));
        item.insert("transaction_count".to_string(), AttributeValue::N(metrics.transaction_count.to_string()));
        item.insert("storage_usage_bytes".to_string(), AttributeValue::N(metrics.storage_usage_bytes.to_string()));
        item.insert("api_requests".to_string(), AttributeValue::N(metrics.api_requests.to_string()));
        item.insert("concurrent_calculations".to_string(), AttributeValue::N(metrics.concurrent_calculations.to_string()));
        item.insert("updated_at".to_string(), AttributeValue::S(metrics.updated_at.to_rfc3339()));
        
        // Put the item in DynamoDB
        self.client.put_item()
            .table_name(&self.metrics_table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| CalculationError::Database(format!("Failed to update tenant metrics: {}", e)))?;
        
        // Return the updated metrics
        Ok(metrics)
    }
    
    async fn increment_metric(&self, tenant_id: &str, metric: &str, amount: usize) -> Result<TenantUsageMetrics, CalculationError> {
        // Validate the metric name
        let update_expression = match metric {
            "portfolios" => "SET portfolio_count = if_not_exists(portfolio_count, :zero) + :amount, updated_at = :updated_at",
            "accounts" => "SET account_count = if_not_exists(account_count, :zero) + :amount, updated_at = :updated_at",
            "securities" => "SET security_count = if_not_exists(security_count, :zero) + :amount, updated_at = :updated_at",
            "transactions" => "SET transaction_count = if_not_exists(transaction_count, :zero) + :amount, updated_at = :updated_at",
            "storage" => "SET storage_usage_bytes = if_not_exists(storage_usage_bytes, :zero) + :amount, updated_at = :updated_at",
            "calculations" => "SET concurrent_calculations = if_not_exists(concurrent_calculations, :zero) + :amount, updated_at = :updated_at",
            _ => return Err(CalculationError::InvalidInput(format!("Invalid metric: {}", metric))),
        };
        
        // Create expression attribute values
        let mut expression_values = HashMap::new();
        expression_values.insert(":amount".to_string(), AttributeValue::N(amount.to_string()));
        expression_values.insert(":zero".to_string(), AttributeValue::N("0".to_string()));
        expression_values.insert(":updated_at".to_string(), AttributeValue::S(Utc::now().to_rfc3339()));
        
        // Update the item in DynamoDB
        self.client.update_item()
            .table_name(&self.metrics_table_name)
            .key("tenant_id", AttributeValue::S(tenant_id.to_string()))
            .update_expression(update_expression)
            .set_expression_attribute_values(Some(expression_values))
            .send()
            .await
            .map_err(|e| CalculationError::Database(format!("Failed to increment metric: {}", e)))?;
        
        // Get the updated metrics
        self.get_usage_metrics(tenant_id).await
    }
    
    async fn decrement_metric(&self, tenant_id: &str, metric: &str, amount: usize) -> Result<TenantUsageMetrics, CalculationError> {
        // Get current metrics to ensure we don't go below zero
        let current_metrics = self.get_usage_metrics(tenant_id).await?;
        
        // Determine the new value based on the metric
        let (update_expression, new_value) = match metric {
            "portfolios" => {
                let new_value = if current_metrics.portfolio_count >= amount {
                    current_metrics.portfolio_count - amount
                } else {
                    0
                };
                ("SET portfolio_count = :new_value, updated_at = :updated_at", new_value)
            },
            "accounts" => {
                let new_value = if current_metrics.account_count >= amount {
                    current_metrics.account_count - amount
                } else {
                    0
                };
                ("SET account_count = :new_value, updated_at = :updated_at", new_value)
            },
            "securities" => {
                let new_value = if current_metrics.security_count >= amount {
                    current_metrics.security_count - amount
                } else {
                    0
                };
                ("SET security_count = :new_value, updated_at = :updated_at", new_value)
            },
            "transactions" => {
                let new_value = if current_metrics.transaction_count >= amount {
                    current_metrics.transaction_count - amount
                } else {
                    0
                };
                ("SET transaction_count = :new_value, updated_at = :updated_at", new_value)
            },
            "storage" => {
                let new_value = if current_metrics.storage_usage_bytes >= amount as u64 {
                    current_metrics.storage_usage_bytes - amount as u64
                } else {
                    0
                };
                ("SET storage_usage_bytes = :new_value, updated_at = :updated_at", new_value as usize)
            },
            "calculations" => {
                let new_value = if current_metrics.concurrent_calculations >= amount {
                    current_metrics.concurrent_calculations - amount
                } else {
                    0
                };
                ("SET concurrent_calculations = :new_value, updated_at = :updated_at", new_value)
            },
            _ => return Err(CalculationError::InvalidInput(format!("Invalid metric: {}", metric))),
        };
        
        // Create expression attribute values
        let mut expression_values = HashMap::new();
        expression_values.insert(":new_value".to_string(), AttributeValue::N(new_value.to_string()));
        expression_values.insert(":updated_at".to_string(), AttributeValue::S(Utc::now().to_rfc3339()));
        
        // Update the item in DynamoDB
        self.client.update_item()
            .table_name(&self.metrics_table_name)
            .key("tenant_id", AttributeValue::S(tenant_id.to_string()))
            .update_expression(update_expression)
            .set_expression_attribute_values(Some(expression_values))
            .send()
            .await
            .map_err(|e| CalculationError::Database(format!("Failed to decrement metric: {}", e)))?;
        
        // Get the updated metrics
        self.get_usage_metrics(tenant_id).await
    }
    
    async fn track_api_request(&self, tenant_id: &str) -> Result<(), CalculationError> {
        // Create expression attribute values
        let mut expression_values = HashMap::new();
        expression_values.insert(":one".to_string(), AttributeValue::N("1".to_string()));
        expression_values.insert(":updated_at".to_string(), AttributeValue::S(Utc::now().to_rfc3339()));
        
        // Update the item in DynamoDB
        self.client.update_item()
            .table_name(&self.metrics_table_name)
            .key("tenant_id", AttributeValue::S(tenant_id.to_string()))
            .update_expression("SET api_requests = api_requests + :one, updated_at = :updated_at")
            .set_expression_attribute_values(Some(expression_values))
            .send()
            .await
            .map_err(|e| CalculationError::Database(format!("Failed to track API request: {}", e)))?;
        
        Ok(())
    }
    
    async fn reset_api_requests(&self, tenant_id: &str) -> Result<(), CalculationError> {
        // Create expression attribute values
        let mut expression_values = HashMap::new();
        expression_values.insert(":zero".to_string(), AttributeValue::N("0".to_string()));
        expression_values.insert(":updated_at".to_string(), AttributeValue::S(Utc::now().to_rfc3339()));
        
        // Update the item in DynamoDB
        self.client.update_item()
            .table_name(&self.metrics_table_name)
            .key("tenant_id", AttributeValue::S(tenant_id.to_string()))
            .update_expression("SET api_requests = :zero, updated_at = :updated_at")
            .set_expression_attribute_values(Some(expression_values))
            .send()
            .await
            .map_err(|e| CalculationError::Database(format!("Failed to reset API requests: {}", e)))?;
        
        Ok(())
    }
    
    async fn get_billing_records(
        &self, 
        tenant_id: &str,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<TenantBillingRecord>, CalculationError> {
        // Create the key condition expression
        let key_condition_expression = "tenant_id = :tenant_id";
        
        // Create expression attribute values
        let mut expression_values = HashMap::new();
        expression_values.insert(":tenant_id".to_string(), AttributeValue::S(tenant_id.to_string()));
        
        // Add filter expressions for date range if provided
        let mut filter_expressions = Vec::new();
        
        if let Some(start) = start_date {
            filter_expressions.push("period_end >= :start_date");
            expression_values.insert(":start_date".to_string(), AttributeValue::S(start.to_rfc3339()));
        }
        
        if let Some(end) = end_date {
            filter_expressions.push("period_start <= :end_date");
            expression_values.insert(":end_date".to_string(), AttributeValue::S(end.to_rfc3339()));
        }
        
        // Build the query
        let mut query = self.client.query()
            .table_name(&self.billing_table_name)
            .key_condition_expression(key_condition_expression)
            .set_expression_attribute_values(Some(expression_values.clone()));
        
        // Add filter expression if needed
        if !filter_expressions.is_empty() {
            query = query.filter_expression(filter_expressions.join(" AND "));
        }
        
        // Add limit if provided
        if let Some(limit_val) = limit {
            query = query.limit(limit_val as i32);
        }
        
        // Execute the query
        let result = query.send()
            .await
            .map_err(|e| CalculationError::Database(format!("Failed to get billing records: {}", e)))?;
        
        // Convert items to TenantBillingRecord
        let mut records = Vec::new();
        
        let items = result.items();
        for item in items {
            // Extract values from the DynamoDB item
            let id = item.get("id")
                .and_then(|v| v.as_s().ok())
                .ok_or_else(|| CalculationError::Database("Missing id in billing record".to_string()))?
                .to_string();
            
            let period_start = item.get("period_start")
                .and_then(|v| v.as_s().ok())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .ok_or_else(|| CalculationError::Database("Missing or invalid period_start in billing record".to_string()))?;
            
            let period_end = item.get("period_end")
                .and_then(|v| v.as_s().ok())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .ok_or_else(|| CalculationError::Database("Missing or invalid period_end in billing record".to_string()))?;
            
            let subscription_tier = item.get("subscription_tier")
                .and_then(|v| v.as_s().ok())
                .ok_or_else(|| CalculationError::Database("Missing subscription_tier in billing record".to_string()))?
                .to_string();
            
            let base_amount = item.get("base_amount")
                .and_then(|v| v.as_n().ok())
                .and_then(|n| n.parse::<f64>().ok())
                .ok_or_else(|| CalculationError::Database("Missing or invalid base_amount in billing record".to_string()))?;
            
            let total_amount = item.get("total_amount")
                .and_then(|v| v.as_n().ok())
                .and_then(|n| n.parse::<f64>().ok())
                .ok_or_else(|| CalculationError::Database("Missing or invalid total_amount in billing record".to_string()))?;
            
            let currency = item.get("currency")
                .and_then(|v| v.as_s().ok())
                .ok_or_else(|| CalculationError::Database("Missing currency in billing record".to_string()))?
                .to_string();
            
            let status_str = item.get("status")
                .and_then(|v| v.as_s().ok())
                .ok_or_else(|| CalculationError::Database("Missing status in billing record".to_string()))?;
            
            let status = match status_str.as_str() {
                "pending" => BillingStatus::Pending,
                "paid" => BillingStatus::Paid,
                "failed" => BillingStatus::Failed,
                "cancelled" => BillingStatus::Cancelled,
                _ => return Err(CalculationError::Database(format!("Invalid status in billing record: {}", status_str))),
            };
            
            let created_at = item.get("created_at")
                .and_then(|v| v.as_s().ok())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .ok_or_else(|| CalculationError::Database("Missing or invalid created_at in billing record".to_string()))?;
            
            let updated_at = item.get("updated_at")
                .and_then(|v| v.as_s().ok())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .ok_or_else(|| CalculationError::Database("Missing or invalid updated_at in billing record".to_string()))?;
            
            // Extract additional charges
            let additional_charges = if let Some(charges_attr) = item.get("additional_charges") {
                if let Ok(charges_map) = charges_attr.as_m() {
                    let mut charges = HashMap::new();
                    for (key, value) in charges_map {
                        if let Ok(amount_str) = value.as_n() {
                            if let Ok(amount) = amount_str.parse::<f64>() {
                                charges.insert(key.clone(), amount);
                            }
                        }
                    }
                    charges
                } else {
                    HashMap::new()
                }
            } else {
                HashMap::new()
            };
            
            // Create the billing record
            let record = TenantBillingRecord {
                id,
                tenant_id: tenant_id.to_string(),
                period_start,
                period_end,
                subscription_tier,
                base_amount,
                additional_charges,
                total_amount,
                currency,
                status,
                created_at,
                updated_at,
            };
            
            records.push(record);
        }
        
        // Apply offset if provided
        if let Some(offset_val) = offset {
            if offset_val < records.len() {
                records = records[offset_val..].to_vec();
            } else {
                records.clear();
            }
        }
        
        // Sort by period_start (newest first)
        records.sort_by(|a, b| b.period_start.cmp(&a.period_start));
        
        Ok(records)
    }
    
    async fn create_billing_record(&self, record: TenantBillingRecord) -> Result<TenantBillingRecord, CalculationError> {
        // Convert record to DynamoDB item
        let mut item = HashMap::new();
        
        item.insert("id".to_string(), AttributeValue::S(record.id.clone()));
        item.insert("tenant_id".to_string(), AttributeValue::S(record.tenant_id.clone()));
        item.insert("period_start".to_string(), AttributeValue::S(record.period_start.to_rfc3339()));
        item.insert("period_end".to_string(), AttributeValue::S(record.period_end.to_rfc3339()));
        item.insert("subscription_tier".to_string(), AttributeValue::S(record.subscription_tier.clone()));
        item.insert("base_amount".to_string(), AttributeValue::N(record.base_amount.to_string()));
        item.insert("total_amount".to_string(), AttributeValue::N(record.total_amount.to_string()));
        item.insert("currency".to_string(), AttributeValue::S(record.currency.clone()));
        
        // Convert status to string
        let status_str = match record.status {
            BillingStatus::Pending => "pending",
            BillingStatus::Paid => "paid",
            BillingStatus::Failed => "failed",
            BillingStatus::Cancelled => "cancelled",
        };
        item.insert("status".to_string(), AttributeValue::S(status_str.to_string()));
        
        item.insert("created_at".to_string(), AttributeValue::S(record.created_at.to_rfc3339()));
        item.insert("updated_at".to_string(), AttributeValue::S(record.updated_at.to_rfc3339()));
        
        // Convert additional charges to DynamoDB map
        if !record.additional_charges.is_empty() {
            let mut charges_map = HashMap::new();
            for (key, value) in &record.additional_charges {
                charges_map.insert(key.clone(), AttributeValue::N(value.to_string()));
            }
            item.insert("additional_charges".to_string(), AttributeValue::M(charges_map));
        }
        
        // Put the item in DynamoDB
        self.client.put_item()
            .table_name(&self.billing_table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| CalculationError::Database(format!("Failed to create billing record: {}", e)))?;
        
        // Return the created record
        Ok(record)
    }
    
    async fn update_billing_record(&self, record: TenantBillingRecord) -> Result<TenantBillingRecord, CalculationError> {
        // Check if the record exists
        let mut key = HashMap::new();
        key.insert("id".to_string(), AttributeValue::S(record.id.clone()));
        key.insert("tenant_id".to_string(), AttributeValue::S(record.tenant_id.clone()));
        
        let result = self.client.get_item()
            .table_name(&self.billing_table_name)
            .set_key(Some(key.clone()))
            .send()
            .await
            .map_err(|e| CalculationError::Database(format!("Failed to check if billing record exists: {}", e)))?;
        
        if result.item().is_none() {
            return Err(CalculationError::NotFound(format!("Billing record with ID {} not found", record.id)));
        }
        
        // Convert record to DynamoDB item (same as create_billing_record)
        let mut item = HashMap::new();
        
        item.insert("id".to_string(), AttributeValue::S(record.id.clone()));
        item.insert("tenant_id".to_string(), AttributeValue::S(record.tenant_id.clone()));
        item.insert("period_start".to_string(), AttributeValue::S(record.period_start.to_rfc3339()));
        item.insert("period_end".to_string(), AttributeValue::S(record.period_end.to_rfc3339()));
        item.insert("subscription_tier".to_string(), AttributeValue::S(record.subscription_tier.clone()));
        item.insert("base_amount".to_string(), AttributeValue::N(record.base_amount.to_string()));
        item.insert("total_amount".to_string(), AttributeValue::N(record.total_amount.to_string()));
        item.insert("currency".to_string(), AttributeValue::S(record.currency.clone()));
        
        // Convert status to string
        let status_str = match record.status {
            BillingStatus::Pending => "pending",
            BillingStatus::Paid => "paid",
            BillingStatus::Failed => "failed",
            BillingStatus::Cancelled => "cancelled",
        };
        item.insert("status".to_string(), AttributeValue::S(status_str.to_string()));
        
        item.insert("created_at".to_string(), AttributeValue::S(record.created_at.to_rfc3339()));
        item.insert("updated_at".to_string(), AttributeValue::S(record.updated_at.to_rfc3339()));
        
        // Convert additional charges to DynamoDB map
        if !record.additional_charges.is_empty() {
            let mut charges_map = HashMap::new();
            for (key, value) in &record.additional_charges {
                charges_map.insert(key.clone(), AttributeValue::N(value.to_string()));
            }
            item.insert("additional_charges".to_string(), AttributeValue::M(charges_map));
        }
        
        // Put the item in DynamoDB
        self.client.put_item()
            .table_name(&self.billing_table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| CalculationError::Database(format!("Failed to update billing record: {}", e)))?;
        
        // Return the updated record
        Ok(record)
    }
}

/// Get a tenant metrics manager instance
pub async fn get_tenant_metrics_manager() -> Result<Box<dyn TenantMetricsManager>, CalculationError> {
    // Check if we should use DynamoDB
    let use_dynamodb = std::env::var("USE_DYNAMODB_METRICS")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);
    
    if use_dynamodb {
        match DynamoDbTenantMetricsManager::from_env().await {
            Ok(manager) => Ok(Box::new(manager)),
            Err(e) => {
                warn!("Failed to create DynamoDB tenant metrics manager: {}. Falling back to in-memory implementation.", e);
                Ok(Box::new(InMemoryTenantMetricsManager::new()))
            }
        }
    } else {
        Ok(Box::new(InMemoryTenantMetricsManager::new()))
    }
}

/// Wrapper type for Portfolio to implement From for DynamoDB
pub struct PortfolioDynamoDb(Portfolio);

impl From<Portfolio> for PortfolioDynamoDb {
    fn from(portfolio: Portfolio) -> Self {
        PortfolioDynamoDb(portfolio)
    }
}

impl From<PortfolioDynamoDb> for HashMap<String, AttributeValue> {
    fn from(wrapper: PortfolioDynamoDb) -> Self {
        let portfolio = wrapper.0;
        let mut map = HashMap::new();
        map.insert("id".to_string(), AttributeValue::S(portfolio.id));
        map.insert("name".to_string(), AttributeValue::S(portfolio.name));
        map.insert("created_at".to_string(), AttributeValue::S(portfolio.created_at.to_rfc3339()));
        map.insert("updated_at".to_string(), AttributeValue::S(portfolio.updated_at.to_rfc3339()));
        map
    }
} 