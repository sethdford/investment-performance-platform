//! API endpoint definitions

use crate::models::{
    Portfolio, Transaction, Account, Security, Client, Benchmark, Price, Position,
    PerformanceMetrics, ApiResponse, ErrorResponse,
};
use crate::auth::{AuthMiddleware, Claims};
use crate::validation;
use shared::{AppError, repository::PaginationOptions};
use lambda_http::{Request, Response, Body};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{info, error};
use shared::metrics::{MetricsCollector, MetricsMiddleware};

/// API endpoint handler trait
#[async_trait::async_trait]
pub trait ApiHandler {
    /// Handle a request to the API
    async fn handle_request(&self, request: Request) -> Result<Response<Body>, AppError>;
}

/// Portfolio API endpoints
pub struct PortfolioApi<R: shared::repository::Repository> {
    repository: R,
    auth_middleware: AuthMiddleware,
}

impl<R: shared::repository::Repository> PortfolioApi<R> {
    /// Create a new portfolio API
    pub fn new(repository: R, auth_middleware: AuthMiddleware) -> Self {
        Self {
            repository,
            auth_middleware,
        }
    }
    
    /// Get a portfolio by ID
    async fn get_portfolio(&self, id: &str, claims: &Claims) -> Result<Response<Body>, AppError> {
        info!("Getting portfolio with ID: {}", id);
        
        // Verify tenant access
        self.auth_middleware.verify_tenant_access(claims, id).await?;
        
        // Get the portfolio
        let portfolio = self.repository.get_portfolio(id).await?
            .ok_or_else(|| AppError::NotFound(format!("Portfolio not found: {}", id)))?;
        
        // Build response
        let response = json!({
            "portfolio": portfolio,
        });
        
        Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::from(response.to_string()))
            .map_err(|e| AppError::Internal(format!("Failed to build response: {}", e)))?)
    }
    
    /// List portfolios
    async fn list_portfolios(
        &self,
        query_params: &HashMap<String, String>,
        claims: &Claims,
    ) -> Result<Response<Body>, AppError> {
        info!("Listing portfolios");
        
        // Extract pagination parameters
        let limit = query_params.get("limit").and_then(|v| v.parse::<u32>().ok());
        let next_token = query_params.get("next_token").cloned();
        
        // Extract filter parameters
        let client_id = query_params.get("client_id").map(|s| s.as_str());
        
        // Verify tenant access if client_id is provided
        if let Some(client_id) = client_id {
            self.auth_middleware.verify_tenant_access(claims, client_id).await?;
        }
        
        // Create pagination options
        let pagination = Some(PaginationOptions {
            limit,
            next_token,
        });
        
        // Query the repository
        let result = self.repository.list_portfolios(client_id, pagination).await?;
        
        // Build response
        let response = json!({
            "items": result.items,
            "next_token": result.next_token,
        });
        
        Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::from(response.to_string()))
            .map_err(|e| AppError::Internal(format!("Failed to build response: {}", e)))?)
    }
    
    /// Create a portfolio
    async fn create_portfolio(
        &self,
        body: Value,
        claims: &Claims,
    ) -> Result<Response<Body>, AppError> {
        info!("Creating portfolio");
        
        // Validate the request
        let portfolio = validation::validate_portfolio_request(&body)?;
        
        // Verify tenant access
        self.auth_middleware.verify_tenant_access(claims, &portfolio.client_id).await?;
        
        // Create the portfolio
        self.repository.put_portfolio(&portfolio).await?;
        
        // Build response
        let response = json!({
            "portfolio": portfolio,
        });
        
        Ok(Response::builder()
            .status(201)
            .header("Content-Type", "application/json")
            .body(Body::from(response.to_string()))
            .map_err(|e| AppError::Internal(format!("Failed to build response: {}", e)))?)
    }
    
    /// Update a portfolio
    async fn update_portfolio(
        &self,
        id: &str,
        body: Value,
        claims: &Claims,
    ) -> Result<Response<Body>, AppError> {
        info!("Updating portfolio with ID: {}", id);
        
        // Validate the request
        let mut portfolio = validation::validate_portfolio_request(&body)?;
        
        // Ensure ID in path matches ID in body
        if portfolio.id != id {
            return Err(AppError::Validation(format!("Portfolio ID in path ({}) does not match ID in body ({})", id, portfolio.id)));
        }
        
        // Verify tenant access
        self.auth_middleware.verify_tenant_access(claims, id).await?;
        
        // Get the existing portfolio to verify it exists
        let existing_portfolio = self.repository.get_portfolio(id).await?
            .ok_or_else(|| AppError::NotFound(format!("Portfolio not found: {}", id)))?;
        
        // Update the portfolio
        portfolio.created_at = existing_portfolio.created_at; // Preserve created_at
        self.repository.put_portfolio(&portfolio).await?;
        
        // Build response
        let response = json!({
            "portfolio": portfolio,
        });
        
        Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Body::from(response.to_string()))
            .map_err(|e| AppError::Internal(format!("Failed to build response: {}", e)))?)
    }
    
    /// Delete a portfolio
    async fn delete_portfolio(
        &self,
        id: &str,
        claims: &Claims,
    ) -> Result<Response<Body>, AppError> {
        info!("Deleting portfolio with ID: {}", id);
        
        // Verify tenant access
        self.auth_middleware.verify_tenant_access(claims, id).await?;
        
        // Delete the portfolio
        self.repository.delete_portfolio(id).await?;
        
        // Build response
        Ok(Response::builder()
            .status(204)
            .body(Body::empty())
            .map_err(|e| AppError::Internal(format!("Failed to build response: {}", e)))?)
    }
}

