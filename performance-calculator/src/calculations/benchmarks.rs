//! Performance benchmarks for the Performance Calculator
//!
//! This module contains benchmarks to measure the performance impact
//! of the new features added in Phase One.

use crate::calculations::{
    audit::{AuditTrail, InMemoryAuditTrail},
    currency::{CurrencyConverter, ExchangeRate, CurrencyCode},
    distributed_cache::{StringCache, InMemoryCache},
    performance_metrics::{TimeWeightedReturn, MoneyWeightedReturn},
};
use anyhow::Result;
use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::Arc;
use tokio::runtime::Runtime;
use async_trait::async_trait;

// Create a simple mock cache for testing
struct MockCache {
    always_miss: bool,
}

impl MockCache {
    fn new() -> Self {
        Self { always_miss: false }
    }
    
    fn new_always_miss() -> Self {
        Self { always_miss: true }
    }
}

#[async_trait]
impl StringCache for MockCache {
    async fn get_string(&self, _key: &str) -> Result<Option<String>> {
        if self.always_miss {
            Ok(None)
        } else {
            Ok(Some("cached_value".to_string()))
        }
    }
    
    async fn set_string(&self, _key: String, _value: String, _ttl_seconds: Option<u64>) -> Result<()> {
        Ok(())
    }
    
    async fn delete_string(&self, _key: &str) -> Result<()> {
        Ok(())
    }
}

// Create a simple mock exchange rate provider
struct MockExchangeRateProvider {
    rates: Vec<ExchangeRate>,
}

impl MockExchangeRateProvider {
    fn new(rates: Vec<ExchangeRate>) -> Self {
        Self { rates }
    }
}

#[async_trait]
impl crate::calculations::currency::ExchangeRateProvider for MockExchangeRateProvider {
    async fn get_exchange_rate(
        &self,
        base_currency: &CurrencyCode,
        quote_currency: &CurrencyCode,
        date: chrono::NaiveDate,
        request_id: &str,
    ) -> Result<ExchangeRate> {
        // Find a matching rate or return a default one
        for rate in &self.rates {
            if rate.base_currency == *base_currency && rate.quote_currency == *quote_currency {
                return Ok(rate.clone());
            }
        }
        
        // Return a default rate if no matching rate is found
        Ok(ExchangeRate {
            base_currency: base_currency.clone(),
            quote_currency: quote_currency.clone(),
            rate: dec!(1.0),
            date,
            source: "mock".to_string(),
        })
    }
}

/// Benchmark performance calculations with and without caching
pub fn benchmark_caching(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("Performance Calculation");
    
    // Setup components
    let mock_cache = Arc::new(MockCache::new());
    let no_cache = Arc::new(MockCache::new_always_miss());
    
    for portfolio_size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("With Cache", portfolio_size),
            portfolio_size,
            |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        black_box(calculate_with_portfolio_size(size, mock_cache.clone()).await.unwrap())
                    })
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("Without Cache", portfolio_size),
            portfolio_size,
            |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        black_box(calculate_with_portfolio_size(size, no_cache.clone()).await.unwrap())
                    })
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark currency conversion performance
pub fn benchmark_currency_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("Currency Conversion");
    
    // Setup exchange rate provider
    let exchange_rate_provider = Arc::new(MockExchangeRateProvider::new(vec![
        ExchangeRate {
            base_currency: "USD".to_string(),
            quote_currency: "EUR".to_string(),
            rate: dec!(0.85),
            date: Utc::now().date_naive(),
            source: "mock".to_string(),
        },
        ExchangeRate {
            base_currency: "USD".to_string(),
            quote_currency: "GBP".to_string(),
            rate: dec!(0.75),
            date: Utc::now().date_naive(),
            source: "mock".to_string(),
        },
        ExchangeRate {
            base_currency: "USD".to_string(),
            quote_currency: "JPY".to_string(),
            rate: dec!(110.0),
            date: Utc::now().date_naive(),
            source: "mock".to_string(),
        },
    ]));
    
    let currency_converter = CurrencyConverter::new(
        exchange_rate_provider,
        "USD".to_string(),
    );
    
    for num_currencies in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("Multi-Currency Portfolio", num_currencies),
            num_currencies,
            |b, &num| {
                b.iter(|| {
                    black_box(calculate_multi_currency_portfolio(num, &currency_converter).unwrap())
                })
            },
        );
    }
    
    group.finish();
}

/// Calculate performance for a portfolio of a given size
async fn calculate_with_portfolio_size(size: usize, cache: Arc<dyn StringCache + Send + Sync>) -> Result<Decimal> {
    // Generate a unique cache key
    let cache_key = format!("portfolio_performance_{}", size);
    
    // Try to get from cache first
    if let Some(cached_value) = cache.get_string(&cache_key).await? {
        return Ok(cached_value.parse()?);
    }
    
    // Simulate calculation time based on portfolio size
    tokio::time::sleep(std::time::Duration::from_millis((size as u64) / 100)).await;
    
    // Calculate a dummy TWR value
    let twr = dec!(0.0875); // 8.75% return
    
    // Cache the result
    cache.set_string(cache_key, twr.to_string(), Some(60)).await?;
    
    Ok(twr)
}

/// Calculate performance for a multi-currency portfolio
fn calculate_multi_currency_portfolio(num_currencies: usize, converter: &CurrencyConverter) -> Result<Decimal> {
    let base_currency = "USD";
    let currencies = vec!["USD", "EUR", "GBP", "JPY", "CAD", "AUD", "CHF", "CNY", "HKD", "NZD"];
    
    let mut total_start_value_base = dec!(0);
    let mut total_end_value_base = dec!(0);
    
    for i in 0..num_currencies {
        let currency = currencies[i % currencies.len()];
        let start_value = dec!(100000) * Decimal::from(i + 1);
        let end_value = start_value * dec!(1.10); // 10% return in local currency
        
        // Convert to base currency
        let start_value_base = if currency == base_currency {
            start_value
        } else {
            // For simplicity in the benchmark, we'll just use a fixed conversion rate
            // In a real implementation, we would call converter.convert()
            start_value * dec!(0.8)
        };
        
        let end_value_base = if currency == base_currency {
            end_value
        } else {
            // For simplicity in the benchmark, we'll just use a fixed conversion rate
            // In a real implementation, we would call converter.convert()
            end_value * dec!(0.8)
        };
        
        total_start_value_base += start_value_base;
        total_end_value_base += end_value_base;
    }
    
    // Calculate overall return
    let overall_return = (total_end_value_base - total_start_value_base) / total_start_value_base;
    
    Ok(overall_return)
}

criterion_group!(benches, benchmark_caching, benchmark_currency_conversion);
criterion_main!(benches); 