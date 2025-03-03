#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::sleep;
    use thiserror::Error;

    #[derive(Error, Debug)]
    enum TestError {
        #[error("Test error")]
        TestError,
    }

    async fn test_operation() -> Result<String, TestError> {
        Ok("success".to_string())
    }

    async fn test_failing_operation() -> Result<String, TestError> {
        Err(TestError::TestError)
    }

    async fn test_slow_operation() -> Result<String, TestError> {
        sleep(Duration::from_millis(100)).await;
        Ok("slow success".to_string())
    }

    #[tokio::test]
    async fn test_bulkhead_successful_execution() {
        let config = BulkheadConfig {
            max_concurrent_calls: 2,
            max_queue_size: 2,
            execution_timeout_seconds: 1,
        };
        let bulkhead = StandardBulkhead::new(config);

        let result = bulkhead.execute(|| test_operation()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");

        let metrics = bulkhead.metrics();
        assert_eq!(metrics.successful_executions, 1);
        assert_eq!(metrics.failed_executions, 0);
        assert_eq!(metrics.rejected_executions, 0);
        assert_eq!(metrics.timed_out_executions, 0);
        assert_eq!(metrics.concurrent_executions, 0);
    }

    #[tokio::test]
    async fn test_bulkhead_failed_execution() {
        let config = BulkheadConfig {
            max_concurrent_calls: 2,
            max_queue_size: 2,
            execution_timeout_seconds: 1,
        };
        let bulkhead = StandardBulkhead::new(config);

        let result = bulkhead.execute(|| test_failing_operation()).await;
        assert!(result.is_err());
        match result {
            Err(BulkheadError::OperationError(_)) => (),
            _ => panic!("Expected OperationError"),
        }

        let metrics = bulkhead.metrics();
        assert_eq!(metrics.successful_executions, 0);
        assert_eq!(metrics.failed_executions, 1);
        assert_eq!(metrics.rejected_executions, 0);
        assert_eq!(metrics.timed_out_executions, 0);
        assert_eq!(metrics.concurrent_executions, 0);
    }

    #[tokio::test]
    async fn test_bulkhead_timeout() {
        let config = BulkheadConfig {
            max_concurrent_calls: 2,
            max_queue_size: 2,
            execution_timeout_seconds: 0, // Set timeout to 0 to force timeout
        };
        let bulkhead = StandardBulkhead::new(config);

        let result = bulkhead.execute(|| test_slow_operation()).await;
        assert!(result.is_err());
        match result {
            Err(BulkheadError::Timeout) => (),
            _ => panic!("Expected Timeout"),
        }

        let metrics = bulkhead.metrics();
        assert_eq!(metrics.successful_executions, 0);
        assert_eq!(metrics.failed_executions, 0);
        assert_eq!(metrics.rejected_executions, 0);
        assert_eq!(metrics.timed_out_executions, 1);
        assert_eq!(metrics.concurrent_executions, 0);
    }

    #[tokio::test]
    async fn test_bulkhead_full() {
        let config = BulkheadConfig {
            max_concurrent_calls: 1,
            max_queue_size: 0, // No queue allowed
            execution_timeout_seconds: 1,
        };
        let bulkhead = Arc::new(StandardBulkhead::new(config));
        
        // Start a long-running operation
        let bulkhead_clone = bulkhead.clone();
        let handle = tokio::spawn(async move {
            bulkhead_clone.execute(|| async {
                sleep(Duration::from_millis(200)).await;
                Ok::<_, TestError>("long operation".to_string())
            }).await
        });
        
        // Give the first operation time to start
        sleep(Duration::from_millis(50)).await;
        
        // Try to execute another operation, which should be rejected
        let result = bulkhead.execute(|| test_operation()).await;
        assert!(result.is_err());
        match result {
            Err(BulkheadError::Full) => (),
            _ => panic!("Expected Full"),
        }
        
        // Wait for the first operation to complete
        let _ = handle.await.unwrap();
        
        let metrics = bulkhead.metrics();
        assert_eq!(metrics.rejected_executions, 1);
    }

    #[tokio::test]
    async fn test_bulkhead_queue() {
        let config = BulkheadConfig {
            max_concurrent_calls: 1,
            max_queue_size: 1, // Allow one operation in the queue
            execution_timeout_seconds: 1,
        };
        let bulkhead = Arc::new(StandardBulkhead::new(config));
        
        // Start a long-running operation
        let bulkhead_clone = bulkhead.clone();
        let handle1 = tokio::spawn(async move {
            bulkhead_clone.execute(|| async {
                sleep(Duration::from_millis(100)).await;
                Ok::<_, TestError>("first operation".to_string())
            }).await
        });
        
        // Give the first operation time to start
        sleep(Duration::from_millis(50)).await;
        
        // Queue a second operation
        let bulkhead_clone = bulkhead.clone();
        let handle2 = tokio::spawn(async move {
            bulkhead_clone.execute(|| async {
                Ok::<_, TestError>("second operation".to_string())
            }).await
        });
        
        // Try to execute a third operation, which should be rejected
        sleep(Duration::from_millis(50)).await;
        let result = bulkhead.execute(|| test_operation()).await;
        assert!(result.is_err());
        match result {
            Err(BulkheadError::Full) => (),
            _ => panic!("Expected Full"),
        }
        
        // Wait for the operations to complete
        let result1 = handle1.await.unwrap();
        let result2 = handle2.await.unwrap();
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        let metrics = bulkhead.metrics();
        assert_eq!(metrics.successful_executions, 2);
        assert_eq!(metrics.rejected_executions, 1);
    }

    #[tokio::test]
    async fn test_reset_metrics() {
        let config = BulkheadConfig {
            max_concurrent_calls: 2,
            max_queue_size: 2,
            execution_timeout_seconds: 1,
        };
        let bulkhead = StandardBulkhead::new(config);

        // Execute a successful operation
        let _ = bulkhead.execute(|| test_operation()).await;
        
        // Execute a failing operation
        let _ = bulkhead.execute(|| test_failing_operation()).await;
        
        // Check metrics before reset
        let metrics = bulkhead.metrics();
        assert_eq!(metrics.successful_executions, 1);
        assert_eq!(metrics.failed_executions, 1);
        
        // Reset metrics
        bulkhead.reset_metrics();
        
        // Check metrics after reset
        let metrics = bulkhead.metrics();
        assert_eq!(metrics.successful_executions, 0);
        assert_eq!(metrics.failed_executions, 0);
        assert_eq!(metrics.concurrent_executions, 0);
    }

    #[tokio::test]
    async fn test_tenant_bulkhead_registry() {
        let registry = TenantBulkheadRegistry::default();
        
        // Get a bulkhead for tenant1
        let tenant1_bulkhead = registry.get_or_create("tenant1");
        
        // Execute an operation for tenant1
        let result = tenant1_bulkhead.execute(|| test_operation()).await;
        assert!(result.is_ok());
        
        // Set a custom config for tenant2
        let tenant2_config = BulkheadConfig {
            max_concurrent_calls: 5,
            max_queue_size: 10,
            execution_timeout_seconds: 2,
        };
        registry.set_tenant_config("tenant2", tenant2_config.clone());
        
        // Get a bulkhead for tenant2
        let tenant2_bulkhead = registry.get_or_create("tenant2");
        
        // Execute an operation for tenant2
        let result = tenant2_bulkhead.execute(|| test_operation()).await;
        assert!(result.is_ok());
        
        // Check that tenant2 has the custom config
        let config = registry.get_tenant_config("tenant2");
        assert_eq!(config.max_concurrent_calls, 5);
        assert_eq!(config.max_queue_size, 10);
        assert_eq!(config.execution_timeout_seconds, 2);
        
        // Reset all metrics
        registry.reset_all_metrics();
        
        // Check that metrics were reset
        let tenant1_metrics = tenant1_bulkhead.metrics();
        let tenant2_metrics = tenant2_bulkhead.metrics();
        assert_eq!(tenant1_metrics.successful_executions, 0);
        assert_eq!(tenant2_metrics.successful_executions, 0);
        
        // Remove tenant1
        registry.remove("tenant1");
        
        // Check that tenant1 is removed
        let bulkheads = registry.get_all();
        assert!(!bulkheads.contains_key("tenant1"));
        assert!(bulkheads.contains_key("tenant2"));
    }

    #[tokio::test]
    async fn test_global_tenant_bulkhead_registry() {
        // Get a bulkhead for tenant1
        let tenant1_bulkhead = get_tenant_bulkhead("tenant1");
        
        // Execute an operation for tenant1
        let result = tenant1_bulkhead.execute(|| test_operation()).await;
        assert!(result.is_ok());
        
        // Set a custom config for tenant2
        let tenant2_config = BulkheadConfig {
            max_concurrent_calls: 5,
            max_queue_size: 10,
            execution_timeout_seconds: 2,
        };
        set_tenant_bulkhead_config("tenant2", tenant2_config);
        
        // Get a bulkhead for tenant2
        let tenant2_bulkhead = get_tenant_bulkhead("tenant2");
        
        // Execute an operation for tenant2
        let result = tenant2_bulkhead.execute(|| test_operation()).await;
        assert!(result.is_ok());
        
        // Get the registry
        let registry = get_tenant_bulkhead_registry();
        
        // Check that tenant2 has the custom config
        let config = registry.get_tenant_config("tenant2");
        assert_eq!(config.max_concurrent_calls, 5);
        assert_eq!(config.max_queue_size, 10);
        assert_eq!(config.execution_timeout_seconds, 2);
    }
} 