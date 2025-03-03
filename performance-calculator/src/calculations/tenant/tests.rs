use super::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid;
use aws_config;
use aws_sdk_dynamodb;

/// Test fixture for tenant tests
struct TenantTestFixture {
    tenant_manager: Arc<Mutex<InMemoryTenantManager>>,
    metrics_manager: Arc<Mutex<InMemoryTenantMetricsManager>>,
}

impl TenantTestFixture {
    fn new() -> Self {
        Self {
            tenant_manager: Arc::new(Mutex::new(InMemoryTenantManager::new())),
            metrics_manager: Arc::new(Mutex::new(InMemoryTenantMetricsManager::new())),
        }
    }
    
    async fn create_test_tenant(&self, name: &str, tier: SubscriptionTier) -> Tenant {
        let tenant = Tenant::new(name, Some("Test tenant"), tier);
        let manager = self.tenant_manager.lock().await;
        manager.create_tenant(tenant.clone()).await.unwrap()
    }
}

#[tokio::test]
async fn test_tenant_creation() {
    let fixture = TenantTestFixture::new();
    
    // Create a tenant
    let tenant = fixture.create_test_tenant("Test Tenant", SubscriptionTier::Basic).await;
    
    // Verify tenant properties
    assert_eq!(tenant.name, "Test Tenant");
    assert_eq!(tenant.description, Some("Test tenant".to_string()));
    assert_eq!(tenant.status, TenantStatus::Active);
    assert_eq!(tenant.subscription_tier, SubscriptionTier::Basic);
    
    // Verify resource limits for Basic tier
    assert_eq!(tenant.resource_limits.max_portfolios, 20);
    assert_eq!(tenant.resource_limits.max_accounts_per_portfolio, 20);
    assert_eq!(tenant.resource_limits.max_securities, 500);
    assert_eq!(tenant.resource_limits.max_transactions, 10000);
    assert_eq!(tenant.resource_limits.max_concurrent_calculations, 2);
    assert_eq!(tenant.resource_limits.max_storage_bytes, 524_288_000); // 500 MB
    assert_eq!(tenant.resource_limits.max_api_requests_per_minute, 50);
}

#[tokio::test]
async fn test_tenant_retrieval() {
    let fixture = TenantTestFixture::new();
    
    // Create a tenant
    let created_tenant = fixture.create_test_tenant("Retrieval Test", SubscriptionTier::Professional).await;
    
    // Retrieve the tenant
    let manager = fixture.tenant_manager.lock().await;
    let retrieved_tenant = manager.get_tenant(&created_tenant.id).await.unwrap().unwrap();
    
    // Verify tenant properties
    assert_eq!(retrieved_tenant.id, created_tenant.id);
    assert_eq!(retrieved_tenant.name, "Retrieval Test");
    assert_eq!(retrieved_tenant.subscription_tier, SubscriptionTier::Professional);
    
    // Verify non-existent tenant returns None
    let non_existent = manager.get_tenant("non-existent-id").await.unwrap();
    assert!(non_existent.is_none());
}

#[tokio::test]
async fn test_tenant_update() {
    let fixture = TenantTestFixture::new();
    
    // Create a tenant
    let mut tenant = fixture.create_test_tenant("Update Test", SubscriptionTier::Basic).await;
    
    // Update tenant properties
    tenant.name = "Updated Name".to_string();
    tenant.description = Some("Updated description".to_string());
    tenant.subscription_tier = SubscriptionTier::Professional;
    
    // Update the tenant
    let manager = fixture.tenant_manager.lock().await;
    let updated_tenant = manager.update_tenant(tenant).await.unwrap();
    
    // Verify updated properties
    assert_eq!(updated_tenant.name, "Updated Name");
    assert_eq!(updated_tenant.description, Some("Updated description".to_string()));
    assert_eq!(updated_tenant.subscription_tier, SubscriptionTier::Professional);
    
    // Verify resource limits were updated to Professional tier
    assert_eq!(updated_tenant.resource_limits.max_portfolios, 100);
    assert_eq!(updated_tenant.resource_limits.max_accounts_per_portfolio, 50);
    assert_eq!(updated_tenant.resource_limits.max_api_requests_per_minute, 100);
}

