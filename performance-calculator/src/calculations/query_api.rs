use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::calculations::{
    performance_metrics::{
        TimeWeightedReturn, 
        MoneyWeightedReturn, 
        CashFlow,
        PerformanceAttribution,
        calculate_modified_dietz,
        calculate_daily_twr,
        calculate_irr,
        annualize_return,
        calculate_attribution
    },
    risk_metrics::{
        RiskMetrics, 
        ReturnSeries,
        calculate_risk_metrics
    },
    benchmark_comparison::{
        BenchmarkComparison,
        calculate_benchmark_comparison
    },
    periodic_returns::{
        Period,
        PeriodicReturn,
        calculate_all_periodic_returns
    },
    audit::{
        AuditTrailManager,
        CalculationEventBuilder
    },
    distributed_cache::{Cache},
    currency::{
        CurrencyConverter,
        CurrencyCode
    }
};

/// Query parameters for performance calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceQueryParams {
    /// Portfolio or account identifier
    pub portfolio_id: String,
    
    /// Start date for the calculation period
    pub start_date: NaiveDate,
    
    /// End date for the calculation period
    pub end_date: NaiveDate,
    
    /// Calculation method for TWR (Modified Dietz, Daily, etc.)
    pub twr_method: Option<String>,
    
    /// Whether to include risk metrics
    pub include_risk_metrics: Option<bool>,
    
    /// Whether to include periodic returns
    pub include_periodic_returns: Option<bool>,
    
    /// Benchmark identifier for comparison
    pub benchmark_id: Option<String>,
    
    /// Currency for the results
    pub currency: Option<CurrencyCode>,
    
    /// Whether to annualize returns
    pub annualize: Option<bool>,
    
    /// Custom parameters for specific calculations
    pub custom_params: Option<HashMap<String, serde_json::Value>>,
}

/// Query parameters for risk metrics calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskQueryParams {
    /// Portfolio or account identifier
    pub portfolio_id: String,
    
    /// Start date for the calculation period
    pub start_date: NaiveDate,
    
    /// End date for the calculation period
    pub end_date: NaiveDate,
    
    /// Return frequency (daily, weekly, monthly)
    pub return_frequency: String,
    
    /// Confidence level for VaR and CVaR calculations
    pub confidence_level: Option<f64>,
    
    /// Benchmark identifier for comparison
    pub benchmark_id: Option<String>,
    
    /// Risk-free rate for Sharpe ratio calculation
    pub risk_free_rate: Option<f64>,
    
    /// Custom parameters for specific calculations
    pub custom_params: Option<HashMap<String, serde_json::Value>>,
}

/// Query parameters for attribution analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionQueryParams {
    /// Portfolio or account identifier
    pub portfolio_id: String,
    
    /// Start date for the calculation period
    pub start_date: NaiveDate,
    
    /// End date for the calculation period
    pub end_date: NaiveDate,
    
    /// Benchmark identifier for comparison
    pub benchmark_id: String,
    
    /// Asset class field name
    pub asset_class_field: String,
    
    /// Whether to include sector attribution
    pub include_sector: Option<bool>,
    
    /// Whether to include security selection attribution
    pub include_security_selection: Option<bool>,
    
    /// Custom parameters for specific calculations
    pub custom_params: Option<HashMap<String, serde_json::Value>>,
}

/// Query parameters for what-if analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatIfQueryParams {
    /// Portfolio or account identifier
    pub portfolio_id: String,
    
    /// Start date for the calculation period
    pub start_date: NaiveDate,
    
    /// End date for the calculation period
    pub end_date: NaiveDate,
    
    /// Hypothetical transactions to apply
    pub hypothetical_transactions: Vec<HypotheticalTransaction>,
    
    /// Whether to include risk metrics
    pub include_risk_metrics: Option<bool>,
    
    /// Benchmark identifier for comparison
    pub benchmark_id: Option<String>,
    
    /// Custom parameters for specific calculations
    pub custom_params: Option<HashMap<String, serde_json::Value>>,
}

/// Hypothetical transaction for what-if analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypotheticalTransaction {
    /// Transaction date
    pub date: NaiveDate,
    
    /// Security identifier
    pub security_id: String,
    
    /// Transaction type (buy, sell, etc.)
    pub transaction_type: String,
    
    /// Transaction amount
    pub amount: f64,
    
    /// Transaction quantity
    pub quantity: Option<f64>,
    
    /// Transaction currency
    pub currency: CurrencyCode,
}

/// Performance calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceResult {
    /// Query ID
    pub query_id: String,
    
    /// Portfolio ID
    pub portfolio_id: String,
    
    /// Start date
    pub start_date: NaiveDate,
    
    /// End date
    pub end_date: NaiveDate,
    
    /// Time-weighted return
    pub time_weighted_return: Option<TimeWeightedReturn>,
    
    /// Money-weighted return
    pub money_weighted_return: Option<MoneyWeightedReturn>,
    
    /// Risk metrics
    pub risk_metrics: Option<RiskMetrics>,
    
    /// Periodic returns
    pub periodic_returns: Option<Vec<PeriodicReturn>>,
    
    /// Benchmark comparison
    pub benchmark_comparison: Option<BenchmarkComparison>,
    
    /// Attribution analysis
    pub attribution: Option<PerformanceAttribution>,
    
    /// Currency of the results
    pub currency: CurrencyCode,
    
    /// Calculation timestamp
    pub calculation_time: DateTime<Utc>,
}

