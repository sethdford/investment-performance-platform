use anyhow::{Result, Context};
use std::sync::Arc;
use std::collections::HashMap;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use performance_calculator::{
    calculations::{
        analytics::{AnalyticsEngine, AnalyticsConfig, Factor, Scenario},
        visualization::{VisualizationEngine, VisualizationConfig, ChartOptions, ChartType, ChartSeries, ChartDefinition, ChartFormat},
        integration::{IntegrationEngine, IntegrationConfig},
        audit::{AuditTrail, InMemoryAuditTrail},
        distributed_cache::{Cache, BinaryCache, StringCache, InMemoryCache},
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("Performance Calculator - Phase 3 Demo");
    println!("=====================================\n");
    
    // Create a configuration
    let config = performance_calculator::config::Config {
        // Use default configuration
        ..performance_calculator::config::Config::default()
    };
    
    println!("🏭 Creating component factory and in-memory components");
    
    // Create in-memory components
    let audit_trail = Arc::new(InMemoryAuditTrail::new()) as Arc<dyn AuditTrail + Send + Sync>;
    let cache_for_analytics = Arc::new(InMemoryCache::new()) as Arc<dyn Cache + Send + Sync>;
    let string_cache = Arc::new(InMemoryCache::new()) as Arc<dyn StringCache + Send + Sync>;
    let binary_cache = Arc::new(InMemoryCache::new()) as Arc<dyn BinaryCache + Send + Sync>;
    
    // Create analytics engine with default config
    let analytics_engine = Arc::new(AnalyticsEngine::new(
        AnalyticsConfig {
            enabled: true,
            ..AnalyticsConfig::default()
        },
        cache_for_analytics,
        audit_trail.clone(),
    ));
    println!("✅ Created analytics engine");
    
    // Create visualization engine with default config
    let mut visualization_engine = VisualizationEngine::new(
        VisualizationConfig {
            enabled: true,
            ..VisualizationConfig::default()
        },
        binary_cache,
        audit_trail.clone(),
    );
    println!("✅ Created visualization engine");
    
    // Create integration engine with default config
    let integration_config = IntegrationConfig {
        enabled: true,
        ..IntegrationConfig::default()
    };
    let integration_engine = Arc::new(IntegrationEngine::new(
        integration_config.clone(),
        string_cache,
        audit_trail.clone(),
    ));
    println!("✅ Created integration engine");
    
    println!("\n1. Advanced Analytics Demo");
    println!("-------------------------");
    
    // Register sample factors
    let market_factor = Factor {
        id: "MARKET".to_string(),
        name: "Market Factor".to_string(),
        category: "Market".to_string(),
        returns: create_sample_returns(0.08),
    };
    
    let size_factor = Factor {
        id: "SIZE".to_string(),
        name: "Size Factor".to_string(),
        category: "Style".to_string(),
        returns: create_sample_returns(0.03),
    };
    
    let value_factor = Factor {
        id: "VALUE".to_string(),
        name: "Value Factor".to_string(),
        category: "Style".to_string(),
        returns: create_sample_returns(0.02),
    };
    
    analytics_engine.register_factor(market_factor).await?;
    analytics_engine.register_factor(size_factor).await?;
    analytics_engine.register_factor(value_factor).await?;
    println!("✅ Registered factors for analysis");
    
    // Register sample scenarios
    let market_crash = Scenario {
        id: "MARKET_CRASH".to_string(),
        name: "Market Crash".to_string(),
        description: "Simulates a severe market downturn".to_string(),
        factor_shocks: {
            let mut shocks = HashMap::new();
            shocks.insert("MARKET".to_string(), dec!(-0.30));
            shocks.insert("SIZE".to_string(), dec!(-0.15));
            shocks.insert("VALUE".to_string(), dec!(0.05));
            shocks
        },
        reference_period: None,
    };
    
    let inflation_shock = Scenario {
        id: "INFLATION_SHOCK".to_string(),
        name: "Inflation Shock".to_string(),
        description: "Simulates a sudden rise in inflation".to_string(),
        factor_shocks: {
            let mut shocks = HashMap::new();
            shocks.insert("MARKET".to_string(), dec!(-0.10));
            shocks.insert("SIZE".to_string(), dec!(-0.05));
            shocks.insert("VALUE".to_string(), dec!(0.10));
            shocks
        },
        reference_period: None,
    };
    
    analytics_engine.register_scenario(market_crash).await?;
    analytics_engine.register_scenario(inflation_shock).await?;
    println!("✅ Registered scenarios for analysis");
    
    // Perform factor analysis
    let portfolio_id = "PORTFOLIO1";
    let start_date = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2022, 12, 31).unwrap();
    
    let factor_analysis = analytics_engine.perform_factor_analysis(
        portfolio_id,
        start_date,
        end_date,
        None,
        "DEMO-REQUEST-1",
    ).await?;
    
    println!("\nFactor Analysis Results:");
    println!("  Portfolio: {}", portfolio_id);
    println!("  Period: {} to {}", start_date, end_date);
    println!("  Alpha: {}%", (factor_analysis.alpha * dec!(100)).round_dp(2));
    println!("  Model R-squared: {}%", (factor_analysis.model_r_squared * dec!(100)).round_dp(2));
    println!("  Factor Exposures:");
    
    for exposure in factor_analysis.exposures {
        println!("    {}: {}  (t-stat: {})", 
            exposure.factor_id, 
            exposure.exposure.round_dp(2), 
            exposure.t_stat.round_dp(2)
        );
    }
    
    // Perform scenario analysis
    let scenario_id = "MARKET_CRASH";
    let analysis_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    
    let scenario_analysis = analytics_engine.perform_scenario_analysis(
        portfolio_id,
        scenario_id,
        analysis_date,
        "DEMO-REQUEST-2",
    ).await?;
    
    println!("\nScenario Analysis Results:");
    println!("  Portfolio: {}", portfolio_id);
    println!("  Scenario: {}", scenario_id);
    println!("  Analysis Date: {}", analysis_date);
    println!("  Expected Return: {}%", (scenario_analysis.expected_return * dec!(100)).round_dp(2));
    println!("  Value at Risk: {}%", (scenario_analysis.value_at_risk * dec!(100)).round_dp(2));
    
    // Perform risk decomposition
    let risk_decomposition = analytics_engine.perform_risk_decomposition(
        portfolio_id,
        analysis_date,
        None,
        "DEMO-REQUEST-3",
    ).await?;
    
    println!("\nRisk Decomposition Results:");
    println!("  Portfolio: {}", portfolio_id);
    println!("  Analysis Date: {}", analysis_date);
    println!("  Total Risk: {}%", (risk_decomposition.total_risk * dec!(100)).round_dp(2));
    println!("  Systematic Risk: {}%", (risk_decomposition.systematic_risk * dec!(100)).round_dp(2));
    println!("  Specific Risk: {}%", (risk_decomposition.specific_risk * dec!(100)).round_dp(2));
    println!("  Factor Contributions:");
    
    for (factor_id, contribution) in risk_decomposition.factor_contributions {
        println!("    {}: {}%", factor_id, (contribution * dec!(100)).round_dp(2));
    }
    
    println!("\n2. Visualization Demo");
    println!("---------------------");
    
    // Create a performance chart
    let chart_options = ChartOptions {
        title: "Portfolio Performance".to_string(),
        subtitle: Some("Historical Returns".to_string()),
        chart_type: ChartType::Line,
        width: 800,
        height: 400,
        x_axis_title: Some("Date".to_string()),
        y_axis_title: Some("Return (%)".to_string()),
        show_legend: true,
        show_tooltips: true,
        enable_zoom: true,
        stacked: false,
        colors: None,
    };
    
    let portfolio_series = ChartSeries {
        name: "Portfolio".to_string(),
        data: vec![
            ("Jan".to_string(), dec!(2.5)),
            ("Feb".to_string(), dec!(-1.2)),
            ("Mar".to_string(), dec!(3.7)),
            ("Apr".to_string(), dec!(1.9)),
            ("May".to_string(), dec!(-0.8)),
            ("Jun".to_string(), dec!(4.2)),
        ],
        color: Some("#4285F4".to_string()),
        series_type: None,
    };
    
    let benchmark_series = ChartSeries {
        name: "Benchmark".to_string(),
        data: vec![
            ("Jan".to_string(), dec!(2.1)),
            ("Feb".to_string(), dec!(-0.9)),
            ("Mar".to_string(), dec!(3.2)),
            ("Apr".to_string(), dec!(1.5)),
            ("May".to_string(), dec!(-1.1)),
            ("Jun".to_string(), dec!(3.8)),
        ],
        color: Some("#DB4437".to_string()),
        series_type: None,
    };
    
    let chart_definition = ChartDefinition {
        options: chart_options,
        series: vec![portfolio_series, benchmark_series],
    };
    
    let chart_result = visualization_engine.generate_chart(
        chart_definition,
        ChartFormat::SVG,
        "DEMO-REQUEST-4",
    ).await?;
    
    println!("✅ Generated performance chart");
    println!("  Chart ID: {}", chart_result.id);
    println!("  Format: SVG");
    println!("  Size: {} bytes", chart_result.data.len());
    
    println!("\n3. Integration Demo");
    println!("------------------");
    
    // Simulate API integration
    println!("✅ Integration engine ready");
    println!("  Configured endpoints: {}", integration_config.api_endpoints.len());
    println!("  Email notifications: {}", if integration_config.email.enabled { "Enabled" } else { "Disabled" });
    println!("  Webhook notifications: {}", if integration_config.webhooks.enabled { "Enabled" } else { "Disabled" });
    
    println!("\nDemo completed successfully!");
    
    Ok(())
}

// Helper function to create sample returns
fn create_sample_returns(mean_return: f64) -> HashMap<NaiveDate, Decimal> {
    let mut returns = HashMap::new();
    let start_date = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
    
    for i in 0..12 {
        let date = start_date.checked_add_months(chrono::Months::new(i)).unwrap();
        let random_component = (i as f64 % 3.0 - 1.0) * 0.02;
        let return_value = mean_return / 12.0 + random_component;
        returns.insert(date, Decimal::try_from(return_value).unwrap());
    }
    
    returns
} 