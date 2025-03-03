use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use crate::calculations::error_handling::with_retry;

/// Currency code (ISO 4217)
pub type CurrencyCode = String;

/// Exchange rate between two currencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRate {
    /// Base currency (the currency being converted from)
    pub base_currency: CurrencyCode,
    /// Quote currency (the currency being converted to)
    pub quote_currency: CurrencyCode,
    /// Rate (how many units of quote currency per 1 unit of base currency)
    pub rate: Decimal,
    /// Date of the exchange rate
    pub date: NaiveDate,
    /// Source of the exchange rate data
    pub source: String,
}

/// Currency conversion service
#[derive(Clone)]
pub struct CurrencyConverter {
    /// Exchange rate provider
    exchange_rate_provider: Arc<dyn ExchangeRateProvider + Send + Sync>,
    /// Base currency for the portfolio/account
    base_currency: CurrencyCode,
}

impl CurrencyConverter {
    /// Create a new currency converter
    pub fn new(
        exchange_rate_provider: Arc<dyn ExchangeRateProvider + Send + Sync>,
        base_currency: CurrencyCode,
    ) -> Self {
        Self {
            exchange_rate_provider,
            base_currency,
        }
    }
    
    /// Convert an amount from one currency to another
    pub async fn convert(
        &self,
        amount: Decimal,
        from_currency: &CurrencyCode,
        to_currency: &CurrencyCode,
        date: NaiveDate,
        request_id: &str,
    ) -> Result<Decimal> {
        // If currencies are the same, no conversion needed
        if from_currency == to_currency {
            return Ok(amount);
        }
        
        // Get exchange rate
        let exchange_rate = self.exchange_rate_provider.get_exchange_rate(
            from_currency,
            to_currency,
            date,
            request_id,
        ).await?;
        
        // Convert amount
        Ok(amount * exchange_rate.rate)
    }
    
    /// Convert an amount to the base currency
    pub async fn convert_to_base(
        &self,
        amount: Decimal,
        from_currency: &CurrencyCode,
        date: NaiveDate,
        request_id: &str,
    ) -> Result<Decimal> {
        self.convert(
            amount,
            from_currency,
            &self.base_currency,
            date,
            request_id,
        ).await
    }
    
    /// Convert multiple amounts in different currencies to the base currency
    pub async fn convert_multiple_to_base(
        &self,
        amounts: &HashMap<CurrencyCode, Decimal>,
        date: NaiveDate,
        request_id: &str,
    ) -> Result<Decimal> {
        let mut total_base_amount = Decimal::ZERO;
        
        for (currency, amount) in amounts {
            let base_amount = self.convert_to_base(*amount, currency, date, request_id).await?;
            total_base_amount += base_amount;
        }
        
        Ok(total_base_amount)
    }
    
    /// Calculate FX impact for a position
    pub async fn calculate_fx_impact(
        &self,
        beginning_amount: Decimal,
        ending_amount: Decimal,
        currency: &CurrencyCode,
        beginning_date: NaiveDate,
        ending_date: NaiveDate,
        request_id: &str,
    ) -> Result<Decimal> {
        // If currency is the base currency, no FX impact
        if currency == &self.base_currency {
            return Ok(Decimal::ZERO);
        }
        
        // Get exchange rates
        let beginning_rate = self.exchange_rate_provider.get_exchange_rate(
            currency,
            &self.base_currency,
            beginning_date,
            request_id,
        ).await?.rate;
        
        let ending_rate = self.exchange_rate_provider.get_exchange_rate(
            currency,
            &self.base_currency,
            ending_date,
            request_id,
        ).await?.rate;
        
        // Calculate FX impact
        let beginning_base = beginning_amount * beginning_rate;
        let ending_base_at_beginning_rate = ending_amount * beginning_rate;
        let ending_base_at_ending_rate = ending_amount * ending_rate;
        
        let fx_impact = ending_base_at_ending_rate - ending_base_at_beginning_rate;
        
        Ok(fx_impact)
    }
}

/// Exchange rate provider interface
#[async_trait::async_trait]
pub trait ExchangeRateProvider {
    /// Get exchange rate between two currencies on a specific date
    async fn get_exchange_rate(
        &self,
        base_currency: &CurrencyCode,
        quote_currency: &CurrencyCode,
        date: NaiveDate,
        request_id: &str,
    ) -> Result<ExchangeRate>;
}

