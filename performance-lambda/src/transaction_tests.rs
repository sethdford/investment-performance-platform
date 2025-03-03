use crate::tenant::{TenantContext, TenantManager, TenantMiddleware, TenantAware, TenantError};
use crate::transactions::{TransactionManager, TransactionOperation, TransactionError};
use aws_sdk_dynamodb::{Client as DynamoDbClient, model::AttributeValue};
use std::collections::HashMap;
use std::error::Error;
use aws_config::SdkConfig;

// Mock DynamoDB client for testing
#[derive(Debug, Clone)]
struct MockDynamoDbClient {
    tenant_id: String,
    items: HashMap<String, HashMap<String, AttributeValue>>,
}

impl MockDynamoDbClient {
    fn new(tenant_id: impl Into<String>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            items: HashMap::new(),
        }
    }
    
    fn add_item(&mut self, key: String, item: HashMap<String, AttributeValue>) {
        self.items.insert(key, item);
    }
    
    fn get_item(&self, key: &str) -> Option<&HashMap<String, AttributeValue>> {
        self.items.get(key)
    }
    
    fn execute_transaction(
        &mut self,
        operations: Vec<TransactionOperation>,
        tenant_id: &str,
    ) -> Result<(), TransactionError> {
        // Verify that all operations have the correct tenant ID
        for op in &operations {
            match op {
                TransactionOperation::Put { item, .. } => {
                    if let Some(op_tenant_id) = item.get("tenant_id").and_then(|v| v.as_s().ok()) {
                        if op_tenant_id != tenant_id {
                            return Err(TransactionError::ValidationError(
                                format!("Tenant ID mismatch: expected {}, got {}", tenant_id, op_tenant_id)
                            ));
                        }
                    } else {
                        return Err(TransactionError::ValidationError(
                            "Missing tenant ID in item".to_string()
                        ));
                    }
                },
                TransactionOperation::Delete { key, .. } => {
                    if let Some(op_tenant_id) = key.get("tenant_id").and_then(|v| v.as_s().ok()) {
                        if op_tenant_id != tenant_id {
                            return Err(TransactionError::ValidationError(
                                format!("Tenant ID mismatch: expected {}, got {}", tenant_id, op_tenant_id)
                            ));
                        }
                    } else {
                        return Err(TransactionError::ValidationError(
                            "Missing tenant ID in key".to_string()
                        ));
                    }
                },
                TransactionOperation::Update { key, .. } => {
                    if let Some(op_tenant_id) = key.get("tenant_id").and_then(|v| v.as_s().ok()) {
                        if op_tenant_id != tenant_id {
                            return Err(TransactionError::ValidationError(
                                format!("Tenant ID mismatch: expected {}, got {}", tenant_id, op_tenant_id)
                            ));
                        }
                    } else {
                        return Err(TransactionError::ValidationError(
                            "Missing tenant ID in key".to_string()
                        ));
                    }
                },
                TransactionOperation::ConditionCheck { key, .. } => {
                    if let Some(op_tenant_id) = key.get("tenant_id").and_then(|v| v.as_s().ok()) {
                        if op_tenant_id != tenant_id {
                            return Err(TransactionError::ValidationError(
                                format!("Tenant ID mismatch: expected {}, got {}", tenant_id, op_tenant_id)
                            ));
                        }
                    } else {
                        return Err(TransactionError::ValidationError(
                            "Missing tenant ID in key".to_string()
                        ));
                    }
                },
            }
        }
        
        // Execute the operations
        for op in operations {
            match op {
                TransactionOperation::Put { table_name, item, .. } => {
                    let id = item.get("id").and_then(|v| v.as_s().ok()).unwrap_or_default();
                    let key = format!("{}:{}", table_name, id);
                    self.add_item(key, item);
                },
                TransactionOperation::Delete { table_name, key, .. } => {
                    let id = key.get("id").and_then(|v| v.as_s().ok()).unwrap_or_default();
                    let item_key = format!("{}:{}", table_name, id);
                    self.items.remove(&item_key);
                },
                TransactionOperation::Update { table_name, key, .. } => {
                    let id = key.get("id").and_then(|v| v.as_s().ok()).unwrap_or_default();
                    let item_key = format!("{}:{}", table_name, id);
                    
                    if let Some(item) = self.items.get_mut(&item_key) {
                        // In a real implementation, we would apply the update expression
                        // For testing, we just mark the item as updated
                        item.insert(
                            "updated".to_string(),
                            AttributeValue::Bool(true),
                        );
                    }
                },
                TransactionOperation::ConditionCheck { .. } => {
                    // In a real implementation, we would check the condition
                    // For testing, we just assume the condition is met
                },
            }
        }
        
        Ok(())
    }
}

