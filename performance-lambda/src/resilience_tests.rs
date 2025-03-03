#[cfg(test)]
mod resilience_tests {
    use crate::RepositoryError;
    use aws_sdk_dynamodb::Error as DynamoDbError;
    use aws_sdk_dynamodb::error::ProvisionedThroughputExceededException;
    use aws_sdk_dynamodb::error::ThrottlingException;
    use aws_sdk_dynamodb::error::InternalServerError;
    use aws_sdk_dynamodb::error::ServiceUnavailable;
    use performance_calculator::resilience::{CircuitBreaker, CircuitBreakerState, RetryPolicy};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::Mutex;
    use tokio::time::sleep;
    
    // Mock function that simulates a DynamoDB operation with failures
    struct MockDynamoDbOperation {
        failure_count: Arc<Mutex<usize>>,
        max_failures: usize,
        failure_type: FailureType,
    }
    
    enum FailureType {
        ProvisionedThroughputExceeded,
        Throttling,
        InternalServer,
        ServiceUnavailable,
        Permanent,
    }
    
    impl MockDynamoDbOperation {
        fn new(max_failures: usize, failure_type: FailureType) -> Self {
            Self {
                failure_count: Arc::new(Mutex::new(0)),
                max_failures,
                failure_type,
            }
        }
        
        async fn execute(&self) -> Result<String, DynamoDbError> {
            let mut count = self.failure_count.lock().await;
            
            if *count < self.max_failures {
                *count += 1;
                
                match self.failure_type {
                    FailureType::ProvisionedThroughputExceeded => {
                        Err(DynamoDbError::from(ProvisionedThroughputExceededException::builder().build()))
                    },
                    FailureType::Throttling => {
                        Err(DynamoDbError::from(ThrottlingException::builder().build()))
                    },
                    FailureType::InternalServer => {
                        Err(DynamoDbError::from(InternalServerError::builder().build()))
                    },
                    FailureType::ServiceUnavailable => {
                        Err(DynamoDbError::from(ServiceUnavailable::builder().build()))
                    },
                    FailureType::Permanent => {
                        Err(DynamoDbError::Unhandled(Box::new("Permanent error")))
                    },
                }
            } else {
                Ok("Success".to_string())
            }
        }
    }
    
    #[tokio::test]
    async fn test_circuit_breaker() {
        // Create a circuit breaker
        let circuit_breaker = CircuitBreaker::new(
            "test-circuit-breaker",
            3,                      // failure_threshold
            Duration::from_secs(1), // reset_timeout
        );
        
        // Initially, the circuit breaker should be closed
        assert_eq!(circuit_breaker.state(), CircuitBreakerState::Closed);
        
        // Simulate 3 failures
        for _ in 0..3 {
            circuit_breaker.record_failure();
        }
        
        // After 3 failures, the circuit breaker should be open
        assert_eq!(circuit_breaker.state(), CircuitBreakerState::Open);
        
        // Wait for the reset timeout
        sleep(Duration::from_secs(1)).await;
        
        // After the reset timeout, the circuit breaker should be half-open
        assert_eq!(circuit_breaker.state(), CircuitBreakerState::HalfOpen);
        
        // Record a success
        circuit_breaker.record_success();
        
        // After a success in half-open state, the circuit breaker should be closed
        assert_eq!(circuit_breaker.state(), CircuitBreakerState::Closed);
    }
    
    #[tokio::test]
    async fn test_retry_policy_with_transient_errors() {
        // Create a retry policy
        let retry_policy = RetryPolicy::new(
            3,                       // max_retries
            Duration::from_millis(10), // initial_backoff
            2.0,                     // backoff_multiplier
            Duration::from_millis(100), // max_backoff
            0.1,                     // jitter
        );
        
        // Create a mock DynamoDB operation that fails 2 times with a transient error
        let operation = MockDynamoDbOperation::new(2, FailureType::ProvisionedThroughputExceeded);
        
        // Execute the operation with the retry policy
        let result = retry_policy.execute(
            || async {
                operation.execute().await.map_err(|e| RepositoryError::DynamoDb(e))
            }
        ).await;
        
        // The operation should eventually succeed
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
    }
    