#[tokio::test]
async fn test_tenant_status_changes() {
    let fixture = TenantTestFixture::new();
    
    // Create a tenant
    let tenant = fixture.create_test_tenant("Status Test", SubscriptionTier::Basic).await;
    assert_eq!(tenant.status, TenantStatus::Active);
    assert!(tenant.is_active());
    
    // Suspend the tenant
    let manager = fixture.tenant_manager.lock().await;
    let suspended_tenant = manager.suspend_tenant(&tenant.id).await.unwrap();
    assert_eq!(suspended_tenant.status, TenantStatus::Suspended);
    assert!(!suspended_tenant.is_active());
    
    // Reactivate the tenant
    let reactivated_tenant = manager.activate_tenant(&tenant.id).await.unwrap();
    assert_eq!(reactivated_tenant.status, TenantStatus::Active);
    assert!(reactivated_tenant.is_active());
    
    // Deactivate the tenant
    let deactivated_tenant = manager.deactivate_tenant(&tenant.id).await.unwrap();
    assert_eq!(deactivated_tenant.status, TenantStatus::Deactivated);
    assert!(!deactivated_tenant.is_active());
}

#[tokio::test]
async fn test_tenant_resource_limits() {
    let fixture = TenantTestFixture::new();
    
    // Create a tenant with Free tier (limited resources)
    let tenant = fixture.create_test_tenant("Resource Test", SubscriptionTier::Free).await;
    
    // Verify resource limits for Free tier
    assert_eq!(tenant.resource_limits.max_portfolios, 5);
    assert_eq!(tenant.resource_limits.max_accounts_per_portfolio, 10);
    assert_eq!(tenant.resource_limits.max_securities, 100);
    assert_eq!(tenant.resource_limits.max_transactions, 1000);
    
    // Test resource limit checks
    assert!(!tenant.would_exceed_limit("portfolios", 3)); // 3 < 5, not exceeded
    assert!(tenant.would_exceed_limit("portfolios", 6));  // 6 > 5, exceeded
    
    assert!(!tenant.would_exceed_limit("accounts", 8));   // 8 < 10, not exceeded
    assert!(tenant.would_exceed_limit("accounts", 15));   // 15 > 10, exceeded
    
    // Update resource limits
    let manager = fixture.tenant_manager.lock().await;
    let custom_limits = ResourceLimits {
        max_portfolios: 10,
        max_accounts_per_portfolio: 20,
        max_securities: 200,
        max_transactions: 2000,
        max_concurrent_calculations: 2,
        max_storage_bytes: 200_000_000,
        max_api_requests_per_minute: 20,
    };
    
    let updated_tenant = manager.update_resource_limits(&tenant.id, custom_limits.clone()).await.unwrap();
    
    // Verify updated limits
    assert_eq!(updated_tenant.resource_limits.max_portfolios, 10);
    assert_eq!(updated_tenant.resource_limits.max_accounts_per_portfolio, 20);
    assert_eq!(updated_tenant.resource_limits.max_securities, 200);
    
    // Test updated resource limit checks
    assert!(!updated_tenant.would_exceed_limit("portfolios", 8)); // 8 < 10, not exceeded
    assert!(updated_tenant.would_exceed_limit("portfolios", 12)); // 12 > 10, exceeded
}

