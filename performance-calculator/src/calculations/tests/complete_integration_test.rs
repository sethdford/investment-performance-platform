use std::collections::HashMap;
use std::sync::Arc;
use chrono::{NaiveDate, Utc};
use rust_decimal_macros::dec;
use anyhow::Result;

use crate::calculations::{
    config::Config,
    factory::ComponentFactory,
    portfolio::{Portfolio, Holding, CashBalance, Transaction},
    events::{Event, TransactionEvent, PriceUpdateEvent, TransactionType},
    analytics::{Factor, Scenario},
    visualization::{ChartType, ChartOptions, ChartSeries, ChartDefinition, ChartFormat, ReportTemplate},
    integration::{EmailNotification, WebhookNotification},
};

/// This test demonstrates the complete workflow of the Performance Calculator
/// with all three phases integrated together.
#[tokio::test]
async fn test_complete_workflow() -> Result<()> {
    // Initialize logging
    let _ = env_logger::builder().is_test(true).try_init();
    
    // PHASE 1: Core Functionality
    // --------------------------
    
    // Create configuration with all features enabled
    let mut config = Config::default();
    
    // Enable Redis cache
    config.redis_cache = Some(crate::calculations::distributed_cache::RedisCacheConfig {
        enabled: true,
        url: "redis://localhost:6379".to_string(),
        prefix: "test:".to_string(),
        ttl_seconds: 3600,
    });
    
    // Enable streaming
    config.streaming = Some(crate::calculations::streaming::StreamingConfig {
        enabled: true,
        buffer_size: 1000,
        batch_size: 100,
        processing_interval_ms: 1000,
    });
    
    // Enable query API
    config.query_api = Some(crate::calculations::query::QueryApiConfig {
        enabled: true,
        max_page_size: 100,
        default_page_size: 20,
        enable_caching: true,
        cache_ttl_seconds: 3600,
    });
    
    // Enable scheduler
    config.scheduler = Some(crate::calculations::scheduler::SchedulerConfig {
        enabled: true,
        poll_interval_seconds: 60,
        max_concurrent_jobs: 10,
        enable_caching: true,
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
            max_file_size_bytes: 10_000_000,
            allowed_formats: vec!["CSV".to_string(), "JSON".to_string()],
        },
        enable_caching: true,
        cache_ttl_seconds: 3600,
    });
    
    // Create component factory
    let factory = ComponentFactory::new(config);
    
    // Create portfolio
    let mut portfolio = Portfolio::new("TEST-PORTFOLIO", "USD");
    
    // Add initial cash balance
    portfolio.add_cash_balance(CashBalance {
        currency: "USD".to_string(),
        amount: dec!(10000),
    });
    
    // Add transactions
    let transactions = vec![
        Transaction {
            id: "T1".to_string(),
            transaction_date: NaiveDate::from_ymd_opt(2022, 1, 15).unwrap(),
            settlement_date: Some(NaiveDate::from_ymd_opt(2022, 1, 17).unwrap()),
            transaction_type: TransactionType::Buy,
            symbol: Some("AAPL".to_string()),
            quantity: Some(dec!(10)),
            price: Some(dec!(150)),
            amount: dec!(1500),
            currency: "USD".to_string(),
            fees: Some(dec!(7.99)),
            taxes: None,
            notes: None,
        },
        Transaction {
            id: "T2".to_string(),
            transaction_date: NaiveDate::from_ymd_opt(2022, 2, 10).unwrap(),
            settlement_date: Some(NaiveDate::from_ymd_opt(2022, 2, 12).unwrap()),
            transaction_type: TransactionType::Buy,
            symbol: Some("MSFT".to_string()),
            quantity: Some(dec!(5)),
            price: Some(dec!(280)),
            amount: dec!(1400),
            currency: "USD".to_string(),
            fees: Some(dec!(7.99)),
            taxes: None,
            notes: None,
        },
        Transaction {
            id: "T3".to_string(),
            transaction_date: NaiveDate::from_ymd_opt(2022, 3, 15).unwrap(),
            settlement_date: Some(NaiveDate::from_ymd_opt(2022, 3, 17).unwrap()),
            transaction_type: TransactionType::Dividend,
            symbol: Some("AAPL".to_string()),
            quantity: None,
            price: None,
            amount: dec!(15),
            currency: "USD".to_string(),
            fees: None,
            taxes: Some(dec!(2.25)),
            notes: None,
        },
        Transaction {
            id: "T4".to_string(),
            transaction_date: NaiveDate::from_ymd_opt(2022, 4, 20).unwrap(),
            settlement_date: Some(NaiveDate::from_ymd_opt(2022, 4, 22).unwrap()),
            transaction_type: TransactionType::Sell,
            symbol: Some("AAPL".to_string()),
            quantity: Some(dec!(3)),
            price: Some(dec!(165)),
            amount: dec!(495),
            currency: "USD".to_string(),
            fees: Some(dec!(7.99)),
            taxes: None,
            notes: None,
        },
    ];
    
    for transaction in transactions {
        portfolio.add_transaction(transaction);
    }
    
    // Add holdings
    portfolio.add_holding(Holding {
        symbol: "AAPL".to_string(),
        quantity: dec!(7),
        cost_basis: Some(dec!(1050)),
        currency: "USD".to_string(),
    });
    
    portfolio.add_holding(Holding {
        symbol: "MSFT".to_string(),
        quantity: dec!(5),
        cost_basis: Some(dec!(1400)),
        currency: "USD".to_string(),
    });
    
    // Create TWR calculator
    let twr_calculator = factory.create_twr_calculator();
    
    // Calculate TWR
    let twr_result = twr_calculator.calculate_twr(
        &portfolio,
        NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 4, 30).unwrap(),
    ).await?;
    
    println!("TWR Result: {}", twr_result);
    
    // Create MWR calculator
    let mwr_calculator = factory.create_mwr_calculator();
    
    // Calculate MWR
    let mwr_result = mwr_calculator.calculate_mwr(
        &portfolio,
        NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 4, 30).unwrap(),
    ).await?;
    
    println!("MWR Result: {}", mwr_result);
    
    // Create risk calculator
    let risk_calculator = factory.create_risk_calculator();
    
    // Calculate volatility
    let volatility = risk_calculator.calculate_volatility(
        &portfolio,
        NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 4, 30).unwrap(),
    ).await?;
    
    println!("Volatility: {}", volatility);
    
    // PHASE 2: Advanced Processing
    // ---------------------------
    
    // Create streaming processor
    let streaming_processor = factory.create_streaming_processor().await?;
    
    // Create transaction events
    let transaction_events = portfolio.transactions.iter().map(|t| {
        Event::Transaction(TransactionEvent {
            id: t.id.clone(),
            portfolio_id: portfolio.id.clone(),
            transaction_date: t.transaction_date,
            settlement_date: t.settlement_date,
            transaction_type: t.transaction_type.clone(),
            symbol: t.symbol.clone(),
            quantity: t.quantity,
            price: t.price,
            amount: t.amount,
            currency: t.currency.clone(),
            fees: t.fees,
            taxes: t.taxes,
            notes: t.notes.clone(),
            timestamp: Utc::now(),
        })
    }).collect::<Vec<_>>();
    
    // Submit transaction events
    for event in transaction_events {
        streaming_processor.submit_event(event).await?;
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
        streaming_processor.submit_event(event).await?;
    }
    
    // Create query API
    let query_api = factory.create_query_api().await?;
    
    // Execute performance query
    let query_result = query_api.execute_query(
        "portfolio",
        Some("id = 'TEST-PORTFOLIO'"),
        Some("transaction_date"),
        Some(1),
        Some(10),
        "TEST-REQUEST-1",
    ).await?;
    
    println!("Query Result: {:?}", query_result);
    
    // Create scheduler
    let scheduler = factory.create_scheduler().await?;
    
    // Schedule a one-time job
    let one_time_job_id = scheduler.schedule_job(
        "calculate_twr",
        HashMap::from([
            ("portfolio_id".to_string(), "TEST-PORTFOLIO".to_string()),
            ("start_date".to_string(), "2022-01-01".to_string()),
            ("end_date".to_string(), "2022-04-30".to_string()),
        ]),
        chrono::Utc::now() + chrono::Duration::minutes(5),
        None,
        "TEST-REQUEST-2",
    ).await?;
    
    println!("Scheduled one-time job: {}", one_time_job_id);
    
    // PHASE 3: Enterprise Features
    // ---------------------------
    
    // Create analytics engine
    let analytics_engine = factory.create_analytics_engine().unwrap();
    
    // Register factors for analysis
    let market_factor = Factor {
        id: "MARKET".to_string(),
        name: "Market Factor".to_string(),
        category: "Market".to_string(),
        returns: create_test_returns(),
    };
    
    let size_factor = Factor {
        id: "SIZE".to_string(),
        name: "Size Factor".to_string(),
        category: "Style".to_string(),
        returns: create_test_returns(),
    };
    
    analytics_engine.register_factor(market_factor).await?;
    analytics_engine.register_factor(size_factor).await?;
    
    // Register a scenario for analysis
    let market_crash_scenario = Scenario {
        id: "MARKET_CRASH".to_string(),
        name: "Market Crash".to_string(),
        description: "Severe market downturn scenario".to_string(),
        factor_shocks: {
            let mut shocks = HashMap::new();
            shocks.insert("MARKET".to_string(), dec!(-0.30));
            shocks.insert("SIZE".to_string(), dec!(-0.15));
            shocks
        },
        reference_period: Some((
            NaiveDate::from_ymd_opt(2008, 9, 1).unwrap(),
            NaiveDate::from_ymd_opt(2009, 3, 31).unwrap(),
        )),
    };
    
    analytics_engine.register_scenario(market_crash_scenario).await?;
    
    // Perform factor analysis
    let factor_analysis = analytics_engine.perform_factor_analysis(
        "TEST-PORTFOLIO",
        NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 4, 30).unwrap(),
        None,
        "TEST-REQUEST-3",
    ).await?;
    
    println!("Factor Analysis: R-squared = {}, Alpha = {}", 
             factor_analysis.model_r_squared, factor_analysis.alpha);
    
    // Perform scenario analysis
    let scenario_analysis = analytics_engine.perform_scenario_analysis(
        "TEST-PORTFOLIO",
        "MARKET_CRASH",
        NaiveDate::from_ymd_opt(2022, 4, 30).unwrap(),
        "TEST-REQUEST-4",
    ).await?;
    
    println!("Scenario Analysis: Expected Return = {}", scenario_analysis.expected_return);
    
    // Create visualization engine
    let mut visualization_engine = factory.create_visualization_engine().unwrap();
    
    // Create a performance chart
    let performance_chart = ChartDefinition {
        options: ChartOptions {
            title: "Portfolio Performance".to_string(),
            subtitle: Some("Jan-Apr 2022".to_string()),
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
                data: vec![
                    ("Jan".to_string(), dec!(0.5)),
                    ("Feb".to_string(), dec!(1.2)),
                    ("Mar".to_string(), dec!(0.8)),
                    ("Apr".to_string(), dec!(1.5)),
                ],
                color: Some("#4285F4".to_string()),
                series_type: None,
            },
            ChartSeries {
                name: "Benchmark".to_string(),
                data: vec![
                    ("Jan".to_string(), dec!(0.3)),
                    ("Feb".to_string(), dec!(0.9)),
                    ("Mar".to_string(), dec!(0.6)),
                    ("Apr".to_string(), dec!(1.1)),
                ],
                color: Some("#DB4437".to_string()),
                series_type: None,
            },
        ],
    };
    
    // Generate a chart
    let chart_result = visualization_engine.generate_chart(
        performance_chart.clone(),
        ChartFormat::SVG,
        "TEST-REQUEST-5",
    ).await?;
    
    println!("Chart generated: ID = {}, Size = {} bytes", 
             chart_result.id, chart_result.data.len());
    
    // Create a report template
    let report_template = ReportTemplate {
        id: "PERFORMANCE_REPORT".to_string(),
        name: "Performance Report".to_string(),
        description: "Monthly performance report".to_string(),
        content: "# Performance Report\n\n## Performance Metrics\n\nTWR: {{twr}}\nMWR: {{mwr}}\nVolatility: {{volatility}}\n\n{{chart:performance_chart}}\n\n## Factor Analysis\n\nR-squared: {{r_squared}}\nAlpha: {{alpha}}".to_string(),
        charts: vec![performance_chart],
        tables: vec![],
    };
    
    visualization_engine.register_template(report_template)?;
    
    // Generate a report
    let mut report_params = HashMap::new();
    report_params.insert("twr".to_string(), twr_result.to_string());
    report_params.insert("mwr".to_string(), mwr_result.to_string());
    report_params.insert("volatility".to_string(), volatility.to_string());
    report_params.insert("r_squared".to_string(), factor_analysis.model_r_squared.to_string());
    report_params.insert("alpha".to_string(), factor_analysis.alpha.to_string());
    
    let report_result = visualization_engine.generate_report(
        "PERFORMANCE_REPORT",
        report_params,
        crate::calculations::visualization::ReportFormat::HTML,
        "TEST-REQUEST-6",
    ).await?;
    
    println!("Report generated: ID = {}, Size = {} bytes", 
             report_result.id, report_result.data.len());
    
    // Create integration engine
    let integration_engine = factory.create_integration_engine().unwrap();
    
    // Send an email notification
    let email_notification = EmailNotification {
        subject: "Performance Report Available".to_string(),
        body: "Your monthly performance report is now available.".to_string(),
        recipients: vec!["user@example.com".to_string()],
        cc: None,
        bcc: None,
        attachments: Some(vec![
            (report_result.id.clone(), report_result.data.clone(), "text/html".to_string())
        ]),
        is_html: false,
    };
    
    let email_result = integration_engine.send_email(email_notification, "TEST-REQUEST-7").await;
    println!("Email notification result: {:?}", email_result.is_ok());
    
    // Send a webhook notification
    let webhook_notification = WebhookNotification {
        event_type: "report_generated".to_string(),
        data: serde_json::json!({
            "report_id": report_result.id,
            "report_name": report_result.name,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }),
        target_webhooks: None,
    };
    
    let webhook_result = integration_engine.send_webhook(webhook_notification, "TEST-REQUEST-8").await;
    println!("Webhook notification result: {:?}", webhook_result.is_ok());
    
    // Complete the workflow
    println!("Complete workflow test finished successfully");
    
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