use aws_sdk_dynamodb::{
    model::{
        AttributeValue, 
        TransactWriteItem, 
        Put, 
        Delete, 
        Update, 
        ConditionCheck,
        ReturnValuesOnConditionCheckFailure,
    },
    Client as DynamoDbClient,
    Error as DynamoDbError,
};
use std::collections::HashMap;
use std::fmt;
use std::error::Error;
use tracing::{debug, error, info, warn};
use crate::tenant::{TenantContext, TenantAware, TenantError};

/// Represents errors that can occur during transaction operations
#[derive(Debug)]
pub enum TransactionError {
    /// Transaction was cancelled due to a condition check failure
    ConditionCheckFailed {
        item: Option<HashMap<String, AttributeValue>>,
        message: String,
    },
    /// Transaction was cancelled due to a conflict with another transaction
    TransactionConflict(String),
    /// Transaction was cancelled due to exceeding the provisioned throughput
    ProvisionedThroughputExceeded(String),
    /// Transaction was cancelled due to an item collection size limit exceeded
    ItemCollectionSizeLimitExceeded {
        table_name: String,
        message: String,
    },
    /// Transaction was cancelled due to a validation error
    ValidationError(String),
    /// Transaction was cancelled due to an internal server error
    InternalServerError(String),
    /// Transaction was cancelled due to a service error
    ServiceError(String),
    /// Transaction was cancelled due to a client error
    ClientError(String),
    /// Transaction was cancelled due to an unknown error
    Unknown(String),
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionError::ConditionCheckFailed { message, .. } => {
                write!(f, "Condition check failed: {}", message)
            }
            TransactionError::TransactionConflict(message) => {
                write!(f, "Transaction conflict: {}", message)
            }
            TransactionError::ProvisionedThroughputExceeded(message) => {
                write!(f, "Provisioned throughput exceeded: {}", message)
            }
            TransactionError::ItemCollectionSizeLimitExceeded { table_name, message } => {
                write!(
                    f,
                    "Item collection size limit exceeded for table {}: {}",
                    table_name, message
                )
            }
            TransactionError::ValidationError(message) => {
                write!(f, "Validation error: {}", message)
            }
            TransactionError::InternalServerError(message) => {
                write!(f, "Internal server error: {}", message)
            }
            TransactionError::ServiceError(message) => {
                write!(f, "Service error: {}", message)
            }
            TransactionError::ClientError(message) => {
                write!(f, "Client error: {}", message)
            }
            TransactionError::Unknown(message) => {
                write!(f, "Unknown error: {}", message)
            }
        }
    }
}

impl Error for TransactionError {}

impl From<DynamoDbError> for TransactionError {
    fn from(err: DynamoDbError) -> Self {
        match &err {
            DynamoDbError::TransactionCanceledException(e) => {
                if let Some(reasons) = &e.cancellation_reasons {
                    // Check if any condition check failed
                    for reason in reasons {
                        if let Some(code) = reason.code.as_ref() {
                            match code.as_str() {
                                "ConditionalCheckFailed" => {
                                    let item = reason
                                        .item
                                        .clone()
                                        .map(|i| i.into_iter().collect());
                                    
                                    return TransactionError::ConditionCheckFailed {
                                        item,
                                        message: reason.message.clone().unwrap_or_default(),
                                    };
                                }
                                "TransactionConflict" => {
                                    return TransactionError::TransactionConflict(
                                        reason.message.clone().unwrap_or_default(),
                                    );
                                }
                                "ProvisionedThroughputExceeded" => {
                                    return TransactionError::ProvisionedThroughputExceeded(
                                        reason.message.clone().unwrap_or_default(),
                                    );
                                }
                                "ItemCollectionSizeLimitExceeded" => {
                                    return TransactionError::ItemCollectionSizeLimitExceeded {
                                        table_name: reason
                                            .table_name
                                            .clone()
                                            .unwrap_or_default(),
                                        message: reason.message.clone().unwrap_or_default(),
                                    };
                                }
                                "ValidationError" => {
                                    return TransactionError::ValidationError(
                                        reason.message.clone().unwrap_or_default(),
                                    );
                                }
                                "InternalServerError" => {
                                    return TransactionError::InternalServerError(
                                        reason.message.clone().unwrap_or_default(),
                                    );
                                }
                                _ => {
                                    return TransactionError::Unknown(format!(
                                        "Unknown cancellation reason: {}",
                                        code
                                    ));
                                }
                            }
                        }
                    }
                }
                
                TransactionError::Unknown(format!("Transaction cancelled: {:?}", err))
            }
            _ => TransactionError::ServiceError(format!("{:?}", err)),
        }
    }
}

