use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use rust_decimal::prelude::FromPrimitive;
use shared::models::{
    Portfolio, Transaction, Security, Position, SecurityType, AssetClass,
    TransactionType as ModelTransactionType, Status, Holding,
};
use crate::calculations::{
    events::{self, Event, TransactionEvent, TransactionType as EventTransactionType},
    analytics::{Factor, Scenario},
    visualization::{ChartType, ChartOptions, ChartSeries, ChartDefinition, ChartFormat, ReportTemplate, ReportFormat},
    streaming::StreamingEvent,
    integration::{EmailNotification, WebhookNotification},
};
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use uuid::Uuid;
use serde_json;

use crate::calculations::{
    config::{Config, StreamingConfig, QueryApiConfig, SchedulerConfig, RedisCacheConfig},
    factory::ComponentFactory,
    events::{PriceUpdateEvent},
    integration::{EmailAttachment, NotificationService as IntegrationNotificationService},
    query_api::{DataAccessService, PortfolioData, PortfolioHoldingsWithReturns, BenchmarkHoldingsWithReturns, HypotheticalTransaction},
    risk_metrics::ReturnSeries,
};
use shared::models::{
    Account, AccountType, TaxStatus,
};

/// Mock notification service for testing
struct MockNotificationService;

#[async_trait]
impl crate::calculations::integration::NotificationService for MockNotificationService {
    async fn send_email(&self, _notification: EmailNotification) -> Result<()> {
        Ok(())
    }
    
    async fn send_webhook(&self, _notification: WebhookNotification) -> Result<()> {
        Ok(())
    }
}

/// Mock data service for testing
struct MockDataService {
    portfolios: HashMap<String, Portfolio>,
    accounts: HashMap<String, Account>,
    securities: HashMap<String, Security>,
    positions: HashMap<String, Position>,
    transactions: HashMap<String, Transaction>,
}

