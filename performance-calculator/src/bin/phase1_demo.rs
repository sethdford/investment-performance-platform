//! Phase 1 Demo Application
//! 
//! This application demonstrates the core functionality of the Performance Calculator:
//! - Multi-Currency Support
//! - Distributed Caching with Redis
//! - Audit Trail and Calculation Lineage
//! - Configuration Management
//! - Component Factory
//! - Performance Metrics
//! - Risk Metrics

use anyhow::{Result, Context};
use chrono::{NaiveDate, Utc};
use performance_calculator::calculations::{
    audit::AuditRecord,
    config::Config,
    factory::ComponentFactory,
    portfolio::{Portfolio, Holding, CashBalance, Transaction},
    events::TransactionType,
};
use rust_decimal_macros::dec;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    println!("Performance Calculator - Phase 1 Demo");
    println!("=====================================");
    
    // Create configuration
    let mut config = Config::default();
    
    // Enable Redis cache
    config.redis_cache = Some(performance_calculator::calculations::distributed_cache::RedisCacheConfig {
        enabled: true,
        url: "redis://localhost:6379".to_string(),
        prefix: "demo:".to_string(),
        ttl_seconds: 3600,
    });
    
    println!("\n📋 Created configuration");
    
    // Create component factory
    let factory = ComponentFactory::new(config);
    println!("🏭 Created component factory");
    
    // Create components
    println!("\n🧩 Creating components...");
    
    let cache = factory.create_redis_cache().await?;
    println!("✅ Created Redis cache");
    
    let currency_converter = factory.create_currency_converter().await?;
    println!("✅ Created currency converter");
    
    let audit_trail = factory.create_audit_trail().await?;
    println!("✅ Created audit trail");
    
    let twr_calculator = factory.create_twr_calculator();
    println!("✅ Created TWR calculator");
    
    let mwr_calculator = factory.create_mwr_calculator();
    println!("✅ Created MWR calculator");
    
    let risk_calculator = factory.create_risk_calculator();
    println!("✅ Created risk calculator");
    
    // Create a portfolio
    println!("\n📊 Creating portfolio...");
    let mut portfolio = Portfolio::new("DEMO-PORTFOLIO", "USD");
    
    // Add initial cash balance
    portfolio.add_cash_balance(CashBalance {
        currency: "USD".to_string(),
        amount: dec!(10000),
    });
    
    // Add transactions
    let transactions = vec![
        Transaction {
            id: "T1".to_string(),
            transaction_date: NaiveDate::from_ymd_opt(2023, 1, 15).unwrap(),
            settlement_date: Some(NaiveDate::from_ymd_opt(2023, 1, 17).unwrap()),
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
            transaction_date: NaiveDate::from_ymd_opt(2023, 2, 10).unwrap(),
            settlement_date: Some(NaiveDate::from_ymd_opt(2023, 2, 12).unwrap()),
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
            transaction_date: NaiveDate::from_ymd_opt(2023, 3, 15).unwrap(),
            settlement_date: Some(NaiveDate::from_ymd_opt(2023, 3, 17).unwrap()),
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
            transaction_date: NaiveDate::from_ymd_opt(2023, 4, 20).unwrap(),
            settlement_date: Some(NaiveDate::from_ymd_opt(2023, 4, 22).unwrap()),
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
    
    println!("✅ Created portfolio with {} transactions and {} holdings", 
             portfolio.transactions.len(), portfolio.holdings.len());
    
    // Define calculation period
    let start_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2023, 4, 30).unwrap();
    
    // Calculate TWR
    println!("\n📈 Calculating Time-Weighted Return (TWR)...");
    let twr_result = twr_calculator.calculate_twr(
        &portfolio,
        start_date,
        end_date,
    ).await?;
    
    println!("TWR Result: {:.2}%", twr_result * dec!(100));
    
    // Calculate MWR
    println!("\n📉 Calculating Money-Weighted Return (MWR)...");
    let mwr_result = mwr_calculator.calculate_mwr(
        &portfolio,
        start_date,
        end_date,
    ).await?;
    
    println!("MWR Result: {:.2}%", mwr_result * dec!(100));
    
    // Calculate risk metrics
    println!("\n⚠️ Calculating Risk Metrics...");
    let volatility = risk_calculator.calculate_volatility(
        &portfolio,
        start_date,
        end_date,
    ).await?;
    
    println!("Volatility: {:.2}%", volatility * dec!(100));
    
    let sharpe_ratio = risk_calculator.calculate_sharpe_ratio(
        &portfolio,
        start_date,
        end_date,
        dec!(0.02), // 2% risk-free rate
    ).await?;
    
    println!("Sharpe Ratio: {:.2}", sharpe_ratio);
    
    let max_drawdown = risk_calculator.calculate_max_drawdown(
        &portfolio,
        start_date,
        end_date,
    ).await?;
    
    println!("Maximum Drawdown: {:.2}%", max_drawdown * dec!(100));
    
    // Demonstrate multi-currency conversion
    if let Some(converter) = currency_converter {
        println!("\n💱 Testing currency conversion:");
        
        let amount = dec!(1000);
        let from_currency = "USD";
        let to_currencies = vec!["EUR", "GBP", "JPY"];
        
        for to_currency in to_currencies {
            match converter.convert(amount, from_currency, to_currency, None) {
                Ok(converted) => {
                    println!("  {} {} = {} {}", amount, from_currency, converted, to_currency);
                },
                Err(e) => {
                    println!("  Error converting {} {} to {}: {}", amount, from_currency, to_currency, e);
                }
            }
        }
    }
    
    // Demonstrate audit trail
    if let Some(audit) = audit_trail {
        println!("\n📝 Recording to audit trail...");
        
        // Record calculation to audit trail
        let audit_record = AuditRecord {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            entity_id: portfolio.id.clone(),
            entity_type: "portfolio".to_string(),
            action: "calculate_performance".to_string(),
            user_id: "demo_user".to_string(),
            parameters: format!(
                "start_date={},end_date={},base_currency={}",
                start_date, end_date, portfolio.base_currency
            ),
            result: format!(
                r#"{{"twr":{},"mwr":{},"volatility":{},"sharpe_ratio":{},"max_drawdown":{}}}"#,
                twr_result, mwr_result, volatility, sharpe_ratio, max_drawdown
            ),
        };
        
        audit.record(audit_record).await?;
        println!("✅ Recorded calculation to audit trail");
        
        // Retrieve audit records
        let records = audit.get_records_for_entity("portfolio", &portfolio.id).await?;
        println!("Retrieved {} audit records", records.len());
        
        for (i, record) in records.iter().enumerate() {
            println!("  Record {}: {} - {} by {}", 
                i + 1, 
                record.timestamp.format("%Y-%m-%d %H:%M:%S"),
                record.action,
                record.user_id
            );
        }
    }
    
    println!("\n✅ Phase 1 demo completed successfully");
    
    Ok(())
} 