/// Represents a transaction operation
#[derive(Debug)]
pub enum TransactionOperation {
    /// Put operation
    Put {
        table_name: String,
        item: HashMap<String, AttributeValue>,
        condition_expression: Option<String>,
        expression_attribute_names: Option<HashMap<String, String>>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    },
    /// Delete operation
    Delete {
        table_name: String,
        key: HashMap<String, AttributeValue>,
        condition_expression: Option<String>,
        expression_attribute_names: Option<HashMap<String, String>>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    },
    /// Update operation
    Update {
        table_name: String,
        key: HashMap<String, AttributeValue>,
        update_expression: String,
        condition_expression: Option<String>,
        expression_attribute_names: Option<HashMap<String, String>>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    },
    /// Condition check operation
    ConditionCheck {
        table_name: String,
        key: HashMap<String, AttributeValue>,
        condition_expression: String,
        expression_attribute_names: Option<HashMap<String, String>>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    },
}

impl TransactionOperation {
    /// Create a new put operation
    pub fn put(
        table_name: impl Into<String>,
        item: HashMap<String, AttributeValue>,
    ) -> Self {
        TransactionOperation::Put {
            table_name: table_name.into(),
            item,
            condition_expression: None,
            expression_attribute_names: None,
            expression_attribute_values: None,
        }
    }
    
    /// Create a new put operation with a condition expression
    pub fn put_with_condition(
        table_name: impl Into<String>,
        item: HashMap<String, AttributeValue>,
        condition_expression: impl Into<String>,
        expression_attribute_names: Option<HashMap<String, String>>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    ) -> Self {
        TransactionOperation::Put {
            table_name: table_name.into(),
            item,
            condition_expression: Some(condition_expression.into()),
            expression_attribute_names,
            expression_attribute_values,
        }
    }
    
    /// Create a new delete operation
    pub fn delete(
        table_name: impl Into<String>,
        key: HashMap<String, AttributeValue>,
    ) -> Self {
        TransactionOperation::Delete {
            table_name: table_name.into(),
            key,
            condition_expression: None,
            expression_attribute_names: None,
            expression_attribute_values: None,
        }
    }
    
    /// Create a new delete operation with a condition expression
    pub fn delete_with_condition(
        table_name: impl Into<String>,
        key: HashMap<String, AttributeValue>,
        condition_expression: impl Into<String>,
        expression_attribute_names: Option<HashMap<String, String>>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    ) -> Self {
        TransactionOperation::Delete {
            table_name: table_name.into(),
            key,
            condition_expression: Some(condition_expression.into()),
            expression_attribute_names,
            expression_attribute_values,
        }
    }
    
    /// Create a new update operation
    pub fn update(
        table_name: impl Into<String>,
        key: HashMap<String, AttributeValue>,
        update_expression: impl Into<String>,
        expression_attribute_names: Option<HashMap<String, String>>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    ) -> Self {
        TransactionOperation::Update {
            table_name: table_name.into(),
            key,
            update_expression: update_expression.into(),
            condition_expression: None,
            expression_attribute_names,
            expression_attribute_values,
        }
    }
    
