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
use crate::visualization::{
    VisualizationService, PerformanceChartParams, ComparisonChartParams,
    RiskReturnChartParams, AllocationChartParams, AttributionChartParams,
    ChartFormat, TimeInterval, AllocationChartType, AttributionChartType
};
use std::str::FromStr;
use chrono::NaiveDate;
use std::sync::Arc;
use std::time::Duration;

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

pub struct Api {
    // Existing fields
    visualization_service: Arc<VisualizationService>,
}

impl Api {
    // Update constructor to include visualization service
    pub fn new(
        dynamodb_repo: Arc<DynamoDbRepository>,
        timestream_repo: Arc<TimestreamRepository>,
        auth_service: Arc<AuthService>,
        cache: Arc<Cache>,
    ) -> Self {
        let visualization_service = Arc::new(VisualizationService::new(
            timestream_repo.clone(),
            dynamodb_repo.clone(),
        ));
        
        Self {
            dynamodb_repo,
            timestream_repo,
            auth_service,
            cache,
            visualization_service,
        }
    }

    // Add visualization endpoints
    
    /// Generate a performance chart for a portfolio
    pub async fn generate_performance_chart(
        &self,
        portfolio_id: String,
        query_params: HashMap<String, String>,
        _user_id: String,
    ) -> Result<Response<Body>, AppError> {
        // Parse query parameters
        let start_date = parse_date_param(&query_params, "start_date")?;
        let end_date = parse_date_param(&query_params, "end_date")?;
        let benchmark_id = query_params.get("benchmark_id").cloned();
        
        let metrics = query_params
            .get("metrics")
            .map(|m| m.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|| vec!["twr".to_string()]);
        
        let interval = query_params
            .get("interval")
            .and_then(|i| TimeInterval::from_str(i).ok())
            .unwrap_or_default();
        
        let width = query_params
            .get("width")
            .and_then(|w| w.parse::<u32>().ok())
            .unwrap_or(800);
        
        let height = query_params
            .get("height")
            .and_then(|h| h.parse::<u32>().ok())
            .unwrap_or(400);
        
        let format = query_params
            .get("format")
            .and_then(|f| ChartFormat::from_str(f).ok())
            .unwrap_or_default();
        
        // Create parameters
        let params = PerformanceChartParams {
            portfolio_id,
            start_date,
            end_date,
            benchmark_id,
            metrics,
            interval,
            width,
            height,
            format,
        };
        
        // Generate chart
        let (chart_data, content_type) = self.visualization_service.generate_performance_chart(params).await?;
        
        // Return response with binary data
        let response = Response::builder()
            .status(200)
            .header("Content-Type", content_type)
            .header("Content-Disposition", format!("inline; filename=\"performance_chart.{}\"", format.extension()))
            .body(Body::from(chart_data))
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        
        Ok(response)
    }
    
    /// Generate a comparison chart for multiple portfolios
    pub async fn generate_comparison_chart(
        &self,
        query_params: HashMap<String, String>,
        _user_id: String,
    ) -> Result<Response<Body>, AppError> {
        // Parse query parameters
        let portfolio_ids = query_params
            .get("portfolio_ids")
            .ok_or_else(|| AppError::BadRequest("portfolio_ids parameter is required".to_string()))?
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();
        
        if portfolio_ids.is_empty() {
            return Err(AppError::BadRequest("No portfolio IDs provided".to_string()));
        }
        
        let start_date = parse_date_param(&query_params, "start_date")?;
        let end_date = parse_date_param(&query_params, "end_date")?;
        let benchmark_id = query_params.get("benchmark_id").cloned();
        
        let metric = query_params
            .get("metric")
            .cloned()
            .unwrap_or_else(|| "twr".to_string());
        
        let interval = query_params
            .get("interval")
            .and_then(|i| TimeInterval::from_str(i).ok())
            .unwrap_or_default();
        
        let width = query_params
            .get("width")
            .and_then(|w| w.parse::<u32>().ok())
            .unwrap_or(800);
        
        let height = query_params
            .get("height")
            .and_then(|h| h.parse::<u32>().ok())
            .unwrap_or(400);
        
        let format = query_params
            .get("format")
            .and_then(|f| ChartFormat::from_str(f).ok())
            .unwrap_or_default();
        
        // Create parameters
        let params = ComparisonChartParams {
            portfolio_ids,
            start_date,
            end_date,
            benchmark_id,
            metric,
            interval,
            width,
            height,
            format,
        };
        
        // Generate chart
        let (chart_data, content_type) = self.visualization_service.generate_comparison_chart(params).await?;
        
        // Return response with binary data
        let response = Response::builder()
            .status(200)
            .header("Content-Type", content_type)
            .header("Content-Disposition", format!("inline; filename=\"comparison_chart.{}\"", format.extension()))
            .body(Body::from(chart_data))
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        
        Ok(response)
    }
    
