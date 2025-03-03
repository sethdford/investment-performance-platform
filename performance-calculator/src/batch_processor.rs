use crate::{
    calculator::PerformanceCalculator,
    models::{CalculationResult, CalculationRequest},
    error::AppError,
};
use shared::repository::Repository;
use std::collections::HashMap;
use tokio::task;
use futures::{stream, StreamExt};
use tracing::{info, warn, error, debug, instrument};
use std::sync::Arc;

/// Batch calculation request
#[derive(Debug, Clone)]
pub struct BatchCalculationRequest {
    /// Portfolio IDs
    pub portfolio_ids: Vec<String>,
    /// Start date
    pub start_date: chrono::NaiveDate,
    /// End date
    pub end_date: chrono::NaiveDate,
    /// Include details flag
    pub include_details: bool,
}

/// Batch calculation result
#[derive(Debug, Clone)]
pub struct BatchCalculationResult {
    /// Results by portfolio ID
    pub results: HashMap<String, Result<CalculationResult, String>>,
    /// Duration in milliseconds
    pub duration_ms: f64,
}

/// Batch processor for performance calculations
pub struct BatchProcessor<R: Repository> {
    /// Repository
    repository: R,
    /// Maximum batch size
    max_batch_size: usize,
    /// Maximum concurrency
    max_concurrency: usize,
}

impl<R: Repository + Clone + Send + Sync + 'static> BatchProcessor<R> {
    /// Create a new batch processor
    pub fn new(repository: R, max_batch_size: usize, max_concurrency: usize) -> Self {
        Self {
            repository,
            max_batch_size,
            max_concurrency,
        }
    }

    /// Process a batch calculation request
    #[instrument(skip(self), fields(portfolio_count = request.portfolio_ids.len()))]
    pub async fn process(&self, request: BatchCalculationRequest) -> Result<BatchCalculationResult, AppError> {
        let start_time = std::time::Instant::now();
        
        info!(
            "Processing batch calculation for {} portfolios from {} to {}",
            request.portfolio_ids.len(),
            request.start_date,
            request.end_date
        );
        
        // Split portfolios into batches
        let batches = self.split_into_batches(&request.portfolio_ids);
        debug!("Split into {} batches", batches.len());
        
        // Create shared repository
        let repository = Arc::new(self.repository.clone());
        
        // Process batches concurrently
        let mut results = HashMap::new();
        
        let batch_results = stream::iter(batches)
            .map(|batch| {
                let repository = repository.clone();
                let request = request.clone();
                
                async move {
                    let mut batch_results = HashMap::new();
                    
                    // Process each portfolio in the batch
                    for portfolio_id in batch {
                        let calculator = PerformanceCalculator::new(repository.as_ref().clone());
                        
                        let calculation_request = CalculationRequest {
                            portfolio_id: portfolio_id.clone(),
                            start_date: request.start_date,
                            end_date: request.end_date,
                            include_details: request.include_details,
                        };
                        
                        match calculator.calculate(calculation_request).await {
                            Ok(result) => {
                                batch_results.insert(portfolio_id, Ok(result));
                            },
                            Err(e) => {
                                error!("Failed to calculate performance for portfolio {}: {}", portfolio_id, e);
                                batch_results.insert(portfolio_id, Err(e.to_string()));
                            }
                        }
                    }
                    
                    batch_results
                }
            })
            .buffer_unordered(self.max_concurrency)
            .collect::<Vec<HashMap<String, Result<CalculationResult, String>>>>()
            .await;
        
        // Combine batch results
        for batch_result in batch_results {
            results.extend(batch_result);
        }
        
        let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        
        info!(
            "Completed batch calculation for {} portfolios in {:.2} ms",
            request.portfolio_ids.len(),
            duration_ms
        );
        
        Ok(BatchCalculationResult {
            results,
            duration_ms,
        })
    }

    /// Split portfolio IDs into batches
    fn split_into_batches(&self, portfolio_ids: &[String]) -> Vec<Vec<String>> {
        let mut batches = Vec::new();
        let mut current_batch = Vec::new();
        
        for portfolio_id in portfolio_ids {
            current_batch.push(portfolio_id.clone());
            
            if current_batch.len() >= self.max_batch_size {
                batches.push(current_batch);
                current_batch = Vec::new();
            }
        }
        
        if !current_batch.is_empty() {
            batches.push(current_batch);
        }
        
        batches
    }
} 