#[tokio::test]
async fn test_tenant_deletion() {
    let fixture = TenantTestFixture::new();
    
    // Create a tenant
    let tenant = fixture.create_test_tenant("Deletion Test", SubscriptionTier::Basic).await;
    
    // Verify tenant exists
    let manager = fixture.tenant_manager.lock().await;
    let exists_before = manager.tenant_exists(&tenant.id).await.unwrap();
    assert!(exists_before);
    
    // Delete the tenant
    manager.delete_tenant(&tenant.id).await.unwrap();
    
    // Verify tenant no longer exists
    let exists_after = manager.tenant_exists(&tenant.id).await.unwrap();
    assert!(!exists_after);
    
    // Verify get_tenant returns None
    let retrieved = manager.get_tenant(&tenant.id).await.unwrap();
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_tenant_metrics_tracking() {
    let fixture = TenantTestFixture::new();
    
    // Create a tenant
    let tenant = fixture.create_test_tenant("Metrics Test", SubscriptionTier::Basic).await;
    
    // Get initial metrics
    let metrics_manager = fixture.metrics_manager.lock().await;
    let initial_metrics = metrics_manager.get_usage_metrics(&tenant.id).await.unwrap();
    
    // Verify initial metrics are zero
    assert_eq!(initial_metrics.portfolio_count, 0);
    assert_eq!(initial_metrics.account_count, 0);
    assert_eq!(initial_metrics.transaction_count, 0);
    assert_eq!(initial_metrics.api_requests, 0);
    
    // Increment metrics
    metrics_manager.increment_metric(&tenant.id, "portfolios", 3).await.unwrap();
    metrics_manager.increment_metric(&tenant.id, "accounts", 5).await.unwrap();
    metrics_manager.increment_metric(&tenant.id, "transactions", 10).await.unwrap();
    
    // Track API requests
    for _ in 0..15 {
        metrics_manager.track_api_request(&tenant.id).await.unwrap();
    }
    
    // Get updated metrics
    let updated_metrics = metrics_manager.get_usage_metrics(&tenant.id).await.unwrap();
    
    // Verify metrics were incremented
    assert_eq!(updated_metrics.portfolio_count, 3);
    assert_eq!(updated_metrics.account_count, 5);
    assert_eq!(updated_metrics.transaction_count, 10);
    assert_eq!(updated_metrics.api_requests, 15);
    
    // Check resource usage percentages
    let portfolio_usage = updated_metrics.usage_percentage("portfolios", &tenant.resource_limits);
    assert_eq!(portfolio_usage, (3.0 / 20.0) * 100.0); // 3/20 = 15%
    
    // Reset API requests
    metrics_manager.reset_api_requests(&tenant.id).await.unwrap();
    
    // Verify API requests were reset
    let after_reset = metrics_manager.get_usage_metrics(&tenant.id).await.unwrap();
    assert_eq!(after_reset.api_requests, 0);
    assert_eq!(after_reset.portfolio_count, 3); // Other metrics unchanged
}

#[tokio::test]
async fn test_tenant_billing_records() {
    let fixture = TenantTestFixture::new();
    
    // Create a tenant
    let tenant = fixture.create_test_tenant("Billing Test", SubscriptionTier::Professional).await;
    
    // Create billing period
    let now = Utc::now();
    let period_start = now - chrono::Duration::days(30);
    let period_end = now;
    
    // Create billing record
    let metrics_manager = fixture.metrics_manager.lock().await;
    let billing_record = TenantBillingRecord::new(
        &tenant.id,
        period_start,
        period_end,
        "Professional",
        99.99,
        "USD"
    );
    
    // Save billing record
    let saved_record = metrics_manager.create_billing_record(billing_record).await.unwrap();
    
    // Verify billing record
    assert_eq!(saved_record.tenant_id, tenant.id);
    assert_eq!(saved_record.subscription_tier, "Professional");
    assert_eq!(saved_record.base_amount, 99.99);
    assert_eq!(saved_record.total_amount, 99.99);
    assert_eq!(saved_record.status, BillingStatus::Pending);
    
    // Add additional charges
    let mut updated_record = saved_record.clone();
    updated_record.add_charge("Extra storage", 10.00);
    updated_record.add_charge("Premium support", 20.00);
    
    // Update billing record
    let updated = metrics_manager.update_billing_record(updated_record).await.unwrap();
    
    // Verify additional charges
    assert_eq!(updated.additional_charges.len(), 2);
    assert_eq!(updated.additional_charges.get("Extra storage"), Some(&10.00));
    assert_eq!(updated.additional_charges.get("Premium support"), Some(&20.00));
    assert_eq!(updated.total_amount, 99.99 + 10.00 + 20.00);
    
    // Update status to paid
    let mut paid_record = updated.clone();
    paid_record.update_status(BillingStatus::Paid);
    
    // Update billing record
    let paid = metrics_manager.update_billing_record(paid_record).await.unwrap();
    
    // Verify status
    assert_eq!(paid.status, BillingStatus::Paid);
    
    // Retrieve billing records
    let records = metrics_manager.get_billing_records(&tenant.id, None, None, None, None).await.unwrap();
    
    // Verify records
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].id, paid.id);
    assert_eq!(records[0].status, BillingStatus::Paid);
}