/// Exchange rate provider that uses a remote API
pub struct RemoteExchangeRateProvider {
    client: reqwest::Client,
    api_url: String,
    api_key: String,
}

impl RemoteExchangeRateProvider {
    /// Create a new remote exchange rate provider
    pub fn new(api_url: String, api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_url,
            api_key,
        }
    }
}

#[async_trait::async_trait]
impl ExchangeRateProvider for RemoteExchangeRateProvider {
    async fn get_exchange_rate(
        &self,
        base_currency: &CurrencyCode,
        quote_currency: &CurrencyCode,
        date: NaiveDate,
        request_id: &str,
    ) -> Result<ExchangeRate> {
        // Retry configuration for API calls
        let retry_config = crate::calculations::error_handling::RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 200,
            backoff_factor: 2.0,
            max_delay_ms: 2000,
        };
        
        // Define the operation to retry
        let operation = || {
            let client = self.client.clone();
            let api_url = self.api_url.clone();
            let api_key = self.api_key.clone();
            let base = base_currency.clone();
            let quote = quote_currency.clone();
            let date_str = date.format("%Y-%m-%d").to_string();
            
            async move {
                let url = format!(
                    "{}/exchangerates?base={}&symbols={}&date={}&access_key={}",
                    api_url, base, quote, date_str, api_key
                );
                
                let response = client.get(&url).send().await?;
                
                if !response.status().is_success() {
                    return Err(anyhow!(
                        "Failed to get exchange rate: HTTP {}", 
                        response.status()
                    ));
                }
                
                let data: serde_json::Value = response.json().await?;
                
                // Extract rate from response
                let rate_str = data["rates"][quote].as_str()
                    .ok_or_else(|| anyhow!("Rate not found in response"))?;
                
                let rate = Decimal::from_str(rate_str)
                    .map_err(|e| anyhow!("Failed to parse rate: {}", e))?;
                
                Ok(ExchangeRate {
                    base_currency: base,
                    quote_currency: quote,
                    rate,
                    date,
                    source: "API".to_string(),
                })
            }
        };
        
        // Execute with retry
        with_retry(
            operation,
            retry_config,
            "get_exchange_rate",
            request_id,
        ).await
    }
}

/// Exchange rate provider that uses a local cache
pub struct CachedExchangeRateProvider {
    delegate: Arc<dyn ExchangeRateProvider + Send + Sync>,
    cache: Arc<crate::calculations::cache::CalculationCache<String, ExchangeRate>>,
}

impl CachedExchangeRateProvider {
    /// Create a new cached exchange rate provider
    pub fn new(
        delegate: Arc<dyn ExchangeRateProvider + Send + Sync>,
        ttl_seconds: i64,
    ) -> Self {
        Self {
            delegate,
            cache: Arc::new(crate::calculations::cache::CalculationCache::new(ttl_seconds)),
        }
    }
    
    /// Generate cache key for exchange rate
    fn generate_cache_key(
        base_currency: &CurrencyCode,
        quote_currency: &CurrencyCode,
        date: NaiveDate,
    ) -> String {
        format!(
            "exchange_rate:{}:{}:{}",
            base_currency,
            quote_currency,
            date.format("%Y-%m-%d")
        )
    }
}

#[async_trait::async_trait]
impl ExchangeRateProvider for CachedExchangeRateProvider {
    async fn get_exchange_rate(
        &self,
        base_currency: &CurrencyCode,
        quote_currency: &CurrencyCode,
        date: NaiveDate,
        request_id: &str,
    ) -> Result<ExchangeRate> {
        let cache_key = Self::generate_cache_key(base_currency, quote_currency, date);
        
        let delegate = self.delegate.clone();
        let base = base_currency.clone();
        let quote = quote_currency.clone();
        
        // Try to get from cache, or compute if not present
        self.cache.get_or_compute(cache_key, || async move {
            delegate.get_exchange_rate(&base, &quote, date, request_id).await
        }).await
    }
}