#[async_trait::async_trait]
impl<R: shared::repository::Repository + Send + Sync> ApiHandler for PortfolioApi<R> {
    async fn handle_request(&self, request: Request) -> Result<Response<Body>, AppError> {
        // Extract request components
        let method = request.method().as_str();
        let path = request.uri().path();
        let query_params: HashMap<String, String> = request.uri().query()
            .map(|q| url::form_urlencoded::parse(q.as_bytes())
                .into_owned()
                .collect())
            .unwrap_or_default();
        
        // Authenticate the request
        let claims = self.auth_middleware.authenticate(&request).await?;
        
        // Parse request body if present
        let body = match request.body() {
            Body::Text(text) => serde_json::from_str(text)?,
            Body::Binary(binary) => serde_json::from_slice(binary)?,
            Body::Empty => json!({}),
        };
        
        // Route the request
        match (method, path) {
            // Get a portfolio
            ("GET", p) if p.starts_with("/api/portfolios/") => {
                let id = p.trim_start_matches("/api/portfolios/");
                self.get_portfolio(id, &claims).await
            },
            
            // List portfolios
            ("GET", "/api/portfolios") => {
                self.list_portfolios(&query_params, &claims).await
            },
            
            // Create a portfolio
            ("POST", "/api/portfolios") => {
                self.create_portfolio(body, &claims).await
            },
            
            // Update a portfolio
            ("PUT", p) if p.starts_with("/api/portfolios/") => {
                let id = p.trim_start_matches("/api/portfolios/");
                self.update_portfolio(id, body, &claims).await
            },
            
            // Delete a portfolio
            ("DELETE", p) if p.starts_with("/api/portfolios/") => {
                let id = p.trim_start_matches("/api/portfolios/");
                self.delete_portfolio(id, &claims).await
            },
            
            // Unknown endpoint
            _ => {
                Err(AppError::NotFound(format!("Endpoint not found: {} {}", method, path)))
            }
        }
    }
}

/// Metrics API
pub struct MetricsApi {
    /// Metrics collector
    metrics_collector: MetricsCollector,
}

impl MetricsApi {
    /// Create a new metrics API
    pub fn new(metrics_collector: MetricsCollector) -> Self {
        Self {
            metrics_collector,
        }
    }
    
    /// Get metrics in Prometheus format
    async fn get_metrics(&self) -> Result<Response<Body>, AppError> {
        let metrics_middleware = MetricsMiddleware::new(self.metrics_collector.clone());
        let metrics = metrics_middleware.get_prometheus_metrics();
        
        Ok(Response::builder()
            .status(200)
            .header("Content-Type", "text/plain")
            .body(Body::from(metrics))
            .map_err(|e| AppError::Internal(format!("Failed to build response: {}", e)))?)
    }
}

#[async_trait::async_trait]
impl ApiHandler for MetricsApi {
    async fn handle_request(&self, request: Request) -> Result<Response<Body>, AppError> {
        // Extract request components
        let method = request.method().as_str();
        let path = request.uri().path();
        
        // Route the request
        match (method, path) {
            // Get metrics
            ("GET", "/metrics") => {
                self.get_metrics().await
            },
            
            // Unknown endpoint
            _ => {
                Err(AppError::NotFound(format!("Endpoint not found: {} {}", method, path)))
            }
        }
    }
}

// Similarly implement other API endpoints (TransactionApi, AccountApi, etc.) 