use anyhow::Result;
use futures::future::join_all;
use std::sync::Arc;
use tokio::task;
use tracing::{info, warn};

/// Process a batch of items in parallel with a maximum concurrency limit
pub async fn process_batch<T, F, Fut, R>(
    items: Vec<T>,
    max_concurrency: usize,
    process_fn: F,
    request_id: &str,
) -> Result<Vec<R>>
where
    T: Send + Sync + 'static,
    R: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = Result<R>> + Send,
{
    info!(
        request_id = %request_id,
        batch_size = items.len(),
        max_concurrency = max_concurrency,
        "Processing batch of items in parallel"
    );

    // Use a semaphore to limit concurrency
    let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrency));
    
    // Create futures for each item
    let futures = items
        .into_iter()
        .map(|item| {
            let semaphore = semaphore.clone();
            let process_fn = process_fn.clone();
            let request_id = request_id.to_string();
            
            task::spawn(async move {
                // Acquire permit from semaphore
                let _permit = semaphore.acquire().await.unwrap();
                
                // Process the item
                match process_fn(item).await {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        warn!(request_id = %request_id, error = %e, "Error processing item");
                        Err(e)
                    }
                }
            })
        })
        .collect::<Vec<_>>();
    
    // Wait for all futures to complete
    let results = join_all(futures).await;
    
    // Collect results, filtering out errors
    let mut successful_results = Vec::new();
    let mut error_count = 0;
    
    for result in results {
        match result {
            Ok(Ok(item_result)) => successful_results.push(item_result),
            Ok(Err(_)) => error_count += 1,
            Err(_) => error_count += 1,
        }
    }
    
    info!(
        request_id = %request_id,
        successful = successful_results.len(),
        errors = error_count,
        "Batch processing completed"
    );
    
    Ok(successful_results)
}

/// Process a batch of portfolio IDs in parallel
pub async fn process_portfolios<F, Fut, R>(
    portfolio_ids: Vec<String>,
    process_fn: F,
    request_id: &str,
) -> Result<Vec<R>>
where
    R: Send + 'static,
    F: Fn(String) -> Fut + Send + Sync + Clone + 'static,
    Fut: std::future::Future<Output = Result<R>> + Send,
{
    // Use a reasonable concurrency limit to avoid overwhelming the database
    let max_concurrency = 5;
    
    process_batch(portfolio_ids, max_concurrency, process_fn, request_id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_process_batch() {
        // Create a test batch
        let items = vec![1, 2, 3, 4, 5];
        
        // Process function that doubles the input
        let process_fn = |item: i32| async move {
            // Simulate some work
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(item * 2)
        };
        
        // Process the batch
        let results = process_batch(items, 2, process_fn, "test-request").await.unwrap();
        
        // Check results
        assert_eq!(results.len(), 5);
        assert_eq!(results, vec![2, 4, 6, 8, 10]);
    }
    
    #[tokio::test]
    async fn test_process_batch_with_errors() {
        // Create a test batch
        let items = vec![1, 2, 3, 4, 5];
        
        // Process function that fails for even numbers
        let process_fn = |item: i32| async move {
            // Simulate some work
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            if item % 2 == 0 {
                Err(anyhow::anyhow!("Even number"))
            } else {
                Ok(item * 2)
            }
        };
        
        // Process the batch
        let results = process_batch(items, 2, process_fn, "test-request").await.unwrap();
        
        // Check results - should only have odd numbers doubled
        assert_eq!(results.len(), 3);
        assert_eq!(results, vec![2, 6, 10]);
    }
} 