    /// Create a new update operation with a condition expression
    pub fn update_with_condition(
        table_name: impl Into<String>,
        key: HashMap<String, AttributeValue>,
        update_expression: impl Into<String>,
        condition_expression: impl Into<String>,
        expression_attribute_names: Option<HashMap<String, String>>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    ) -> Self {
        TransactionOperation::Update {
            table_name: table_name.into(),
            key,
            update_expression: update_expression.into(),
            condition_expression: Some(condition_expression.into()),
            expression_attribute_names,
            expression_attribute_values,
        }
    }
    
    /// Create a new condition check operation
    pub fn condition_check(
        table_name: impl Into<String>,
        key: HashMap<String, AttributeValue>,
        condition_expression: impl Into<String>,
        expression_attribute_names: Option<HashMap<String, String>>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    ) -> Self {
        TransactionOperation::ConditionCheck {
            table_name: table_name.into(),
            key,
            condition_expression: condition_expression.into(),
            expression_attribute_names,
            expression_attribute_values,
        }
    }
    
    /// Convert to a TransactWriteItem
    fn to_transact_write_item(&self) -> TransactWriteItem {
        match self {
            TransactionOperation::Put {
                table_name,
                item,
                condition_expression,
                expression_attribute_names,
                expression_attribute_values,
            } => {
                let mut put = Put::builder()
                    .table_name(table_name)
                    .set_item(Some(item.clone()));
                
                if let Some(condition) = condition_expression {
                    put = put.condition_expression(condition);
                }
                
                if let Some(names) = expression_attribute_names {
                    put = put.set_expression_attribute_names(Some(names.clone()));
                }
                
                if let Some(values) = expression_attribute_values {
                    put = put.set_expression_attribute_values(Some(values.clone()));
                }
                
                TransactWriteItem::builder()
                    .put(put.build())
                    .build()
            }
            TransactionOperation::Delete {
                table_name,
                key,
                condition_expression,
                expression_attribute_names,
                expression_attribute_values,
            } => {
                let mut delete = Delete::builder()
                    .table_name(table_name)
                    .set_key(Some(key.clone()));
                
                if let Some(condition) = condition_expression {
                    delete = delete.condition_expression(condition);
                }
                
                if let Some(names) = expression_attribute_names {
                    delete = delete.set_expression_attribute_names(Some(names.clone()));
                }
                
                if let Some(values) = expression_attribute_values {
                    delete = delete.set_expression_attribute_values(Some(values.clone()));
                }
                
                TransactWriteItem::builder()
                    .delete(delete.build())
                    .build()
            }
            TransactionOperation::Update {
                table_name,
                key,
                update_expression,
                condition_expression,
                expression_attribute_names,
                expression_attribute_values,
            } => {
                let mut update = Update::builder()
                    .table_name(table_name)
                    .set_key(Some(key.clone()))
                    .update_expression(update_expression);
                
                if let Some(condition) = condition_expression {
                    update = update.condition_expression(condition);
                }
                
                if let Some(names) = expression_attribute_names {
                    update = update.set_expression_attribute_names(Some(names.clone()));
                }
                
                if let Some(values) = expression_attribute_values {
                    update = update.set_expression_attribute_values(Some(values.clone()));
                }
                
                TransactWriteItem::builder()
                    .update(update.build())
                    .build()
            }
            TransactionOperation::ConditionCheck {
                table_name,
                key,
                condition_expression,
                expression_attribute_names,
                expression_attribute_values,
            } => {
                let mut check = ConditionCheck::builder()
                    .table_name(table_name)
                    .set_key(Some(key.clone()))
                    .condition_expression(condition_expression)
                    .return_values_on_condition_check_failure(ReturnValuesOnConditionCheckFailure::All);
                
                if let Some(names) = expression_attribute_names {
                    check = check.set_expression_attribute_names(Some(names.clone()));
                }
                
                if let Some(values) = expression_attribute_values {
                    check = check.set_expression_attribute_values(Some(values.clone()));
                }
                
                TransactWriteItem::builder()
                    .condition_check(check.build())
                    .build()
            }
        }
    }
}

