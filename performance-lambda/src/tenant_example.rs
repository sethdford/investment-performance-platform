use crate::tenant::{TenantContext, TenantManager, TenantMiddleware, TenantAware, TenantError};
use aws_sdk_dynamodb::{Client as DynamoDbClient, model::AttributeValue};
use aws_sdk_timestreamwrite::Client as TimestreamWriteClient;
use aws_sdk_timestreamquery::Client as TimestreamQueryClient;
use std::collections::HashMap;
use std::error::Error;
use tracing::{info, error};

// Example repository for portfolios
#[derive(Debug, Clone)]
pub struct PortfolioRepository {
    client: DynamoDbClient,
    table_name: String,
    tenant_context: TenantContext,
}

impl PortfolioRepository {
    pub fn new(
        client: DynamoDbClient,
        table_name: impl Into<String>,
        tenant_context: TenantContext,
    ) -> Self {
        Self {
            client,
            table_name: table_name.into(),
            tenant_context,
        }
    }
    
    // Get a portfolio by ID
    pub async fn get_portfolio(
        &self,
        portfolio_id: &str,
    ) -> Result<Option<Portfolio>, Box<dyn Error + Send + Sync>> {
        // Create the key with tenant ID
        let mut key = HashMap::new();
        key.insert(
            "id".to_string(),
            AttributeValue::S(portfolio_id.to_string()),
        );
        key.insert(
            "entity_type".to_string(),
            AttributeValue::S("PORTFOLIO".to_string()),
        );
        
        // Add tenant ID to the key
        self.add_tenant_id_to_item(&mut key);
        
        // Get the item from DynamoDB
        let result = self.client
            .get_item()
            .table_name(&self.table_name)
            .set_key(Some(key))
            .send()
            .await?;
        
        // Convert the item to a Portfolio
        if let Some(item) = result.item {
            // Validate tenant ownership
            self.validate_tenant_ownership(&item)
                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
            
            // Convert the item to a Portfolio
            let portfolio = Portfolio {
                id: item.get("id").and_then(|v| v.as_s().ok()).unwrap_or_default().to_string(),
                name: item.get("name").and_then(|v| v.as_s().ok()).unwrap_or_default().to_string(),
                description: item.get("description").and_then(|v| v.as_s().ok()).unwrap_or_default().to_string(),
                tenant_id: item.get("tenant_id").and_then(|v| v.as_s().ok()).unwrap_or_default().to_string(),
            };
            
            Ok(Some(portfolio))
        } else {
            Ok(None)
        }
    }
    
    // List portfolios for the current tenant
    pub async fn list_portfolios(
        &self,
    ) -> Result<Vec<Portfolio>, Box<dyn Error + Send + Sync>> {
        // Create the query with tenant filter
        let key_condition_expression = "entity_type = :entity_type";
        
        let mut expression_attribute_values = HashMap::new();
        expression_attribute_values.insert(
            ":entity_type".to_string(),
            AttributeValue::S("PORTFOLIO".to_string()),
        );
        
        // Add tenant ID to the expression attribute values
        let tenant_values = self.tenant_expression_attribute_values();
        expression_attribute_values.extend(tenant_values);
        
        // Add tenant filter expression
        let filter_expression = self.tenant_condition_expression();
        
        // Query DynamoDB
        let result = self.client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression(key_condition_expression)
            .filter_expression(filter_expression)
            .set_expression_attribute_values(Some(expression_attribute_values))
            .send()
            .await?;
        
        // Convert the items to Portfolios
        let mut portfolios = Vec::new();
        
        if let Some(items) = result.items {
            for item in items {
                // Validate tenant ownership
                if let Err(e) = self.validate_tenant_ownership(&item) {
                    error!("Tenant validation failed: {}", e);
                    continue;
                }
                
                // Convert the item to a Portfolio
                let portfolio = Portfolio {
                    id: item.get("id").and_then(|v| v.as_s().ok()).unwrap_or_default().to_string(),
                    name: item.get("name").and_then(|v| v.as_s().ok()).unwrap_or_default().to_string(),
                    description: item.get("description").and_then(|v| v.as_s().ok()).unwrap_or_default().to_string(),
                    tenant_id: item.get("tenant_id").and_then(|v| v.as_s().ok()).unwrap_or_default().to_string(),
                };
                
                portfolios.push(portfolio);
            }
        }
        
        Ok(portfolios)
    }
    
