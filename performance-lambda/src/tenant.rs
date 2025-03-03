use aws_sdk_dynamodb::model::AttributeValue;
use std::collections::HashMap;
use std::fmt;
use std::error::Error;
use tracing::{debug, error, info, warn};

/// Represents errors that can occur during tenant operations
#[derive(Debug)]
pub enum TenantError {
    /// Tenant ID is missing
    MissingTenantId,
    /// Tenant ID is invalid
    InvalidTenantId(String),
    /// Tenant access is denied
    AccessDenied(String),
    /// Tenant operation failed
    OperationFailed(String),
}

impl fmt::Display for TenantError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TenantError::MissingTenantId => write!(f, "Tenant ID is missing"),
            TenantError::InvalidTenantId(msg) => write!(f, "Tenant ID is invalid: {}", msg),
            TenantError::AccessDenied(msg) => write!(f, "Tenant access denied: {}", msg),
            TenantError::OperationFailed(msg) => write!(f, "Tenant operation failed: {}", msg),
        }
    }
}

impl Error for TenantError {}

/// Represents a tenant context
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TenantContext {
    /// The tenant ID
    pub tenant_id: String,
    /// Additional tenant metadata
    pub metadata: HashMap<String, String>,
}

impl TenantContext {
    /// Create a new tenant context
    pub fn new(tenant_id: impl Into<String>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            metadata: HashMap::new(),
        }
    }
    
    /// Create a new tenant context with metadata
    pub fn with_metadata(
        tenant_id: impl Into<String>,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            metadata,
        }
    }
    
    /// Add metadata to the tenant context
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
    
    /// Get metadata from the tenant context
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Validate the tenant context
    pub fn validate(&self) -> Result<(), TenantError> {
        if self.tenant_id.is_empty() {
            return Err(TenantError::MissingTenantId);
        }
        
        // Add additional validation rules as needed
        
        Ok(())
    }
}

/// Trait for tenant-aware repositories
pub trait TenantAware {
    /// Get the current tenant context
    fn tenant_context(&self) -> &TenantContext;
    
    /// Create a new instance with a different tenant context
    fn with_tenant(self, tenant_context: TenantContext) -> Self;
    
    /// Add tenant ID to a DynamoDB item
    fn add_tenant_id_to_item(&self, item: &mut HashMap<String, AttributeValue>) {
        item.insert(
            "tenant_id".to_string(),
            AttributeValue::S(self.tenant_context().tenant_id.clone()),
        );
    }
    
    /// Create a tenant condition expression for DynamoDB operations
    fn tenant_condition_expression(&self) -> String {
        "tenant_id = :tenant_id".to_string()
    }
    
    /// Create tenant expression attribute values for DynamoDB operations
    fn tenant_expression_attribute_values(&self) -> HashMap<String, AttributeValue> {
        let mut values = HashMap::new();
        values.insert(
            ":tenant_id".to_string(),
            AttributeValue::S(self.tenant_context().tenant_id.clone()),
        );
        values
    }
    
    /// Validate that an item belongs to the current tenant
    fn validate_tenant_ownership(
        &self,
        item: &HashMap<String, AttributeValue>,
    ) -> Result<(), TenantError> {
        if let Some(tenant_id) = item.get("tenant_id") {
            if let Some(id) = tenant_id.as_s() {
                if id == &self.tenant_context().tenant_id {
                    return Ok(());
                }
            }
        }
        
        Err(TenantError::AccessDenied(
            "Item does not belong to the current tenant".to_string(),
        ))
    }
}

/// Tenant manager for handling tenant operations
#[derive(Debug, Clone)]
pub struct TenantManager {
    /// The current tenant context
    tenant_context: TenantContext,
}

impl TenantManager {
    /// Create a new tenant manager
    pub fn new(tenant_context: TenantContext) -> Result<Self, TenantError> {
        tenant_context.validate()?;
        
        Ok(Self { tenant_context })
    }
    
    /// Get the current tenant context
    pub fn tenant_context(&self) -> &TenantContext {
        &self.tenant_context
    }
    
    /// Create a new tenant manager with a different tenant context
    pub fn with_tenant(self, tenant_context: TenantContext) -> Result<Self, TenantError> {
        tenant_context.validate()?;
        
        Ok(Self { tenant_context })
    }
    
    /// Add tenant ID to a DynamoDB key
    pub fn add_tenant_id_to_key(&self, key: &mut HashMap<String, AttributeValue>) {
        key.insert(
            "tenant_id".to_string(),
            AttributeValue::S(self.tenant_context.tenant_id.clone()),
        );
    }
    
    /// Add tenant ID to a DynamoDB item
    pub fn add_tenant_id_to_item(&self, item: &mut HashMap<String, AttributeValue>) {
        item.insert(
            "tenant_id".to_string(),
            AttributeValue::S(self.tenant_context.tenant_id.clone()),
        );
    }
    
    /// Create a tenant key condition expression for DynamoDB operations
    pub fn tenant_key_condition_expression(&self) -> String {
        "tenant_id = :tenant_id".to_string()
    }
    
    /// Create a tenant filter expression for DynamoDB operations
    pub fn tenant_filter_expression(&self) -> String {
        "tenant_id = :tenant_id".to_string()
    }
    
    /// Create tenant expression attribute values for DynamoDB operations
    pub fn tenant_expression_attribute_values(&self) -> HashMap<String, AttributeValue> {
        let mut values = HashMap::new();
        values.insert(
            ":tenant_id".to_string(),
            AttributeValue::S(self.tenant_context.tenant_id.clone()),
        );
        values
    }
    