// Create a mock DynamoDB client for testing
fn create_mock_client() -> DynamoDbClient {
    let config = aws_config::from_env().region("us-east-1").build();
    DynamoDbClient::new(&config)
}

#[tokio::test]
async fn test_transaction_manager_tenant_isolation() {
    // Create tenant contexts
    let tenant1_context = TenantContext::new("tenant1");
    let tenant2_context = TenantContext::new("tenant2");
    
    // Create transaction managers
    let client = create_mock_client();
    let transaction_manager1 = TransactionManager::new(client.clone(), tenant1_context.clone());
    let transaction_manager2 = TransactionManager::new(client.clone(), tenant2_context.clone());
    
    // Create a Put operation for tenant1
    let mut item1 = HashMap::new();
    item1.insert("id".to_string(), AttributeValue::S("item1".to_string()));
    
    let put_op1 = TransactionOperation::put("table", item1.clone());
    
    // Create a Put operation for tenant2
    let mut item2 = HashMap::new();
    item2.insert("id".to_string(), AttributeValue::S("item2".to_string()));
    
    let put_op2 = TransactionOperation::put("table", item2.clone());
    
    // Add tenant IDs to operations
    let tenant1_operations = transaction_manager1.add_tenant_to_operations(vec![put_op1]).unwrap();
    let tenant2_operations = transaction_manager2.add_tenant_to_operations(vec![put_op2]).unwrap();
    
    // Verify that tenant IDs were added correctly
    let tenant1_op = &tenant1_operations[0];
    let tenant2_op = &tenant2_operations[0];
    
    match tenant1_op {
        TransactionOperation::Put { item, .. } => {
            assert_eq!(
                item.get("tenant_id").unwrap().as_s().unwrap(),
                "tenant1"
            );
        },
        _ => panic!("Expected Put operation"),
    }
    
    match tenant2_op {
        TransactionOperation::Put { item, .. } => {
            assert_eq!(
                item.get("tenant_id").unwrap().as_s().unwrap(),
                "tenant2"
            );
        },
        _ => panic!("Expected Put operation"),
    }
    
    // Create a mock DynamoDB client
    let mut mock_client = MockDynamoDbClient::new("tenant1");
    
    // Execute the transaction for tenant1
    mock_client.execute_transaction(tenant1_operations, "tenant1").unwrap();
    
    // Verify that the item was added
    let item = mock_client.get_item("table:item1").unwrap();
    assert_eq!(
        item.get("tenant_id").unwrap().as_s().unwrap(),
        "tenant1"
    );
    
    // Try to execute tenant2's transaction with tenant1's client
    // This should fail due to tenant isolation
    let result = mock_client.execute_transaction(tenant2_operations, "tenant1");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_transaction_manager_with_tenant() {
    // Create tenant contexts
    let tenant1_context = TenantContext::new("tenant1");
    let tenant2_context = TenantContext::new("tenant2");
    
    // Create a transaction manager for tenant1
    let client = create_mock_client();
    let transaction_manager1 = TransactionManager::new(client.clone(), tenant1_context.clone());
    
    // Switch to tenant2
    let transaction_manager2 = transaction_manager1.with_tenant(tenant2_context.clone());
    
    // Verify that the tenant context was updated
    assert_eq!(
        transaction_manager2.tenant_context().tenant_id,
        "tenant2"
    );
    
    // Create a Put operation
    let mut item = HashMap::new();
    item.insert("id".to_string(), AttributeValue::S("item".to_string()));
    
    let put_op = TransactionOperation::put("table", item.clone());
    
    // Add tenant ID to the operation using tenant2's manager
    let tenant2_operations = transaction_manager2.add_tenant_to_operations(vec![put_op]).unwrap();
    
    // Verify that tenant2's ID was added
    let tenant2_op = &tenant2_operations[0];
    
    match tenant2_op {
        TransactionOperation::Put { item, .. } => {
            assert_eq!(
                item.get("tenant_id").unwrap().as_s().unwrap(),
                "tenant2"
            );
        },
        _ => panic!("Expected Put operation"),
    }
}

#[tokio::test]
async fn test_transaction_manager_add_tenant_condition() {
    // Create a tenant context
    let tenant_context = TenantContext::new("tenant1");
    
    // Create a transaction manager
    let client = create_mock_client();
    let transaction_manager = TransactionManager::new(client, tenant_context);
    
    // Test with no existing condition
    let (condition, values) = transaction_manager.add_tenant_condition(None, None);
    
    assert_eq!(condition, "tenant_id = :tenant_id");
    assert_eq!(
        values.get(":tenant_id").unwrap().as_s().unwrap(),
        "tenant1"
    );
    
    // Test with an existing condition
    let (condition, values) = transaction_manager.add_tenant_condition(
        Some("attribute_exists(id)".to_string()),
        None,
    );
    
    assert_eq!(condition, "(attribute_exists(id)) AND tenant_id = :tenant_id");
    assert_eq!(
        values.get(":tenant_id").unwrap().as_s().unwrap(),
        "tenant1"
    );
    
    // Test with existing expression attribute values
    let mut expr_attr_values = HashMap::new();
    expr_attr_values.insert(":val".to_string(), AttributeValue::S("value".to_string()));
    
    let (condition, values) = transaction_manager.add_tenant_condition(
        Some("attribute_exists(id) AND #attr = :val".to_string()),
        Some(expr_attr_values),
    );
    
    assert_eq!(condition, "(attribute_exists(id) AND #attr = :val) AND tenant_id = :tenant_id");
    assert_eq!(
        values.get(":tenant_id").unwrap().as_s().unwrap(),
        "tenant1"
    );
    assert_eq!(
        values.get(":val").unwrap().as_s().unwrap(),
        "value"
    );
}

#[tokio::test]
async fn test_transaction_manager_with_middleware() {
    // Create tenant contexts
    let tenant1_context = TenantContext::new("tenant1");
    let tenant2_context = TenantContext::new("tenant2");
    
    // Create tenant managers
    let tenant1_manager = TenantManager::new(tenant1_context.clone()).unwrap();
    let tenant2_manager = TenantManager::new(tenant2_context.clone()).unwrap();
    
    // Create transaction managers
    let client = create_mock_client();
    let transaction_manager1 = TransactionManager::new(client.clone(), tenant1_context);
    let transaction_manager2 = TransactionManager::new(client.clone(), tenant2_context);
    
    // Create tenant middlewares
    let middleware1 = TenantMiddleware::new(transaction_manager1, tenant1_manager);
    let middleware2 = TenantMiddleware::new(transaction_manager2, tenant2_manager);
    
    // Verify that the tenant contexts are correct
    assert_eq!(
        middleware1.tenant_manager().tenant_context().tenant_id,
        "tenant1"
    );
    assert_eq!(
        middleware2.tenant_manager().tenant_context().tenant_id,
        "tenant2"
    );
    
    // Get the transaction managers from the middlewares
    let transaction_manager1 = middleware1.inner();
    let transaction_manager2 = middleware2.inner();
    
    // Create Put operations
    let mut item1 = HashMap::new();
    item1.insert("id".to_string(), AttributeValue::S("item1".to_string()));
    
    let mut item2 = HashMap::new();
    item2.insert("id".to_string(), AttributeValue::S("item2".to_string()));
    
    let put_op1 = TransactionOperation::put("table", item1.clone());
    let put_op2 = TransactionOperation::put("table", item2.clone());
    
    // Add tenant IDs to operations
    let tenant1_operations = transaction_manager1.add_tenant_to_operations(vec![put_op1]).unwrap();
    let tenant2_operations = transaction_manager2.add_tenant_to_operations(vec![put_op2]).unwrap();
    
    // Verify that tenant IDs were added correctly
    let tenant1_op = &tenant1_operations[0];
    let tenant2_op = &tenant2_operations[0];
    
    match tenant1_op {
        TransactionOperation::Put { item, .. } => {
            assert_eq!(
                item.get("tenant_id").unwrap().as_s().unwrap(),
                "tenant1"
            );
        },
        _ => panic!("Expected Put operation"),
    }
    
    match tenant2_op {
        TransactionOperation::Put { item, .. } => {
            assert_eq!(
                item.get("tenant_id").unwrap().as_s().unwrap(),
                "tenant2"
            );
        },
        _ => panic!("Expected Put operation"),
    }
} 