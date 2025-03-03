use crate::tenant::{TenantContext, TenantManager, TenantMiddleware, TenantAware, TenantError};
use crate::transactions::{TransactionManager, TransactionOperation, TransactionError};
use aws_sdk_dynamodb::{Client as DynamoDbClient, model::AttributeValue};
use std::collections::HashMap;
use std::error::Error;
use tracing::{info, error, debug};
use uuid::Uuid;

/// Example of using tenant-aware transactions to update a portfolio and its accounts atomically
pub async fn update_portfolio_with_accounts_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create AWS clients
    let config = aws_config::load_from_env().await;
    let dynamodb_client = DynamoDbClient::new(&config);
    
    // Create tenant context
    let tenant_context = TenantContext::new("tenant1");
    
    // Create transaction manager with tenant context
    let transaction_manager = TransactionManager::new(
        dynamodb_client.clone(),
        tenant_context.clone(),
    );
    
    // Define the table name
    let table_name = "performance-data";
    
    // Create a portfolio update operation
    let portfolio_id = Uuid::new_v4().to_string();
    let mut portfolio_item = HashMap::new();
    portfolio_item.insert("id".to_string(), AttributeValue::S(portfolio_id.clone()));
    portfolio_item.insert("entity_type".to_string(), AttributeValue::S("PORTFOLIO".to_string()));
    portfolio_item.insert("name".to_string(), AttributeValue::S("Updated Portfolio".to_string()));
    portfolio_item.insert("description".to_string(), AttributeValue::S("Updated description".to_string()));
    portfolio_item.insert("status".to_string(), AttributeValue::S("ACTIVE".to_string()));
    
    let portfolio_put = TransactionOperation::put(
        table_name,
        portfolio_item,
    );
    
    // Create account operations
    let account_operations = create_account_operations(&portfolio_id, table_name);
    
    // Combine all operations
    let mut operations = vec![portfolio_put];
    operations.extend(account_operations);
    
    info!("Executing transaction with {} operations", operations.len());
    
    // Execute the transaction
    transaction_manager.execute_transaction(operations).await?;
    
    info!("Transaction executed successfully");
    
    // Now let's try with a different tenant
    let tenant2_context = TenantContext::new("tenant2");
    
    // Create a new transaction manager with the second tenant
    let transaction_manager2 = transaction_manager.with_tenant(tenant2_context);
    
    // Create a portfolio update operation for tenant2
    let portfolio2_id = Uuid::new_v4().to_string();
    let mut portfolio2_item = HashMap::new();
    portfolio2_item.insert("id".to_string(), AttributeValue::S(portfolio2_id.clone()));
    portfolio2_item.insert("entity_type".to_string(), AttributeValue::S("PORTFOLIO".to_string()));
    portfolio2_item.insert("name".to_string(), AttributeValue::S("Tenant2 Portfolio".to_string()));
    portfolio2_item.insert("description".to_string(), AttributeValue::S("Portfolio for tenant2".to_string()));
    portfolio2_item.insert("status".to_string(), AttributeValue::S("ACTIVE".to_string()));
    
    let portfolio2_put = TransactionOperation::put(
        table_name,
        portfolio2_item,
    );
    
    // Create account operations for tenant2
    let account2_operations = create_account_operations(&portfolio2_id, table_name);
    
    // Combine all operations for tenant2
    let mut operations2 = vec![portfolio2_put];
    operations2.extend(account2_operations);
    
    info!("Executing transaction for tenant2 with {} operations", operations2.len());
    
    // Execute the transaction for tenant2
    transaction_manager2.execute_transaction(operations2).await?;
    
    info!("Transaction for tenant2 executed successfully");
    
    // Example of a transaction that would fail due to tenant isolation
    // Try to update a portfolio from tenant1 using tenant2's transaction manager
    let mut tenant1_portfolio_key = HashMap::new();
    tenant1_portfolio_key.insert("id".to_string(), AttributeValue::S(portfolio_id.clone()));
    tenant1_portfolio_key.insert("entity_type".to_string(), AttributeValue::S("PORTFOLIO".to_string()));
    
    let update_expression = "SET #name = :name";
    let mut expression_attribute_names = HashMap::new();
    expression_attribute_names.insert("#name".to_string(), "name".to_string());
    
    let mut expression_attribute_values = HashMap::new();
    expression_attribute_values.insert(":name".to_string(), AttributeValue::S("Attempted Update".to_string()));
    
    let update_op = TransactionOperation::update(
        table_name,
        tenant1_portfolio_key,
        update_expression,
        Some(expression_attribute_names),
        Some(expression_attribute_values),
    );
    
    info!("Attempting to update tenant1's portfolio using tenant2's transaction manager");
    
    // This should fail due to tenant isolation
    match transaction_manager2.execute_transaction(vec![update_op]).await {
        Ok(_) => {
            error!("Transaction should have failed due to tenant isolation");
        },
        Err(e) => {
            info!("Transaction correctly failed due to tenant isolation: {}", e);
        }
    }
    
    Ok(())
}