#[async_trait::async_trait]
impl DataAccessService for MockDataService {
    async fn get_portfolio_data(
        &self,
        _portfolio_id: &str,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> Result<PortfolioData> {
        Ok(PortfolioData {
            beginning_market_value: 10000.0,
            ending_market_value: 10500.0,
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
        Ok(ReturnSeries::new(vec![], vec![]))
    }

    async fn get_benchmark_returns_by_frequency(
        &self,
        _benchmark_id: &str,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
        _frequency: &str,
    ) -> Result<ReturnSeries> {
        Ok(ReturnSeries::new(vec![], vec![]))
    }

    async fn get_portfolio_holdings_with_returns(
        &self,
        _portfolio_id: &str,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> Result<PortfolioHoldingsWithReturns> {
        Ok(PortfolioHoldingsWithReturns {
            total_return: 0.0,
            holdings: vec![],
        })
    }

    async fn get_benchmark_holdings_with_returns(
        &self,
        _benchmark_id: &str,
        _start_date: NaiveDate,
        _end_date: NaiveDate,
    ) -> Result<BenchmarkHoldingsWithReturns> {
        Ok(BenchmarkHoldingsWithReturns {
            total_return: 0.0,
            holdings: vec![],
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
        _transaction: &HypotheticalTransaction,
    ) -> Result<()> {
        Ok(())
    }

    async fn delete_portfolio_data(&self, _portfolio_id: &str) -> Result<()> {
        Ok(())
    }
}

impl MockDataService {
    fn new() -> Self {
        Self {
            portfolios: HashMap::new(),
            accounts: HashMap::new(),
            securities: HashMap::new(),
            positions: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    async fn store_portfolio(&self, _portfolio: &Portfolio) -> Result<()> {
        Ok(())
    }

    async fn store_account(&self, _account: &Account) -> Result<()> {
        Ok(())
    }

    async fn store_security(&self, _security: &Security) -> Result<()> {
        Ok(())
    }

    async fn store_position(&self, _position: &Position) -> Result<()> {
        Ok(())
    }

    async fn store_transaction(&self, _transaction: &Transaction) -> Result<()> {
        Ok(())
    }
}

/// This test demonstrates the complete workflow of the Performance Calculator
/// with all three phases integrated together.
#[tokio::test]
async fn test_complete_workflow() -> anyhow::Result<()> {
    // Initialize logging
    let _ = env_logger::builder().is_test(true).try_init();
    
    // PHASE 1: Core Functionality
    // --------------------------
    
    // Create test configuration
    let mut config = Config::default();
    
    // Enable streaming
    config.streaming = Some(StreamingConfig {
        kafka_bootstrap_servers: Some("localhost:9092".to_string()),
        kafka_topics: vec!["test-topic".to_string()],
        kafka_consumer_group_id: Some("test-group".to_string()),
        max_parallel_events: 10,
        enabled: true,
        buffer_size: 10000,
        enable_batch_processing: true,
        max_batch_size: 100,
        batch_wait_ms: 1000,
    });

    // Enable query API
    config.query_api = Some(QueryApiConfig {
        cache_ttl_seconds: 300,
        max_concurrent_queries: 5,
        default_query_timeout_seconds: 30,
        enable_caching: true,
        max_query_complexity: 50,
        endpoint: "http://localhost:8080".to_string(),
        api_key: None,
        enabled: true,
    });

    // Enable Redis cache
    config.redis_cache = Some(RedisCacheConfig {
        url: "redis://localhost:6379".to_string(),
        max_connections: 5,
        default_ttl_seconds: 300,
    });

    // Enable scheduler
    config.scheduler = Some(SchedulerConfig {
        check_interval_seconds: 60,
        max_concurrent_calculations: 2,
        enabled: true,
        default_notification_channels: vec![],
        max_results_per_schedule: 10,
        cron_expression: "0 0 * * *".to_string(),
    });
    
    // Enable analytics
    config.analytics = Some(crate::calculations::analytics::AnalyticsConfig {
        enabled: true,
        max_concurrent_scenarios: 5,
        max_factors: 10,
        enable_caching: true,
        cache_ttl_seconds: 3600,
    });
    
    // Enable visualization
    config.visualization = Some(crate::calculations::visualization::VisualizationConfig {
        enabled: true,
        max_data_points: 1000,
        default_chart_width: 800,
        default_chart_height: 600,
        enable_caching: true,
        cache_ttl_seconds: 3600,
    });
    
    // Enable integration
    config.integration = Some(crate::calculations::integration::IntegrationConfig {
        enabled: true,
        api_endpoints: HashMap::new(),
        email: crate::calculations::integration::EmailConfig::default(),
        webhooks: crate::calculations::integration::WebhookConfig::default(),
        data_import: crate::calculations::integration::DataImportConfig {
            enabled: true,
            supported_formats: vec!["CSV".to_string(), "JSON".to_string()],
            max_file_size: 10_000_000,
            validate_data: true,
            backup_before_import: false,
        },
        enable_caching: true,
        cache_ttl_seconds: 3600,
    });
    
    // Create component factory
    let factory = ComponentFactory::new(config.into_app_config());
    
    // Create test portfolio
    let mut portfolio = Portfolio {
        id: "portfolio-1".to_string(),
        name: "Test Portfolio".to_string(),
        client_id: "client-1".to_string(),
        inception_date: Utc::now().date_naive(),
        benchmark_id: Some("benchmark-1".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        status: Status::Active,
        metadata: HashMap::new(),
        transactions: Vec::new(),
        holdings: Vec::new(),
    };
    
    // Create test transactions
    let transactions = vec![
        Transaction {
            id: "TXN-1".to_string(),
            account_id: "ACC-1".to_string(),
            security_id: Some("SEC-1".to_string()),
            transaction_date: Utc::now().date_naive(),
            settlement_date: Some(Utc::now().date_naive()),
            transaction_type: ModelTransactionType::Buy,
            amount: 10000.0,
            quantity: Some(100.0),
            price: Some(100.0),
            fees: Some(9.99),
            currency: "USD".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        },
        // Add more transactions as needed
    ];
    
    // Add transactions to portfolio (clone them to keep the original vec)
    for transaction in transactions.clone() {
        portfolio.add_transaction(transaction);
    }
    
    // Create test positions
    let positions = vec![
        Position {
            account_id: "ACC-1".to_string(),
            security_id: "SEC-1".to_string(),
            date: Utc::now().date_naive(),
            quantity: 100.0,
            market_value: 10500.0,
            cost_basis: Some(10000.0),
            currency: "USD".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        // Add more positions as needed
    ];
    
    // Create test securities
    let securities = vec![
        Security {
            id: "SEC-1".to_string(),
            symbol: "AAPL".to_string(),
            name: "Apple Inc.".to_string(),
            security_type: SecurityType::Equity,
            asset_class: AssetClass::DomesticEquity,
            cusip: Some("037833100".to_string()),
            isin: Some("US0378331005".to_string()),
            sedol: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        },
        // Add more securities as needed
    ];
    
    // Add holdings
    portfolio.add_holding(Holding {
        symbol: "AAPL".to_string(),
        quantity: dec!(100),
        cost_basis: Some(dec!(15000)),
        currency: "USD".to_string(),
    });
    
    portfolio.add_holding(Holding {
        symbol: "MSFT".to_string(),
        quantity: dec!(50),
        cost_basis: Some(dec!(12500)),
        currency: "USD".to_string(),
    });
    
    // Create risk calculator
    // TODO: Factory doesn't have create_risk_calculator method
    // let risk_calculator = factory.create_risk_calculator();
    
    // Calculate volatility
    // let volatility = risk_calculator.calculate_volatility(
    //     &portfolio,
    //     NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
    //     NaiveDate::from_ymd_opt(2022, 4, 30).unwrap(),
    // )?;
    
    // println!("Volatility: {}", volatility);
    
    // PHASE 2: Advanced Processing
    // ---------------------------
    
    let audit_trail = factory.create_audit_trail().await?;
    // Create an AuditTrailManager with InMemoryAuditTrailStorage
    let storage = Arc::new(crate::calculations::audit::InMemoryAuditTrailStorage::new());
    let audit_manager = Arc::new(crate::calculations::audit::AuditTrailManager::new(storage));
    let streaming_processor = factory.create_streaming_processor(audit_manager).await?;
    
    // Create transaction events
    let transaction_events = portfolio.transactions.iter().map(|t| {
        Event::Transaction(TransactionEvent {
            id: t.id.clone(),
            portfolio_id: portfolio.id.clone(),
            transaction_date: t.transaction_date,
            settlement_date: t.settlement_date,
            transaction_type: match t.transaction_type {
                ModelTransactionType::Buy => EventTransactionType::Buy,
                ModelTransactionType::Sell => EventTransactionType::Sell,
                ModelTransactionType::Deposit => EventTransactionType::Deposit,
                ModelTransactionType::Withdrawal => EventTransactionType::Withdrawal,
                ModelTransactionType::Dividend => EventTransactionType::Dividend,
                ModelTransactionType::Interest => EventTransactionType::Interest,
                ModelTransactionType::Fee => EventTransactionType::Fee,
                ModelTransactionType::Transfer => EventTransactionType::Transfer,
                ModelTransactionType::Split => EventTransactionType::Split,
                ModelTransactionType::Other(ref s) => EventTransactionType::Fee,
            },
            symbol: t.security_id.clone(),
            quantity: t.quantity.map(|q| Decimal::from_f64(q).unwrap_or_default()),
            price: t.price.map(|p| Decimal::from_f64(p).unwrap_or_default()),
            amount: Decimal::from_f64(t.amount).unwrap_or_default(),
            currency: t.currency.clone(),
            fees: t.fees.map(|f| Decimal::from_f64(f).unwrap_or_default()),
            taxes: None,
            notes: None,
            timestamp: Utc::now(),
        })
    }).collect::<Vec<_>>();
    
    // Submit events to streaming processor
    for event in transaction_events {
        let streaming_event = match event {
            Event::Transaction(t) => StreamingEvent {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                event_type: "transaction".to_string(),
                source: "test".to_string(),
                entity_id: t.portfolio_id.clone(),
                payload: serde_json::to_value(t).unwrap_or_default(),
            },
            _ => continue,
        };
        streaming_processor.submit_event(streaming_event).await?;
    }
    
    // Create price update events
    let price_update_events = vec![
        Event::PriceUpdate(PriceUpdateEvent {
            symbol: "AAPL".to_string(),
            price_date: NaiveDate::from_ymd_opt(2022, 4, 30).unwrap(),
            price: dec!(175),
            currency: "USD".to_string(),
            source: "TEST".to_string(),
            timestamp: Utc::now(),
        }),
        Event::PriceUpdate(PriceUpdateEvent {
            symbol: "MSFT".to_string(),
            price_date: NaiveDate::from_ymd_opt(2022, 4, 30).unwrap(),
            price: dec!(290),
            currency: "USD".to_string(),
            source: "TEST".to_string(),
            timestamp: Utc::now(),
        }),
    ];
    
    // Submit price update events
    for event in price_update_events {
        let streaming_event = match event {
            Event::PriceUpdate(p) => StreamingEvent {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                event_type: "price_update".to_string(),
                source: "test".to_string(),
                entity_id: p.symbol.clone(),
                payload: serde_json::to_value(p).unwrap_or_default(),
            },
            _ => continue,
        };
        streaming_processor.submit_event(streaming_event).await?;
    }
    
    // Create query API
    let query_api = factory.create_query_api().await?;
    
    // Execute performance query
    let query_result = query_api.calculate_performance(
        crate::calculations::query_api::PerformanceQueryParams {
            portfolio_id: "TEST-PORTFOLIO".to_string(),
            start_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2022, 4, 30).unwrap(),
            twr_method: Some("daily".to_string()),
            include_risk_metrics: Some(true),
            include_periodic_returns: Some(true),
            benchmark_id: None,
            currency: None,
            annualize: Some(true),
            custom_params: None,
        }
    ).await?;
    
    println!("Query Result: {:?}", query_result);
    
    // Create scheduler
    let scheduler = factory.create_scheduler().await?.unwrap_or_else(|| {
        panic!("Scheduler creation failed");
    });
    
    // Schedule a one-time job
    // TODO: CalculationScheduler doesn't have schedule_job method
    // let one_time_job_id = scheduler.schedule_job(
    //     "calculate_twr",
    //     HashMap::from([
    //         ("portfolio_id".to_string(), "TEST-PORTFOLIO".to_string()),
    //         ("start_date".to_string(), "2022-01-01".to_string()),
    //         ("end_date".to_string(), "2022-04-30".to_string()),
    //     ]),
    //     ScheduleFrequency::Once(Utc::now() + chrono::Duration::hours(1)),
    // )?;
    
    // println!("Scheduled one-time job: {}", one_time_job_id);
    
    // PHASE 3: Enterprise Features
    // ---------------------------
    
    // Create test accounts
    let accounts = vec![
        Account {
            id: "ACC-1".to_string(),
            account_number: "123456789".to_string(),
            name: "Test Account".to_string(),
            portfolio_id: portfolio.id.clone(),
            account_type: AccountType::Individual,
            inception_date: Utc::now().date_naive(),
            tax_status: TaxStatus::Taxable,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: Status::Active,
            metadata: HashMap::new(),
        },
    ];

    // Create test data service
    let data_service = Arc::new(MockDataService::new());

    // Store test data
    data_service.store_portfolio(&portfolio).await?;
    for account in &accounts {
        data_service.store_account(account).await?;
    }
    for security in &securities {
        data_service.store_security(security).await?;
    }
    for position in &positions {
        data_service.store_position(position).await?;
    }
    for transaction in &transactions {
        data_service.store_transaction(transaction).await?;
    }

    // Create analytics engine
    let analytics_engine = factory.create_analytics_engine()
        .ok_or_else(|| anyhow!("Failed to create analytics engine"))?;

    // Register factors for analysis
    analytics_engine.register_factor(Factor {
        id: "MARKET".to_string(),
        name: "Market".to_string(),
        category: "Market".to_string(),
        returns: HashMap::from([
            (NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(), dec!(0.02)),
            (NaiveDate::from_ymd_opt(2022, 2, 1).unwrap(), dec!(-0.01)),
            (NaiveDate::from_ymd_opt(2022, 3, 1).unwrap(), dec!(0.03)),
            (NaiveDate::from_ymd_opt(2022, 4, 1).unwrap(), dec!(0.01)),
        ]),
    }).await?;

    // Create scenario for analysis
    let scenario = Scenario {
        id: "BASE_CASE".to_string(),
        name: "Base Case".to_string(),
        description: "Base case scenario".to_string(),
        factor_shocks: HashMap::from([
            ("MARKET".to_string(), dec!(0.0)),
        ]),
        reference_period: None,
    };

    // Register scenario
    analytics_engine.register_scenario(scenario.clone()).await?;

    // Perform factor analysis
    let factor_analysis = analytics_engine.perform_factor_analysis(
        &portfolio.id,
        NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 4, 30).unwrap(),
        None,
        "TEST-REQUEST",
    ).await?;

    // Create visualization engine
    let visualization_engine = factory.create_visualization_engine()
        .ok_or_else(|| anyhow!("Failed to create visualization engine"))?;

    // Generate charts
    let charts = vec![
        ChartDefinition {
            options: ChartOptions {
                title: "Portfolio Returns".to_string(),
                subtitle: None,
                chart_type: ChartType::Line,
                width: 800,
                height: 400,
                x_axis_title: Some("Date".to_string()),
                y_axis_title: Some("Return (%)".to_string()),
                show_legend: true,
                show_tooltips: true,
                enable_zoom: false,
                stacked: false,
                colors: None,
            },
            series: vec![
                ChartSeries {
                    name: "Portfolio".to_string(),
                    data: factor_analysis.exposures.iter().map(|e| (
                        e.factor_id.clone(),
                        e.exposure,
                    )).collect(),
                    color: Some("#4285F4".to_string()),
                    series_type: None,
                },
            ],
        },
    ];

    // Create report template
    let report_template = ReportTemplate {
        id: "PERFORMANCE_REPORT".to_string(),
        name: "Portfolio Analysis Report".to_string(),
        description: "Analysis of portfolio performance and risk metrics".to_string(),
        content: "# Portfolio Analysis Report\n\n## Performance Analysis\n\n{{chart:portfolio_returns}}".to_string(),
        charts: charts.clone(),
        tables: vec![],
    };

    // Generate report
    let report_result = visualization_engine.generate_report(
        &report_template.id,
        HashMap::new(),
        ReportFormat::PDF,
        "TEST-REQUEST",
    ).await?;

    // Create notification service
    let notification_service: Arc<dyn crate::calculations::integration::NotificationService> = Arc::new(MockNotificationService);
    
    // Send email notification
    let email = EmailNotification {
        subject: "Performance Report".to_string(),
        body: "Performance report for TEST-PORTFOLIO is ready.".to_string(),
        recipients: vec!["user@example.com".to_string()],
        cc: None,
        bcc: None,
        attachments: None,
        is_html: false,
    };

    notification_service.send_email(email).await?;

    // Send webhook notification
    let webhook = WebhookNotification {
        event_type: "performance_calculated".to_string(),
        data: serde_json::json!({
            "portfolio_id": "TEST-PORTFOLIO",
            "timestamp": Utc::now().to_rfc3339(),
        }),
        target_webhooks: None,
    };

    notification_service.send_webhook(webhook).await?;

    Ok(())
}

// Helper function to create test returns
fn create_test_returns() -> HashMap<NaiveDate, rust_decimal::Decimal> {
    let mut returns = HashMap::new();
    
    // Create sample returns for 2022
    for month in 1..=12 {
        let date = NaiveDate::from_ymd_opt(2022, month, 1).unwrap();
        let return_value = rust_decimal::Decimal::new(rand::random::<i64>() % 200 - 100, 3); // Random return between -10% and 10%
        returns.insert(date, return_value);
    }
    
    returns
} 