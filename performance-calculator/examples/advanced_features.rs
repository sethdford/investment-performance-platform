use anyhow::Result;
use chrono::{Utc, NaiveDate};
use std::sync::Arc;
use tokio::signal;
use performance_calculator::calculations::{
    config::AppConfig,
    factory::ComponentFactory,
    audit::{AuditTrailManager, InMemoryAuditTrailStorage},
    currency::{CurrencyConverter, ExchangeRateProvider, ExchangeRate, CurrencyCode},
    query_api::{PerformanceQueryParams, DataAccessService, PortfolioData},
    scheduler::{ScheduledCalculation, ScheduledCalculationType, ScheduleFrequency, NotificationChannel},
    risk_metrics::ReturnSeries,
};
use std::collections::HashMap;
use async_trait::async_trait;

// Create custom mock implementations
struct MockExchangeRateProvider;

#[async_trait]
impl ExchangeRateProvider for MockExchangeRateProvider {
    async fn get_exchange_rate(
        &self,
        _base_currency: &CurrencyCode,
        _quote_currency: &CurrencyCode,
        date: NaiveDate,
        _request_id: &str,
    ) -> Result<ExchangeRate> {
        Ok(ExchangeRate {
            base_currency: "USD".to_string(),
            quote_currency: "EUR".to_string(),
            rate: rust_decimal::Decimal::new(85, 2), // 0.85
            date,
            source: "MOCK".to_string(),
        })
    }
}

struct MockDataAccessService;

#[async_trait]
impl DataAccessService for MockDataAccessService {
    async fn get_portfolio_data(
        &self,
        _portfolio_id: &str,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> Result<PortfolioData> {
        Ok(PortfolioData {
            beginning_market_value: 1000.0,
            ending_market_value: 1100.0,
            cash_flows: vec![],
            daily_market_values: HashMap::new(),
            daily_returns: HashMap::new(),
            currency: "USD".to_string(),
        })
    }

    async fn get_portfolio_returns(
        &self,
        _portfolio_id: &str,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
        _frequency: &str,
    ) -> Result<HashMap<NaiveDate, f64>> {
        Ok(HashMap::new())
    }

    async fn get_benchmark_returns(
        &self,
        _benchmark_id: &str,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> Result<ReturnSeries> {
        Ok(ReturnSeries {
            dates: vec![],
            values: vec![],
        })
    }

    async fn get_benchmark_returns_by_frequency(
        &self,
        _benchmark_id: &str,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
        _frequency: &str,
    ) -> Result<ReturnSeries> {
        Ok(ReturnSeries {
            dates: vec![],
            values: vec![],
        })
    }
    
    async fn get_portfolio_holdings_with_returns(
        &self,
        _portfolio_id: &str,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> Result<performance_calculator::calculations::query_api::PortfolioHoldingsWithReturns> {
        Ok(performance_calculator::calculations::query_api::PortfolioHoldingsWithReturns {
            holdings: vec![],
            total_return: 0.0,
        })
    }
    
    async fn get_benchmark_holdings_with_returns(
        &self,
        _benchmark_id: &str,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> Result<performance_calculator::calculations::query_api::BenchmarkHoldingsWithReturns> {
        Ok(performance_calculator::calculations::query_api::BenchmarkHoldingsWithReturns {
            holdings: vec![],
            total_return: 0.0,
        })
    }
    
    async fn clone_portfolio_data(
        &self,
        _source_portfolio_id: &str,
        _target_portfolio_id: &str,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> Result<()> {
        Ok(())
    }
    
    async fn apply_hypothetical_transaction(
        &self,
        _portfolio_id: &str,
        _transaction: &performance_calculator::calculations::query_api::HypotheticalTransaction,
    ) -> Result<()> {
        Ok(())
    }
    
    async fn delete_portfolio_data(&self, _portfolio_id: &str) -> Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = AppConfig::from_env();
    
    // Create component factory
    let factory = ComponentFactory::new(config);
    
    // Create audit trail manager
    let storage = Arc::new(InMemoryAuditTrailStorage::new());
    let audit_manager = Arc::new(AuditTrailManager::new(storage.clone()));
    
    // Create currency converter
    let exchange_rate_provider = Arc::new(MockExchangeRateProvider);
    let currency_converter = Arc::new(CurrencyConverter::new(
        exchange_rate_provider,
        "USD".to_string(),
    ));
    
    // Create data access service
    let data_service = Arc::new(MockDataAccessService);
    
    // Create query API
    let query_api = factory.create_query_api().await?;
    
    // Create streaming processor
    let streaming_processor = match factory.create_streaming_processor(audit_manager.clone()).await {
        Ok(processor) => Some(processor),
        Err(e) => {
            println!("Warning: Failed to create streaming processor: {}", e);
            None
        }
    };
    
    // Create calculation scheduler
    let scheduler: Option<Arc<dyn std::marker::Send + std::marker::Sync>> = None;
    println!("Calculation scheduler not implemented yet");
    
    // Start streaming processor if available
    if let Some(processor) = &streaming_processor {
        println!("Streaming processor would start here if implemented");
        // processor.start().await?; - Not implemented yet
    }
    
    // Start scheduler if available
    if let Some(scheduler) = &scheduler {
        println!("Calculation scheduler would start here if implemented");
        // scheduler.start().await?; - Not implemented yet
    }
    
    // Execute a sample query
    let query_params = PerformanceQueryParams {
        portfolio_id: "sample-portfolio".to_string(),
        start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2023, 3, 31).unwrap(),
        twr_method: Some("daily".to_string()),
        include_risk_metrics: Some(true),
        include_periodic_returns: Some(true),
        benchmark_id: Some("SPY".to_string()),
        currency: None,
        annualize: Some(false),
        custom_params: None,
    };
    
    let result = query_api.calculate_performance(query_params).await?;
    println!("Query result: {:?}", result);
    
    // Wait for Ctrl+C
    println!("Press Ctrl+C to exit");
    signal::ctrl_c().await?;
    
    // Stop services
    if let Some(processor) = &streaming_processor {
        println!("Streaming processor would stop here if implemented");
        // processor.stop().await?; - Not implemented yet
    }
    
    if let Some(scheduler) = &scheduler {
        println!("Calculation scheduler would stop here if implemented");
        // scheduler.stop().await?; - Not implemented yet
    }
    
    println!("Application shutdown complete");
    
    Ok(())
} 