/// Create account operations for a portfolio
fn create_account_operations(portfolio_id: &str, table_name: &str) -> Vec<TransactionOperation> {
    let mut operations = Vec::new();
    
    // Create three accounts for the portfolio
    for i in 1..=3 {
        let account_id = Uuid::new_v4().to_string();
        let mut account_item = HashMap::new();
        account_item.insert("id".to_string(), AttributeValue::S(account_id));
        account_item.insert("entity_type".to_string(), AttributeValue::S("ACCOUNT".to_string()));
        account_item.insert("portfolio_id".to_string(), AttributeValue::S(portfolio_id.to_string()));
        account_item.insert("name".to_string(), AttributeValue::S(format!("Account {}", i)));
        account_item.insert("account_type".to_string(), AttributeValue::S("INVESTMENT".to_string()));
        account_item.insert("status".to_string(), AttributeValue::S("ACTIVE".to_string()));
        
        let account_put = TransactionOperation::put(
            table_name,
            account_item,
        );
        
        operations.push(account_put);
    }
    
    operations
}

/// Example of using tenant-aware transactions with condition checks
pub async fn conditional_transaction_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create AWS clients
    let config = aws_config::load_from_env().await;
    let dynamodb_client = DynamoDbClient::new(&config);
    
    // Create tenant context
    let tenant_context = TenantContext::new("tenant1");
    
    // Create transaction manager with tenant context
    let transaction_manager = TransactionManager::new(
        dynamodb_client.clone(),
        tenant_context.clone(),
    );
    
    // Define the table name
    let table_name = "performance-data";
    
    // Create a portfolio
    let portfolio_id = Uuid::new_v4().to_string();
    let mut portfolio_item = HashMap::new();
    portfolio_item.insert("id".to_string(), AttributeValue::S(portfolio_id.clone()));
    portfolio_item.insert("entity_type".to_string(), AttributeValue::S("PORTFOLIO".to_string()));
    portfolio_item.insert("name".to_string(), AttributeValue::S("Conditional Portfolio".to_string()));
    portfolio_item.insert("status".to_string(), AttributeValue::S("DRAFT".to_string()));
    
    // Put the portfolio
    let portfolio_put = TransactionOperation::put(
        table_name,
        portfolio_item,
    );
    
    // Execute the transaction to create the portfolio
    transaction_manager.execute_transaction(vec![portfolio_put]).await?;
    
    info!("Portfolio created successfully");
    
    // Now try to update the portfolio with a condition check
    let mut portfolio_key = HashMap::new();
    portfolio_key.insert("id".to_string(), AttributeValue::S(portfolio_id.clone()));
    portfolio_key.insert("entity_type".to_string(), AttributeValue::S("PORTFOLIO".to_string()));
    
    // Condition: portfolio must be in DRAFT status
    let condition_check = TransactionOperation::condition_check(
        table_name,
        portfolio_key.clone(),
        "status = :draft_status",
        None,
        Some(HashMap::from([(":draft_status".to_string(), AttributeValue::S("DRAFT".to_string()))])),
    );
    
    // Update the portfolio to ACTIVE status
    let update_expression = "SET #status = :active_status";
    let mut expression_attribute_names = HashMap::new();
    expression_attribute_names.insert("#status".to_string(), "status".to_string());
    
    let mut expression_attribute_values = HashMap::new();
    expression_attribute_values.insert(":active_status".to_string(), AttributeValue::S("ACTIVE".to_string()));
    
    let update_op = TransactionOperation::update(
        table_name,
        portfolio_key.clone(),
        update_expression,
        Some(expression_attribute_names),
        Some(expression_attribute_values),
    );
    
    // Execute the conditional transaction
    let operations = vec![condition_check, update_op];
    transaction_manager.execute_transaction(operations).await?;
    
    info!("Portfolio updated to ACTIVE status successfully");
    
    // Now try to update it again with the same condition (should fail)
    let condition_check2 = TransactionOperation::condition_check(
        table_name,
        portfolio_key.clone(),
        "status = :draft_status",
        None,
        Some(HashMap::from([(":draft_status".to_string(), AttributeValue::S("DRAFT".to_string()))])),
    );
    
    let update_expression2 = "SET description = :description";
    let mut expression_attribute_values2 = HashMap::new();
    expression_attribute_values2.insert(":description".to_string(), AttributeValue::S("Updated description".to_string()));
    
    let update_op2 = TransactionOperation::update(
        table_name,
        portfolio_key.clone(),
        update_expression2,
        None,
        Some(expression_attribute_values2),
    );
    
    // This should fail because the portfolio is no longer in DRAFT status
    let operations2 = vec![condition_check2, update_op2];
    match transaction_manager.execute_transaction(operations2).await {
        Ok(_) => {
            error!("Transaction should have failed due to condition check");
        },
        Err(e) => {
            info!("Transaction correctly failed due to condition check: {}", e);
        }
    }
    
    Ok(())
}