    /// Validate that an item belongs to the current tenant
    pub fn validate_tenant_ownership(
        &self,
        item: &HashMap<String, AttributeValue>,
    ) -> Result<(), TenantError> {
        if let Some(tenant_id) = item.get("tenant_id") {
            if let Some(id) = tenant_id.as_s() {
                if id == &self.tenant_context.tenant_id {
                    return Ok(());
                }
            }
        }
        
        Err(TenantError::AccessDenied(
            "Item does not belong to the current tenant".to_string(),
        ))
    }
    
    /// Create a tenant-specific table name
    pub fn tenant_table_name(&self, base_table_name: &str) -> String {
        format!("{}-{}", base_table_name, self.tenant_context.tenant_id)
    }
}

/// Middleware for enforcing tenant isolation in repositories
pub struct TenantMiddleware<T> {
    /// The inner repository
    inner: T,
    /// The tenant manager
    tenant_manager: TenantManager,
}

impl<T> TenantMiddleware<T> {
    /// Create a new tenant middleware
    pub fn new(inner: T, tenant_manager: TenantManager) -> Self {
        Self {
            inner,
            tenant_manager,
        }
    }
    
    /// Get the inner repository
    pub fn inner(&self) -> &T {
        &self.inner
    }
    
    /// Get the tenant manager
    pub fn tenant_manager(&self) -> &TenantManager {
        &self.tenant_manager
    }
    
    /// Create a new tenant middleware with a different tenant context
    pub fn with_tenant(
        self,
        tenant_context: TenantContext,
    ) -> Result<Self, TenantError> {
        let tenant_manager = self.tenant_manager.with_tenant(tenant_context)?;
        
        Ok(Self {
            inner: self.inner,
            tenant_manager,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tenant_context_new() {
        let context = TenantContext::new("tenant1");
        
        assert_eq!(context.tenant_id, "tenant1");
        assert!(context.metadata.is_empty());
    }
    
    #[test]
    fn test_tenant_context_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());
        
        let context = TenantContext::with_metadata("tenant1", metadata.clone());
        
        assert_eq!(context.tenant_id, "tenant1");
        assert_eq!(context.metadata, metadata);
    }
    
    #[test]
    fn test_tenant_context_add_metadata() {
        let mut context = TenantContext::new("tenant1");
        context.add_metadata("key1", "value1");
        
        assert_eq!(context.get_metadata("key1"), Some(&"value1".to_string()));
    }
    
    #[test]
    fn test_tenant_context_validate() {
        let valid_context = TenantContext::new("tenant1");
        assert!(valid_context.validate().is_ok());
        
        let invalid_context = TenantContext::new("");
        assert!(invalid_context.validate().is_err());
    }
    
    #[test]
    fn test_tenant_manager_new() {
        let context = TenantContext::new("tenant1");
        let manager = TenantManager::new(context.clone()).unwrap();
        
        assert_eq!(manager.tenant_context(), &context);
    }
    
    #[test]
    fn test_tenant_manager_with_tenant() {
        let context1 = TenantContext::new("tenant1");
        let context2 = TenantContext::new("tenant2");
        
        let manager1 = TenantManager::new(context1).unwrap();
        let manager2 = manager1.with_tenant(context2.clone()).unwrap();
        
        assert_eq!(manager2.tenant_context(), &context2);
    }
    
    #[test]
    fn test_tenant_manager_add_tenant_id_to_item() {
        let context = TenantContext::new("tenant1");
        let manager = TenantManager::new(context).unwrap();
        
        let mut item = HashMap::new();
        item.insert(
            "id".to_string(),
            AttributeValue::S("123".to_string()),
        );
        
        manager.add_tenant_id_to_item(&mut item);
        
        assert_eq!(
            item.get("tenant_id").unwrap().as_s().unwrap(),
            "tenant1"
        );
    }
    
    #[test]
    fn test_tenant_manager_validate_tenant_ownership() {
        let context = TenantContext::new("tenant1");
        let manager = TenantManager::new(context).unwrap();
        
        let mut item = HashMap::new();
        item.insert(
            "id".to_string(),
            AttributeValue::S("123".to_string()),
        );
        item.insert(
            "tenant_id".to_string(),
            AttributeValue::S("tenant1".to_string()),
        );
        
        assert!(manager.validate_tenant_ownership(&item).is_ok());
        
        let mut wrong_tenant_item = HashMap::new();
        wrong_tenant_item.insert(
            "id".to_string(),
            AttributeValue::S("123".to_string()),
        );
        wrong_tenant_item.insert(
            "tenant_id".to_string(),
            AttributeValue::S("tenant2".to_string()),
        );
        
        assert!(manager.validate_tenant_ownership(&wrong_tenant_item).is_err());
    }
    
    #[test]
    fn test_tenant_manager_tenant_table_name() {
        let context = TenantContext::new("tenant1");
        let manager = TenantManager::new(context).unwrap();
        
        assert_eq!(
            manager.tenant_table_name("table"),
            "table-tenant1"
        );
    }
    
    struct MockRepository;
    
    #[test]
    fn test_tenant_middleware_new() {
        let context = TenantContext::new("tenant1");
        let manager = TenantManager::new(context).unwrap();
        
        let middleware = TenantMiddleware::new(MockRepository, manager.clone());
        
        assert_eq!(middleware.tenant_manager().tenant_context(), &context);
    }
    
    #[test]
    fn test_tenant_middleware_with_tenant() {
        let context1 = TenantContext::new("tenant1");
        let context2 = TenantContext::new("tenant2");
        
        let manager = TenantManager::new(context1).unwrap();
        let middleware1 = TenantMiddleware::new(MockRepository, manager);
        
        let middleware2 = middleware1.with_tenant(context2.clone()).unwrap();
        
        assert_eq!(middleware2.tenant_manager().tenant_context(), &context2);
    }
} 