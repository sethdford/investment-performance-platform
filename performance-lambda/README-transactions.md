# Transaction Support with Tenant Isolation in the Investment Performance Calculator

This document describes the transaction support implementation in the Investment Performance Calculator, with a focus on how it integrates with the tenant isolation system.

## Overview

The transaction support implementation provides a way to perform atomic operations on multiple items in DynamoDB, ensuring that either all operations succeed or none of them do. The implementation is fully integrated with the tenant isolation system, ensuring that transactions are properly isolated between tenants.

## Components

### TransactionError

The `TransactionError` enum represents errors that can occur during transaction operations:

```rust
#[derive(Debug)]
pub enum TransactionError {
    ConditionCheckFailed { item: Option<HashMap<String, AttributeValue>>, message: String },
    TransactionConflict(String),
    ProvisionedThroughputExceeded(String),
    ItemCollectionSizeLimitExceeded { table_name: String, message: String },
    ValidationError(String),
    InternalServerError(String),
    ServiceError(String),
    ClientError(String),
    Unknown(String),
}
```

### TransactionOperation

The `TransactionOperation` enum represents the different types of operations that can be performed in a transaction:

```rust
pub enum TransactionOperation {
    Put {
        table_name: String,
        item: HashMap<String, AttributeValue>,
        condition_expression: Option<String>,
        expression_attribute_names: Option<HashMap<String, String>>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    },
    Delete {
        table_name: String,
        key: HashMap<String, AttributeValue>,
        condition_expression: Option<String>,
        expression_attribute_names: Option<HashMap<String, String>>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    },
    Update {
        table_name: String,
        key: HashMap<String, AttributeValue>,
        update_expression: String,
        condition_expression: Option<String>,
        expression_attribute_names: Option<HashMap<String, String>>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    },
    ConditionCheck {
        table_name: String,
        key: HashMap<String, AttributeValue>,
        condition_expression: String,
        expression_attribute_names: Option<HashMap<String, String>>,
        expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    },
}
```

The `TransactionOperation` enum provides methods for creating each type of operation:

- `put`: Create a Put operation
- `put_with_condition`: Create a Put operation with a condition
- `delete`: Create a Delete operation
- `delete_with_condition`: Create a Delete operation with a condition
- `update`: Create an Update operation
- `update_with_condition`: Create an Update operation with a condition
- `condition_check`: Create a ConditionCheck operation

### TransactionManager

The `TransactionManager` struct is responsible for executing transactions:

```rust
#[derive(Debug, Clone)]
pub struct TransactionManager {
    client: DynamoDbClient,
    tenant_context: TenantContext,
}
```

The `TransactionManager` provides methods for:

- Executing transactions: `execute_transaction`
- Executing transactions with idempotency: `execute_transaction_with_token`
- Adding tenant IDs to transaction operations: `add_tenant_to_operations`
- Adding tenant conditions to expressions: `add_tenant_condition`

## Integration with Tenant Isolation

The transaction support is fully integrated with the tenant isolation system:

1. The `TransactionManager` implements the `TenantAware` trait, allowing it to be used with the tenant isolation system.
2. Transaction operations automatically include the tenant ID in keys and items.
3. Condition expressions automatically include tenant conditions to ensure tenant isolation.
4. The `TransactionManager` can be wrapped in a `TenantMiddleware` for additional isolation.

## Usage

### Creating a Transaction Manager

To create a transaction manager:

```rust
// Create a tenant context
let tenant_context = TenantContext::new("tenant1");

// Create a transaction manager
let transaction_manager = TransactionManager::new(
    dynamodb_client,
    tenant_context,
);
```

### Creating Transaction Operations

To create transaction operations:

```rust
// Create a Put operation
let put_op = TransactionOperation::put(
    "table_name",
    item,
);

// Create a Delete operation
let delete_op = TransactionOperation::delete(
    "table_name",
    key,
);

// Create an Update operation
let update_op = TransactionOperation::update(
    "table_name",
    key,
    "SET #attr = :val",
    Some(expression_attribute_names),
    Some(expression_attribute_values),
);

// Create a ConditionCheck operation
let condition_op = TransactionOperation::condition_check(
    "table_name",
    key,
    "attribute_exists(id)",
    None,
    None,
);
```

### Executing Transactions

To execute a transaction:

```rust
// Create operations
let operations = vec![put_op, update_op, delete_op, condition_op];

// Execute the transaction
transaction_manager.execute_transaction(operations).await?;
```

### Executing Transactions with Idempotency

To execute a transaction with idempotency:

```rust
// Create operations
let operations = vec![put_op, update_op, delete_op, condition_op];

// Generate a client request token
let client_request_token = Uuid::new_v4().to_string();

// Execute the transaction with the token
transaction_manager.execute_transaction_with_token(operations, client_request_token).await?;
```

### Using Transactions with Different Tenants

To use transactions with different tenants:

```rust
// Create a transaction manager for tenant1
let transaction_manager1 = TransactionManager::new(
    dynamodb_client.clone(),
    tenant1_context,
);

// Create a transaction manager for tenant2
let transaction_manager2 = TransactionManager::new(
    dynamodb_client.clone(),
    tenant2_context,
);

// Or switch tenants on an existing manager
let transaction_manager2 = transaction_manager1.with_tenant(tenant2_context);
```

### Using Transactions with Tenant Middleware

To use transactions with tenant middleware:

```rust
// Create a tenant context
let tenant_context = TenantContext::new("tenant1");

// Create a tenant manager
let tenant_manager = TenantManager::new(tenant_context.clone())?;

// Create a transaction manager
let transaction_manager = TransactionManager::new(
    dynamodb_client.clone(),
    tenant_context,
);

// Create a tenant middleware
let transaction_middleware = TenantMiddleware::new(
    transaction_manager,
    tenant_manager,
);

// Use the transaction manager through the middleware
let transaction_manager = transaction_middleware.inner();
transaction_manager.execute_transaction(operations).await?;

// Switch to a different tenant
let transaction_middleware2 = transaction_middleware.with_tenant(tenant2_context)?;
let transaction_manager2 = transaction_middleware2.inner();
transaction_manager2.execute_transaction(operations2).await?;
```

## Examples

See the `transaction_example.rs` file for complete examples of using transactions with tenant isolation:

1. `update_portfolio_with_accounts_example`: Example of using tenant-aware transactions to update a portfolio and its accounts atomically.
2. `conditional_transaction_example`: Example of using tenant-aware transactions with condition checks.
3. `idempotent_transaction_example`: Example of using tenant-aware transactions with idempotency.
4. `transaction_with_middleware_example`: Example of using tenant-aware transactions with the tenant middleware.

## Best Practices

1. **Use Transactions for Related Operations**: Use transactions when you need to update multiple related items atomically.
2. **Include Condition Checks**: Use condition checks to ensure that items meet certain conditions before updating them.
3. **Use Idempotency Tokens**: Use idempotency tokens when retrying transactions to avoid duplicate operations.
4. **Keep Transactions Small**: Keep transactions as small as possible to avoid hitting DynamoDB limits.
5. **Handle Transaction Errors**: Handle transaction errors appropriately, especially `ConditionCheckFailed` errors.
6. **Test Tenant Isolation**: Test that transactions are properly isolated between tenants.

## Conclusion

The transaction support implementation provides a robust and flexible way to perform atomic operations on multiple items in DynamoDB, while ensuring proper tenant isolation. By using transactions, the Investment Performance Calculator can maintain data consistency and integrity across related items, even in a multi-tenant environment.