/// Transaction manager for handling transaction operations
#[derive(Debug, Clone)]
pub struct TransactionManager {
    client: DynamoDbClient,
    tenant_context: TenantContext,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new(client: DynamoDbClient, tenant_context: TenantContext) -> Self {
        Self { 
            client,
            tenant_context,
        }
    }
    
    /// Execute a transaction
    pub async fn execute_transaction(
        &self,
        operations: Vec<TransactionOperation>,
    ) -> Result<(), TransactionError> {
        // Add tenant validation to each operation
        let tenant_operations = self.add_tenant_to_operations(operations)?;
        
        // Convert operations to TransactWriteItems
        let transact_items: Vec<TransactWriteItem> = tenant_operations
            .iter()
            .map(|op| op.to_transact_write_item())
            .collect();
        
        debug!("Executing transaction with {} operations", transact_items.len());
        
        // Execute the transaction
        self.client
            .transact_write_items()
            .set_transact_items(Some(transact_items))
            .send()
            .await
            .map_err(|e| e.into())?;
        
        Ok(())
    }
    
    /// Execute a transaction with a client request token for idempotency
    pub async fn execute_transaction_with_token(
        &self,
        operations: Vec<TransactionOperation>,
        client_request_token: impl Into<String>,
    ) -> Result<(), TransactionError> {
        // Add tenant validation to each operation
        let tenant_operations = self.add_tenant_to_operations(operations)?;
        
        // Convert operations to TransactWriteItems
        let transact_items: Vec<TransactWriteItem> = tenant_operations
            .iter()
            .map(|op| op.to_transact_write_item())
            .collect();
        
        debug!(
            "Executing transaction with {} operations and token {}",
            transact_items.len(),
            client_request_token.into()
        );
        
        // Execute the transaction with the client request token
        self.client
            .transact_write_items()
            .set_transact_items(Some(transact_items))
            .client_request_token(client_request_token)
            .send()
            .await
            .map_err(|e| e.into())?;
        
        Ok(())
    }
    
    /// Add tenant ID to transaction operations
    fn add_tenant_to_operations(
        &self,
        operations: Vec<TransactionOperation>,
    ) -> Result<Vec<TransactionOperation>, TransactionError> {
        let mut tenant_operations = Vec::with_capacity(operations.len());
        
        for op in operations {
            let tenant_op = match op {
                TransactionOperation::Put {
                    table_name,
                    mut item,
                    condition_expression,
                    expression_attribute_names,
                    expression_attribute_values,
                } => {
                    // Add tenant ID to the item
                    item.insert(
                        "tenant_id".to_string(),
                        AttributeValue::S(self.tenant_context.tenant_id.clone()),
                    );
                    
                    // Add tenant condition if not already present
                    let (cond_expr, expr_attr_values) = self.add_tenant_condition(
                        condition_expression,
                        expression_attribute_values,
                    );
                    
                    TransactionOperation::Put {
                        table_name,
                        item,
                        condition_expression: Some(cond_expr),
                        expression_attribute_names,
                        expression_attribute_values: Some(expr_attr_values),
                    }
                },
                TransactionOperation::Delete {
                    table_name,
                    mut key,
                    condition_expression,
                    expression_attribute_names,
                    expression_attribute_values,
                } => {
                    // Add tenant ID to the key
                    key.insert(
                        "tenant_id".to_string(),
                        AttributeValue::S(self.tenant_context.tenant_id.clone()),
                    );
                    
                    // Add tenant condition if not already present
                    let (cond_expr, expr_attr_values) = self.add_tenant_condition(
                        condition_expression,
                        expression_attribute_values,
                    );
                    
                    TransactionOperation::Delete {
                        table_name,
                        key,
                        condition_expression: Some(cond_expr),
                        expression_attribute_names,
                        expression_attribute_values: Some(expr_attr_values),
                    }
                },
                TransactionOperation::Update {
                    table_name,
                    mut key,
                    update_expression,
                    condition_expression,
                    expression_attribute_names,
                    expression_attribute_values,
                } => {
                    // Add tenant ID to the key
                    key.insert(
                        "tenant_id".to_string(),
                        AttributeValue::S(self.tenant_context.tenant_id.clone()),
                    );
                    
                    // Add tenant condition if not already present
                    let (cond_expr, expr_attr_values) = self.add_tenant_condition(
                        condition_expression,
                        expression_attribute_values,
                    );
                    
                    TransactionOperation::Update {
                        table_name,
                        key,
                        update_expression,
                        condition_expression: Some(cond_expr),
                        expression_attribute_names,
                        expression_attribute_values: Some(expr_attr_values),
                    }
                },
                TransactionOperation::ConditionCheck {
                    table_name,
                    mut key,
                    condition_expression,
                    expression_attribute_names,
                    expression_attribute_values,
                } => {
                    // Add tenant ID to the key
                    key.insert(
                        "tenant_id".to_string(),
                        AttributeValue::S(self.tenant_context.tenant_id.clone()),
                    );
                    
                    // Add tenant condition if not already present
                    let (cond_expr, expr_attr_values) = self.add_tenant_condition(
                        Some(condition_expression),
                        expression_attribute_values,
                    );
                    
                    TransactionOperation::ConditionCheck {
                        table_name,
                        key,
                        condition_expression: cond_expr,
                        expression_attribute_names,
                        expression_attribute_values: Some(expr_attr_values),
                    }
                },
            };
            
            tenant_operations.push(tenant_op);
        }
        
        Ok(tenant_operations)
    }
    