/// Risk metrics calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskResult {
    /// Query ID
    pub query_id: String,
    
    /// Portfolio ID
    pub portfolio_id: String,
    
    /// Start date
    pub start_date: NaiveDate,
    
    /// End date
    pub end_date: NaiveDate,
    
    /// Risk metrics
    pub risk_metrics: RiskMetrics,
    
    /// Return frequency used
    pub return_frequency: String,
    
    /// Confidence level used for VaR and CVaR
    pub confidence_level: Option<f64>,
    
    /// Benchmark ID if used
    pub benchmark_id: Option<String>,
    
    /// Calculation timestamp
    pub calculation_time: DateTime<Utc>,
}

/// Attribution analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionResult {
    /// Query ID
    pub query_id: String,
    
    /// Portfolio ID
    pub portfolio_id: String,
    
    /// Start date
    pub start_date: NaiveDate,
    
    /// End date
    pub end_date: NaiveDate,
    
    /// Benchmark ID
    pub benchmark_id: String,
    
    /// Overall attribution
    pub overall_attribution: PerformanceAttribution,
    
    /// Attribution by asset class
    pub asset_class_attribution: HashMap<String, PerformanceAttribution>,
    
    /// Attribution by sector (if requested)
    pub sector_attribution: Option<HashMap<String, PerformanceAttribution>>,
    
    /// Attribution by security (if requested)
    pub security_attribution: Option<HashMap<String, PerformanceAttribution>>,
    
    /// Calculation timestamp
    pub calculation_time: DateTime<Utc>,
}

/// What-if analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatIfResult {
    /// Query ID
    pub query_id: String,
    
    /// Portfolio ID
    pub portfolio_id: String,
    
    /// Start date
    pub start_date: NaiveDate,
    
    /// End date
    pub end_date: NaiveDate,
    
    /// Original performance
    pub original_performance: PerformanceResult,
    
    /// Hypothetical performance
    pub hypothetical_performance: PerformanceResult,
    
    /// Difference in performance
    pub performance_difference: f64,
    
    /// Calculation timestamp
    pub calculation_time: DateTime<Utc>,
}

/// Interactive query API for performance calculations
pub struct QueryApi {
    /// Audit trail manager
    audit_manager: Arc<AuditTrailManager>,
    
    /// Cache for query results
    cache: Arc<dyn Cache<String, serde_json::Value> + Send + Sync>,
    
    /// Currency converter
    currency_converter: Arc<CurrencyConverter>,
    
    /// Data access service (simplified)
    data_service: Arc<dyn DataAccessService>,
}

impl QueryApi {
    /// Create a new query API
    pub fn new(
        audit_manager: Arc<AuditTrailManager>,
        cache: Arc<dyn Cache<String, serde_json::Value> + Send + Sync>,
        currency_converter: Arc<CurrencyConverter>,
        data_service: Arc<dyn DataAccessService>,
    ) -> Self {
        Self {
            audit_manager,
            cache,
            currency_converter,
            data_service,
        }
    }
    