    /// Generate a risk-return chart for multiple portfolios
    pub async fn generate_risk_return_chart(
        &self,
        query_params: HashMap<String, String>,
        _user_id: String,
    ) -> Result<Response<Body>, AppError> {
        // Parse query parameters
        let portfolio_ids = query_params
            .get("portfolio_ids")
            .ok_or_else(|| AppError::BadRequest("portfolio_ids parameter is required".to_string()))?
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();
        
        if portfolio_ids.is_empty() {
            return Err(AppError::BadRequest("No portfolio IDs provided".to_string()));
        }
        
        let start_date = parse_date_param(&query_params, "start_date")?;
        let end_date = parse_date_param(&query_params, "end_date")?;
        let benchmark_id = query_params.get("benchmark_id").cloned();
        
        let return_metric = query_params
            .get("return_metric")
            .cloned()
            .unwrap_or_else(|| "twr".to_string());
        
        let risk_metric = query_params
            .get("risk_metric")
            .cloned()
            .unwrap_or_else(|| "volatility".to_string());
        
        let width = query_params
            .get("width")
            .and_then(|w| w.parse::<u32>().ok())
            .unwrap_or(800);
        
        let height = query_params
            .get("height")
            .and_then(|h| h.parse::<u32>().ok())
            .unwrap_or(400);
        
        let format = query_params
            .get("format")
            .and_then(|f| ChartFormat::from_str(f).ok())
            .unwrap_or_default();
        
        // Create parameters
        let params = RiskReturnChartParams {
            portfolio_ids,
            start_date,
            end_date,
            benchmark_id,
            return_metric,
            risk_metric,
            width,
            height,
            format,
        };
        
        // Generate chart
        let (chart_data, content_type) = self.visualization_service.generate_risk_return_chart(params).await?;
        
        // Return response with binary data
        let response = Response::builder()
            .status(200)
            .header("Content-Type", content_type)
            .header("Content-Disposition", format!("inline; filename=\"risk_return_chart.{}\"", format.extension()))
            .body(Body::from(chart_data))
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        
        Ok(response)
    }
    
    /// Generate an allocation chart for a portfolio
    pub async fn generate_allocation_chart(
        &self,
        portfolio_id: String,
        query_params: HashMap<String, String>,
        _user_id: String,
    ) -> Result<Response<Body>, AppError> {
        // Parse query parameters
        let date = query_params
            .get("date")
            .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
            .unwrap_or_else(|| chrono::Utc::now().date_naive());
        
        let group_by = query_params
            .get("group_by")
            .cloned()
            .unwrap_or_else(|| "type".to_string());
        
        let chart_type = query_params
            .get("chart_type")
            .and_then(|t| AllocationChartType::from_str(t).ok())
            .unwrap_or_default();
        
        let width = query_params
            .get("width")
            .and_then(|w| w.parse::<u32>().ok())
            .unwrap_or(800);
        
        let height = query_params
            .get("height")
            .and_then(|h| h.parse::<u32>().ok())
            .unwrap_or(400);
        
        let format = query_params
            .get("format")
            .and_then(|f| ChartFormat::from_str(f).ok())
            .unwrap_or_default();
        
        // Create parameters
        let params = AllocationChartParams {
            portfolio_id,
            date,
            group_by,
            chart_type,
            width,
            height,
            format,
        };
        
        // Generate chart
        let (chart_data, content_type) = self.visualization_service.generate_allocation_chart(params).await?;
        
        // Return response with binary data
        let response = Response::builder()
            .status(200)
            .header("Content-Type", content_type)
            .header("Content-Disposition", format!("inline; filename=\"allocation_chart.{}\"", format.extension()))
            .body(Body::from(chart_data))
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        
        Ok(response)
    }
    
