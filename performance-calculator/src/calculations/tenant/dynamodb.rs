use aws_sdk_dynamodb::{Client as DynamoDbClient, Error as DynamoDbError};
use aws_sdk_dynamodb::model::AttributeValue;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::env;
use tracing::{error, info};
use uuid::Uuid;

use crate::calculations::error_handling::CalculationError;
use super::{Tenant, TenantManager, TenantStatus, SubscriptionTier, ResourceLimits};

/// DynamoDB implementation of the TenantManager
pub struct DynamoDbTenantManager {
    client: DynamoDbClient,
    table_name: String,
}

impl DynamoDbTenantManager {
    /// Create a new DynamoDbTenantManager
    pub fn new(client: DynamoDbClient, table_name: String) -> Self {
        Self {
            client,
            table_name,
        }
    }
    
    /// Create a new DynamoDbTenantManager from environment variables
    pub async fn from_env() -> Result<Self, CalculationError> {
        let config = aws_config::load_from_env().await;
        let client = DynamoDbClient::new(&config);
        
        let table_name = env::var("DYNAMODB_TABLE")
            .unwrap_or_else(|_| "investment-performance".to_string());
        
        Ok(Self {
            client,
            table_name,
        })
    }
    
    /// Convert a Tenant to a DynamoDB item
    fn tenant_to_item(&self, tenant: &Tenant) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();
        
        // Primary key
        item.insert("PK".to_string(), AttributeValue::S(format!("TENANT#{}", tenant.id)));
        item.insert("SK".to_string(), AttributeValue::S("METADATA".to_string()));
        
        // Tenant attributes
        item.insert("id".to_string(), AttributeValue::S(tenant.id.clone()));
        item.insert("name".to_string(), AttributeValue::S(tenant.name.clone()));
        
        if let Some(description) = &tenant.description {
            item.insert("description".to_string(), AttributeValue::S(description.clone()));
        } else {
            item.insert("description".to_string(), AttributeValue::Null(true));
        }
        
        // Status
        let status = match tenant.status {
            TenantStatus::Active => "ACTIVE",
            TenantStatus::Suspended => "SUSPENDED",
            TenantStatus::Deactivated => "DEACTIVATED",
        };
        item.insert("status".to_string(), AttributeValue::S(status.to_string()));
        
        // Subscription tier
        let tier = match tenant.subscription_tier {
            SubscriptionTier::Free => "FREE",
            SubscriptionTier::Basic => "BASIC",
            SubscriptionTier::Professional => "PROFESSIONAL",
            SubscriptionTier::Enterprise => "ENTERPRISE",
            SubscriptionTier::Custom => "CUSTOM",
        };
        item.insert("subscription_tier".to_string(), AttributeValue::S(tier.to_string()));
        
        // Resource limits
        let limits = &tenant.resource_limits;
        item.insert("max_portfolios".to_string(), AttributeValue::N(limits.max_portfolios.to_string()));
        item.insert("max_accounts_per_portfolio".to_string(), AttributeValue::N(limits.max_accounts_per_portfolio.to_string()));
        item.insert("max_securities".to_string(), AttributeValue::N(limits.max_securities.to_string()));
        item.insert("max_transactions".to_string(), AttributeValue::N(limits.max_transactions.to_string()));
        item.insert("max_concurrent_calculations".to_string(), AttributeValue::N(limits.max_concurrent_calculations.to_string()));
        item.insert("max_storage_bytes".to_string(), AttributeValue::N(limits.max_storage_bytes.to_string()));
        item.insert("max_api_requests_per_minute".to_string(), AttributeValue::N(limits.max_api_requests_per_minute.to_string()));
        
        // Custom config
        let mut custom_config = HashMap::new();
        for (key, value) in &tenant.custom_config {
            custom_config.insert(key.clone(), AttributeValue::S(value.clone()));
        }
        item.insert("custom_config".to_string(), AttributeValue::M(custom_config));
        
        // Timestamps
        item.insert("created_at".to_string(), AttributeValue::S(tenant.created_at.to_rfc3339()));
        item.insert("updated_at".to_string(), AttributeValue::S(tenant.updated_at.to_rfc3339()));
        
        // Entity type for filtering
        item.insert("entity_type".to_string(), AttributeValue::S("TENANT".to_string()));
        