    /// Calculate performance metrics
    pub async fn calculate_performance(&self, params: PerformanceQueryParams) -> Result<PerformanceResult> {
        let query_id = Uuid::new_v4().to_string();
        let request_id = format!("query:{}", query_id);
        
        // Start audit trail
        let mut input_params = HashMap::new();
        input_params.insert("portfolio_id".to_string(), serde_json::json!(params.portfolio_id));
        input_params.insert("start_date".to_string(), serde_json::json!(params.start_date.to_string()));
        input_params.insert("end_date".to_string(), serde_json::json!(params.end_date.to_string()));
        
        if let Some(twr_method) = &params.twr_method {
            input_params.insert("twr_method".to_string(), serde_json::json!(twr_method));
        }
        
        if let Some(benchmark_id) = &params.benchmark_id {
            input_params.insert("benchmark_id".to_string(), serde_json::json!(benchmark_id));
        }
        
        let event = self.audit_manager.start_calculation(
            "interactive_performance_query",
            &request_id,
            "query_api",
            input_params,
            vec![format!("portfolio:{}", params.portfolio_id)],
        ).await?;
        
        // Check cache first
        let cache_key = format!(
            "performance:{}:{}:{}:{}",
            params.portfolio_id,
            params.start_date,
            params.end_date,
            serde_json::to_string(&params).unwrap_or_default()
        );
        
        let result = self.cache.get_or_compute(
            cache_key,
            3600, // 1 hour TTL
            || async {
                // This closure will be called if the result is not in the cache
                
                // Fetch portfolio data
                let portfolio_data = self.data_service.get_portfolio_data(
                    &params.portfolio_id,
                    params.start_date,
                    params.end_date,
                ).await?;
                
                // Calculate TWR
                let twr_method = params.twr_method.as_deref().unwrap_or("daily");
                let time_weighted_return = match twr_method {
                    "modified_dietz" => {
                        // Calculate Modified Dietz TWR
                        let twr = calculate_modified_dietz(
                            portfolio_data.beginning_market_value,
                            portfolio_data.ending_market_value,
                            &portfolio_data.cash_flows,
                        )?;
                        
                        Some(TimeWeightedReturn {
                            return_value: twr,
                            calculation_method: "Modified Dietz".to_string(),
                            sub_period_returns: Vec::new(), // Simplified
                            is_annualized: false,
                        })
                    },
                    "daily" => {
                        // Calculate Daily TWR
                        let twr = calculate_daily_twr(
                            &portfolio_data.daily_market_values,
                            &portfolio_data.cash_flows,
                        )?;
                        
                        Some(TimeWeightedReturn {
                            return_value: twr,
                            calculation_method: "Daily".to_string(),
                            sub_period_returns: Vec::new(), // Simplified
                            is_annualized: false,
                        })
                    },
                    _ => None,
                };
                
                // Calculate MWR (IRR)
                let money_weighted_return = if !portfolio_data.cash_flows.is_empty() {
                    let mwr = calculate_irr(
                        portfolio_data.beginning_market_value,
                        portfolio_data.ending_market_value,
                        &portfolio_data.cash_flows,
                    )?;
                    
                    Some(MoneyWeightedReturn {
                        return_value: mwr,
                        calculation_method: "IRR".to_string(),
                        is_annualized: false,
                    })
                } else {
                    None
                };
                
                // Calculate risk metrics if requested
                let risk_metrics = if params.include_risk_metrics.unwrap_or(false) {
                    // Create return series from daily returns
                    let return_series = ReturnSeries {
                        dates: portfolio_data.daily_returns.keys().cloned().collect(),
                        returns: portfolio_data.daily_returns.values().cloned().collect(),
                    };
                    
                    // Get benchmark returns if specified
                    let benchmark_returns = if let Some(benchmark_id) = &params.benchmark_id {
                        Some(self.data_service.get_benchmark_returns(
                            benchmark_id,
                            params.start_date,
                            params.end_date,
                        ).await?)
                    } else {
                        None
                    };
                    
                    // Calculate risk metrics
                    Some(calculate_risk_metrics(&return_series, benchmark_returns.as_ref())?)
                } else {
                    None
                };
                
                // Calculate periodic returns if requested
                let periodic_returns = if params.include_periodic_returns.unwrap_or(false) {
                    // Create return series from daily returns
                    let return_series = ReturnSeries {
                        dates: portfolio_data.daily_returns.keys().cloned().collect(),
                        returns: portfolio_data.daily_returns.values().cloned().collect(),
                    };
                    
                    // Calculate all periodic returns
                    Some(calculate_all_periodic_returns(&return_series)?)
                } else {
                    None
                };
                
                // Calculate benchmark comparison if benchmark specified
                let benchmark_comparison = if let Some(benchmark_id) = &params.benchmark_id {
                    // Get benchmark returns
                    let benchmark_returns = self.data_service.get_benchmark_returns(
                        benchmark_id,
                        params.start_date,
                        params.end_date,
                    ).await?;
                    
                    // Create return series
                    let portfolio_return_series = ReturnSeries {
                        dates: portfolio_data.daily_returns.keys().cloned().collect(),
                        returns: portfolio_data.daily_returns.values().cloned().collect(),
                    };
                    
                    // Calculate benchmark comparison
                    Some(calculate_benchmark_comparison(
                        &portfolio_return_series,
                        &benchmark_returns,
                    )?)
                } else {
                    None
                };
                
                // Convert currency if needed
                let currency = params.currency.clone().unwrap_or_else(|| portfolio_data.currency.clone());
                
                // Create result
                let result = PerformanceResult {
                    query_id,
                    portfolio_id: params.portfolio_id,
                    start_date: params.start_date,
                    end_date: params.end_date,
                    time_weighted_return,
                    money_weighted_return,
                    risk_metrics,
                    periodic_returns,
                    benchmark_comparison,
                    attribution: None, // Attribution requires separate calculation
                    currency,
                    calculation_time: Utc::now(),
                };
                
                // Serialize to JSON for caching
                let json_result = serde_json::to_value(&result)
                    .map_err(|e| anyhow!("Failed to serialize performance result: {}", e))?;
                
                Ok(json_result)
            },
        ).await?;
        
        // Deserialize from JSON
        let performance_result: PerformanceResult = serde_json::from_value(result)
            .map_err(|e| anyhow!("Failed to deserialize performance result: {}", e))?;
        
        // Complete audit trail
        self.audit_manager.complete_calculation(
            &event.event_id,
            vec![format!("performance_result:{}", query_id)],
        ).await?;
        
        Ok(performance_result)
    }
    
