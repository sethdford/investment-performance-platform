# Tenant Isolation in the Investment Performance Calculator

This document describes the tenant isolation implementation in the Investment Performance Calculator. The tenant isolation ensures that data from different tenants is properly segregated and that tenants can only access their own data.

## Overview

The tenant isolation implementation uses a combination of techniques:

1. **Tenant Context**: A `TenantContext` struct that contains the tenant ID and optional metadata.
2. **Tenant Manager**: A `TenantManager` that provides utility methods for working with tenant data.
3. **Tenant-Aware Repositories**: Repositories that implement the `TenantAware` trait to ensure tenant isolation.
4. **Tenant Middleware**: A middleware pattern that wraps repositories to enforce tenant isolation.

## Components

### TenantContext

The `TenantContext` struct contains the tenant ID and optional metadata:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TenantContext {
    pub tenant_id: String,
    pub metadata: HashMap<String, String>,
}
```

The `TenantContext` provides methods for creating and validating tenant contexts, as well as managing metadata.

### TenantManager

The `TenantManager` provides utility methods for working with tenant data:

```rust
#[derive(Debug, Clone)]
pub struct TenantManager {
    tenant_context: TenantContext,
}
```

The `TenantManager` provides methods for:

- Adding tenant IDs to DynamoDB keys and items
- Creating tenant-specific condition expressions and attribute values
- Validating tenant ownership of items
- Creating tenant-specific table names

### TenantAware Trait

The `TenantAware` trait defines the interface for tenant-aware repositories:

```rust
pub trait TenantAware {
    fn tenant_context(&self) -> &TenantContext;
    fn with_tenant(self, tenant_context: TenantContext) -> Self;
    // ... other methods ...
}
```

Repositories that implement this trait can:

- Access the current tenant context
- Create new instances with different tenant contexts
- Add tenant IDs to DynamoDB items
- Create tenant-specific condition expressions and attribute values
- Validate tenant ownership of items

### TenantMiddleware

The `TenantMiddleware` wraps repositories to enforce tenant isolation:

```rust
pub struct TenantMiddleware<T> {
    inner: T,
    tenant_manager: TenantManager,
}
```

The middleware pattern allows for:

- Transparent access to the underlying repository
- Enforcing tenant isolation at the middleware level
- Switching between tenants without changing the repository implementation

## Usage

### Creating Tenant-Aware Repositories

To make a repository tenant-aware, implement the `TenantAware` trait:

```rust
impl TenantAware for DynamoDbRepository {
    fn tenant_context(&self) -> &TenantContext {
        &self.tenant_context
    }
    
    fn with_tenant(mut self, tenant_context: TenantContext) -> Self {
        self.tenant_context = tenant_context;
        self
    }
}
```

### Using Tenant-Aware Repositories

When using tenant-aware repositories, ensure that:

1. The tenant ID is included in all keys and items
2. Tenant ownership is validated for all retrieved items
3. Tenant-specific condition expressions are used for queries and scans

Example:

```rust
// Create a tenant context
let tenant_context = TenantContext::new("tenant1");

// Create a repository with the tenant context
let repository = DynamoDbRepository::new(
    dynamodb_client,
    "table_name",
    tenant_context,
);

// Use the repository
let portfolio = repository.get_portfolio("portfolio1").await?;
```

### Using Tenant Middleware

The tenant middleware provides an additional layer of isolation:

```rust
// Create a tenant context
let tenant_context = TenantContext::new("tenant1");

// Create a tenant manager
let tenant_manager = TenantManager::new(tenant_context.clone())?;

// Create a repository
let repository = DynamoDbRepository::new(
    dynamodb_client,
    "table_name",
    tenant_context,
);

// Create a middleware
let middleware = TenantMiddleware::new(repository, tenant_manager);

// Use the repository through the middleware
let repo = middleware.inner();
let portfolio = repo.get_portfolio("portfolio1").await?;
```

### Switching Tenants

To switch to a different tenant:

```rust
// Create a new tenant context
let tenant2_context = TenantContext::new("tenant2");

// Create a new middleware with the new tenant
let middleware2 = middleware.with_tenant(tenant2_context)?;

// Use the repository with the new tenant
let repo2 = middleware2.inner();
let portfolio2 = repo2.get_portfolio("portfolio1").await?;
```

## Best Practices

1. **Always Include Tenant ID**: Always include the tenant ID in all keys and items.
2. **Validate Tenant Ownership**: Always validate tenant ownership for all retrieved items.
3. **Use Tenant-Specific Condition Expressions**: Use tenant-specific condition expressions for queries and scans.
4. **Use Tenant Middleware**: Use the tenant middleware pattern for an additional layer of isolation.
5. **Test Tenant Isolation**: Write tests to ensure that tenant isolation is properly enforced.

## Example

See the `tenant_example.rs` file for a complete example of using tenant isolation in a real-world scenario.

## Testing

The tenant isolation implementation includes comprehensive tests in the `tenant_tests.rs` file. These tests ensure that:

1. Tenant contexts are properly created and validated
2. Tenant managers provide the correct utility methods
3. Tenant-aware repositories properly enforce tenant isolation
4. Tenant middleware properly wraps repositories and enforces tenant isolation
5. Switching between tenants works correctly

## Conclusion

The tenant isolation implementation provides a robust and flexible way to ensure that data from different tenants is properly segregated. By using a combination of tenant contexts, tenant managers, tenant-aware repositories, and tenant middleware, the Investment Performance Calculator can safely handle data from multiple tenants in a single deployment. 