    /// Generate an attribution chart for a portfolio
    pub async fn generate_attribution_chart(
        &self,
        portfolio_id: String,
        query_params: HashMap<String, String>,
        _user_id: String,
    ) -> Result<Response<Body>, AppError> {
        // Parse query parameters
        let start_date = parse_date_param(&query_params, "start_date")?;
        let end_date = parse_date_param(&query_params, "end_date")?;
        
        let group_by = query_params
            .get("group_by")
            .cloned()
            .unwrap_or_else(|| "type".to_string());
        
        let chart_type = query_params
            .get("chart_type")
            .and_then(|t| AttributionChartType::from_str(t).ok())
            .unwrap_or_default();
        
        let width = query_params
            .get("width")
            .and_then(|w| w.parse::<u32>().ok())
            .unwrap_or(800);
        
        let height = query_params
            .get("height")
            .and_then(|h| h.parse::<u32>().ok())
            .unwrap_or(400);
        
        let format = query_params
            .get("format")
            .and_then(|f| ChartFormat::from_str(f).ok())
            .unwrap_or_default();
        
        // Create parameters
        let params = AttributionChartParams {
            portfolio_id,
            start_date,
            end_date,
            group_by,
            chart_type,
            width,
            height,
            format,
        };
        
        // Generate chart
        let (chart_data, content_type) = self.visualization_service.generate_attribution_chart(params).await?;
        
        // Return response with binary data
        let response = Response::builder()
            .status(200)
            .header("Content-Type", content_type)
            .header("Content-Disposition", format!("inline; filename=\"attribution_chart.{}\"", format.extension()))
            .body(Body::from(chart_data))
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        
        Ok(response)
    }
}

// Helper function to parse date parameters
fn parse_date_param(
    query_params: &HashMap<String, String>,
    param_name: &str,
) -> Result<NaiveDate, AppError> {
    query_params
        .get(param_name)
        .ok_or_else(|| AppError::BadRequest(format!("{} parameter is required", param_name)))?
        .parse::<NaiveDate>()
        .map_err(|_| AppError::BadRequest(format!("Invalid {} format, expected YYYY-MM-DD", param_name)))
}

// Add FromStr implementations for enum types
impl FromStr for ChartFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "png" => Ok(ChartFormat::Png),
            "jpg" | "jpeg" => Ok(ChartFormat::Jpg),
            "svg" => Ok(ChartFormat::Svg),
            "pdf" => Ok(ChartFormat::Pdf),
            _ => Err(format!("Unknown chart format: {}", s)),
        }
    }
}

impl FromStr for TimeInterval {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "daily" => Ok(TimeInterval::Daily),
            "weekly" => Ok(TimeInterval::Weekly),
            "monthly" => Ok(TimeInterval::Monthly),
            "quarterly" => Ok(TimeInterval::Quarterly),
            "yearly" => Ok(TimeInterval::Yearly),
            _ => Err(format!("Unknown time interval: {}", s)),
        }
    }
}

impl FromStr for AllocationChartType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pie" => Ok(AllocationChartType::Pie),
            "donut" => Ok(AllocationChartType::Donut),
            "bar" => Ok(AllocationChartType::Bar),
            "treemap" => Ok(AllocationChartType::Treemap),
            _ => Err(format!("Unknown allocation chart type: {}", s)),
        }
    }
}