    // Save a portfolio
    pub async fn save_portfolio(
        &self,
        portfolio: &Portfolio,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Validate that the portfolio belongs to the current tenant
        if portfolio.tenant_id != self.tenant_context.tenant_id {
            return Err(Box::new(TenantError::AccessDenied(
                "Portfolio does not belong to the current tenant".to_string(),
            )));
        }
        
        // Convert the portfolio to a DynamoDB item
        let mut item = HashMap::new();
        item.insert(
            "id".to_string(),
            AttributeValue::S(portfolio.id.clone()),
        );
        item.insert(
            "entity_type".to_string(),
            AttributeValue::S("PORTFOLIO".to_string()),
        );
        item.insert(
            "name".to_string(),
            AttributeValue::S(portfolio.name.clone()),
        );
        item.insert(
            "description".to_string(),
            AttributeValue::S(portfolio.description.clone()),
        );
        
        // Add tenant ID to the item
        self.add_tenant_id_to_item(&mut item);
        
        // Put the item in DynamoDB
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(item))
            .send()
            .await?;
        
        Ok(())
    }
    
    // Delete a portfolio
    pub async fn delete_portfolio(
        &self,
        portfolio_id: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Create the key with tenant ID
        let mut key = HashMap::new();
        key.insert(
            "id".to_string(),
            AttributeValue::S(portfolio_id.to_string()),
        );
        key.insert(
            "entity_type".to_string(),
            AttributeValue::S("PORTFOLIO".to_string()),
        );
        
        // Add tenant ID to the key
        self.add_tenant_id_to_item(&mut key);
        
        // Add condition expression to ensure tenant ownership
        let condition_expression = self.tenant_condition_expression();
        let expression_attribute_values = self.tenant_expression_attribute_values();
        
        // Delete the item from DynamoDB
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .set_key(Some(key))
            .condition_expression(condition_expression)
            .set_expression_attribute_values(Some(expression_attribute_values))
            .send()
            .await?;
        
        Ok(())
    }
}

impl TenantAware for PortfolioRepository {
    fn tenant_context(&self) -> &TenantContext {
        &self.tenant_context
    }
    
    fn with_tenant(mut self, tenant_context: TenantContext) -> Self {
        self.tenant_context = tenant_context;
        self
    }
}

// Example repository for performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceRepository {
    write_client: TimestreamWriteClient,
    query_client: TimestreamQueryClient,
    database_name: String,
    table_name: String,
    tenant_context: TenantContext,
}

impl PerformanceRepository {
    pub fn new(
        write_client: TimestreamWriteClient,
        query_client: TimestreamQueryClient,
        database_name: impl Into<String>,
        table_name: impl Into<String>,
        tenant_context: TenantContext,
    ) -> Self {
        Self {
            write_client,
            query_client,
            database_name: database_name.into(),
            table_name: table_name.into(),
            tenant_context,
        }
    }
    
    // Save performance metrics
    pub async fn save_performance_metrics(
        &self,
        portfolio_id: &str,
        metrics: &PerformanceMetrics,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Implementation would include tenant ID in the dimensions
        info!(
            "Saving performance metrics for portfolio {} in tenant {}",
            portfolio_id,
            self.tenant_context.tenant_id
        );
        
        // In a real implementation, we would add the tenant ID as a dimension
        // when writing to Timestream
        
        Ok(())
    }
    
    // Get performance history
    pub async fn get_performance_history(
        &self,
        portfolio_id: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<PerformanceMetrics>, Box<dyn Error + Send + Sync>> {
        // Implementation would include tenant ID in the query filter
        info!(
            "Getting performance history for portfolio {} in tenant {} from {} to {}",
            portfolio_id,
            self.tenant_context.tenant_id,
            start_date,
            end_date
        );
        
        // In a real implementation, we would add the tenant ID as a filter
        // when querying Timestream
        
        Ok(Vec::new())
    }
}

impl TenantAware for PerformanceRepository {
    fn tenant_context(&self) -> &TenantContext {
        &self.tenant_context
    }
    
    fn with_tenant(mut self, tenant_context: TenantContext) -> Self {
        self.tenant_context = tenant_context;
        self
    }
}

// Example service that uses the repositories
#[derive(Debug, Clone)]
pub struct PerformanceService {
    portfolio_repository: PortfolioRepository,
    performance_repository: PerformanceRepository,
}

impl PerformanceService {
    pub fn new(
        portfolio_repository: PortfolioRepository,
        performance_repository: PerformanceRepository,
    ) -> Self {
        Self {
            portfolio_repository,
            performance_repository,
        }
    }
    
    // Calculate and save performance for a portfolio
    pub async fn calculate_and_save_performance(
        &self,
        portfolio_id: &str,
    ) -> Result<PerformanceMetrics, Box<dyn Error + Send + Sync>> {
        // Get the portfolio
        let portfolio = self.portfolio_repository
            .get_portfolio(portfolio_id)
            .await?
            .ok_or_else(|| "Portfolio not found".to_string())?;
        
        // Calculate performance metrics
        let metrics = PerformanceMetrics {
            portfolio_id: portfolio.id.clone(),
            date: "2023-01-01".to_string(),
            twr: 0.05,
            mwr: 0.04,
            volatility: 0.1,
            sharpe_ratio: 0.5,
            sortino_ratio: 0.6,
            max_drawdown: -0.1,
            tenant_id: portfolio.tenant_id.clone(),
        };
        
        // Save performance metrics
        self.performance_repository
            .save_performance_metrics(portfolio_id, &metrics)
            .await?;
        
        Ok(metrics)
    }
    