    /// Add tenant condition to expression
    fn add_tenant_condition(
        &self,
        condition_expression: Option<String>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    ) -> (String, HashMap<String, AttributeValue>) {
        let tenant_condition = "tenant_id = :tenant_id";
        
        // Create or update the condition expression
        let condition = match condition_expression {
            Some(expr) => format!("({}) AND {}", expr, tenant_condition),
            None => tenant_condition.to_string(),
        };
        
        // Create or update the expression attribute values
        let mut values = expression_attribute_values.unwrap_or_default();
        values.insert(
            ":tenant_id".to_string(),
            AttributeValue::S(self.tenant_context.tenant_id.clone()),
        );
        
        (condition, values)
    }
}

impl TenantAware for TransactionManager {
    fn tenant_context(&self) -> &TenantContext {
        &self.tenant_context
    }
    
    fn with_tenant(mut self, tenant_context: TenantContext) -> Self {
        self.tenant_context = tenant_context;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_dynamodb::model::AttributeValue;
    use std::collections::HashMap;
    
    #[test]
    fn test_transaction_operation_put() {
        let mut item = HashMap::new();
        item.insert("id".to_string(), AttributeValue::S("123".to_string()));
        
        let op = TransactionOperation::put("table", item.clone());
        
        match op {
            TransactionOperation::Put {
                table_name,
                item: op_item,
                condition_expression,
                expression_attribute_names,
                expression_attribute_values,
            } => {
                assert_eq!(table_name, "table");
                assert_eq!(op_item, item);
                assert_eq!(condition_expression, None);
                assert_eq!(expression_attribute_names, None);
                assert_eq!(expression_attribute_values, None);
            }
            _ => panic!("Expected Put operation"),
        }
    }
    
    #[test]
    fn test_transaction_operation_put_with_condition() {
        let mut item = HashMap::new();
        item.insert("id".to_string(), AttributeValue::S("123".to_string()));
        
        let mut expr_attr_values = HashMap::new();
        expr_attr_values.insert(":val".to_string(), AttributeValue::S("value".to_string()));
        
        let op = TransactionOperation::put_with_condition(
            "table",
            item.clone(),
            "attribute_not_exists(id)",
            None,
            Some(expr_attr_values.clone()),
        );
        
        match op {
            TransactionOperation::Put {
                table_name,
                item: op_item,
                condition_expression,
                expression_attribute_names,
                expression_attribute_values,
            } => {
                assert_eq!(table_name, "table");
                assert_eq!(op_item, item);
                assert_eq!(condition_expression, Some("attribute_not_exists(id)".to_string()));
                assert_eq!(expression_attribute_names, None);
                assert_eq!(expression_attribute_values, Some(expr_attr_values));
            }
            _ => panic!("Expected Put operation"),
        }
    }
    
    #[test]
    fn test_transaction_operation_delete() {
        let mut key = HashMap::new();
        key.insert("id".to_string(), AttributeValue::S("123".to_string()));
        
        let op = TransactionOperation::delete("table", key.clone());
        
        match op {
            TransactionOperation::Delete {
                table_name,
                key: op_key,
                condition_expression,
                expression_attribute_names,
                expression_attribute_values,
            } => {
                assert_eq!(table_name, "table");
                assert_eq!(op_key, key);
                assert_eq!(condition_expression, None);
                assert_eq!(expression_attribute_names, None);
                assert_eq!(expression_attribute_values, None);
            }
            _ => panic!("Expected Delete operation"),
        }
    }
    
    #[test]
    fn test_transaction_operation_update() {
        let mut key = HashMap::new();
        key.insert("id".to_string(), AttributeValue::S("123".to_string()));
        
        let mut expr_attr_values = HashMap::new();
        expr_attr_values.insert(":val".to_string(), AttributeValue::S("value".to_string()));
        
        let op = TransactionOperation::update(
            "table",
            key.clone(),
            "SET #attr = :val",
            None,
            Some(expr_attr_values.clone()),
        );
        
        match op {
            TransactionOperation::Update {
                table_name,
                key: op_key,
                update_expression,
                condition_expression,
                expression_attribute_names,
                expression_attribute_values,
            } => {
                assert_eq!(table_name, "table");
                assert_eq!(op_key, key);
                assert_eq!(update_expression, "SET #attr = :val");
                assert_eq!(condition_expression, None);
                assert_eq!(expression_attribute_names, None);
                assert_eq!(expression_attribute_values, Some(expr_attr_values));
            }
            _ => panic!("Expected Update operation"),
        }
    }
    
    #[test]
    fn test_transaction_operation_condition_check() {
        let mut key = HashMap::new();
        key.insert("id".to_string(), AttributeValue::S("123".to_string()));
        
        let op = TransactionOperation::condition_check(
            "table",
            key.clone(),
            "attribute_exists(id)",
            None,
            None,
        );
        
        match op {
            TransactionOperation::ConditionCheck {
                table_name,
                key: op_key,
                condition_expression,
                expression_attribute_names,
                expression_attribute_values,
            } => {
                assert_eq!(table_name, "table");
                assert_eq!(op_key, key);
                assert_eq!(condition_expression, "attribute_exists(id)");
                assert_eq!(expression_attribute_names, None);
                assert_eq!(expression_attribute_values, None);
            }
            _ => panic!("Expected ConditionCheck operation"),
        }
    }
    
    #[test]
    fn test_to_transact_write_item_put() {
        let mut item = HashMap::new();
        item.insert("id".to_string(), AttributeValue::S("123".to_string()));
        
        let op = TransactionOperation::put("table", item.clone());
        let transact_item = op.to_transact_write_item();
        
        assert!(transact_item.put().is_some());
        assert!(transact_item.delete().is_none());
        assert!(transact_item.update().is_none());
        assert!(transact_item.condition_check().is_none());
        
        let put = transact_item.put().unwrap();
        assert_eq!(put.table_name(), "table");
        assert_eq!(put.item().unwrap(), &item);
    }
    
    #[test]
    fn test_to_transact_write_item_delete() {
        let mut key = HashMap::new();
        key.insert("id".to_string(), AttributeValue::S("123".to_string()));
        
        let op = TransactionOperation::delete("table", key.clone());
        let transact_item = op.to_transact_write_item();
        
        assert!(transact_item.put().is_none());
        assert!(transact_item.delete().is_some());
        assert!(transact_item.update().is_none());
        assert!(transact_item.condition_check().is_none());
        
        let delete = transact_item.delete().unwrap();
        assert_eq!(delete.table_name(), "table");
        assert_eq!(delete.key().unwrap(), &key);
    }
    
    #[test]
    fn test_add_tenant_to_operations() {
        let client = DynamoDbClient::new(&aws_config::from_env().region("us-east-1").build());
        let tenant_context = TenantContext::new("tenant1");
        let transaction_manager = TransactionManager::new(client, tenant_context);
        
        // Create a Put operation
        let mut item = HashMap::new();
        item.insert("id".to_string(), AttributeValue::S("item1".to_string()));
        
        let put_op = TransactionOperation::put("table", item.clone());
        
        // Create a Delete operation
        let mut key = HashMap::new();
        key.insert("id".to_string(), AttributeValue::S("item1".to_string()));
        
        let delete_op = TransactionOperation::delete("table", key.clone());
        
        // Create an Update operation
        let update_op = TransactionOperation::update(
            "table",
            key.clone(),
            "SET #name = :name",
            Some(HashMap::from([("#name".to_string(), "name".to_string())])),
            Some(HashMap::from([(":name".to_string(), AttributeValue::S("New Name".to_string()))])),
        );
        
        // Create a ConditionCheck operation
        let condition_op = TransactionOperation::condition_check(
            "table",
            key.clone(),
            "attribute_exists(id)",
            None,
            None,
        );
        
        // Add tenant to operations
        let operations = vec![put_op, delete_op, update_op, condition_op];
        let tenant_operations = transaction_manager.add_tenant_to_operations(operations).unwrap();
        
        // Verify that tenant ID was added to each operation
        for op in tenant_operations {
            match op {
                TransactionOperation::Put { item, .. } => {
                    assert_eq!(
                        item.get("tenant_id").unwrap().as_s().unwrap(),
                        "tenant1"
                    );
                },
                TransactionOperation::Delete { key, .. } => {
                    assert_eq!(
                        key.get("tenant_id").unwrap().as_s().unwrap(),
                        "tenant1"
                    );
                },
                TransactionOperation::Update { key, .. } => {
                    assert_eq!(
                        key.get("tenant_id").unwrap().as_s().unwrap(),
                        "tenant1"
                    );
                },
                TransactionOperation::ConditionCheck { key, .. } => {
                    assert_eq!(
                        key.get("tenant_id").unwrap().as_s().unwrap(),
                        "tenant1"
                    );
                },
            }
        }
    }
    
    #[test]
    fn test_add_tenant_condition() {
        let client = DynamoDbClient::new(&aws_config::from_env().region("us-east-1").build());
        let tenant_context = TenantContext::new("tenant1");
        let transaction_manager = TransactionManager::new(client, tenant_context);
        
        // Test with no existing condition
        let (condition, values) = transaction_manager.add_tenant_condition(None, None);
        
        assert_eq!(condition, "tenant_id = :tenant_id");
        assert_eq!(
            values.get(":tenant_id").unwrap().as_s().unwrap(),
            "tenant1"
        );
        
        // Test with existing condition
        let mut expr_attr_values = HashMap::new();
        expr_attr_values.insert(":val".to_string(), AttributeValue::S("value".to_string()));
        
        let (condition, values) = transaction_manager.add_tenant_condition(
            Some("attribute_exists(id)".to_string()),
            Some(expr_attr_values),
        );
        
        assert_eq!(condition, "(attribute_exists(id)) AND tenant_id = :tenant_id");
        assert_eq!(
            values.get(":tenant_id").unwrap().as_s().unwrap(),
            "tenant1"
        );
        assert_eq!(
            values.get(":val").unwrap().as_s().unwrap(),
            "value"
        );
    }
}