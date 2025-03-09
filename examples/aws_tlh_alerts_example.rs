// This example demonstrates how to use the AWS TLH Alerts service
// This is a mock implementation for demonstration purposes
use std::collections::HashMap;
use anyhow::Result;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Mock types
struct AlgorithmicTLHConfig {
    min_loss_threshold: f64,
    min_holding_period_days: u32,
    max_wash_sale_window_days: u32,
    enable_cross_account_tlh: bool,
}

struct PortfolioHolding {
    security_id: String,
    quantity: f64,
    cost_basis: f64,
    market_value: f64,
    purchase_date: DateTime<Utc>,
}

struct AwsTlhAlertConfig {
    sns_topic_arn: String,
    region: String,
    min_alert_amount: f64,
    include_wash_sale_warnings: bool,
}

struct AwsTlhAlertService {
    config: AwsTlhAlertConfig,
}

impl AwsTlhAlertService {
    fn new(config: AwsTlhAlertConfig) -> Self {
        Self { config }
    }
    
    async fn send_tlh_alerts(&self, opportunities: Vec<TaxLossHarvestingOpportunity>) -> Result<()> {
        println!("Sending TLH alerts to SNS topic: {}", self.config.sns_topic_arn);
        
        for opportunity in opportunities {
            println!("  Alert for portfolio {}: {} (${:.2})", 
                opportunity.portfolio_id,
                opportunity.security_id,
                opportunity.loss_amount);
                
            if let Some(warnings) = opportunity.wash_sale_warnings {
                println!("    Wash sale warnings:");
                for warning in warnings {
                    println!("      {}", warning);
                }
            }
        }
        
        Ok(())
    }
}

struct TaxLossHarvestingOpportunity {
    portfolio_id: String,
    security_id: String,
    quantity: f64,
    loss_amount: f64,
    purchase_date: DateTime<Utc>,
    wash_sale_warnings: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== AWS Tax-Loss Harvesting Alerts Example ===\n");
    
    // Initialize TLH configuration
    let tlh_config = AlgorithmicTLHConfig {
        min_loss_threshold: 1000.0,
        min_holding_period_days: 30,
        max_wash_sale_window_days: 30,
        enable_cross_account_tlh: true,
    };
    
    // Create sample portfolio holdings
    let holdings = vec![
        PortfolioHolding {
            security_id: "AAPL".to_string(),
            quantity: 100.0,
            cost_basis: 15000.0,  // $150 per share
            market_value: 14000.0, // $140 per share (loss)
            purchase_date: Utc::now() - chrono::Duration::days(60),
        },
        PortfolioHolding {
            security_id: "MSFT".to_string(),
            quantity: 50.0,
            cost_basis: 15000.0,  // $300 per share
            market_value: 14000.0, // $280 per share (loss)
            purchase_date: Utc::now() - chrono::Duration::days(45),
        },
        PortfolioHolding {
            security_id: "GOOGL".to_string(),
            quantity: 20.0,
            cost_basis: 5000.0,   // $250 per share
            market_value: 5200.0,  // $260 per share (gain)
            purchase_date: Utc::now() - chrono::Duration::days(90),
        },
    ];
    
    // Initialize AWS TLH Alert Service
    let alert_config = AwsTlhAlertConfig {
        sns_topic_arn: "arn:aws:sns:us-east-1:123456789012:tax-loss-harvesting-alerts".to_string(),
        region: "us-east-1".to_string(),
        min_alert_amount: 500.0,
        include_wash_sale_warnings: true,
    };
    
    let alert_service = AwsTlhAlertService::new(alert_config);
    
    // Identify TLH opportunities (simplified algorithm)
    let mut opportunities = Vec::new();
    
    for holding in holdings {
        let loss = holding.cost_basis - holding.market_value;
        
        if loss > tlh_config.min_loss_threshold {
            let days_held = (Utc::now() - holding.purchase_date).num_days();
            
            if days_held >= tlh_config.min_holding_period_days as i64 {
                // Check for potential wash sales (simplified)
                let wash_sale_warnings = if holding.security_id == "AAPL" {
                    Some(vec![
                        "Recently purchased similar ETF in IRA account".to_string(),
                        "Consider alternative replacement security".to_string(),
                    ])
                } else {
                    None
                };
                
                opportunities.push(TaxLossHarvestingOpportunity {
                    portfolio_id: format!("portfolio-{}", Uuid::new_v4()),
                    security_id: holding.security_id,
                    quantity: holding.quantity,
                    loss_amount: loss,
                    purchase_date: holding.purchase_date,
                    wash_sale_warnings,
                });
            }
        }
    }
    
    // Send alerts
    if opportunities.is_empty() {
        println!("No tax-loss harvesting opportunities found");
    } else {
        println!("Found {} tax-loss harvesting opportunities", opportunities.len());
        alert_service.send_tlh_alerts(opportunities).await?;
    }
    
    println!("\nAWS TLH Alerts Example completed successfully!");
    Ok(())
} 