//! Performance benchmarks for the Performance Calculator
//!
//! This module contains benchmarks to measure the performance impact
//! of the new features added in Phase One.

use crate::calculations::{
    audit::{AuditTrail, InMemoryAuditTrail},
    currency::{CurrencyConverter, MockExchangeRateProvider, ExchangeRate},
    distributed_cache::{Cache, MockCache, RedisCache},
    performance_metrics::{calculate_twr, calculate_mwr},
};
use anyhow::Result;
use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Benchmark performance calculations with and without caching
pub fn benchmark_caching(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("Performance Calculation");
    
    // Setup components
    let mock_cache = Arc::new(MockCache::new());
    let no_cache = Arc::new(MockCache::new_always_miss());
    
    for portfolio_size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("Without Cache", portfolio_size), 
            portfolio_size,
            |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        black_box(calculate_with_portfolio_size(size, no_cache.clone()).await.unwrap())
                    })
                })
            }
        );
        
        group.bench_with_input(
            BenchmarkId::new("With Cache", portfolio_size), 
            portfolio_size,
            |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        black_box(calculate_with_portfolio_size(size, mock_cache.clone()).await.unwrap())
                    })
                })
            }
        );
    }
    
    group.finish();
}

/// Benchmark multi-currency calculations
pub fn benchmark_currency_conversion(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("Currency Conversion");
    
    // Setup exchange rate provider
    let exchange_rate_provider = Arc::new(MockExchangeRateProvider::new(vec![
        ExchangeRate {
            base_currency: "USD".to_string(),
            quote_currency: "EUR".to_string(),
            rate: dec!(0.85),
            date: Utc::now(),
            source: "test".to_string(),
        },
        ExchangeRate {
            base_currency: "USD".to_string(),
            quote_currency: "GBP".to_string(),
            rate: dec!(0.75),
            date: Utc::now(),
            source: "test".to_string(),
        },
        ExchangeRate {
            base_currency: "USD".to_string(),
            quote_currency: "JPY".to_string(),
            rate: dec!(110.0),
            date: Utc::now(),
            source: "test".to_string(),
        },
    ]));
    
    let currency_converter = CurrencyConverter::new(exchange_rate_provider);
    
    for num_currencies in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("Multi-Currency Portfolio", num_currencies), 
            num_currencies,
            |b, &num| {
                b.iter(|| {
                    rt.block_on(async {
                        black_box(calculate_multi_currency_portfolio(num, &currency_converter).unwrap())
                    })
                })
            }
        );
    }
    
    group.finish();
}

/// Helper function to calculate performance with different portfolio sizes
async fn calculate_with_portfolio_size(size: usize, cache: Arc<dyn Cache>) -> Result<Decimal> {
    let portfolio_id = format!("portfolio-{}", size);
    let cache_key = format!("benchmark:{}:twr", portfolio_id);
    
    // Try to get from cache
    if let Some(cached) = cache.get(&cache_key).await? {
        return Ok(cached.parse()?);
    }
    
    // Simulate calculation with different portfolio sizes
    // In a real scenario, this would involve processing transactions
    let start_value = Decimal::from(1000 * size);
    let end_value = start_value * dec!(1.1); // 10% return
    
    // Simulate some processing time based on portfolio size
    tokio::time::sleep(std::time::Duration::from_micros((size as u64).min(1000))).await;
    
    let twr = calculate_twr(start_value, end_value, Decimal::ZERO);
    
    // Cache the result
    cache.set(&cache_key, &twr.to_string()).await?;
    
    Ok(twr)
}

/// Helper function to calculate performance for multi-currency portfolios
fn calculate_multi_currency_portfolio(num_currencies: usize, converter: &CurrencyConverter) -> Result<Decimal> {
    let currencies = ["USD", "EUR", "GBP", "JPY", "CAD", "AUD", "CHF", "HKD", "SGD", "NZD"];
    let base_currency = "USD";
    
    let mut total_start_value_base = Decimal::ZERO;
    let mut total_end_value_base = Decimal::ZERO;
    
    // Use only as many currencies as requested, but no more than available
    let currencies_to_use = currencies.iter().take(num_currencies.min(currencies.len()));
    
    for &currency in currencies_to_use {
        let start_value = dec!(10000); // Same value in each currency
        let end_value = start_value * dec!(1.1); // 10% return in each currency
        
        // Convert to base currency
        let start_value_base = if currency == base_currency {
            start_value
        } else {
            converter.convert(start_value, currency, base_currency, None)?
        };
        
        let end_value_base = if currency == base_currency {
            end_value
        } else {
            converter.convert(end_value, currency, base_currency, None)?
        };
        
        total_start_value_base += start_value_base;
        total_end_value_base += end_value_base;
    }
    
    // Calculate TWR in base currency
    let twr = calculate_twr(total_start_value_base, total_end_value_base, Decimal::ZERO);
    
    Ok(twr)
}

criterion_group!(benches, benchmark_caching, benchmark_currency_conversion);
criterion_main!(benches); 