    /// Calculate risk metrics
    pub async fn calculate_risk(&self, params: RiskQueryParams) -> Result<RiskResult> {
        let query_id = Uuid::new_v4().to_string();
        let request_id = format!("query:{}", query_id);
        
        // Start audit trail
        let mut input_params = HashMap::new();
        input_params.insert("portfolio_id".to_string(), serde_json::json!(params.portfolio_id));
        input_params.insert("start_date".to_string(), serde_json::json!(params.start_date.to_string()));
        input_params.insert("end_date".to_string(), serde_json::json!(params.end_date.to_string()));
        input_params.insert("return_frequency".to_string(), serde_json::json!(params.return_frequency));
        
        if let Some(confidence_level) = params.confidence_level {
            input_params.insert("confidence_level".to_string(), serde_json::json!(confidence_level));
        }
        
        if let Some(benchmark_id) = &params.benchmark_id {
            input_params.insert("benchmark_id".to_string(), serde_json::json!(benchmark_id));
        }
        
        let event = self.audit_manager.start_calculation(
            "interactive_risk_query",
            &request_id,
            "query_api",
            input_params,
            vec![format!("portfolio:{}", params.portfolio_id)],
        ).await?;
        
        // Check cache first
        let cache_key = format!(
            "risk:{}:{}:{}:{}",
            params.portfolio_id,
            params.start_date,
            params.end_date,
            serde_json::to_string(&params).unwrap_or_default()
        );
        
        let result = self.cache.get_or_compute(
            cache_key,
            3600, // 1 hour TTL
            || async {
                // Get returns based on frequency
                let returns = self.data_service.get_portfolio_returns(
                    &params.portfolio_id,
                    params.start_date,
                    params.end_date,
                    &params.return_frequency,
                ).await?;
                
                // Create return series
                let return_series = ReturnSeries {
                    dates: returns.keys().cloned().collect(),
                    returns: returns.values().cloned().collect(),
                };
                
                // Get benchmark returns if specified
                let benchmark_returns = if let Some(benchmark_id) = &params.benchmark_id {
                    Some(self.data_service.get_benchmark_returns_by_frequency(
                        benchmark_id,
                        params.start_date,
                        params.end_date,
                        &params.return_frequency,
                    ).await?)
                } else {
                    None
                };
                
                // Set risk-free rate if provided
                let risk_free_rate = params.risk_free_rate;
                
                // Calculate risk metrics
                let risk_metrics = calculate_risk_metrics(&return_series, benchmark_returns.as_ref())?;
                
                // Create result
                let result = RiskResult {
                    query_id: query_id.clone(),
                    portfolio_id: params.portfolio_id.clone(),
                    start_date: params.start_date,
                    end_date: params.end_date,
                    risk_metrics,
                    return_frequency: params.return_frequency.clone(),
                    confidence_level: params.confidence_level,
                    benchmark_id: params.benchmark_id.clone(),
                    calculation_time: Utc::now(),
                };
                
                // Serialize to JSON for caching
                let json_result = serde_json::to_value(&result)
                    .map_err(|e| anyhow!("Failed to serialize risk result: {}", e))?;
                
                Ok(json_result)
            }
        ).await?;
        
        // Deserialize from JSON
        let risk_result: RiskResult = serde_json::from_value(result)
            .map_err(|e| anyhow!("Failed to deserialize risk result: {}", e))?;
        
        // Complete audit trail
        self.audit_manager.complete_calculation(
            &event.event_id,
            vec![format!("risk_result:{}", query_id)],
        ).await?;
        
        Ok(risk_result)
    }
    
