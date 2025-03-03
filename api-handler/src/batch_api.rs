//! Batch API for performance calculations

use lambda_http::{Request, Response, Body};
use serde_json::json;
use shared::{
    AppError,
    validation,
    models::Portfolio,
};
use performance_calculator::batch_processor::{BatchProcessor, BatchCalculationRequest};
use chrono::NaiveDate;
use tracing::{info, error};
use crate::auth::Claims;

/// Batch API handler
pub struct BatchApi<R: shared::repository::Repository> {
    /// Repository
    repository: R,
}

impl<R: shared::repository::Repository + Clone + Send + Sync + 'static> BatchApi<R> {
    /// Create a new batch API handler
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
    
    /// Handle batch calculation request
    pub async fn batch_calculate(
        &self,
        request: &serde_json::Value,
        claims: &Claims,
    ) -> Result<Response<Body>, AppError> {
        info!("Processing batch calculation request");
        
        // Validate request
        let portfolio_ids = match request.get("portfolio_ids") {
            Some(ids) if ids.is_array() => {
                ids.as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|id| id.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            },
            _ => {
                return Err(AppError::Validation("portfolio_ids must be an array of strings".to_string()));
            }
        };
        
        if portfolio_ids.is_empty() {
            return Err(AppError::Validation("portfolio_ids cannot be empty".to_string()));
        }
        
        // Validate start_date
        let start_date = match request.get("start_date") {
            Some(date) if date.is_string() => {
                let date_str = date.as_str().unwrap();
                NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                    .map_err(|_| AppError::Validation(format!("Invalid start_date format: {}", date_str)))?
            },
            _ => {
                return Err(AppError::Validation("start_date is required and must be a string in YYYY-MM-DD format".to_string()));
            }
        };
        
        // Validate end_date
        let end_date = match request.get("end_date") {
            Some(date) if date.is_string() => {
                let date_str = date.as_str().unwrap();
                NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                    .map_err(|_| AppError::Validation(format!("Invalid end_date format: {}", date_str)))?
            },
            _ => {
                return Err(AppError::Validation("end_date is required and must be a string in YYYY-MM-DD format".to_string()));
            }
        };
        
        // Validate date range
        if end_date < start_date {
            return Err(AppError::Validation("end_date cannot be before start_date".to_string()));
        }
        
        // Get include_details flag
        let include_details = request.get("include_details")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        // Create batch processor
        let batch_processor = BatchProcessor::new(
            self.repository.clone(),
            10, // max_batch_size
            4,  // max_concurrency
        );
        
        // Create batch calculation request
        let batch_request = BatchCalculationRequest {
            portfolio_ids,
            start_date,
            end_date,
            include_details,
        };
        
        // Process batch calculation
        let result = batch_processor.process(batch_request).await?;
        
        // Build response
        let response = json!({
            "results": result.results,
            "duration_ms": result.duration_ms,
        });
        
        Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::from(response.to_string()))
            .map_err(|e| error!("Failed to build response: {}", e))
        )
    }
} 