/// Currency-aware performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyAwarePerformance {
    /// Base currency for the performance metrics
    pub base_currency: CurrencyCode,
    /// Local currency performance (in the currency of the security/account)
    pub local_performance: Decimal,
    /// Base currency performance (converted to the base currency)
    pub base_performance: Decimal,
    /// FX impact (contribution of currency movements to performance)
    pub fx_impact: Decimal,
}

impl CurrencyAwarePerformance {
    /// Create a new currency-aware performance metric
    pub fn new(
        base_currency: CurrencyCode,
        local_performance: Decimal,
        base_performance: Decimal,
        fx_impact: Decimal,
    ) -> Self {
        Self {
            base_currency,
            local_performance,
            base_performance,
            fx_impact,
        }
    }
    
    /// Calculate the percentage of performance attributable to FX
    pub fn fx_impact_percentage(&self) -> Decimal {
        if self.base_performance == Decimal::ZERO {
            return Decimal::ZERO;
        }
        
        (self.fx_impact / self.base_performance) * Decimal::from(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use rust_decimal_macros::dec;
    use mockall::predicate::*;
    use mockall::mock;
    
    // Mock exchange rate provider for testing
    mock! {
        ExchangeRateProviderMock {}
        
        #[async_trait::async_trait]
        impl ExchangeRateProvider for ExchangeRateProviderMock {
            async fn get_exchange_rate(
                &self,
                base_currency: &CurrencyCode,
                quote_currency: &CurrencyCode,
                date: NaiveDate,
                request_id: &str,
            ) -> Result<ExchangeRate>;
        }
    }
    
    #[tokio::test]
    async fn test_currency_conversion() {
        let mut mock_provider = MockExchangeRateProviderMock::new();
        
        // Set up mock expectations
        mock_provider
            .expect_get_exchange_rate()
            .with(
                eq("USD".to_string()),
                eq("EUR".to_string()),
                eq(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
                eq("test-request"),
            )
            .returning(|_, _, date, _| {
                Ok(ExchangeRate {
                    base_currency: "USD".to_string(),
                    quote_currency: "EUR".to_string(),
                    rate: dec!(0.85),
                    date,
                    source: "TEST".to_string(),
                })
            });
        
        let converter = CurrencyConverter::new(
            Arc::new(mock_provider),
            "USD".to_string(),
        );
        
        // Test conversion
        let result = converter.convert(
            dec!(100),
            &"USD".to_string(),
            &"EUR".to_string(),
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            "test-request",
        ).await.unwrap();
        
        assert_eq!(result, dec!(85));
    }
    
    #[tokio::test]
    async fn test_fx_impact() {
        let mut mock_provider = MockExchangeRateProviderMock::new();
        
        // Set up mock expectations for beginning date
        mock_provider
            .expect_get_exchange_rate()
            .with(
                eq("EUR".to_string()),
                eq("USD".to_string()),
                eq(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
                eq("test-request"),
            )
            .returning(|_, _, date, _| {
                Ok(ExchangeRate {
                    base_currency: "EUR".to_string(),
                    quote_currency: "USD".to_string(),
                    rate: dec!(1.10),
                    date,
                    source: "TEST".to_string(),
                })
            });
        
        // Set up mock expectations for ending date
        mock_provider
            .expect_get_exchange_rate()
            .with(
                eq("EUR".to_string()),
                eq("USD".to_string()),
                eq(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()),
                eq("test-request"),
            )
            .returning(|_, _, date, _| {
                Ok(ExchangeRate {
                    base_currency: "EUR".to_string(),
                    quote_currency: "USD".to_string(),
                    rate: dec!(1.20),
                    date,
                    source: "TEST".to_string(),
                })
            });
        
        let converter = CurrencyConverter::new(
            Arc::new(mock_provider),
            "USD".to_string(),
        );
        
        // Test FX impact calculation
        // Beginning: 100 EUR * 1.10 = 110 USD
        // Ending: 100 EUR * 1.20 = 120 USD
        // FX impact: 120 - (100 * 1.10) = 10 USD
        let result = converter.calculate_fx_impact(
            dec!(100),
            dec!(100),
            &"EUR".to_string(),
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 12, 31).unwrap(),
            "test-request",
        ).await.unwrap();
        
        assert_eq!(result, dec!(10));
    }
} 