        item
    }
    
    /// Convert a DynamoDB item to a Tenant
    fn item_to_tenant(&self, item: &HashMap<String, AttributeValue>) -> Result<Tenant, CalculationError> {
        // Extract tenant ID
        let id = item.get("id")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| CalculationError::InvalidData("Missing tenant ID".to_string()))?
            .clone();
        
        // Extract tenant name
        let name = item.get("name")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| CalculationError::InvalidData("Missing tenant name".to_string()))?
            .clone();
        
        // Extract description (optional)
        let description = item.get("description")
            .and_then(|v| {
                if v.as_null().is_some() {
                    None
                } else {
                    v.as_s().ok()
                }
            })
            .map(|s| s.clone());
        
        // Extract status
        let status_str = item.get("status")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| CalculationError::InvalidData("Missing tenant status".to_string()))?;
        
        let status = match status_str.as_str() {
            "ACTIVE" => TenantStatus::Active,
            "SUSPENDED" => TenantStatus::Suspended,
            "DEACTIVATED" => TenantStatus::Deactivated,
            _ => return Err(CalculationError::InvalidData(format!("Invalid tenant status: {}", status_str))),
        };
        
        // Extract subscription tier
        let tier_str = item.get("subscription_tier")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| CalculationError::InvalidData("Missing subscription tier".to_string()))?;
        
        let subscription_tier = match tier_str.as_str() {
            "FREE" => SubscriptionTier::Free,
            "BASIC" => SubscriptionTier::Basic,
            "PROFESSIONAL" => SubscriptionTier::Professional,
            "ENTERPRISE" => SubscriptionTier::Enterprise,
            "CUSTOM" => SubscriptionTier::Custom,
            _ => return Err(CalculationError::InvalidData(format!("Invalid subscription tier: {}", tier_str))),
        };
        
        // Extract resource limits
        let max_portfolios = item.get("max_portfolios")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<usize>().ok())
            .unwrap_or(100);
        
        let max_accounts_per_portfolio = item.get("max_accounts_per_portfolio")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<usize>().ok())
            .unwrap_or(50);
        
        let max_securities = item.get("max_securities")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<usize>().ok())
            .unwrap_or(1000);
        
        let max_transactions = item.get("max_transactions")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<usize>().ok())
            .unwrap_or(100000);
        
        let max_concurrent_calculations = item.get("max_concurrent_calculations")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<usize>().ok())
            .unwrap_or(5);
        
        let max_storage_bytes = item.get("max_storage_bytes")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<u64>().ok())
            .unwrap_or(1_073_741_824); // 1 GB
        
        let max_api_requests_per_minute = item.get("max_api_requests_per_minute")
            .and_then(|v| v.as_n().ok())
            .and_then(|n| n.parse::<u32>().ok())
            .unwrap_or(100);
        
        let resource_limits = ResourceLimits {
            max_portfolios,
            max_accounts_per_portfolio,
            max_securities,
            max_transactions,
            max_concurrent_calculations,
            max_storage_bytes,
            max_api_requests_per_minute,
        };
        
        // Extract custom config
        let custom_config = item.get("custom_config")
            .and_then(|v| v.as_m().ok())
            .map(|m| {
                let mut config = HashMap::new();
                for (key, value) in m {
                    if let Some(s) = value.as_s().ok() {
                        config.insert(key.clone(), s.clone());
                    }
                }
                config
            })
            .unwrap_or_else(HashMap::new);
        
        // Extract timestamps
        let created_at_str = item.get("created_at")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| CalculationError::InvalidData("Missing created_at timestamp".to_string()))?;
        
        let created_at = DateTime::parse_from_rfc3339(created_at_str)
            .map_err(|e| CalculationError::InvalidData(format!("Invalid created_at timestamp: {}", e)))?
            .with_timezone(&Utc);
        
        let updated_at_str = item.get("updated_at")
            .and_then(|v| v.as_s().ok())
            .ok_or_else(|| CalculationError::InvalidData("Missing updated_at timestamp".to_string()))?;
        
        let updated_at = DateTime::parse_from_rfc3339(updated_at_str)
            .map_err(|e| CalculationError::InvalidData(format!("Invalid updated_at timestamp: {}", e)))?
            .with_timezone(&Utc);
        
        Ok(Tenant {
            id,
            name,
            description,
            status,
            subscription_tier,
            resource_limits,
            custom_config,
            created_at,
            updated_at,
        })
    }
}