    /// Calculate attribution analysis
    pub async fn calculate_attribution(&self, params: AttributionQueryParams) -> Result<AttributionResult> {
        let query_id = Uuid::new_v4().to_string();
        let request_id = format!("query:{}", query_id);
        
        // Start audit trail
        let mut input_params = HashMap::new();
        input_params.insert("portfolio_id".to_string(), serde_json::json!(params.portfolio_id));
        input_params.insert("start_date".to_string(), serde_json::json!(params.start_date.to_string()));
        input_params.insert("end_date".to_string(), serde_json::json!(params.end_date.to_string()));
        input_params.insert("benchmark_id".to_string(), serde_json::json!(params.benchmark_id));
        input_params.insert("asset_class_field".to_string(), serde_json::json!(params.asset_class_field));
        
        let event = self.audit_manager.start_calculation(
            "interactive_attribution_query",
            &request_id,
            "query_api",
            input_params,
            vec![
                format!("portfolio:{}", params.portfolio_id),
                format!("benchmark:{}", params.benchmark_id),
            ],
        ).await?;
        
        // Check cache first
        let cache_key = format!(
            "attribution:{}:{}:{}:{}:{}",
            params.portfolio_id,
            params.benchmark_id,
            params.start_date,
            params.end_date,
            serde_json::to_string(&params).unwrap_or_default()
        );
        
        let result = self.cache.get_or_compute(
            cache_key,
            3600, // 1 hour TTL
            || async {
                // Get portfolio and benchmark data
                let portfolio_data = self.data_service.get_portfolio_holdings_with_returns(
                    &params.portfolio_id,
                    params.start_date,
                    params.end_date,
                ).await?;
                
                let benchmark_data = self.data_service.get_benchmark_holdings_with_returns(
                    &params.benchmark_id,
                    params.start_date,
                    params.end_date,
                ).await?;
                
                // Calculate overall attribution
                let overall_attribution = calculate_attribution(
                    portfolio_data.total_return,
                    benchmark_data.total_return,
                    &portfolio_data.holdings,
                    &benchmark_data.holdings,
                    &params.asset_class_field,
                )?;
                
                // Calculate attribution by asset class
                let mut asset_class_attribution = HashMap::new();
                
                // Group holdings by asset class
                let portfolio_by_asset_class = group_holdings_by_field(
                    &portfolio_data.holdings,
                    &params.asset_class_field,
                );
                
                let benchmark_by_asset_class = group_holdings_by_field(
                    &benchmark_data.holdings,
                    &params.asset_class_field,
                );
                
                // Calculate attribution for each asset class
                for (asset_class, _) in &portfolio_by_asset_class {
                    let portfolio_holdings = portfolio_by_asset_class.get(asset_class)
                        .cloned()
                        .unwrap_or_default();
                    
                    let benchmark_holdings = benchmark_by_asset_class.get(asset_class)
                        .cloned()
                        .unwrap_or_default();
                    
                    // Calculate attribution for this asset class
                    let attribution = calculate_attribution(
                        portfolio_data.total_return,
                        benchmark_data.total_return,
                        &portfolio_holdings,
                        &benchmark_holdings,
                        "security_id", // Use security ID as the field for this level
                    )?;
                    
                    asset_class_attribution.insert(asset_class.clone(), attribution);
                }
                
                // Calculate sector attribution if requested
                let sector_attribution = if params.include_sector.unwrap_or(false) {
                    // Similar to asset class attribution but using sector field
                    // Simplified for brevity
                    Some(HashMap::new())
                } else {
                    None
                };
                
                // Calculate security attribution if requested
                let security_attribution = if params.include_security_selection.unwrap_or(false) {
                    // Calculate attribution at security level
                    // Simplified for brevity
                    Some(HashMap::new())
                } else {
                    None
                };
                
                // Create result
                let result = AttributionResult {
                    query_id: query_id.clone(),
                    portfolio_id: params.portfolio_id.clone(),
                    start_date: params.start_date,
                    end_date: params.end_date,
                    benchmark_id: params.benchmark_id.clone(),
                    overall_attribution,
                    asset_class_attribution,
                    sector_attribution,
                    security_attribution,
                    calculation_time: Utc::now(),
                };
                
                // Serialize to JSON for caching
                let json_result = serde_json::to_value(&result)
                    .map_err(|e| anyhow!("Failed to serialize attribution result: {}", e))?;
                
                Ok(json_result)
            }
        ).await?;
        
        // Deserialize from JSON
        let attribution_result: AttributionResult = serde_json::from_value(result)
            .map_err(|e| anyhow!("Failed to deserialize attribution result: {}", e))?;
        
        // Complete audit trail
        self.audit_manager.complete_calculation(
            &event.event_id,
            vec![format!("attribution_result:{}", query_id)],
        ).await?;
        
        Ok(attribution_result)
    }
    
    /// Perform what-if analysis
    pub async fn perform_what_if_analysis(&self, params: WhatIfQueryParams) -> Result<WhatIfResult> {
        let query_id = Uuid::new_v4().to_string();
        let request_id = format!("query:{}", query_id);
        
        // Start audit trail
        let mut input_params = HashMap::new();
        input_params.insert("portfolio_id".to_string(), serde_json::json!(params.portfolio_id));
        input_params.insert("start_date".to_string(), serde_json::json!(params.start_date.to_string()));
        input_params.insert("end_date".to_string(), serde_json::json!(params.end_date.to_string()));
        input_params.insert("hypothetical_transactions".to_string(), serde_json::json!(params.hypothetical_transactions));
        
        let event = self.audit_manager.start_calculation(
            "interactive_what_if_query",
            &request_id,
            "query_api",
            input_params,
            vec![format!("portfolio:{}", params.portfolio_id)],
        ).await?;
        
        // First, calculate original performance
        let original_params = PerformanceQueryParams {
            portfolio_id: params.portfolio_id.clone(),
            start_date: params.start_date,
            end_date: params.end_date,
            twr_method: Some("daily".to_string()),
            include_risk_metrics: params.include_risk_metrics,
            include_periodic_returns: Some(false),
            benchmark_id: params.benchmark_id.clone(),
            currency: None,
            annualize: Some(false),
            custom_params: None,
        };
        
        let original_performance = self.calculate_performance(original_params).await?;
        
        // Now, apply hypothetical transactions and recalculate
        let hypothetical_portfolio_id = format!("what_if_{}", query_id);
        
        // Clone the original portfolio data
        self.data_service.clone_portfolio_data(
            &params.portfolio_id,
            &hypothetical_portfolio_id,
            params.start_date,
            params.end_date,
        ).await?;
        
        // Apply hypothetical transactions
        for transaction in &params.hypothetical_transactions {
            self.data_service.apply_hypothetical_transaction(
                &hypothetical_portfolio_id,
                transaction,
            ).await?;
        }
        
        // Calculate performance for the hypothetical portfolio
        let hypothetical_params = PerformanceQueryParams {
            portfolio_id: hypothetical_portfolio_id.clone(),
            start_date: params.start_date,
            end_date: params.end_date,
            twr_method: Some("daily".to_string()),
            include_risk_metrics: params.include_risk_metrics,
            include_periodic_returns: Some(false),
            benchmark_id: params.benchmark_id.clone(),
            currency: None,
            annualize: Some(false),
            custom_params: None,
        };
        
        let hypothetical_performance = self.calculate_performance(hypothetical_params).await?;
        
        // Calculate performance difference
        let original_twr = original_performance.time_weighted_return
            .as_ref()
            .map(|twr| twr.return_value)
            .unwrap_or_default();
        
        let hypothetical_twr = hypothetical_performance.time_weighted_return
            .as_ref()
            .map(|twr| twr.return_value)
            .unwrap_or_default();
        
        let performance_difference = hypothetical_twr - original_twr;
        
        // Create result
        let result = WhatIfResult {
            query_id,
            portfolio_id: params.portfolio_id.clone(),
            start_date: params.start_date,
            end_date: params.end_date,
            original_performance,
            hypothetical_performance,
            performance_difference,
            calculation_time: Utc::now(),
        };
        
        // Clean up temporary portfolio
        self.data_service.delete_portfolio_data(&hypothetical_portfolio_id).await?;
        
        // Complete audit trail
        self.audit_manager.complete_calculation(
            &event.event_id,
            vec![format!("what_if_result:{}", query_id)],
        ).await?;
        
        Ok(result)
    }
}

