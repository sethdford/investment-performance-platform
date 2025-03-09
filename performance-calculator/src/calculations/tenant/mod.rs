use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::calculations::error_handling::CalculationError;

// Import submodules
mod dynamodb;
mod cached;
mod metrics;
#[cfg(test)]
mod tests;

// Re-export implementations
pub use dynamodb::DynamoDbTenantManager;
pub use cached::CachedTenantManager;
pub use metrics::{
    TenantUsageMetrics, 
    TenantBillingRecord, 
    BillingStatus, 
    TenantMetricsManager, 
    InMemoryTenantMetricsManager,
    DynamoDbTenantMetricsManager,
    get_tenant_metrics_manager
};

/// Configuration for the tenant management system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    /// Whether tenant management is enabled
    pub enabled: bool,
    /// Default resource limits for new tenants
    pub default_resource_limits: ResourceLimits,
    /// Whether to enable tenant caching
    pub enable_caching: bool,
    /// TTL for tenant cache in seconds
    pub cache_ttl_seconds: u64,
}

impl Default for TenantConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_resource_limits: ResourceLimits::default(),
            enable_caching: true,
            cache_ttl_seconds: 3600,
        }
    }
}

/// Resource limits for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum number of portfolios
    pub max_portfolios: usize,
    /// Maximum number of accounts per portfolio
    pub max_accounts_per_portfolio: usize,
    /// Maximum number of securities
    pub max_securities: usize,
    /// Maximum number of transactions
    pub max_transactions: usize,
    /// Maximum number of concurrent calculations
    pub max_concurrent_calculations: usize,
    /// Maximum storage in bytes
    pub max_storage_bytes: u64,
    /// Maximum API requests per minute
    pub max_api_requests_per_minute: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_portfolios: 100,
            max_accounts_per_portfolio: 50,
            max_securities: 1000,
            max_transactions: 100000,
            max_concurrent_calculations: 5,
            max_storage_bytes: 1_073_741_824, // 1 GB
            max_api_requests_per_minute: 100,
        }
    }
}

/// Subscription tier for a tenant
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionTier {
    Free,
    Basic,
    Professional,
    Enterprise,
    Custom,
}

/// Tenant status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantStatus {
    Active,
    Suspended,
    Deactivated,
}