    #[tokio::test]
    async fn test_retry_policy_with_permanent_errors() {
        // Create a retry policy
        let retry_policy = RetryPolicy::new(
            3,                       // max_retries
            Duration::from_millis(10), // initial_backoff
            2.0,                     // backoff_multiplier
            Duration::from_millis(100), // max_backoff
            0.1,                     // jitter
        );
        
        // Create a mock DynamoDB operation that always fails with a permanent error
        let operation = MockDynamoDbOperation::new(10, FailureType::Permanent);
        
        // Execute the operation with the retry policy
        let result = retry_policy.execute(
            || async {
                operation.execute().await.map_err(|e| RepositoryError::DynamoDb(e))
            }
        ).await;
        
        // The operation should fail after all retries
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_circuit_breaker_with_retry_policy() {
        // Create a circuit breaker
        let circuit_breaker = CircuitBreaker::new(
            "test-circuit-breaker",
            3,                      // failure_threshold
            Duration::from_secs(1), // reset_timeout
        );
        
        // Create a retry policy
        let retry_policy = RetryPolicy::new(
            3,                       // max_retries
            Duration::from_millis(10), // initial_backoff
            2.0,                     // backoff_multiplier
            Duration::from_millis(100), // max_backoff
            0.1,                     // jitter
        );
        
        // Create a mock DynamoDB operation that fails 10 times with a transient error
        let operation = MockDynamoDbOperation::new(10, FailureType::ProvisionedThroughputExceeded);
        
        // Execute the operation with the circuit breaker and retry policy
        let result = async {
            if circuit_breaker.state() == CircuitBreakerState::Open {
                return Err(RepositoryError::CircuitBreakerOpen("Circuit breaker is open".to_string()));
            }
            
            let result = retry_policy.execute(
                || async {
                    operation.execute().await.map_err(|e| RepositoryError::DynamoDb(e))
                }
            ).await;
            
            match result {
                Ok(value) => {
                    circuit_breaker.record_success();
                    Ok(value)
                },
                Err(err) => {
                    circuit_breaker.record_failure();
                    Err(err)
                },
            }
        }.await;
        
        // The operation should fail because the circuit breaker will open after 3 failures
        assert!(result.is_err());
        
        // The circuit breaker should be open
        assert_eq!(circuit_breaker.state(), CircuitBreakerState::Open);
        
        // Wait for the reset timeout
        sleep(Duration::from_secs(1)).await;
        
        // After the reset timeout, the circuit breaker should be half-open
        assert_eq!(circuit_breaker.state(), CircuitBreakerState::HalfOpen);
        
        // Create a new mock DynamoDB operation that succeeds immediately
        let operation = MockDynamoDbOperation::new(0, FailureType::ProvisionedThroughputExceeded);
        
        // Execute the operation with the circuit breaker and retry policy
        let result = async {
            if circuit_breaker.state() == CircuitBreakerState::Open {
                return Err(RepositoryError::CircuitBreakerOpen("Circuit breaker is open".to_string()));
            }
            
            let result = retry_policy.execute(
                || async {
                    operation.execute().await.map_err(|e| RepositoryError::DynamoDb(e))
                }
            ).await;
            
            match result {
                Ok(value) => {
                    circuit_breaker.record_success();
                    Ok(value)
                },
                Err(err) => {
                    circuit_breaker.record_failure();
                    Err(err)
                },
            }
        }.await;
        
        // The operation should succeed
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
        
        // The circuit breaker should be closed
        assert_eq!(circuit_breaker.state(), CircuitBreakerState::Closed);
    }
    
    #[tokio::test]
    async fn test_health_check() {
        // Create a health check for DynamoDB
        let health_check = |healthy: bool| {
            async move {
                if healthy {
                    Ok(())
                } else {
                    Err("DynamoDB is unhealthy".to_string())
                }
            }
        };
        
        // Register the health check
        performance_calculator::resilience::register_health_check("dynamodb", health_check(true));
        
        // Get the health check results
        let results = performance_calculator::resilience::get_health_check_results().await;
        
        // The health check should be healthy
        assert!(results.contains_key("dynamodb"));
        assert!(results["dynamodb"].is_ok());
        
        // Register an unhealthy health check
        performance_calculator::resilience::register_health_check("dynamodb", health_check(false));
        
        // Get the health check results
        let results = performance_calculator::resilience::get_health_check_results().await;
        
        // The health check should be unhealthy
        assert!(results.contains_key("dynamodb"));
        assert!(results["dynamodb"].is_err());
        assert_eq!(results["dynamodb"].as_ref().unwrap_err(), "DynamoDB is unhealthy");
    }
} 