/// Example of using tenant-aware transactions with idempotency
pub async fn idempotent_transaction_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create AWS clients
    let config = aws_config::load_from_env().await;
    let dynamodb_client = DynamoDbClient::new(&config);
    
    // Create tenant context
    let tenant_context = TenantContext::new("tenant1");
    
    // Create transaction manager with tenant context
    let transaction_manager = TransactionManager::new(
        dynamodb_client.clone(),
        tenant_context.clone(),
    );
    
    // Define the table name
    let table_name = "performance-data";
    
    // Create a portfolio
    let portfolio_id = Uuid::new_v4().to_string();
    let mut portfolio_item = HashMap::new();
    portfolio_item.insert("id".to_string(), AttributeValue::S(portfolio_id.clone()));
    portfolio_item.insert("entity_type".to_string(), AttributeValue::S("PORTFOLIO".to_string()));
    portfolio_item.insert("name".to_string(), AttributeValue::S("Idempotent Portfolio".to_string()));
    portfolio_item.insert("status".to_string(), AttributeValue::S("ACTIVE".to_string()));
    
    // Put the portfolio
    let portfolio_put = TransactionOperation::put(
        table_name,
        portfolio_item,
    );
    
    // Generate a client request token for idempotency
    let client_request_token = Uuid::new_v4().to_string();
    
    // Execute the transaction with the token
    transaction_manager
        .execute_transaction_with_token(vec![portfolio_put.clone()], client_request_token.clone())
        .await?;
    
    info!("Portfolio created successfully");
    
    // Try to execute the same transaction again with the same token
    // This should be idempotent and not create a duplicate
    info!("Executing the same transaction again with the same token");
    
    transaction_manager
        .execute_transaction_with_token(vec![portfolio_put], client_request_token)
        .await?;
    
    info!("Second transaction executed successfully (idempotent)");
    
    Ok(())
}

/// Example of using tenant-aware transactions with the tenant middleware
pub async fn transaction_with_middleware_example() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create AWS clients
    let config = aws_config::load_from_env().await;
    let dynamodb_client = DynamoDbClient::new(&config);
    
    // Create tenant context
    let tenant_context = TenantContext::new("tenant1");
    
    // Create tenant manager
    let tenant_manager = TenantManager::new(tenant_context.clone())?;
    
    // Create transaction manager with tenant context
    let transaction_manager = TransactionManager::new(
        dynamodb_client.clone(),
        tenant_context.clone(),
    );
    
    // Create tenant middleware for the transaction manager
    let transaction_middleware = TenantMiddleware::new(
        transaction_manager,
        tenant_manager.clone(),
    );
    
    // Define the table name
    let table_name = "performance-data";
    
    // Create a portfolio
    let portfolio_id = Uuid::new_v4().to_string();
    let mut portfolio_item = HashMap::new();
    portfolio_item.insert("id".to_string(), AttributeValue::S(portfolio_id.clone()));
    portfolio_item.insert("entity_type".to_string(), AttributeValue::S("PORTFOLIO".to_string()));
    portfolio_item.insert("name".to_string(), AttributeValue::S("Middleware Portfolio".to_string()));
    portfolio_item.insert("status".to_string(), AttributeValue::S("ACTIVE".to_string()));
    
    // Put the portfolio
    let portfolio_put = TransactionOperation::put(
        table_name,
        portfolio_item,
    );
    
    // Get the transaction manager from the middleware
    let transaction_manager = transaction_middleware.inner();
    
    // Execute the transaction
    transaction_manager.execute_transaction(vec![portfolio_put]).await?;
    
    info!("Portfolio created successfully using middleware");
    
    // Switch to a different tenant
    let tenant2_context = TenantContext::new("tenant2");
    let transaction_middleware2 = transaction_middleware.with_tenant(tenant2_context)?;
    
    // Get the transaction manager for tenant2
    let transaction_manager2 = transaction_middleware2.inner();
    
    // Create a portfolio for tenant2
    let portfolio2_id = Uuid::new_v4().to_string();
    let mut portfolio2_item = HashMap::new();
    portfolio2_item.insert("id".to_string(), AttributeValue::S(portfolio2_id.clone()));
    portfolio2_item.insert("entity_type".to_string(), AttributeValue::S("PORTFOLIO".to_string()));
    portfolio2_item.insert("name".to_string(), AttributeValue::S("Tenant2 Middleware Portfolio".to_string()));
    portfolio2_item.insert("status".to_string(), AttributeValue::S("ACTIVE".to_string()));
    
    // Put the portfolio for tenant2
    let portfolio2_put = TransactionOperation::put(
        table_name,
        portfolio2_item,
    );
    
    // Execute the transaction for tenant2
    transaction_manager2.execute_transaction(vec![portfolio2_put]).await?;
    
    info!("Portfolio created successfully for tenant2 using middleware");
    
    Ok(())
} 