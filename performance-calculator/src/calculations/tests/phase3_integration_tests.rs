use std::collections::HashMap;
use std::sync::Arc;
use chrono::{NaiveDate, Utc};
use rust_decimal_macros::dec;
use tokio::sync::Mutex;
use std::time::Instant;
use async_trait::async_trait;
use serde_json::Value;
use std::default::Default;
use rand::Rng;

use crate::calculations::{
    config::{Config, RedisCacheConfig, AppConfig},
    factory::ComponentFactory,
    analytics::{Factor, Scenario},
    visualization::{ChartType, ChartOptions, ChartSeries, ChartDefinition, ChartFormat, ReportTemplate, TableDefinition, TableColumn, ReportFormat},
    integration::{EmailNotification, WebhookNotification, ApiRequest, DataImportRequest, ApiResponse, DataImportResult, ApiClient, DataImportService},
    distributed_cache::StringCache,
    audit::AuditTrail,
};

// Mock API client for testing
#[derive(Clone)]
struct MockApiClient {
    requests_sent: Arc<Mutex<Vec<ApiRequest>>>,
}

impl MockApiClient {
    fn new() -> Self {
        Self {
            requests_sent: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl StringCache for MockApiClient {
    async fn get_string(&self, _key: &str) -> anyhow::Result<Option<String>> {
        Ok(None)
    }

    async fn set_string(&self, _key: String, _value: String, _ttl_seconds: Option<u64>) -> anyhow::Result<()> {
        Ok(())
    }

    async fn delete_string(&self, _key: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

#[async_trait]
impl ApiClient for MockApiClient {
    async fn send_request(&self, request: ApiRequest) -> anyhow::Result<ApiResponse> {
        let mut requests = self.requests_sent.lock().await;
        requests.push(request);
        
        Ok(ApiResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: Some(serde_json::json!({"result": "success"})),
            error: None,
        })
    }

    async fn get_oauth2_token(&self, endpoint_id: &str) -> anyhow::Result<String> {
        Ok("mock_token".to_string())
    }
}

// Mock data import service for testing
#[derive(Clone)]
struct MockDataImportService {
    imports_processed: Arc<Mutex<Vec<DataImportRequest>>>,
}

impl MockDataImportService {
    fn new() -> Self {
        Self {
            imports_processed: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl AuditTrail for MockDataImportService {
    async fn record(&self, _record: crate::calculations::audit::AuditRecord) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get_events_for_resource(&self, _resource_id: &str) -> anyhow::Result<Vec<crate::calculations::audit::AuditRecord>> {
        Ok(vec![])
    }

    async fn get_events_for_tenant(&self, _tenant_id: &str) -> anyhow::Result<Vec<crate::calculations::audit::AuditRecord>> {
        Ok(vec![])
    }
}

#[tokio::test]
async fn test_analytics_engine() {
    // Create configuration with analytics enabled
    let mut config = Config::default();
    config.analytics = Some(crate::calculations::analytics::AnalyticsConfig {
        enabled: true,
        max_concurrent_scenarios: 5,
        max_factors: 10,
        enable_caching: true,
        cache_ttl_seconds: 3600,
    });

    // Create component factory
    let factory = ComponentFactory::new(config.into_app_config());
    
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
    
    analytics_engine.register_factor(market_factor).await.unwrap();
    analytics_engine.register_factor(size_factor).await.unwrap();
    
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
    
    analytics_engine.register_scenario(market_crash_scenario).await.unwrap();
    
    // Perform factor analysis
    let factor_analysis = analytics_engine.perform_factor_analysis(
        "PORTFOLIO1",
        NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 12, 31).unwrap(),
        None,
        "TEST-REQUEST-1",
    ).await.unwrap();
    
    // Verify factor analysis results
    assert_eq!(factor_analysis.portfolio_id, "PORTFOLIO1");
    assert!(factor_analysis.model_r_squared > dec!(0));
    assert!(factor_analysis.exposures.len() >= 2);
    
    // Perform scenario analysis
    let scenario_analysis = analytics_engine.perform_scenario_analysis(
        "PORTFOLIO1",
        "MARKET_CRASH",
        NaiveDate::from_ymd_opt(2022, 12, 31).unwrap(),
        "TEST-REQUEST-2",
    ).await.unwrap();
    
    // Verify scenario analysis results
    assert_eq!(scenario_analysis.portfolio_id, "PORTFOLIO1");
    assert_eq!(scenario_analysis.scenario_id, "MARKET_CRASH");
    assert!(scenario_analysis.expected_return < dec!(0)); // Should be negative in a crash scenario
    
    // Perform risk decomposition
    let risk_decomposition = analytics_engine.perform_risk_decomposition(
        "PORTFOLIO1",
        NaiveDate::from_ymd_opt(2022, 12, 31).unwrap(),
        None,
        "TEST-REQUEST-3",
    ).await.unwrap();
    
    // Verify risk decomposition results
    assert_eq!(risk_decomposition.portfolio_id, "PORTFOLIO1");
    assert!(risk_decomposition.total_risk > dec!(0));
    assert!(risk_decomposition.factor_contributions.len() >= 2);
}

#[tokio::test]
async fn test_visualization_engine() {
    // Create configuration with visualization enabled
    let mut config = Config::default();
    config.visualization = Some(crate::calculations::visualization::VisualizationConfig {
        enabled: true,
        max_data_points: 1000,
        default_chart_width: 800,
        default_chart_height: 600,
        enable_caching: true,
        cache_ttl_seconds: 3600,
    });

    // Create component factory
    let factory = ComponentFactory::new(config.into_app_config());
    
    // Create visualization engine
    let mut visualization_engine = factory.create_visualization_engine().unwrap();
    
    // Create a chart definition
    let chart_definition = ChartDefinition {
        options: ChartOptions {
            title: "Test Chart".to_string(),
            subtitle: Some("Test Subtitle".to_string()),
            chart_type: ChartType::Line,
            width: 800,
            height: 400,
            x_axis_title: Some("Date".to_string()),
            y_axis_title: Some("Value".to_string()),
            show_legend: true,
            show_tooltips: true,
            enable_zoom: false,
            stacked: false,
            colors: None,
        },
        series: vec![
            ChartSeries {
                name: "Series 1".to_string(),
                data: vec![
                    ("Jan".to_string(), dec!(1.0)),
                    ("Feb".to_string(), dec!(2.0)),
                    ("Mar".to_string(), dec!(3.0)),
                    ("Apr".to_string(), dec!(2.5)),
                    ("May".to_string(), dec!(4.0)),
                ],
                color: Some("#4285F4".to_string()),
                series_type: None,
            },
            ChartSeries {
                name: "Series 2".to_string(),
                data: vec![
                    ("Jan".to_string(), dec!(0.5)),
                    ("Feb".to_string(), dec!(1.5)),
                    ("Mar".to_string(), dec!(2.0)),
                    ("Apr".to_string(), dec!(1.8)),
                    ("May".to_string(), dec!(3.0)),
                ],
                color: Some("#DB4437".to_string()),
                series_type: None,
            },
        ],
    };
    
    // Generate a chart
    let chart_result = visualization_engine.generate_chart(
        chart_definition.clone(),
        ChartFormat::SVG,
        "TEST-REQUEST-4",
    ).await.unwrap();
    
    // Verify chart results
    assert!(!chart_result.id.is_empty());
    assert_eq!(chart_result.format, ChartFormat::SVG);
    assert!(!chart_result.data.is_empty());
    
    // Create a report template
    let report_template = ReportTemplate {
        id: "TEST_REPORT".to_string(),
        name: "Test Report".to_string(),
        description: "Test report template".to_string(),
        content: "# Test Report\n\n{{chart:test_chart}}\n\n{{table:test_table}}".to_string(),
        charts: vec![chart_definition],
        tables: vec![
            TableDefinition {
                id: "test_table".to_string(),
                title: "Test Table".to_string(),
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
                        row.insert("metric".to_string(), serde_json::json!("Metric 1"));
                        row.insert("value".to_string(), serde_json::json!(1.23));
                        row
                    },
                    {
                        let mut row = HashMap::new();
                        row.insert("metric".to_string(), serde_json::json!("Metric 2"));
                        row.insert("value".to_string(), serde_json::json!(4.56));
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
    
    // Register the template
    visualization_engine.register_template(report_template).unwrap();
    
    // Generate a report
    let report_params = HashMap::new();
    let report_result = visualization_engine.generate_report(
        "TEST_REPORT",
        report_params,
        ReportFormat::HTML,
        "TEST-REQUEST-5",
    ).await.unwrap();
    
    // Verify report results
    assert!(!report_result.id.is_empty());
    assert_eq!(report_result.name, "Test Report");
    assert_eq!(report_result.format, ReportFormat::HTML);
    assert!(!report_result.data.is_empty());
    
    // Export data
    let export_result = visualization_engine.export_data(
        vec![{
            let mut row = HashMap::new();
            row.insert("Header 1".to_string(), serde_json::json!("Value 1"));
            row.insert("Header 2".to_string(), serde_json::json!("Value 2"));
            row
        }],
        ReportFormat::CSV,
        "TEST-REQUEST-6",
    ).await.unwrap();
    
    // Verify export results
    assert!(!export_result.is_empty());
}

#[tokio::test]
async fn test_integration_engine() {
    // Create a mock notification service for testing
    struct MockNotificationService {
        emails_sent: Arc<Mutex<Vec<EmailNotification>>>,
        webhooks_sent: Arc<Mutex<Vec<WebhookNotification>>>,
    }
    
    #[async_trait]
    impl crate::calculations::integration::NotificationService for MockNotificationService {
        async fn send_email(&self, notification: EmailNotification) -> anyhow::Result<()> {
            let mut emails = self.emails_sent.lock().await;
            emails.push(notification);
            Ok(())
        }
        
        async fn send_webhook(&self, notification: WebhookNotification) -> anyhow::Result<()> {
            let mut webhooks = self.webhooks_sent.lock().await;
            webhooks.push(notification);
            Ok(())
        }
    }
    
    // Create a mock API client for testing
    let api_client = MockApiClient::new();
    
    // Create a mock data import service for testing
    let data_import_service = MockDataImportService::new();
    
    // Create configuration with integration enabled
    let mut config = Config::default();
    config.integration = Some(crate::calculations::integration::IntegrationConfig {
        enabled: true,
        api_endpoints: {
            let mut endpoints = HashMap::new();
            endpoints.insert("TEST_API".to_string(), crate::calculations::integration::ApiEndpointConfig {
                url: "https://api.example.com".to_string(),
                auth_type: crate::calculations::integration::AuthType::None,
                timeout_seconds: 30,
                retry_enabled: true,
                max_retries: 3,
                retry_delay_seconds: 1,
            });
            endpoints
        },
        email: crate::calculations::integration::EmailConfig {
            enabled: true,
            smtp_server: "smtp.example.com".to_string(),
            smtp_port: 587,
            smtp_username: "user".to_string(),
            smtp_password: "password".to_string(),
            from_address: "noreply@example.com".to_string(),
            default_recipients: vec![],
            use_tls: true,
        },
        webhooks: crate::calculations::integration::WebhookConfig {
            enabled: true,
            webhooks: {
                let mut map = HashMap::new();
                map.insert("TEST_WEBHOOK".to_string(), crate::calculations::integration::WebhookEndpoint {
                    url: "https://webhook.example.com/callback".to_string(),
                    method: "POST".to_string(),
                    headers: HashMap::new(),
                    event_types: vec!["*".to_string()],
                    retry_enabled: true,
                    max_retries: 3,
                });
                map
            },
        },
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
    let factory = ComponentFactory::new(config.clone().into_app_config());
    
    // Create cache and audit trail
    let cache = factory.create_redis_cache().await.unwrap_or_else(|_| {
        Arc::new(crate::calculations::distributed_cache::NoOpCache::new())
    });
    
    let audit_trail = factory.create_audit_trail().await.unwrap();
    
    // Create mock services
    let emails_sent = Arc::new(Mutex::new(Vec::new()));
    let webhooks_sent = Arc::new(Mutex::new(Vec::new()));
    
    let notification_service = Arc::new(MockNotificationService {
        emails_sent: emails_sent.clone(),
        webhooks_sent: webhooks_sent.clone(),
    });
    
    // Create integration engine with mock services
    let integration_config = config.integration.unwrap();
    let api_client_clone = api_client.clone();
    let data_import_service_clone = data_import_service.clone();
    
    let integration_engine = crate::calculations::integration::IntegrationEngine::new(
        integration_config,
        Arc::new(api_client_clone) as Arc<dyn StringCache + Send + Sync>,
        Arc::new(data_import_service_clone) as Arc<dyn AuditTrail + Send + Sync>,
    );
    
    // Test sending an email notification
    let email_notification = EmailNotification {
        subject: "Test Email".to_string(),
        body: "This is a test email".to_string(),
        recipients: vec!["user@example.com".to_string()],
        cc: None,
        bcc: None,
        attachments: None,
        is_html: false,
    };
    
    integration_engine.send_email(email_notification.clone(), "TEST-REQUEST-7").await.unwrap();
    
    // Verify email was sent
    let emails = emails_sent.lock().await;
    assert_eq!(emails.len(), 1);
    assert_eq!(emails[0].subject, "Test Email");
    
    // Test sending a webhook notification
    let webhook_notification = WebhookNotification {
        event_type: "test_event".to_string(),
        data: serde_json::json!({"test": "data"}),
        target_webhooks: None,
    };
    
    integration_engine.send_webhook(webhook_notification.clone(), "TEST-REQUEST-8").await.unwrap();
    
    // Verify webhook was sent
    let webhooks = webhooks_sent.lock().await;
    assert_eq!(webhooks.len(), 1);
    assert_eq!(webhooks[0].event_type, "test_event");
    
    // Test sending an API request
    let api_request = ApiRequest {
        endpoint_id: "TEST_API".to_string(),
        method: "GET".to_string(),
        path: "/api/v1/data".to_string(),
        query_params: Some({
            let mut params = HashMap::new();
            params.insert("param1".to_string(), "value1".to_string());
            params
        }),
        headers: None,
        body: None,
    };
    
    integration_engine.send_api_request(api_request.clone(), "TEST-REQUEST-9").await.unwrap();
    
    // Verify API request was sent
    let requests = api_client.requests_sent.lock().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].endpoint_id, "TEST_API");
    assert_eq!(requests[0].method, "GET");
    
    // Test importing data
    let import_request = DataImportRequest {
        import_type: "TEST_IMPORT".to_string(),
        format: "CSV".to_string(),
        data: b"header1,header2\nvalue1,value2\nvalue3,value4".to_vec(),
        options: None,
    };
    
    integration_engine.import_data(import_request.clone(), "TEST-REQUEST-10").await.unwrap();
    
    // Verify data import was processed
    let imports = data_import_service.imports_processed.lock().await;
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].import_type, "TEST_IMPORT");
    assert_eq!(imports[0].format, "CSV");
}

#[tokio::test]
async fn test_phase3_integration() -> anyhow::Result<()> {
    // Create configuration with all Phase 3 features enabled
    let mut config = Config::default();
    
    // Enable Redis cache
    config.redis_cache = Some(crate::calculations::config::RedisCacheConfig {
        url: "redis://localhost:6379".to_string(),
        max_connections: 10,
        default_ttl_seconds: 3600,
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
        webhooks: crate::calculations::integration::WebhookConfig {
            enabled: true,
            webhooks: {
                let mut map = HashMap::new();
                map.insert("TEST_WEBHOOK".to_string(), crate::calculations::integration::WebhookEndpoint {
                    url: "https://webhook.example.com/callback".to_string(),
                    method: "POST".to_string(),
                    headers: HashMap::new(),
                    event_types: vec!["*".to_string()],
                    retry_enabled: true,
                    max_retries: 3,
                });
                map
            },
        },
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
    let factory = ComponentFactory::new(config.clone().into_app_config());
    
    // Create Phase 3 components
    let analytics_engine = factory.create_analytics_engine().unwrap();
    let mut visualization_engine = factory.create_visualization_engine().unwrap();
    let integration_engine = factory.create_integration_engine().unwrap();
    
    // Register a factor for analysis
    let market_factor = Factor {
        id: "MARKET".to_string(),
        name: "Market Factor".to_string(),
        category: "Market".to_string(),
        returns: create_test_returns(),
    };
    
    analytics_engine.register_factor(market_factor).await.unwrap();
    
    // Perform factor analysis
    let factor_analysis = analytics_engine.perform_factor_analysis(
        "PORTFOLIO1",
        NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2022, 12, 31).unwrap(),
        None,
        "TEST-REQUEST-11",
    ).await.unwrap();
    
    // Create a chart based on factor analysis
    let chart_definition = ChartDefinition {
        options: ChartOptions {
            title: "Factor Exposures".to_string(),
            subtitle: Some("Portfolio 1".to_string()),
            chart_type: ChartType::Bar,
            width: 800,
            height: 400,
            x_axis_title: Some("Factor".to_string()),
            y_axis_title: Some("Exposure".to_string()),
            show_legend: true,
            show_tooltips: true,
            enable_zoom: false,
            stacked: false,
            colors: None,
        },
        series: vec![
            ChartSeries {
                name: "Exposure".to_string(),
                data: factor_analysis.exposures.iter().map(|e| (e.factor_id.clone(), e.exposure)).collect(),
                color: Some("#4285F4".to_string()),
                series_type: None,
            },
        ],
    };
    
    // Generate a chart
    let chart_result = visualization_engine.generate_chart(
        chart_definition.clone(),
        ChartFormat::SVG,
        "TEST-REQUEST-12",
    ).await.unwrap();
    
    // Create a report template
    let report_template = ReportTemplate {
        id: "FACTOR_ANALYSIS_REPORT".to_string(),
        name: "Factor Analysis Report".to_string(),
        description: "Report showing factor analysis results".to_string(),
        content: "# Factor Analysis Report\n\n{{chart:factor_exposures}}\n\n## Analysis Results\n\nR-squared: {{r_squared}}\nAlpha: {{alpha}}".to_string(),
        charts: vec![chart_definition],
        tables: vec![],
    };
    
    visualization_engine.register_template(report_template).unwrap();
    
    // Generate a report
    let mut report_params = HashMap::new();
    report_params.insert("r_squared".to_string(), serde_json::json!(factor_analysis.model_r_squared.to_string()));
    report_params.insert("alpha".to_string(), serde_json::json!(factor_analysis.alpha.to_string()));
    
    let report_result = visualization_engine.generate_report(
        "FACTOR_ANALYSIS_REPORT",
        report_params,
        ReportFormat::HTML,
        "TEST-REQUEST-13",
    ).await.unwrap();
    
    // Send an email notification with the report
    let email_notification = EmailNotification {
        subject: "Factor Analysis Report".to_string(),
        body: "Please find attached the factor analysis report.".to_string(),
        recipients: vec!["user@example.com".to_string()],
        cc: None,
        bcc: None,
        attachments: Some(vec![
            crate::calculations::integration::EmailAttachment {
                name: report_result.id.clone(),
                data: report_result.data.clone(),
                content_type: "text/html".to_string(),
            }
        ]),
        is_html: false,
    };
    
    integration_engine.send_email(email_notification, "TEST-REQUEST-14").await.unwrap();
    
    // Import data and analyze it
    let import_request = DataImportRequest {
        import_type: "POSITIONS".to_string(),
        format: "CSV".to_string(),
        data: b"symbol,weight,sector\nAAPL,0.05,Technology\nMSFT,0.04,Technology\nJPM,0.03,Financials".to_vec(),
        options: None,
    };
    
    let import_result = integration_engine.import_data(import_request, "TEST-REQUEST-15").await.unwrap();
    
    // Verify the integration of all components
    assert!(!factor_analysis.portfolio_id.is_empty());
    assert!(!chart_result.id.is_empty());
    assert!(!report_result.id.is_empty());
    assert!(!import_result.import_id.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_analytics_error_handling() -> anyhow::Result<()> {
    let config = Config::default();
    let factory = ComponentFactory::new(config.into_app_config());
    let analytics_engine = factory.create_analytics_engine().unwrap();
    
    // Test invalid date range
    let result = analytics_engine.perform_factor_analysis(
        "PORTFOLIO1",
        NaiveDate::from_ymd_opt(2022, 12, 31).unwrap(),
        NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(), // End before start
        None,
        "TEST-REQUEST",
    ).await;
    
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_visualization_options() -> anyhow::Result<()> {
    let config = Config::default();
    let factory = ComponentFactory::new(config.into_app_config());
    let mut visualization_engine = factory.create_visualization_engine().unwrap();
    
    // Test different chart types
    let chart_types = vec![
        ChartType::Line,
        ChartType::Bar,
        ChartType::Pie,
        ChartType::Scatter,
    ];
    
    for chart_type in chart_types {
        let chart_def = ChartDefinition {
            options: ChartOptions {
                title: format!("Test {:?} Chart", chart_type),
                chart_type,
                width: 800,
                height: 400,
                ..Default::default()
            },
            series: vec![ChartSeries {
                name: "Test Series".to_string(),
                data: vec![("A".to_string(), dec!(1.0)), ("B".to_string(), dec!(2.0))],
                color: None,
                series_type: None,
            }],
        };
        
        let result = visualization_engine.generate_chart(
            chart_def,
            ChartFormat::SVG,
            "TEST-REQUEST",
        ).await;
        
        assert!(result.is_ok());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_integration_error_handling() -> anyhow::Result<()> {
    let config = Config::default();
    let factory = ComponentFactory::new(config.into_app_config());
    let integration_engine = factory.create_integration_engine().unwrap();
    
    // Test invalid email
    let invalid_email = EmailNotification {
        subject: "Test".to_string(),
        body: "Test".to_string(),
        recipients: vec!["invalid@".to_string()],
        cc: None,
        bcc: None,
        attachments: None,
        is_html: false,
    };
    
    let result = integration_engine.send_email(invalid_email, "TEST-REQUEST").await;
    assert!(result.is_err());
    
    // Test invalid webhook
    let invalid_webhook = WebhookNotification {
        event_type: "test".to_string(),
        data: serde_json::json!(null),
        target_webhooks: Some(vec!["non-existent".to_string()]),
    };
    
    let result = integration_engine.send_webhook(invalid_webhook, "TEST-REQUEST").await;
    assert!(result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_retry_mechanisms() -> anyhow::Result<()> {
    let config = Config::default();
    let factory = ComponentFactory::new(config.into_app_config());
    let integration_engine = factory.create_integration_engine().unwrap();
    
    // Test API retry
    let api_request = ApiRequest {
        endpoint_id: "TEST_API".to_string(),
        method: "GET".to_string(),
        path: "/slow-endpoint".to_string(),
        query_params: None,
        headers: None,
        body: None,
    };
    
    let start = Instant::now();
    let result = integration_engine.send_api_request(api_request, "TEST-REQUEST").await;
    let duration = start.elapsed();
    
    // Should have attempted retries
    assert!(duration.as_secs() >= 2);
    
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