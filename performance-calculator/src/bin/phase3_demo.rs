use anyhow::{Result, Context};
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use performance_calculator::calculations::{
    config::Config,
    factory::ComponentFactory,
    analytics::{Factor, Scenario, FactorAnalysisResult, ScenarioAnalysisResult},
    visualization::{ChartType, ChartOptions, ChartSeries, ChartDefinition, ChartFormat, ReportTemplate, TableDefinition, TableColumn, ReportFormat},
    integration::{EmailNotification, WebhookNotification, ApiRequest, DataImportRequest},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("Performance Calculator - Phase 3 Demo");
    println!("=====================================");
    
    // Create configuration with Phase 3 features enabled
    let mut config = Config::default();
    
    // Enable Redis cache
    config.redis_cache = Some(performance_calculator::calculations::distributed_cache::RedisCacheConfig {
        enabled: true,
        url: "redis://localhost:6379".to_string(),
        prefix: "demo:".to_string(),
        ttl_seconds: 3600,
    });
    
    // Enable analytics
    config.analytics = Some(performance_calculator::calculations::analytics::AnalyticsConfig {
        enabled: true,
        max_concurrent_scenarios: 5,
        max_factors: 10,
        enable_caching: true,
        cache_ttl_seconds: 3600,
    });
    
    // Enable visualization
    config.visualization = Some(performance_calculator::calculations::visualization::VisualizationConfig {
        enabled: true,
        max_data_points: 1000,
        default_chart_width: 800,
        default_chart_height: 600,
        enable_caching: true,
        cache_ttl_seconds: 3600,
    });
    
    // Enable integration
    config.integration = Some(performance_calculator::calculations::integration::IntegrationConfig {
        enabled: true,
        api_endpoints: HashMap::new(),
        email: performance_calculator::calculations::integration::EmailConfig::default(),
        webhooks: performance_calculator::calculations::integration::WebhookConfig::default(),
        data_import: performance_calculator::calculations::integration::DataImportConfig::default(),
        enable_caching: true,
        cache_ttl_seconds: 3600,
    });
    
    // Create component factory
    let factory = ComponentFactory::new(config);
    
    // Create Phase 3 components
    let analytics_engine = factory.create_analytics_engine()
        .context("Failed to create analytics engine")?;
    
    let mut visualization_engine = factory.create_visualization_engine()
        .context("Failed to create visualization engine")?;
    
    let integration_engine = factory.create_integration_engine()
        .context("Failed to create integration engine")?;
    
    println!("\n1. Advanced Analytics Demo");
    println!("------------------------");
    
    // Register factors for analysis
    let market_factor = Factor {
        id: "MARKET".to_string(),
        name: "Market Factor".to_string(),
        category: "Market".to_string(),
        returns: create_sample_returns(),
    };
    
    let size_factor = Factor {
        id: "SIZE".to_string(),
        name: "Size Factor".to_string(),
        category: "Style".to_string(),
        returns: create_sample_returns(),
    };
    
    let value_factor = Factor {
        id: "VALUE".to_string(),
        name: "Value Factor".to_string(),
        category: "Style".to_string(),
        returns: create_sample_returns(),
    };
    
    analytics_engine.register_factor(market_factor).await?;
    analytics_engine.register_factor(size_factor).await?;
    analytics_engine.register_factor(value_factor).await?;
    
    // Register scenarios for analysis
    let market_crash_scenario = Scenario {
        id: "MARKET_CRASH".to_string(),
        name: "Market Crash".to_string(),
        description: "Severe market downturn scenario".to_string(),
        factor_shocks: {
            let mut shocks = HashMap::new();
            shocks.insert("MARKET".to_string(), dec!(-0.30));
            shocks.insert("SIZE".to_string(), dec!(-0.15));
            shocks.insert("VALUE".to_string(), dec!(0.05));
            shocks
        },
        reference_period: Some((
            NaiveDate::from_ymd_opt(2008, 9, 1).unwrap(),
            NaiveDate::from_ymd_opt(2009, 3, 31).unwrap(),
        )),
    };
    
    let inflation_scenario = Scenario {
        id: "INFLATION".to_string(),
        name: "High Inflation".to_string(),
        description: "High inflation scenario".to_string(),
        factor_shocks: {
            let mut shocks = HashMap::new();
            shocks.insert("MARKET".to_string(), dec!(-0.10));
            shocks.insert("SIZE".to_string(), dec!(-0.05));
            shocks.insert("VALUE".to_string(), dec!(0.15));
            shocks
        },
        reference_period: Some((
            NaiveDate::from_ymd_opt(1977, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(1982, 12, 31).unwrap(),
        )),
    };
    
    analytics_engine.register_scenario(market_crash_scenario).await?;
    analytics_engine.register_scenario(inflation_scenario).await?;
    
    // Perform factor analysis
    println!("Performing factor analysis...");
    let factor_analysis = analytics_engine.perform_factor_analysis(
        "PORTFOLIO1",
        NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 12, 31).unwrap(),
        None,
        "DEMO-REQUEST-1",
    ).await?;
    
    println!("Factor Analysis Results:");
    println!("  Portfolio: {}", factor_analysis.portfolio_id);
    println!("  Period: {} to {}", factor_analysis.start_date, factor_analysis.end_date);
    println!("  Model R-squared: {}", factor_analysis.model_r_squared);
    println!("  Alpha: {}", factor_analysis.alpha);
    println!("  Tracking Error: {}", factor_analysis.tracking_error);
    println!("  Information Ratio: {}", factor_analysis.information_ratio);
    println!("  Factor Exposures:");
    
    for exposure in &factor_analysis.exposures {
        println!("    {}: Exposure={}, t-stat={}, R-squared={}",
            exposure.factor_id, exposure.exposure, exposure.t_stat, exposure.r_squared);
    }
    
    // Perform scenario analysis
    println!("\nPerforming scenario analysis...");
    let scenario_analysis = analytics_engine.perform_scenario_analysis(
        "PORTFOLIO1",
        "MARKET_CRASH",
        NaiveDate::from_ymd_opt(2022, 12, 31).unwrap(),
        "DEMO-REQUEST-2",
    ).await?;
    
    println!("Scenario Analysis Results:");
    println!("  Portfolio: {}", scenario_analysis.portfolio_id);
    println!("  Scenario: {}", scenario_analysis.scenario_id);
    println!("  Expected Return: {}", scenario_analysis.expected_return);
    println!("  Expected Value: {}", scenario_analysis.expected_value);
    println!("  Value at Risk: {}", scenario_analysis.value_at_risk);
    println!("  Expected Shortfall: {}", scenario_analysis.expected_shortfall);
    
    // Perform risk decomposition
    println!("\nPerforming risk decomposition...");
    let risk_decomposition = analytics_engine.perform_risk_decomposition(
        "PORTFOLIO1",
        NaiveDate::from_ymd_opt(2022, 12, 31).unwrap(),
        None,
        "DEMO-REQUEST-3",
    ).await?;
    
    println!("Risk Decomposition Results:");
    println!("  Portfolio: {}", risk_decomposition.portfolio_id);
    println!("  Analysis Date: {}", risk_decomposition.analysis_date);
    println!("  Total Risk: {}", risk_decomposition.total_risk);
    println!("  Systematic Risk: {}", risk_decomposition.systematic_risk);
    println!("  Specific Risk: {}", risk_decomposition.specific_risk);
    println!("  Factor Contributions:");
    
    for (factor_id, contribution) in &risk_decomposition.factor_contributions {
        println!("    {}: {}", factor_id, contribution);
    }
    
    println!("\n2. Visualization Demo");
    println!("--------------------");
    
    // Create a chart
    println!("Generating performance chart...");
    let chart_definition = ChartDefinition {
        options: ChartOptions {
            title: "Portfolio Performance".to_string(),
            subtitle: Some("2022 Performance".to_string()),
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
                    ("Jan".to_string(), dec!(1.2)),
                    ("Feb".to_string(), dec!(-0.5)),
                    ("Mar".to_string(), dec!(0.8)),
                    ("Apr".to_string(), dec!(1.5)),
                    ("May".to_string(), dec!(-1.0)),
                    ("Jun".to_string(), dec!(0.3)),
                    ("Jul".to_string(), dec!(1.7)),
                    ("Aug".to_string(), dec!(0.9)),
                    ("Sep".to_string(), dec!(-0.2)),
                    ("Oct".to_string(), dec!(1.1)),
                    ("Nov".to_string(), dec!(0.6)),
                    ("Dec".to_string(), dec!(0.4)),
                ],
                color: Some("#4285F4".to_string()),
                series_type: None,
            },
            ChartSeries {
                name: "Benchmark".to_string(),
                data: vec![
                    ("Jan".to_string(), dec!(1.0)),
                    ("Feb".to_string(), dec!(-0.3)),
                    ("Mar".to_string(), dec!(0.5)),
                    ("Apr".to_string(), dec!(1.2)),
                    ("May".to_string(), dec!(-0.8)),
                    ("Jun".to_string(), dec!(0.1)),
                    ("Jul".to_string(), dec!(1.4)),
                    ("Aug".to_string(), dec!(0.7)),
                    ("Sep".to_string(), dec!(-0.1)),
                    ("Oct".to_string(), dec!(0.9)),
                    ("Nov".to_string(), dec!(0.4)),
                    ("Dec".to_string(), dec!(0.2)),
                ],
                color: Some("#DB4437".to_string()),
                series_type: None,
            },
        ],
    };
    
    let chart_result = visualization_engine.generate_chart(
        chart_definition,
        ChartFormat::SVG,
        "DEMO-REQUEST-4",
    ).await?;
    
    println!("Chart generated with ID: {}", chart_result.id);
    println!("Chart format: {:?}", chart_result.format);
    println!("Chart data length: {} bytes", chart_result.data.len());
    
    // Create a report template
    println!("\nRegistering report template...");
    let report_template = ReportTemplate {
        id: "PERFORMANCE_REPORT".to_string(),
        name: "Performance Report".to_string(),
        description: "Monthly performance report".to_string(),
        content: "# Performance Report\n\nThis report shows the performance of the portfolio for the selected period.\n\n## Performance Chart\n\n{{chart:performance_chart}}\n\n## Performance Metrics\n\n{{table:performance_metrics}}\n\n## Risk Metrics\n\n{{table:risk_metrics}}".to_string(),
        charts: vec![chart_result.definition],
        tables: vec![
            TableDefinition {
                id: "performance_metrics".to_string(),
                title: "Performance Metrics".to_string(),
                columns: vec![
                    TableColumn {
                        id: "metric".to_string(),
                        title: "Metric".to_string(),
                        data_type: "string".to_string(),
                        format: None,
                        width: None,
                        sortable: true,
                        filterable: true,
                    },
                    TableColumn {
                        id: "value".to_string(),
                        title: "Value".to_string(),
                        data_type: "decimal".to_string(),
                        format: Some("0.00%".to_string()),
                        width: None,
                        sortable: true,
                        filterable: false,
                    },
                    TableColumn {
                        id: "benchmark".to_string(),
                        title: "Benchmark".to_string(),
                        data_type: "decimal".to_string(),
                        format: Some("0.00%".to_string()),
                        width: None,
                        sortable: true,
                        filterable: false,
                    },
                ],
                data: vec![
                    {
                        let mut row = HashMap::new();
                        row.insert("metric".to_string(), serde_json::json!("1 Month Return"));
                        row.insert("value".to_string(), serde_json::json!(0.0123));
                        row.insert("benchmark".to_string(), serde_json::json!(0.0098));
                        row
                    },
                    {
                        let mut row = HashMap::new();
                        row.insert("metric".to_string(), serde_json::json!("3 Month Return"));
                        row.insert("value".to_string(), serde_json::json!(0.0345));
                        row.insert("benchmark".to_string(), serde_json::json!(0.0278));
                        row
                    },
                    {
                        let mut row = HashMap::new();
                        row.insert("metric".to_string(), serde_json::json!("YTD Return"));
                        row.insert("value".to_string(), serde_json::json!(0.0567));
                        row.insert("benchmark".to_string(), serde_json::json!(0.0489));
                        row
                    },
                    {
                        let mut row = HashMap::new();
                        row.insert("metric".to_string(), serde_json::json!("1 Year Return"));
                        row.insert("value".to_string(), serde_json::json!(0.0789));
                        row.insert("benchmark".to_string(), serde_json::json!(0.0654));
                        row
                    },
                ],
                default_sort: Some("metric".to_string()),
                default_sort_ascending: Some(true),
                paginated: false,
                page_size: None,
            },
            TableDefinition {
                id: "risk_metrics".to_string(),
                title: "Risk Metrics".to_string(),
                columns: vec![
                    TableColumn {
                        id: "metric".to_string(),
                        title: "Metric".to_string(),
                        data_type: "string".to_string(),
                        format: None,
                        width: None,
                        sortable: true,
                        filterable: true,
                    },
                    TableColumn {
                        id: "value".to_string(),
                        title: "Value".to_string(),
                        data_type: "decimal".to_string(),
                        format: Some("0.00".to_string()),
                        width: None,
                        sortable: true,
                        filterable: false,
                    },
                ],
                data: vec![
                    {
                        let mut row = HashMap::new();
                        row.insert("metric".to_string(), serde_json::json!("Volatility"));
                        row.insert("value".to_string(), serde_json::json!(0.1234));
                        row
                    },
                    {
                        let mut row = HashMap::new();
                        row.insert("metric".to_string(), serde_json::json!("Sharpe Ratio"));
                        row.insert("value".to_string(), serde_json::json!(0.8765));
                        row
                    },
                    {
                        let mut row = HashMap::new();
                        row.insert("metric".to_string(), serde_json::json!("Maximum Drawdown"));
                        row.insert("value".to_string(), serde_json::json!(0.1543));
                        row
                    },
                    {
                        let mut row = HashMap::new();
                        row.insert("metric".to_string(), serde_json::json!("Beta"));
                        row.insert("value".to_string(), serde_json::json!(1.0567));
                        row
                    },
                ],
                default_sort: Some("metric".to_string()),
                default_sort_ascending: Some(true),
                paginated: false,
                page_size: None,
            },
        ],
    };
    
    visualization_engine.register_template(report_template.clone())?;
    
    // Generate a report
    println!("Generating performance report...");
    let report_params = HashMap::new();
    let report_result = visualization_engine.generate_report(
        "PERFORMANCE_REPORT",
        report_params,
        ReportFormat::HTML,
        "DEMO-REQUEST-5",
    ).await?;
    
    println!("Report generated with ID: {}", report_result.id);
    println!("Report name: {}", report_result.name);
    println!("Report format: {:?}", report_result.format);
    println!("Report data length: {} bytes", report_result.data.len());
    
    println!("\n3. Enterprise Integration Demo");
    println!("-----------------------------");
    
    // Send an email notification
    println!("Sending email notification...");
    let email_notification = EmailNotification {
        subject: "Performance Report Available".to_string(),
        body: "Your monthly performance report is now available. Please log in to view it.".to_string(),
        recipients: vec!["user@example.com".to_string()],
        cc: None,
        bcc: None,
        attachments: None,
        is_html: false,
    };
    
    integration_engine.send_email(email_notification, "DEMO-REQUEST-6").await?;
    println!("Email notification sent");
    
    // Send a webhook notification
    println!("\nSending webhook notification...");
    let webhook_notification = WebhookNotification {
        event_type: "report_generated".to_string(),
        data: serde_json::json!({
            "report_id": report_result.id,
            "report_name": report_result.name,
            "timestamp": Utc::now().to_rfc3339(),
        }),
        target_webhooks: None,
    };
    
    integration_engine.send_webhook(webhook_notification, "DEMO-REQUEST-7").await?;
    println!("Webhook notification sent");
    
    // Send an API request
    println!("\nSending API request...");
    let api_request = ApiRequest {
        endpoint_id: "MARKET_DATA_API".to_string(),
        method: "GET".to_string(),
        path: "/api/v1/prices".to_string(),
        query_params: Some({
            let mut params = HashMap::new();
            params.insert("symbol".to_string(), "AAPL".to_string());
            params.insert("date".to_string(), "2022-12-31".to_string());
            params
        }),
        headers: None,
        body: None,
    };
    
    // This will fail because the endpoint doesn't exist, but it demonstrates the API
    let api_result = integration_engine.send_api_request(api_request, "DEMO-REQUEST-8").await;
    match api_result {
        Ok(response) => {
            println!("API request successful");
            println!("Status code: {}", response.status_code);
            println!("Response body: {:?}", response.body);
        },
        Err(e) => {
            println!("API request failed: {}", e);
            println!("This is expected in the demo since we didn't configure any API endpoints");
        },
    }
    
    // Import data
    println!("\nImporting data...");
    let import_request = DataImportRequest {
        import_type: "TRANSACTIONS".to_string(),
        format: "CSV".to_string(),
        data: b"date,symbol,quantity,price,currency\n2022-01-15,AAPL,100,150.25,USD\n2022-02-10,MSFT,50,300.75,USD\n2022-03-05,GOOGL,10,2500.50,USD".to_vec(),
        options: None,
    };
    
    let import_result = integration_engine.import_data(import_request, "DEMO-REQUEST-9").await?;
    
    println!("Data import completed");
    println!("Import ID: {}", import_result.import_id);
    println!("Records processed: {}", import_result.records_processed);
    println!("Records imported: {}", import_result.records_imported);
    println!("Records with errors: {}", import_result.records_with_errors);
    
    if !import_result.errors.is_empty() {
        println!("Errors:");
        for error in &import_result.errors {
            println!("  {}", error);
        }
    }
    
    println!("\nPhase 3 Demo Completed");
    println!("======================");
    
    Ok(())
}

fn create_sample_returns() -> HashMap<NaiveDate, Decimal> {
    let mut returns = HashMap::new();
    
    // Create sample returns for 2022
    for month in 1..=12 {
        let date = NaiveDate::from_ymd_opt(2022, month, 1).unwrap();
        let return_value = Decimal::new(rand::random::<i64>() % 200 - 100, 3); // Random return between -10% and 10%
        returns.insert(date, return_value);
    }
    
    returns
} 