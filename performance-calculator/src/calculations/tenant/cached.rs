use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use chrono::Utc;
use tracing::{info, warn};

use crate::calculations::error_handling::CalculationError;
use super::{Tenant, TenantManager, ResourceLimits};

/// Cache entry for a tenant
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    /// Create a new cache entry
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Instant::now() + ttl,
        }
    }
    
    /// Check if the cache entry is expired
    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// Cached implementation of the TenantManager
pub struct CachedTenantManager<T: TenantManager> {
    inner: T,
    tenant_cache: Arc<RwLock<HashMap<String, CacheEntry<Tenant>>>>,
    cache_ttl: Duration,
}

impl<T: TenantManager> CachedTenantManager<T> {
    /// Create a new CachedTenantManager
    pub fn new(inner: T, cache_ttl_seconds: u64) -> Self {
        Self {
            inner,
            tenant_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(cache_ttl_seconds),
        }
    }
    
    /// Get a tenant from the cache
    fn get_from_cache(&self, tenant_id: &str) -> Option<Tenant> {
        let cache = match self.tenant_cache.read() {
            Ok(cache) => cache,
            Err(e) => {
                warn!("Failed to acquire read lock on tenant cache: {}", e);
                return None;
            }
        };
        
        if let Some(entry) = cache.get(tenant_id) {
            if entry.is_expired() {
                return None;
            }
            
            return Some(entry.value.clone());
        }
        
        None
    }
    
    /// Add a tenant to the cache
    fn add_to_cache(&self, tenant: Tenant) {
        let mut cache = match self.tenant_cache.write() {
            Ok(cache) => cache,
            Err(e) => {
                warn!("Failed to acquire write lock on tenant cache: {}", e);
                return;
            }
        };
        
        let entry = CacheEntry::new(tenant.clone(), self.cache_ttl);
        cache.insert(tenant.id.clone(), entry);
    }
    
    /// Remove a tenant from the cache
    fn remove_from_cache(&self, tenant_id: &str) {
        let mut cache = match self.tenant_cache.write() {
            Ok(cache) => cache,
            Err(e) => {
                warn!("Failed to acquire write lock on tenant cache: {}", e);
                return;
            }
        };
        
        cache.remove(tenant_id);
    }
    
    /// Clear the cache
    pub fn clear_cache(&self) {
        let mut cache = match self.tenant_cache.write() {
            Ok(cache) => cache,
            Err(e) => {
                warn!("Failed to acquire write lock on tenant cache: {}", e);
                return;
            }
        };
        
        cache.clear();
        info!("Tenant cache cleared");
    }
}

#[async_trait::async_trait]
impl<T: TenantManager + Send + Sync> TenantManager for CachedTenantManager<T> {
    async fn create_tenant(&self, tenant: Tenant) -> Result<Tenant, CalculationError> {
        let created_tenant = self.inner.create_tenant(tenant).await?;
        self.add_to_cache(created_tenant.clone());
        Ok(created_tenant)
    }
    
    async fn get_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>, CalculationError> {
        // Try to get from cache first
        if let Some(tenant) = self.get_from_cache(tenant_id) {
            return Ok(Some(tenant));
        }
        
        // If not in cache, get from inner manager
        let tenant = self.inner.get_tenant(tenant_id).await?;
        
        // Add to cache if found
        if let Some(tenant) = &tenant {
            self.add_to_cache(tenant.clone());
        }
        
        Ok(tenant)
    }
    
    async fn update_tenant(&self, tenant: Tenant) -> Result<Tenant, CalculationError> {
        let updated_tenant = self.inner.update_tenant(tenant).await?;
        self.add_to_cache(updated_tenant.clone());
        Ok(updated_tenant)
    }
    
    async fn delete_tenant(&self, tenant_id: &str) -> Result<(), CalculationError> {
        let result = self.inner.delete_tenant(tenant_id).await;
        if result.is_ok() {
            self.remove_from_cache(tenant_id);
        }
        result
    }
    
    async fn list_tenants(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<Tenant>, CalculationError> {
        // Always get from inner manager for list operations
        let tenants = self.inner.list_tenants(limit, offset).await?;
        
        // Update cache with fetched tenants
        for tenant in &tenants {
            self.add_to_cache(tenant.clone());
        }
        
        Ok(tenants)
    }
    
    async fn activate_tenant(&self, tenant_id: &str) -> Result<Tenant, CalculationError> {
        let activated_tenant = self.inner.activate_tenant(tenant_id).await?;
        self.add_to_cache(activated_tenant.clone());
        Ok(activated_tenant)
    }
    
    async fn suspend_tenant(&self, tenant_id: &str) -> Result<Tenant, CalculationError> {
        let suspended_tenant = self.inner.suspend_tenant(tenant_id).await?;
        self.add_to_cache(suspended_tenant.clone());
        Ok(suspended_tenant)
    }
    
    async fn deactivate_tenant(&self, tenant_id: &str) -> Result<Tenant, CalculationError> {
        let deactivated_tenant = self.inner.deactivate_tenant(tenant_id).await?;
        self.add_to_cache(deactivated_tenant.clone());
        Ok(deactivated_tenant)
    }
    
    async fn update_resource_limits(&self, tenant_id: &str, limits: ResourceLimits) -> Result<Tenant, CalculationError> {
        let updated_tenant = self.inner.update_resource_limits(tenant_id, limits).await?;
        self.add_to_cache(updated_tenant.clone());
        Ok(updated_tenant)
    }
    
    async fn tenant_exists(&self, tenant_id: &str) -> Result<bool, CalculationError> {
        // Try to get from cache first
        if self.get_from_cache(tenant_id).is_some() {
            return Ok(true);
        }
        
        // If not in cache, check with inner manager
        self.inner.tenant_exists(tenant_id).await
    }
    
    async fn validate_tenant_access(&self, tenant_id: &str, resource_id: &str) -> Result<bool, CalculationError> {
        // Always delegate to inner manager for access validation
        self.inner.validate_tenant_access(tenant_id, resource_id).await
    }
} 