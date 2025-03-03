use crate::tenant::{TenantContext, TenantManager, TenantMiddleware, TenantAware, TenantError};
use aws_sdk_dynamodb::model::AttributeValue;
use std::collections::HashMap;

// Mock repository for testing
#[derive(Debug, Clone)]
struct MockRepository {
    tenant_context: TenantContext,
    items: HashMap<String, HashMap<String, AttributeValue>>,
}

impl MockRepository {
    fn new(tenant_context: TenantContext) -> Self {
        Self {
            tenant_context,
            items: HashMap::new(),
        }
    }
    
    fn add_item(&mut self, id: &str, item: HashMap<String, AttributeValue>) {
        self.items.insert(id.to_string(), item);
    }
    
    fn get_item(&self, id: &str) -> Option<&HashMap<String, AttributeValue>> {
        self.items.get(id)
    }
}

impl TenantAware for MockRepository {
    fn tenant_context(&self) -> &TenantContext {
        &self.tenant_context
    }
    
    fn with_tenant(mut self, tenant_context: TenantContext) -> Self {
        self.tenant_context = tenant_context;
        self
    }
}

#[tokio::test]
async fn test_tenant_middleware_isolation() {
    // Create tenant contexts
    let tenant1_context = TenantContext::new("tenant1");
    let tenant2_context = TenantContext::new("tenant2");
    
    // Create tenant managers
    let tenant1_manager = TenantManager::new(tenant1_context.clone()).unwrap();
    let tenant2_manager = TenantManager::new(tenant2_context.clone()).unwrap();
    
    // Create mock repositories
    let mut repo1 = MockRepository::new(tenant1_context.clone());
    let mut repo2 = MockRepository::new(tenant2_context.clone());
    
    // Add items to repositories
    let mut item1 = HashMap::new();
    item1.insert("id".to_string(), AttributeValue::S("item1".to_string()));
    item1.insert("tenant_id".to_string(), AttributeValue::S("tenant1".to_string()));
    
    let mut item2 = HashMap::new();
    item2.insert("id".to_string(), AttributeValue::S("item2".to_string()));
    item2.insert("tenant_id".to_string(), AttributeValue::S("tenant2".to_string()));
    
    repo1.add_item("item1", item1.clone());
    repo2.add_item("item2", item2.clone());
    
    // Create tenant middlewares
    let middleware1 = TenantMiddleware::new(repo1, tenant1_manager);
    let middleware2 = TenantMiddleware::new(repo2, tenant2_manager);
    
    // Test tenant isolation
    assert_eq!(
        middleware1.tenant_manager().tenant_context().tenant_id,
        "tenant1"
    );
    assert_eq!(
        middleware2.tenant_manager().tenant_context().tenant_id,
        "tenant2"
    );
    
    // Test item access
    let repo1 = middleware1.inner();
    let repo2 = middleware2.inner();
    
    let item1_from_repo1 = repo1.get_item("item1").unwrap();
    assert_eq!(
        item1_from_repo1.get("tenant_id").unwrap().as_s().unwrap(),
        "tenant1"
    );
    
    let item2_from_repo2 = repo2.get_item("item2").unwrap();
    assert_eq!(
        item2_from_repo2.get("tenant_id").unwrap().as_s().unwrap(),
        "tenant2"
    );
    
    // Test tenant validation
    assert!(middleware1.tenant_manager().validate_tenant_ownership(&item1).is_ok());
    assert!(middleware1.tenant_manager().validate_tenant_ownership(&item2).is_err());
    assert!(middleware2.tenant_manager().validate_tenant_ownership(&item2).is_ok());
    assert!(middleware2.tenant_manager().validate_tenant_ownership(&item1).is_err());
}

#[tokio::test]
async fn test_tenant_manager_table_name() {
    // Create tenant contexts
    let tenant1_context = TenantContext::new("tenant1");
    let tenant2_context = TenantContext::new("tenant2");
    
    // Create tenant managers
    let tenant1_manager = TenantManager::new(tenant1_context).unwrap();
    let tenant2_manager = TenantManager::new(tenant2_context).unwrap();
    
    // Test tenant-specific table names
    assert_eq!(
        tenant1_manager.tenant_table_name("performance"),
        "performance-tenant1"
    );
    assert_eq!(
        tenant2_manager.tenant_table_name("performance"),
        "performance-tenant2"
    );
}