#[async_trait::async_trait]
impl TenantManager for DynamoDbTenantManager {
    async fn create_tenant(&self, tenant: Tenant) -> Result<Tenant, CalculationError> {
        let item = self.tenant_to_item(&tenant);
        
        self.client.put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to create tenant: {:?}", e);
                CalculationError::DatabaseError(format!("Failed to create tenant: {}", e))
            })?;
        
        info!("Created tenant: {}", tenant.id);
        Ok(tenant)
    }
    
    async fn get_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>, CalculationError> {
        let response = self.client.get_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("TENANT#{}", tenant_id)))
            .key("SK", AttributeValue::S("METADATA".to_string()))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to get tenant: {:?}", e);
                CalculationError::DatabaseError(format!("Failed to get tenant: {}", e))
            })?;
        
        if let Some(item) = response.item {
            let tenant = self.item_to_tenant(&item)?;
            Ok(Some(tenant))
        } else {
            Ok(None)
        }
    }
    
    async fn update_tenant(&self, tenant: Tenant) -> Result<Tenant, CalculationError> {
        // Check if tenant exists
        if self.get_tenant(&tenant.id).await?.is_none() {
            return Err(CalculationError::NotFound(format!("Tenant with ID {} not found", tenant.id)));
        }
        
        let item = self.tenant_to_item(&tenant);
        
        self.client.put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to update tenant: {:?}", e);
                CalculationError::DatabaseError(format!("Failed to update tenant: {}", e))
            })?;
        
        info!("Updated tenant: {}", tenant.id);
        Ok(tenant)
    }
    
    async fn delete_tenant(&self, tenant_id: &str) -> Result<(), CalculationError> {
        // Check if tenant exists
        if self.get_tenant(tenant_id).await?.is_none() {
            return Err(CalculationError::NotFound(format!("Tenant with ID {} not found", tenant_id)));
        }
        
        self.client.delete_item()
            .table_name(&self.table_name)
            .key("PK", AttributeValue::S(format!("TENANT#{}", tenant_id)))
            .key("SK", AttributeValue::S("METADATA".to_string()))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to delete tenant: {:?}", e);
                CalculationError::DatabaseError(format!("Failed to delete tenant: {}", e))
            })?;
        
        info!("Deleted tenant: {}", tenant_id);
        Ok(())
    }
    
    async fn list_tenants(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<Tenant>, CalculationError> {
        let mut query = self.client.query()
            .table_name(&self.table_name)
            .key_condition_expression("begins_with(PK, :pk) AND SK = :sk")
            .expression_attribute_values(":pk", AttributeValue::S("TENANT#".to_string()))
            .expression_attribute_values(":sk", AttributeValue::S("METADATA".to_string()));
        
        if let Some(limit_val) = limit {
            query = query.limit(limit_val as i32);
        }
        
        let response = query.send().await.map_err(|e| {
            error!("Failed to list tenants: {:?}", e);
            CalculationError::DatabaseError(format!("Failed to list tenants: {}", e))
        })?;
        
        let items = response.items.unwrap_or_default();
        
        // Apply offset if provided
        let items = if let Some(offset_val) = offset {
            if offset_val < items.len() {
                items[offset_val..].to_vec()
            } else {
                Vec::new()
            }
        } else {
            items
        };
        
        let tenants = items.into_iter()
            .filter_map(|item| {
                match self.item_to_tenant(&item) {
                    Ok(tenant) => Some(tenant),
                    Err(e) => {
                        error!("Failed to parse tenant: {:?}", e);
                        None
                    }
                }
            })
            .collect();
        
        Ok(tenants)
    }
    
    async fn activate_tenant(&self, tenant_id: &str) -> Result<Tenant, CalculationError> {
        let tenant = match self.get_tenant(tenant_id).await? {
            Some(t) => t,
            None => return Err(CalculationError::NotFound(format!("Tenant with ID {} not found", tenant_id))),
        };
        
        let mut updated_tenant = tenant.clone();
        updated_tenant.status = TenantStatus::Active;
        updated_tenant.updated_at = Utc::now();
        
        self.update_tenant(updated_tenant).await
    }
    
    async fn suspend_tenant(&self, tenant_id: &str) -> Result<Tenant, CalculationError> {
        let tenant = match self.get_tenant(tenant_id).await? {
            Some(t) => t,
            None => return Err(CalculationError::NotFound(format!("Tenant with ID {} not found", tenant_id))),
        };
        
        let mut updated_tenant = tenant.clone();
        updated_tenant.status = TenantStatus::Suspended;
        updated_tenant.updated_at = Utc::now();
        
        self.update_tenant(updated_tenant).await
    }
    
    async fn deactivate_tenant(&self, tenant_id: &str) -> Result<Tenant, CalculationError> {
        let tenant = match self.get_tenant(tenant_id).await? {
            Some(t) => t,
            None => return Err(CalculationError::NotFound(format!("Tenant with ID {} not found", tenant_id))),
        };
        
        let mut updated_tenant = tenant.clone();
        updated_tenant.status = TenantStatus::Deactivated;
        updated_tenant.updated_at = Utc::now();
        
        self.update_tenant(updated_tenant).await
    }
    
    async fn update_resource_limits(&self, tenant_id: &str, limits: ResourceLimits) -> Result<Tenant, CalculationError> {
        let tenant = match self.get_tenant(tenant_id).await? {
            Some(t) => t,
            None => return Err(CalculationError::NotFound(format!("Tenant with ID {} not found", tenant_id))),
        };
        
        let mut updated_tenant = tenant.clone();
        updated_tenant.resource_limits = limits;
        updated_tenant.updated_at = Utc::now();
        
        self.update_tenant(updated_tenant).await
    }
    
    async fn tenant_exists(&self, tenant_id: &str) -> Result<bool, CalculationError> {
        let tenant = self.get_tenant(tenant_id).await?;
        Ok(tenant.is_some())
    }
    
    async fn validate_tenant_access(&self, tenant_id: &str, resource_id: &str) -> Result<bool, CalculationError> {
        // Check if tenant exists and is active
        let tenant = match self.get_tenant(tenant_id).await? {
            Some(t) => t,
            None => return Ok(false),
        };
        
        if !tenant.is_active() {
            return Ok(false);
        }
        
        // Check if resource belongs to tenant
        // In a real implementation, we would query the database to check if the resource belongs to the tenant
        // For now, we'll just check if the resource ID starts with the tenant ID
        Ok(resource_id.starts_with(&format!("{}_", tenant_id)))
    }
} 