#[tokio::test]
async fn test_tenant_isolation() {
    let fixture = TenantTestFixture::new();
    
    // Create two tenants
    let tenant1 = fixture.create_test_tenant("Tenant 1", SubscriptionTier::Basic).await;
    let tenant2 = fixture.create_test_tenant("Tenant 2", SubscriptionTier::Basic).await;
    
    // Add metrics for tenant 1
    let metrics_manager = fixture.metrics_manager.lock().await;
    metrics_manager.increment_metric(&tenant1.id, "portfolios", 5).await.unwrap();
    metrics_manager.increment_metric(&tenant1.id, "accounts", 10).await.unwrap();
    
    // Add metrics for tenant 2
    metrics_manager.increment_metric(&tenant2.id, "portfolios", 3).await.unwrap();
    metrics_manager.increment_metric(&tenant2.id, "securities", 15).await.unwrap();
    
    // Verify tenant 1 metrics
    let metrics1 = metrics_manager.get_usage_metrics(&tenant1.id).await.unwrap();
    assert_eq!(metrics1.portfolio_count, 5);
    assert_eq!(metrics1.account_count, 10);
    assert_eq!(metrics1.security_count, 0); // Not set for tenant 1
    
    // Verify tenant 2 metrics
    let metrics2 = metrics_manager.get_usage_metrics(&tenant2.id).await.unwrap();
    assert_eq!(metrics2.portfolio_count, 3);
    assert_eq!(metrics2.account_count, 0); // Not set for tenant 2
    assert_eq!(metrics2.security_count, 15);
    
    // Create billing records for each tenant
    let now = Utc::now();
    let period_start = now - chrono::Duration::days(30);
    let period_end = now;
    
    let billing1 = TenantBillingRecord::new(
        &tenant1.id,
        period_start,
        period_end,
        "Basic",
        49.99,
        "USD"
    );
    
    let billing2 = TenantBillingRecord::new(
        &tenant2.id,
        period_start,
        period_end,
        "Basic",
        49.99,
        "USD"
    );
    
    metrics_manager.create_billing_record(billing1).await.unwrap();
    metrics_manager.create_billing_record(billing2).await.unwrap();
    
    // Verify tenant 1 billing records
    let records1 = metrics_manager.get_billing_records(&tenant1.id, None, None, None, None).await.unwrap();
    assert_eq!(records1.len(), 1);
    assert_eq!(records1[0].tenant_id, tenant1.id);
    
    // Verify tenant 2 billing records
    let records2 = metrics_manager.get_billing_records(&tenant2.id, None, None, None, None).await.unwrap();
    assert_eq!(records2.len(), 1);
    assert_eq!(records2[0].tenant_id, tenant2.id);
}