    // Get performance history for a portfolio
    pub async fn get_performance_history(
        &self,
        portfolio_id: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<PerformanceMetrics>, Box<dyn Error + Send + Sync>> {
        // Get the portfolio
        let portfolio = self.portfolio_repository
            .get_portfolio(portfolio_id)
            .await?
            .ok_or_else(|| "Portfolio not found".to_string())?;
        
        // Get performance history
        let metrics = self.performance_repository
            .get_performance_history(portfolio_id, start_date, end_date)
            .await?;
        
        Ok(metrics)
    }
}

// Example of using tenant middleware
pub async fn example_with_middleware() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create AWS clients
    let config = aws_config::load_from_env().await;
    let dynamodb_client = DynamoDbClient::new(&config);
    let timestream_write_client = TimestreamWriteClient::new(&config);
    let timestream_query_client = TimestreamQueryClient::new(&config);
    
    // Create tenant context
    let tenant_context = TenantContext::new("tenant1");
    
    // Create tenant manager
    let tenant_manager = TenantManager::new(tenant_context.clone())?;
    
    // Create repositories
    let portfolio_repository = PortfolioRepository::new(
        dynamodb_client.clone(),
        "performance-data",
        tenant_context.clone(),
    );
    
    let performance_repository = PerformanceRepository::new(
        timestream_write_client.clone(),
        timestream_query_client.clone(),
        "performance",
        "metrics",
        tenant_context.clone(),
    );
    
    // Create middleware for repositories
    let portfolio_middleware = TenantMiddleware::new(
        portfolio_repository,
        tenant_manager.clone(),
    );
    
    let performance_middleware = TenantMiddleware::new(
        performance_repository,
        tenant_manager.clone(),
    );
    
    // Use the repositories through middleware
    let portfolio_repo = portfolio_middleware.inner();
    let performance_repo = performance_middleware.inner();
    
    // Create a service with the repositories
    let service = PerformanceService::new(
        portfolio_repo.clone(),
        performance_repo.clone(),
    );
    
    // Use the service
    let portfolio = Portfolio {
        id: "portfolio1".to_string(),
        name: "My Portfolio".to_string(),
        description: "A test portfolio".to_string(),
        tenant_id: tenant_context.tenant_id.clone(),
    };
    
    // Save the portfolio
    portfolio_repo.save_portfolio(&portfolio).await?;
    
    // Calculate and save performance
    let metrics = service
        .calculate_and_save_performance(&portfolio.id)
        .await?;
    
    info!("Calculated performance metrics: {:?}", metrics);
    
    // Get performance history
    let history = service
        .get_performance_history(&portfolio.id, "2023-01-01", "2023-12-31")
        .await?;
    
    info!("Performance history: {:?}", history);
    
    // Switch to a different tenant
    let tenant2_context = TenantContext::new("tenant2");
    
    let portfolio_middleware2 = portfolio_middleware
        .with_tenant(tenant2_context.clone())?;
    
    let performance_middleware2 = performance_middleware
        .with_tenant(tenant2_context.clone())?;
    
    // Use the repositories with the new tenant
    let portfolio_repo2 = portfolio_middleware2.inner();
    let performance_repo2 = performance_middleware2.inner();
    
    // Create a service with the new tenant repositories
    let service2 = PerformanceService::new(
        portfolio_repo2.clone(),
        performance_repo2.clone(),
    );
    
    // Use the service with the new tenant
    let portfolio2 = Portfolio {
        id: "portfolio1".to_string(),
        name: "My Portfolio".to_string(),
        description: "A test portfolio".to_string(),
        tenant_id: tenant2_context.tenant_id.clone(),
    };
    
    // Save the portfolio for the new tenant
    portfolio_repo2.save_portfolio(&portfolio2).await?;
    
    // Calculate and save performance for the new tenant
    let metrics2 = service2
        .calculate_and_save_performance(&portfolio2.id)
        .await?;
    
    info!("Calculated performance metrics for tenant2: {:?}", metrics2);
    
    Ok(())
}

// Example domain models
#[derive(Debug, Clone)]
pub struct Portfolio {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub portfolio_id: String,
    pub date: String,
    pub twr: f64,
    pub mwr: f64,
    pub volatility: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub max_drawdown: f64,
    pub tenant_id: String,
} 