#[tokio::test]
async fn test_tenant_context_metadata() {
    // Create tenant context with metadata
    let mut tenant_context = TenantContext::new("tenant1");
    tenant_context.add_metadata("region", "us-west-2");
    tenant_context.add_metadata("environment", "production");
    
    // Test metadata access
    assert_eq!(
        tenant_context.get_metadata("region"),
        Some(&"us-west-2".to_string())
    );
    assert_eq!(
        tenant_context.get_metadata("environment"),
        Some(&"production".to_string())
    );
    assert_eq!(tenant_context.get_metadata("non_existent"), None);
}

#[tokio::test]
async fn test_tenant_middleware_with_tenant() {
    // Create tenant contexts
    let tenant1_context = TenantContext::new("tenant1");
    let tenant2_context = TenantContext::new("tenant2");
    
    // Create tenant manager
    let tenant1_manager = TenantManager::new(tenant1_context.clone()).unwrap();
    
    // Create mock repository
    let repo = MockRepository::new(tenant1_context);
    
    // Create tenant middleware
    let middleware1 = TenantMiddleware::new(repo, tenant1_manager);
    
    // Switch to tenant2
    let middleware2 = middleware1.with_tenant(tenant2_context.clone()).unwrap();
    
    // Test tenant context
    assert_eq!(
        middleware2.tenant_manager().tenant_context().tenant_id,
        "tenant2"
    );
}

#[tokio::test]
async fn test_tenant_validation() {
    // Test valid tenant
    let valid_context = TenantContext::new("tenant1");
    assert!(valid_context.validate().is_ok());
    
    // Test invalid tenant (empty ID)
    let invalid_context = TenantContext::new("");
    assert!(matches!(
        invalid_context.validate(),
        Err(TenantError::MissingTenantId)
    ));
}

#[tokio::test]
async fn test_tenant_expression_values() {
    // Create tenant context
    let tenant_context = TenantContext::new("tenant1");
    
    // Create tenant manager
    let tenant_manager = TenantManager::new(tenant_context).unwrap();
    
    // Test expression attribute values
    let values = tenant_manager.tenant_expression_attribute_values();
    assert_eq!(values.len(), 1);
    assert_eq!(
        values.get(":tenant_id").unwrap().as_s().unwrap(),
        "tenant1"
    );
}

#[tokio::test]
async fn test_tenant_condition_expressions() {
    // Create tenant context
    let tenant_context = TenantContext::new("tenant1");
    
    // Create tenant manager
    let tenant_manager = TenantManager::new(tenant_context).unwrap();
    
    // Test condition expressions
    assert_eq!(
        tenant_manager.tenant_key_condition_expression(),
        "tenant_id = :tenant_id"
    );
    assert_eq!(
        tenant_manager.tenant_filter_expression(),
        "tenant_id = :tenant_id"
    );
}

#[tokio::test]
async fn test_tenant_add_to_item() {
    // Create tenant context
    let tenant_context = TenantContext::new("tenant1");
    
    // Create tenant manager
    let tenant_manager = TenantManager::new(tenant_context).unwrap();
    
    // Test adding tenant ID to item
    let mut item = HashMap::new();
    item.insert(
        "id".to_string(),
        AttributeValue::S("123".to_string()),
    );
    
    tenant_manager.add_tenant_id_to_item(&mut item);
    
    assert_eq!(
        item.get("tenant_id").unwrap().as_s().unwrap(),
        "tenant1"
    );
}

#[tokio::test]
async fn test_tenant_add_to_key() {
    // Create tenant context
    let tenant_context = TenantContext::new("tenant1");
    
    // Create tenant manager
    let tenant_manager = TenantManager::new(tenant_context).unwrap();
    
    // Test adding tenant ID to key
    let mut key = HashMap::new();
    key.insert(
        "id".to_string(),
        AttributeValue::S("123".to_string()),
    );
    
    tenant_manager.add_tenant_id_to_key(&mut key);
    
    assert_eq!(
        key.get("tenant_id").unwrap().as_s().unwrap(),
        "tenant1"
    );
} 