#[tokio::test]
#[ignore] // Ignore by default as it requires a real DynamoDB instance
async fn test_dynamodb_tenant_metrics_manager() {
    // Set up environment variables for testing
    std::env::set_var("TENANT_METRICS_TABLE", "test-tenant-metrics");
    std::env::set_var("TENANT_BILLING_TABLE", "test-tenant-billing");
    
    // Create DynamoDB client with local endpoint
    let config = aws_config::from_env()
        .endpoint_url("http://localhost:8000") // Point to local DynamoDB
        .load()
        .await;
    
    let client = aws_sdk_dynamodb::Client::new(&config);
    
    // Create metrics manager
    let metrics_manager = DynamoDbTenantMetricsManager::new(
        client.clone(),
        "test-tenant-metrics".to_string(),
        "test-tenant-billing".to_string(),
    );
    
    // Create test tenant ID
    let tenant_id = format!("test-tenant-{}", Uuid::new_v4());
    
    // Test get_usage_metrics (should return default metrics for new tenant)
    let initial_metrics = metrics_manager.get_usage_metrics(&tenant_id).await.unwrap();
    assert_eq!(initial_metrics.tenant_id, tenant_id);
    assert_eq!(initial_metrics.portfolio_count, 0);
    assert_eq!(initial_metrics.account_count, 0);
    assert_eq!(initial_metrics.transaction_count, 0);
    assert_eq!(initial_metrics.api_requests, 0);
    
    // Test increment_metric
    let updated_metrics = metrics_manager.increment_metric(&tenant_id, "portfolios", 3).await.unwrap();
    assert_eq!(updated_metrics.portfolio_count, 3);
    
    // Test track_api_request
    metrics_manager.track_api_request(&tenant_id).await.unwrap();
    metrics_manager.track_api_request(&tenant_id).await.unwrap();
    
    let metrics_after_api = metrics_manager.get_usage_metrics(&tenant_id).await.unwrap();
    assert_eq!(metrics_after_api.api_requests, 2);
    
    // Test reset_api_requests
    metrics_manager.reset_api_requests(&tenant_id).await.unwrap();
    
    let metrics_after_reset = metrics_manager.get_usage_metrics(&tenant_id).await.unwrap();
    assert_eq!(metrics_after_reset.api_requests, 0);
    assert_eq!(metrics_after_reset.portfolio_count, 3); // Other metrics unchanged
    
    // Test decrement_metric
    let decremented_metrics = metrics_manager.decrement_metric(&tenant_id, "portfolios", 1).await.unwrap();
    assert_eq!(decremented_metrics.portfolio_count, 2);
    
    // Test update_usage_metrics
    let mut custom_metrics = TenantUsageMetrics::new(&tenant_id);
    custom_metrics.portfolio_count = 5;
    custom_metrics.account_count = 10;
    custom_metrics.security_count = 15;
    
    let updated = metrics_manager.update_usage_metrics(custom_metrics).await.unwrap();
    assert_eq!(updated.portfolio_count, 5);
    assert_eq!(updated.account_count, 10);
    assert_eq!(updated.security_count, 15);
    
    // Test billing records
    let now = Utc::now();
    let period_start = now - chrono::Duration::days(30);
    let period_end = now;
    
    let billing_record = TenantBillingRecord::new(
        &tenant_id,
        period_start,
        period_end,
        "Professional",
        99.99,
        "USD"
    );
    
    // Test create_billing_record
    let created_record = metrics_manager.create_billing_record(billing_record).await.unwrap();
    assert_eq!(created_record.tenant_id, tenant_id);
    assert_eq!(created_record.subscription_tier, "Professional");
    assert_eq!(created_record.base_amount, 99.99);
    
    // Test get_billing_records
    let records = metrics_manager.get_billing_records(&tenant_id, None, None, None, None).await.unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].tenant_id, tenant_id);
    assert_eq!(records[0].subscription_tier, "Professional");
    
    // Test update_billing_record
    let mut updated_record = records[0].clone();
    updated_record.add_charge("Extra storage", 10.00);
    
    let updated = metrics_manager.update_billing_record(updated_record).await.unwrap();
    assert_eq!(updated.additional_charges.len(), 1);
    assert_eq!(updated.additional_charges.get("Extra storage"), Some(&10.00));
    assert_eq!(updated.total_amount, 99.99 + 10.00);
    
    // Clean up (delete test data)
    // Note: In a real test, you would delete the test data from DynamoDB
}

#[tokio::test]
async fn test_tenant_metrics_manager_factory() {
    // Test with in-memory configuration
    std::env::set_var("USE_DYNAMODB_METRICS", "false");
    
    let metrics_manager = get_tenant_metrics_manager().await.unwrap();
    
    // Create a tenant and test basic operations
    let tenant_id = format!("test-tenant-{}", Uuid::new_v4());
    
    // Test get_usage_metrics
    let initial_metrics = metrics_manager.get_usage_metrics(&tenant_id).await.unwrap();
    assert_eq!(initial_metrics.tenant_id, tenant_id);
    assert_eq!(initial_metrics.portfolio_count, 0);
    
    // Test increment_metric
    let updated_metrics = metrics_manager.increment_metric(&tenant_id, "portfolios", 3).await.unwrap();
    assert_eq!(updated_metrics.portfolio_count, 3);
    
    // Test with DynamoDB configuration (should fall back to in-memory in test)
    std::env::set_var("USE_DYNAMODB_METRICS", "true");
    
    let fallback_manager = get_tenant_metrics_manager().await.unwrap();
    
    // Test that we can still perform operations
    let fallback_metrics = fallback_manager.get_usage_metrics(&tenant_id).await.unwrap();
    assert_eq!(fallback_metrics.tenant_id, tenant_id);
} 