impl FromStr for AttributionChartType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bar" => Ok(AttributionChartType::Bar),
            "stacked_bar" => Ok(AttributionChartType::StackedBar),
            "waterfall" => Ok(AttributionChartType::Waterfall),
            _ => Err(format!("Unknown attribution chart type: {}", s)),
        }
    }
}

/// Visualization API endpoints
pub struct VisualizationApi {
    visualization_service: Arc<VisualizationService>,
    auth_middleware: AuthMiddleware,
}

impl VisualizationApi {
    /// Create a new visualization API
    pub fn new(
        visualization_service: Arc<VisualizationService>,
        auth_middleware: AuthMiddleware,
    ) -> Self {
        Self {
            visualization_service,
            auth_middleware,
        }
    }
    
    // Move the visualization methods from Api struct to here
    // generate_performance_chart, generate_comparison_chart, etc.
}

#[async_trait::async_trait]
impl ApiHandler for VisualizationApi {
    async fn handle_request(&self, request: Request) -> Result<Response<Body>, AppError> {
        let path = request.uri().path();
        let method = request.method().as_str();
        let query_params = extract_query_params(request.uri().query());
        
        // Extract auth claims
        let claims = match self.auth_middleware.extract_claims(&request).await {
            Ok(claims) => claims,
            Err(e) => return Err(e),
        };
        
        // Handle visualization routes
        if path.starts_with("/portfolios/") && path.contains("/charts/") {
            // Extract portfolio_id and chart type
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 5 && parts[1] == "portfolios" && parts[3] == "charts" {
                let portfolio_id = parts[2].to_string();
                let chart_type = parts[4];
                
                match (method, chart_type) {
                    ("GET", "performance") => {
                        return self.generate_performance_chart(portfolio_id, query_params, claims.sub).await;
                    }
                    ("GET", "allocation") => {
                        return self.generate_allocation_chart(portfolio_id, query_params, claims.sub).await;
                    }
                    ("GET", "attribution") => {
                        return self.generate_attribution_chart(portfolio_id, query_params, claims.sub).await;
                    }
                    _ => {
                        return Err(AppError::NotFound(format!("Route not found: {}", path)));
                    }
                }
            }
        }

        // Handle global chart routes
        if path.starts_with("/charts/") {
            let chart_type = path.split('/').nth(2).unwrap_or("");
            
            match (method, chart_type) {
                ("GET", "comparison") => {
                    return self.generate_comparison_chart(query_params, claims.sub).await;
                }
                ("GET", "risk-return") => {
                    return self.generate_risk_return_chart(query_params, claims.sub).await;
                }
                _ => {
                    return Err(AppError::NotFound(format!("Route not found: {}", path)));
                }
            }
        }
        
        Err(AppError::NotFound(format!("Route not found: {}", path)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::mock::{MockDynamoDbRepository, MockTimestreamRepository};
    
    #[tokio::test]
    async fn test_generate_performance_chart() {
        // Setup mock repositories
        let dynamodb_repo = Arc::new(MockDynamoDbRepository::new());
        let timestream_repo = Arc::new(MockTimestreamRepository::new());
        
        // Create visualization service
        let service = VisualizationService::new(timestream_repo, dynamodb_repo);
        
        // Test parameters
        let params = PerformanceChartParams {
            portfolio_id: "portfolio-123".to_string(),
            start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2023, 12, 31).unwrap(),
            benchmark_id: Some("benchmark-456".to_string()),
            metrics: vec!["twr".to_string()],
            interval: TimeInterval::Monthly,
            width: 800,
            height: 400,
            format: ChartFormat::Png,
        };
        
        // Generate chart
        let result = service.generate_performance_chart(params).await;
        
        // Assert result
        assert!(result.is_ok());
        let (chart_data, content_type) = result.unwrap();
        assert!(!chart_data.is_empty());
        assert_eq!(content_type, "image/png");
    }
    
    // Add more tests for other chart types...
} 