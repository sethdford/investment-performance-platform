use std::fmt;

/// Tenant context for multi-tenancy support
#[derive(Debug, Clone)]
pub struct TenantContext {
    /// Tenant ID
    pub tenant_id: String,
    /// User ID
    pub user_id: Option<String>,
}

impl TenantContext {
    /// Create a new tenant context
    pub fn new(tenant_id: String, user_id: Option<String>) -> Self {
        Self { tenant_id, user_id }
    }
    
    /// Validate the tenant context
    pub fn validate(&self) -> Result<(), TenantError> {
        if self.tenant_id.is_empty() {
            return Err(TenantError::InvalidTenantId("Tenant ID cannot be empty".to_string()));
        }
        
        Ok(())
    }
}

/// Tenant error
#[derive(Debug)]
pub enum TenantError {
    /// Invalid tenant ID
    InvalidTenantId(String),
    /// Tenant access denied
    AccessDenied(String),
    /// Tenant not found
    NotFound(String),
}

impl fmt::Display for TenantError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TenantError::InvalidTenantId(msg) => write!(f, "Invalid tenant ID: {}", msg),
            TenantError::AccessDenied(msg) => write!(f, "Tenant access denied: {}", msg),
            TenantError::NotFound(msg) => write!(f, "Tenant not found: {}", msg),
        }
    }
}

impl std::error::Error for TenantError {}

/// Tenant manager for handling tenant operations
pub struct TenantManager {
    /// Current tenant context
    context: TenantContext,
}

impl TenantManager {
    /// Create a new tenant manager
    pub fn new(context: TenantContext) -> Result<Self, TenantError> {
        context.validate()?;
        Ok(Self { context })
    }
    
    /// Get the current tenant ID
    pub fn tenant_id(&self) -> &str {
        &self.context.tenant_id
    }
    
    /// Validate that an item belongs to the current tenant
    pub fn validate_tenant_ownership<T>(&self, item: &T) -> Result<(), TenantError>
    where
        T: TenantOwned,
    {
        if !item.belongs_to_tenant(&self.context.tenant_id) {
            return Err(TenantError::AccessDenied(
                "Item does not belong to the current tenant".to_string()
            ));
        }
        
        Ok(())
    }
}

/// Trait for items that belong to a tenant
pub trait TenantOwned {
    /// Check if the item belongs to a tenant
    fn belongs_to_tenant(&self, tenant_id: &str) -> bool;
    
    /// Get the tenant ID of the item
    fn get_tenant_id(&self) -> Option<&str>;
} 