/// Tenant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    /// Unique tenant ID
    pub id: String,
    /// Tenant name
    pub name: String,
    /// Tenant description
    pub description: Option<String>,
    /// Tenant status
    pub status: TenantStatus,
    /// Subscription tier
    pub subscription_tier: SubscriptionTier,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Custom configuration
    pub custom_config: HashMap<String, String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl Tenant {
    /// Create a new tenant
    pub fn new(name: &str, description: Option<&str>, tier: SubscriptionTier) -> Self {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        Self {
            id,
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            status: TenantStatus::Active,
            subscription_tier: tier.clone(),
            resource_limits: match tier {
                SubscriptionTier::Free => ResourceLimits {
                    max_portfolios: 5,
                    max_accounts_per_portfolio: 10,
                    max_securities: 100,
                    max_transactions: 1000,
                    max_concurrent_calculations: 1,
                    max_storage_bytes: 104_857_600, // 100 MB
                    max_api_requests_per_minute: 10,
                },
                SubscriptionTier::Basic => ResourceLimits {
                    max_portfolios: 20,
                    max_accounts_per_portfolio: 20,
                    max_securities: 500,
                    max_transactions: 10000,
                    max_concurrent_calculations: 2,
                    max_storage_bytes: 524_288_000, // 500 MB
                    max_api_requests_per_minute: 50,
                },
                SubscriptionTier::Professional => ResourceLimits {
                    max_portfolios: 100,
                    max_accounts_per_portfolio: 50,
                    max_securities: 1000,
                    max_transactions: 100000,
                    max_concurrent_calculations: 5,
                    max_storage_bytes: 1_073_741_824, // 1 GB
                    max_api_requests_per_minute: 100,
                },
                SubscriptionTier::Enterprise => ResourceLimits {
                    max_portfolios: 500,
                    max_accounts_per_portfolio: 100,
                    max_securities: 5000,
                    max_transactions: 1000000,
                    max_concurrent_calculations: 20,
                    max_storage_bytes: 10_737_418_240, // 10 GB
                    max_api_requests_per_minute: 500,
                },
                SubscriptionTier::Custom => ResourceLimits::default(),
            },
            custom_config: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Check if the tenant is active
    pub fn is_active(&self) -> bool {
        self.status == TenantStatus::Active
    }
    
    /// Check if a resource limit would be exceeded
    pub fn would_exceed_limit(&self, resource_type: &str, current_count: usize) -> bool {
        match resource_type {
            "portfolios" => current_count >= self.resource_limits.max_portfolios,
            "accounts" => current_count >= self.resource_limits.max_accounts_per_portfolio,
            "securities" => current_count >= self.resource_limits.max_securities,
            "transactions" => current_count >= self.resource_limits.max_transactions,
            "concurrent_calculations" => current_count >= self.resource_limits.max_concurrent_calculations,
            _ => false,
        }
    }
}

/// Tenant manager trait
#[async_trait::async_trait]
pub trait TenantManager: Send + Sync {
    /// Create a new tenant
    async fn create_tenant(&self, tenant: Tenant) -> Result<Tenant, CalculationError>;
    
    /// Get a tenant by ID
    async fn get_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>, CalculationError>;
    
    /// Update a tenant
    async fn update_tenant(&self, tenant: Tenant) -> Result<Tenant, CalculationError>;
    
    /// Delete a tenant
    async fn delete_tenant(&self, tenant_id: &str) -> Result<(), CalculationError>;
    
    /// List all tenants
    async fn list_tenants(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<Tenant>, CalculationError>;
    
    /// Activate a tenant
    async fn activate_tenant(&self, tenant_id: &str) -> Result<Tenant, CalculationError>;
    
    /// Suspend a tenant
    async fn suspend_tenant(&self, tenant_id: &str) -> Result<Tenant, CalculationError>;
    
    /// Deactivate a tenant
    async fn deactivate_tenant(&self, tenant_id: &str) -> Result<Tenant, CalculationError>;
    
    /// Update tenant resource limits
    async fn update_resource_limits(&self, tenant_id: &str, limits: ResourceLimits) -> Result<Tenant, CalculationError>;
    
    /// Check if a tenant exists
    async fn tenant_exists(&self, tenant_id: &str) -> Result<bool, CalculationError>;
    
    /// Validate tenant access to a resource
    async fn validate_tenant_access(&self, tenant_id: &str, resource_id: &str) -> Result<bool, CalculationError>;
}

/// In-memory implementation of the TenantManager
pub struct InMemoryTenantManager {
    tenants: std::sync::RwLock<HashMap<String, Tenant>>,
    tenant_resources: std::sync::RwLock<HashMap<String, Vec<String>>>,
}

impl InMemoryTenantManager {
    /// Create a new InMemoryTenantManager
    pub fn new() -> Self {
        Self {
            tenants: std::sync::RwLock::new(HashMap::new()),
            tenant_resources: std::sync::RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl TenantManager for InMemoryTenantManager {
    async fn create_tenant(&self, tenant: Tenant) -> Result<Tenant, CalculationError> {
        let mut tenants = self.tenants.write().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire write lock: {}", e))
        })?;
        
        if tenants.contains_key(&tenant.id) {
            return Err(CalculationError::AlreadyExists(format!("Tenant with ID {} already exists", tenant.id)));
        }
        
        tenants.insert(tenant.id.clone(), tenant.clone());
        
        Ok(tenant)
    }
    
    async fn get_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>, CalculationError> {
        let tenants = self.tenants.read().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire read lock: {}", e))
        })?;
        
        Ok(tenants.get(tenant_id).cloned())
    }
    
    async fn update_tenant(&self, tenant: Tenant) -> Result<Tenant, CalculationError> {
        let mut tenants = self.tenants.write().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire write lock: {}", e))
        })?;
        
        if !tenants.contains_key(&tenant.id) {
            return Err(CalculationError::NotFound(format!("Tenant with ID {} not found", tenant.id)));
        }
        
        tenants.insert(tenant.id.clone(), tenant.clone());
        
        Ok(tenant)
    }
    
    async fn delete_tenant(&self, tenant_id: &str) -> Result<(), CalculationError> {
        let mut tenants = self.tenants.write().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire write lock: {}", e))
        })?;
        
        if !tenants.contains_key(tenant_id) {
            return Err(CalculationError::NotFound(format!("Tenant with ID {} not found", tenant_id)));
        }
        
        tenants.remove(tenant_id);
        
        // Also remove tenant resources
        let mut tenant_resources = self.tenant_resources.write().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire write lock: {}", e))
        })?;
        
        tenant_resources.remove(tenant_id);
        
        Ok(())
    }
    
    async fn list_tenants(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<Tenant>, CalculationError> {
        let tenants = self.tenants.read().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire read lock: {}", e))
        })?;
        
        let mut tenant_list: Vec<Tenant> = tenants.values().cloned().collect();
        
        // Sort by name
        tenant_list.sort_by(|a, b| a.name.cmp(&b.name));
        
        // Apply offset and limit
        let offset = offset.unwrap_or(0);
        if offset >= tenant_list.len() {
            return Ok(Vec::new());
        }
        
        let tenant_list = if let Some(limit) = limit {
            tenant_list[offset..std::cmp::min(offset + limit, tenant_list.len())].to_vec()
        } else {
            tenant_list[offset..].to_vec()
        };
        
        Ok(tenant_list)
    }
    
    async fn activate_tenant(&self, tenant_id: &str) -> Result<Tenant, CalculationError> {
        let mut tenants = self.tenants.write().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire write lock: {}", e))
        })?;
        
        let tenant = tenants.get_mut(tenant_id).ok_or_else(|| {
            CalculationError::NotFound(format!("Tenant with ID {} not found", tenant_id))
        })?;
        
        tenant.status = TenantStatus::Active;
        tenant.updated_at = Utc::now();
        
        Ok(tenant.clone())
    }
    
    async fn suspend_tenant(&self, tenant_id: &str) -> Result<Tenant, CalculationError> {
        let mut tenants = self.tenants.write().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire write lock: {}", e))
        })?;
        
        let tenant = tenants.get_mut(tenant_id).ok_or_else(|| {
            CalculationError::NotFound(format!("Tenant with ID {} not found", tenant_id))
        })?;
        
        tenant.status = TenantStatus::Suspended;
        tenant.updated_at = Utc::now();
        
        Ok(tenant.clone())
    }
    
    async fn deactivate_tenant(&self, tenant_id: &str) -> Result<Tenant, CalculationError> {
        let mut tenants = self.tenants.write().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire write lock: {}", e))
        })?;
        
        let tenant = tenants.get_mut(tenant_id).ok_or_else(|| {
            CalculationError::NotFound(format!("Tenant with ID {} not found", tenant_id))
        })?;
        
        tenant.status = TenantStatus::Deactivated;
        tenant.updated_at = Utc::now();
        
        Ok(tenant.clone())
    }
    
    async fn update_resource_limits(&self, tenant_id: &str, limits: ResourceLimits) -> Result<Tenant, CalculationError> {
        let mut tenants = self.tenants.write().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire write lock: {}", e))
        })?;
        
        let tenant = tenants.get_mut(tenant_id).ok_or_else(|| {
            CalculationError::NotFound(format!("Tenant with ID {} not found", tenant_id))
        })?;
        
        tenant.resource_limits = limits;
        tenant.updated_at = Utc::now();
        
        Ok(tenant.clone())
    }
    
    async fn tenant_exists(&self, tenant_id: &str) -> Result<bool, CalculationError> {
        let tenants = self.tenants.read().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire read lock: {}", e))
        })?;
        
        Ok(tenants.contains_key(tenant_id))
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
        let tenant_resources = self.tenant_resources.read().map_err(|e| {
            CalculationError::Internal(format!("Failed to acquire read lock: {}", e))
        })?;
        
        if let Some(resources) = tenant_resources.get(tenant_id) {
            return Ok(resources.contains(&resource_id.to_string()));
        }
        
        Ok(false)
    }
}

/// Get the appropriate tenant manager based on configuration
pub async fn get_tenant_manager(config: &TenantConfig) -> Result<Box<dyn TenantManager>, CalculationError> {
    if !config.enabled {
        // Return a dummy manager that allows everything
        return Ok(Box::new(InMemoryTenantManager::new()));
    }
    
    // Create the DynamoDB tenant manager
    let dynamodb_manager = DynamoDbTenantManager::from_env().await?;
    
    // Wrap with cache if enabled
    if config.enable_caching {
        let cached_manager = CachedTenantManager::new(dynamodb_manager, config.cache_ttl_seconds);
        Ok(Box::new(cached_manager))
    } else {
        Ok(Box::new(dynamodb_manager))
    }
} 