/// Helper function to group holdings by a field
fn group_holdings_by_field(
    holdings: &[HoldingWithReturn],
    field: &str,
) -> HashMap<String, Vec<HoldingWithReturn>> {
    let mut grouped = HashMap::new();
    
    for holding in holdings {
        let field_value = holding.attributes.get(field)
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        
        grouped.entry(field_value)
            .or_insert_with(Vec::new)
            .push(holding.clone());
    }
    
    grouped
}

/// Data access service interface
#[async_trait::async_trait]
pub trait DataAccessService: Send + Sync {
    /// Get portfolio data for a date range
    async fn get_portfolio_data(
        &self,
        portfolio_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<PortfolioData>;
    
    /// Get portfolio returns for a date range with specified frequency
    async fn get_portfolio_returns(
        &self,
        portfolio_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        frequency: &str,
    ) -> Result<HashMap<NaiveDate, f64>>;
    
    /// Get benchmark returns for a date range
    async fn get_benchmark_returns(
        &self,
        benchmark_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<ReturnSeries>;
    
    /// Get benchmark returns for a date range with specified frequency
    async fn get_benchmark_returns_by_frequency(
        &self,
        benchmark_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        frequency: &str,
    ) -> Result<ReturnSeries>;
    
    /// Get portfolio holdings with returns
    async fn get_portfolio_holdings_with_returns(
        &self,
        portfolio_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<PortfolioHoldingsWithReturns>;
    
    /// Get benchmark holdings with returns
    async fn get_benchmark_holdings_with_returns(
        &self,
        benchmark_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<BenchmarkHoldingsWithReturns>;
    
    /// Clone portfolio data for what-if analysis
    async fn clone_portfolio_data(
        &self,
        source_portfolio_id: &str,
        target_portfolio_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<()>;
    
    /// Apply a hypothetical transaction
    async fn apply_hypothetical_transaction(
        &self,
        portfolio_id: &str,
        transaction: &HypotheticalTransaction,
    ) -> Result<()>;
    
    /// Delete portfolio data
    async fn delete_portfolio_data(&self, portfolio_id: &str) -> Result<()>;
}

/// Portfolio data for calculations
#[derive(Debug, Clone)]
pub struct PortfolioData {
    /// Beginning market value
    pub beginning_market_value: f64,
    
    /// Ending market value
    pub ending_market_value: f64,
    
    /// Cash flows during the period
    pub cash_flows: Vec<CashFlow>,
    
    /// Daily market values
    pub daily_market_values: HashMap<NaiveDate, f64>,
    
    /// Daily returns
    pub daily_returns: HashMap<NaiveDate, f64>,
    
    /// Portfolio currency
    pub currency: String,
}

/// Portfolio holdings with returns
#[derive(Debug, Clone)]
pub struct PortfolioHoldingsWithReturns {
    /// Total portfolio return
    pub total_return: f64,
    
    /// Holdings with returns
    pub holdings: Vec<HoldingWithReturn>,
}

/// Benchmark holdings with returns
#[derive(Debug, Clone)]
pub struct BenchmarkHoldingsWithReturns {
    /// Total benchmark return
    pub total_return: f64,
    
    /// Holdings with returns
    pub holdings: Vec<HoldingWithReturn>,
}

/// Holding with return
#[derive(Debug, Clone)]
pub struct HoldingWithReturn {
    /// Security identifier
    pub security_id: String,
    
    /// Weight in the portfolio or benchmark
    pub weight: f64,
    
    /// Return for the period
    pub return_value: f64,
    
    /// Contribution to total return
    pub contribution: f64,
    
    /// Additional attributes (sector, asset class, etc.)
    pub attributes: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculations::audit::InMemoryAuditTrailStorage;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    
    // Mock data access service for testing
    struct MockDataAccessService;
    
    #[async_trait::async_trait]
    impl DataAccessService for MockDataAccessService {
        async fn get_portfolio_data(
            &self,
            _portfolio_id: &str,
            _start_date: NaiveDate,
            _end_date: NaiveDate,
        ) -> Result<PortfolioData> {
            // Return mock data
            let mut daily_market_values = HashMap::new();
            let mut daily_returns = HashMap::new();
            
            let start = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
            
            // Generate 30 days of data
            for i in 0..30 {
                let date = start.checked_add_days(chrono::Days::new(i)).unwrap();
                let value = 1_000_000.0 * (1.0 + 0.001 * i as f64);
                daily_market_values.insert(date, value);
                
                if i > 0 {
                    let prev_date = start.checked_add_days(chrono::Days::new(i - 1)).unwrap();
                    let prev_value = daily_market_values.get(&prev_date).unwrap();
                    let daily_return = (value / prev_value) - 1.0;
                    daily_returns.insert(date, daily_return);
                }
            }
            
            // Create cash flows
            let cash_flows = vec![
                CashFlow {
                    date: start.checked_add_days(chrono::Days::new(10)).unwrap(),
                    amount: Decimal::from_f64(50_000.0).unwrap(),
                    description: "Deposit".to_string(),
                },
                CashFlow {
                    date: start.checked_add_days(chrono::Days::new(20)).unwrap(),
                    amount: Decimal::from_f64(-30_000.0).unwrap(),
                    description: "Withdrawal".to_string(),
                },
            ];
            
            Ok(PortfolioData {
                beginning_market_value: 1_000_000.0,
                ending_market_value: 1_030_000.0,
                cash_flows,
                daily_market_values,
                daily_returns,
                currency: "USD".to_string(),
            })
        }
        
        // Implement other methods with mock data
        // ...
        
        async fn get_portfolio_returns(
            &self,
            _portfolio_id: &str,
            start_date: NaiveDate,
            end_date: NaiveDate,
            _frequency: &str,
        ) -> Result<HashMap<NaiveDate, f64>> {
            let mut returns = HashMap::new();
            
            let mut current_date = start_date;
            let mut value = 1_000_000.0;
            
            while current_date <= end_date {
                value *= 1.0 + 0.001;
                let daily_return = 0.001;
                returns.insert(current_date, daily_return);
                
                current_date = current_date.checked_add_days(chrono::Days::new(1)).unwrap();
            }
            
            Ok(returns)
        }
        
        async fn get_benchmark_returns(
            &self,
            _benchmark_id: &str,
            start_date: NaiveDate,
            end_date: NaiveDate,
        ) -> Result<ReturnSeries> {
            let mut dates = Vec::new();
            let mut returns = Vec::new();
            
            let mut current_date = start_date;
            
            while current_date <= end_date {
                dates.push(current_date);
                returns.push(0.0008); // Slightly lower than portfolio
                
                current_date = current_date.checked_add_days(chrono::Days::new(1)).unwrap();
            }
            
            Ok(ReturnSeries { dates, returns })
        }
        
        async fn get_benchmark_returns_by_frequency(
            &self,
            benchmark_id: &str,
            start_date: NaiveDate,
            end_date: NaiveDate,
            _frequency: &str,
        ) -> Result<ReturnSeries> {
            // For simplicity, just use daily returns
            self.get_benchmark_returns(benchmark_id, start_date, end_date).await
        }
        
        async fn get_portfolio_holdings_with_returns(
            &self,
            _portfolio_id: &str,
            _start_date: NaiveDate,
            _end_date: NaiveDate,
        ) -> Result<PortfolioHoldingsWithReturns> {
            // Mock holdings data
            let holdings = vec![
                HoldingWithReturn {
                    security_id: "AAPL".to_string(),
                    weight: 0.25,
                    return_value: 0.05,
                    contribution: 0.0125,
                    attributes: {
                        let mut attrs = HashMap::new();
                        attrs.insert("asset_class".to_string(), serde_json::json!("Equity"));
                        attrs.insert("sector".to_string(), serde_json::json!("Technology"));
                        attrs
                    },
                },
                HoldingWithReturn {
                    security_id: "MSFT".to_string(),
                    weight: 0.20,
                    return_value: 0.04,
                    contribution: 0.008,
                    attributes: {
                        let mut attrs = HashMap::new();
                        attrs.insert("asset_class".to_string(), serde_json::json!("Equity"));
                        attrs.insert("sector".to_string(), serde_json::json!("Technology"));
                        attrs
                    },
                },
                HoldingWithReturn {
                    security_id: "GOVT".to_string(),
                    weight: 0.30,
                    return_value: 0.01,
                    contribution: 0.003,
                    attributes: {
                        let mut attrs = HashMap::new();
                        attrs.insert("asset_class".to_string(), serde_json::json!("Fixed Income"));
                        attrs.insert("sector".to_string(), serde_json::json!("Government"));
                        attrs
                    },
                },
                HoldingWithReturn {
                    security_id: "CORP".to_string(),
                    weight: 0.25,
                    return_value: 0.02,
                    contribution: 0.005,
                    attributes: {
                        let mut attrs = HashMap::new();
                        attrs.insert("asset_class".to_string(), serde_json::json!("Fixed Income"));
                        attrs.insert("sector".to_string(), serde_json::json!("Corporate"));
                        attrs
                    },
                },
            ];
            
            Ok(PortfolioHoldingsWithReturns {
                total_return: 0.0285, // Sum of contributions
                holdings,
            })
        }
        
        async fn get_benchmark_holdings_with_returns(
            &self,
            _benchmark_id: &str,
            _start_date: NaiveDate,
            _end_date: NaiveDate,
        ) -> Result<BenchmarkHoldingsWithReturns> {
            // Mock benchmark holdings data
            let holdings = vec![
                HoldingWithReturn {
                    security_id: "AAPL".to_string(),
                    weight: 0.20,
                    return_value: 0.05,
                    contribution: 0.01,
                    attributes: {
                        let mut attrs = HashMap::new();
                        attrs.insert("asset_class".to_string(), serde_json::json!("Equity"));
                        attrs.insert("sector".to_string(), serde_json::json!("Technology"));
                        attrs
                    },
                },
                HoldingWithReturn {
                    security_id: "MSFT".to_string(),
                    weight: 0.15,
                    return_value: 0.04,
                    contribution: 0.006,
                    attributes: {
                        let mut attrs = HashMap::new();
                        attrs.insert("asset_class".to_string(), serde_json::json!("Equity"));
                        attrs.insert("sector".to_string(), serde_json::json!("Technology"));
                        attrs
                    },
                },
                HoldingWithReturn {
                    security_id: "GOVT".to_string(),
                    weight: 0.40,
                    return_value: 0.01,
                    contribution: 0.004,
                    attributes: {
                        let mut attrs = HashMap::new();
                        attrs.insert("asset_class".to_string(), serde_json::json!("Fixed Income"));
                        attrs.insert("sector".to_string(), serde_json::json!("Government"));
                        attrs
                    },
                },
                HoldingWithReturn {
                    security_id: "CORP".to_string(),
                    weight: 0.25,
                    return_value: 0.02,
                    contribution: 0.005,
                    attributes: {
                        let mut attrs = HashMap::new();
                        attrs.insert("asset_class".to_string(), serde_json::json!("Fixed Income"));
                        attrs.insert("sector".to_string(), serde_json::json!("Corporate"));
                        attrs
                    },
                },
            ];
            
            Ok(BenchmarkHoldingsWithReturns {
                total_return: 0.025, // Sum of contributions
                holdings,
            })
        }
        
        async fn clone_portfolio_data(
            &self,
            _source_portfolio_id: &str,
            _target_portfolio_id: &str,
            _start_date: NaiveDate,
            _end_date: NaiveDate,
        ) -> Result<()> {
            // Mock implementation - just return success
            Ok(())
        }
        
        async fn apply_hypothetical_transaction(
            &self,
            _portfolio_id: &str,
            _transaction: &HypotheticalTransaction,
        ) -> Result<()> {
            // Mock implementation - just return success
            Ok(())
        }
        
        async fn delete_portfolio_data(&self, _portfolio_id: &str) -> Result<()> {
            // Mock implementation - just return success
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_calculate_performance() {
        // Create dependencies
        let storage = Arc::new(InMemoryAuditTrailStorage::new());
        let audit_manager = Arc::new(AuditTrailManager::new(storage.clone()));
        let cache = crate::calculations::distributed_cache::CacheFactory::create_mock_cache();
        
        // Create mock currency converter
        let mock_exchange_rate_provider = Arc::new(crate::calculations::currency::MockExchangeRateProviderMock::new());
        let currency_converter = Arc::new(crate::calculations::currency::CurrencyConverter::new(
            mock_exchange_rate_provider,
            "USD".to_string(),
        ));
        
        let data_service = Arc::new(MockDataAccessService);
        
        // Create query API
        let query_api = QueryApi::new(
            audit_manager,
            cache,
            currency_converter,
            data_service,
        );
        
        // Create query parameters
        let params = PerformanceQueryParams {
            portfolio_id: "test_portfolio".to_string(),
            start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
            twr_method: Some("daily".to_string()),
            include_risk_metrics: Some(true),
            include_periodic_returns: Some(true),
            benchmark_id: Some("test_benchmark".to_string()),
            currency: None,
            annualize: Some(false),
            custom_params: None,
        };
        
        // Execute query
        let result = query_api.calculate_performance(params).await.unwrap();
        
        // Verify result
        assert_eq!(result.portfolio_id, "test_portfolio");
        assert!(result.time_weighted_return.is_some());
        assert!(result.risk_metrics.is_some());
        assert!(result.benchmark_comparison.is_some());
    }
} 