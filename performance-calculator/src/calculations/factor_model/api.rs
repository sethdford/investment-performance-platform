//! API endpoints for factor model analysis and visualization
//!
//! This module provides REST API endpoints for factor model analysis,
//! including factor exposure analysis, risk decomposition, performance attribution,
//! portfolio optimization, and factor forecasting.

use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use super::calculator::FactorModelCalculator;
use super::covariance::CovarianceEstimator;
use super::error::FactorModelError;
use super::repository::FactorRepository;
use super::types::{Factor, FactorCategory, FactorCovariance, FactorExposure, FactorReturn};
use super::Result;

/// Request for factor exposure analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorExposureRequest {
    /// Portfolio ID to analyze
    pub portfolio_id: String,
    
    /// Optional benchmark ID for comparison
    pub benchmark_id: Option<String>,
    
    /// Date for analysis
    pub as_of_date: NaiveDate,
    
    /// Optional filter for specific factor categories
    pub factor_categories: Option<Vec<FactorCategory>>,
}

/// Response for factor exposure analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorExposureResponse {
    /// Portfolio factor exposures
    pub portfolio_exposures: HashMap<String, f64>,
    
    /// Benchmark factor exposures (if requested)
    pub benchmark_exposures: Option<HashMap<String, f64>>,
    
    /// Active exposures (portfolio - benchmark)
    pub active_exposures: Option<HashMap<String, f64>>,
    
    /// Factor metadata
    pub factors: HashMap<String, FactorMetadata>,
    
    /// Analysis date
    pub as_of_date: NaiveDate,
}

/// Factor metadata for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorMetadata {
    /// Factor ID
    pub id: String,
    
    /// Factor name
    pub name: String,
    
    /// Factor category
    pub category: FactorCategory,
    
    /// Factor description
    pub description: String,
}

/// Request for risk decomposition analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDecompositionRequest {
    /// Portfolio ID to analyze
    pub portfolio_id: String,
    
    /// Optional benchmark ID for comparison
    pub benchmark_id: Option<String>,
    
    /// Date for analysis
    pub as_of_date: NaiveDate,
    
    /// Whether to include security-level risk contributions
    pub include_security_level: bool,
}

/// Response for risk decomposition analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDecompositionResponse {
    /// Total portfolio risk
    pub total_risk: f64,
    
    /// Factor risk (systematic risk)
    pub factor_risk: f64,
    
    /// Specific risk (idiosyncratic risk)
    pub specific_risk: f64,
    
    /// Risk contribution by factor
    pub factor_risk_contribution: HashMap<String, RiskContribution>,
    
    /// Security-level risk contributions (if requested)
    pub security_risk_contribution: Option<HashMap<String, RiskContribution>>,
    
    /// Analysis date
    pub as_of_date: NaiveDate,
}

/// Risk contribution for a factor or security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskContribution {
    /// Absolute risk contribution
    pub absolute: f64,
    
    /// Percentage of total risk
    pub percentage: f64,
    
    /// Marginal contribution to risk
    pub marginal: f64,
}

/// Request for performance attribution analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAttributionRequest {
    /// Portfolio ID to analyze
    pub portfolio_id: String,
    
    /// Optional benchmark ID for comparison
    pub benchmark_id: Option<String>,
    
    /// Start date for analysis period
    pub start_date: NaiveDate,
    
    /// End date for analysis period
    pub end_date: NaiveDate,
}

/// Response for performance attribution analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAttributionResponse {
    /// Total portfolio return
    pub total_return: f64,
    
    /// Benchmark return (if requested)
    pub benchmark_return: Option<f64>,
    
    /// Active return (portfolio - benchmark)
    pub active_return: Option<f64>,
    
    /// Return attribution by factor
    pub factor_attribution: HashMap<String, AttributionEffect>,
    
    /// Specific return (unexplained by factors)
    pub specific_return: f64,
    
    /// Analysis period
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

/// Attribution effect for a factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributionEffect {
    /// Factor exposure
    pub exposure: f64,
    
    /// Factor return
    pub factor_return: f64,
    
    /// Contribution to return
    pub contribution: f64,
    
    /// Percentage of total return
    pub percentage: f64,
}

/// Request for portfolio optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioOptimizationRequest {
    /// Portfolio ID to optimize
    pub portfolio_id: String,
    
    /// Optional benchmark ID for reference
    pub benchmark_id: Option<String>,
    
    /// Date for optimization
    pub as_of_date: NaiveDate,
    
    /// Optimization objective
    pub objective: OptimizationObjective,
    
    /// Optimization constraints
    pub constraints: Vec<OptimizationConstraint>,
}

/// Optimization objective
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationObjective {
    /// Minimize portfolio risk
    MinimizeRisk,
    
    /// Maximize expected return
    MaximizeReturn,
    
    /// Maximize information ratio
    MaximizeInformationRatio,
    
    /// Minimize tracking error
    MinimizeTrackingError,
}

/// Optimization constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationConstraint {
    /// Constraint on factor exposure
    FactorExposure {
        /// Factor ID
        factor_id: String,
        
        /// Minimum exposure
        min: Option<f64>,
        
        /// Maximum exposure
        max: Option<f64>,
        
        /// Target exposure
        target: Option<f64>,
    },
    
    /// Constraint on total risk
    TotalRisk {
        /// Maximum risk
        max: f64,
    },
    
    /// Constraint on tracking error
    TrackingError {
        /// Maximum tracking error
        max: f64,
    },
    
    /// Constraint on position size
    PositionSize {
        /// Security ID
        security_id: String,
        
        /// Minimum weight
        min: Option<f64>,
        
        /// Maximum weight
        max: Option<f64>,
    },
    
    /// Constraint on turnover
    Turnover {
        /// Maximum turnover
        max: f64,
    },
}

/// Response for portfolio optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioOptimizationResponse {
    /// Optimized portfolio weights
    pub optimized_weights: HashMap<String, f64>,
    
    /// Current portfolio weights
    pub current_weights: HashMap<String, f64>,
    
    /// Suggested trades to reach optimized portfolio
    pub suggested_trades: HashMap<String, f64>,
    
    /// Expected risk of optimized portfolio
    pub expected_risk: f64,
    
    /// Expected return of optimized portfolio
    pub expected_return: f64,
    
    /// Factor exposures of optimized portfolio
    pub factor_exposures: HashMap<String, f64>,
    
    /// Optimization date
    pub as_of_date: NaiveDate,
}

/// Request for factor forecasting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorForecastRequest {
    /// Factor IDs to forecast
    pub factor_ids: Vec<String>,
    
    /// Start date for forecast
    pub start_date: NaiveDate,
    
    /// End date for forecast
    pub end_date: NaiveDate,
    
    /// Forecast horizon in days
    pub horizon: u32,
    
    /// Confidence level for prediction intervals (0-1)
    pub confidence_level: f64,
}

/// Response for factor forecasting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorForecastResponse {
    /// Forecasted factor returns
    pub forecasts: HashMap<String, Vec<FactorForecast>>,
    
    /// Forecast start date
    pub start_date: NaiveDate,
    
    /// Forecast end date
    pub end_date: NaiveDate,
    
    /// Forecast horizon in days
    pub horizon: u32,
    
    /// Confidence level for prediction intervals
    pub confidence_level: f64,
}

/// Forecast for a factor on a specific date
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorForecast {
    /// Forecast date
    pub date: NaiveDate,
    
    /// Forecasted return
    pub forecasted_return: f64,
    
    /// Lower bound of prediction interval
    pub lower_bound: f64,
    
    /// Upper bound of prediction interval
    pub upper_bound: f64,
}

/// Factor model API service
#[async_trait]
pub trait FactorModelApiService: Send + Sync {
    /// Analyze factor exposures
    async fn analyze_factor_exposures(
        &self,
        request: FactorExposureRequest,
    ) -> Result<FactorExposureResponse>;
    
    /// Analyze risk decomposition
    async fn analyze_risk_decomposition(
        &self,
        request: RiskDecompositionRequest,
    ) -> Result<RiskDecompositionResponse>;
    
    /// Analyze performance attribution
    async fn analyze_performance_attribution(
        &self,
        request: PerformanceAttributionRequest,
    ) -> Result<PerformanceAttributionResponse>;
    
    /// Optimize portfolio
    async fn optimize_portfolio(
        &self,
        request: PortfolioOptimizationRequest,
    ) -> Result<PortfolioOptimizationResponse>;
    
    /// Forecast factor returns
    async fn forecast_factor_returns(
        &self,
        request: FactorForecastRequest,
    ) -> Result<FactorForecastResponse>;
}

/// Default implementation of factor model API service
pub struct DefaultFactorModelApiService {
    /// Factor model calculator
    calculator: Arc<dyn FactorModelCalculator>,
    
    /// Factor repository
    repository: Arc<dyn FactorRepository>,
}

impl DefaultFactorModelApiService {
    /// Create a new factor model API service
    pub fn new(
        calculator: Arc<dyn FactorModelCalculator>,
        repository: Arc<dyn FactorRepository>,
    ) -> Self {
        Self {
            calculator,
            repository,
        }
    }
    
    /// Convert a factor to factor metadata
    fn factor_to_metadata(&self, factor: &Factor) -> FactorMetadata {
        FactorMetadata {
            id: factor.id.clone(),
            name: factor.name.clone(),
            category: factor.category.clone(),
            description: factor.description.clone(),
        }
    }
}

#[async_trait]
impl FactorModelApiService for DefaultFactorModelApiService {
    async fn analyze_factor_exposures(
        &self,
        request: FactorExposureRequest,
    ) -> Result<FactorExposureResponse> {
        // Get portfolio factor exposures
        let portfolio_exposures = self.calculator.calculate_portfolio_factor_exposures(
            &request.portfolio_id,
            request.as_of_date,
        ).await?;
        
        // Get benchmark factor exposures if requested
        let (benchmark_exposures, active_exposures) = if let Some(benchmark_id) = &request.benchmark_id {
            let benchmark_exposures = self.calculator.calculate_portfolio_factor_exposures(
                benchmark_id,
                request.as_of_date,
            ).await?;
            
            // Calculate active exposures
            let mut active_exposures = HashMap::new();
            for (factor_id, portfolio_exposure) in &portfolio_exposures {
                let benchmark_exposure = benchmark_exposures.get(factor_id).unwrap_or(&0.0);
                active_exposures.insert(factor_id.clone(), portfolio_exposure - benchmark_exposure);
            }
            
            (Some(benchmark_exposures), Some(active_exposures))
        } else {
            (None, None)
        };
        
        // Get factor metadata
        let mut factors = HashMap::new();
        for factor_id in portfolio_exposures.keys() {
            let factor = self.repository.get_factor(factor_id).await?;
            
            // Filter by category if requested
            if let Some(categories) = &request.factor_categories {
                if !categories.contains(&factor.category) {
                    continue;
                }
            }
            
            factors.insert(factor_id.clone(), self.factor_to_metadata(&factor));
        }
        
        Ok(FactorExposureResponse {
            portfolio_exposures,
            benchmark_exposures,
            active_exposures,
            factors,
            as_of_date: request.as_of_date,
        })
    }
    
    async fn analyze_risk_decomposition(
        &self,
        request: RiskDecompositionRequest,
    ) -> Result<RiskDecompositionResponse> {
        // Calculate total risk
        let total_risk = self.calculator.calculate_total_risk(
            &request.portfolio_id,
            request.as_of_date,
        ).await?;
        
        // Calculate factor risk
        let factor_risk = self.calculator.calculate_factor_risk(
            &request.portfolio_id,
            request.as_of_date,
        ).await?;
        
        // Calculate specific risk
        let specific_risk = self.calculator.calculate_specific_risk(
            &request.portfolio_id,
            request.as_of_date,
        ).await?;
        
        // Calculate marginal contribution to risk
        let mctr = self.calculator.calculate_marginal_contribution_to_risk(
            &request.portfolio_id,
            request.as_of_date,
        ).await?;
        
        // Calculate factor exposures
        let exposures = self.calculator.calculate_portfolio_factor_exposures(
            &request.portfolio_id,
            request.as_of_date,
        ).await?;
        
        // Calculate factor risk contributions
        let mut factor_risk_contribution = HashMap::new();
        for (factor_id, mctr_value) in &mctr {
            let exposure = exposures.get(factor_id).unwrap_or(&0.0);
            let contribution = mctr_value * exposure;
            
            factor_risk_contribution.insert(factor_id.clone(), RiskContribution {
                absolute: contribution,
                percentage: contribution / total_risk * 100.0,
                marginal: *mctr_value,
            });
        }
        
        // We would implement security-level risk contributions here
        // This is a placeholder for now
        let security_risk_contribution = if request.include_security_level {
            Some(HashMap::new())
        } else {
            None
        };
        
        Ok(RiskDecompositionResponse {
            total_risk,
            factor_risk,
            specific_risk,
            factor_risk_contribution,
            security_risk_contribution,
            as_of_date: request.as_of_date,
        })
    }
    
    async fn analyze_performance_attribution(
        &self,
        request: PerformanceAttributionRequest,
    ) -> Result<PerformanceAttributionResponse> {
        // Calculate factor contributions
        let factor_contributions = self.calculator.calculate_factor_contributions(
            &request.portfolio_id,
            request.start_date,
            request.end_date,
        ).await?;
        
        // Get portfolio factor exposures
        let exposures = self.calculator.calculate_portfolio_factor_exposures(
            &request.portfolio_id,
            request.start_date,
        ).await?;
        
        // Get factor returns
        let factor_ids: Vec<String> = exposures.keys().cloned().collect();
        let factor_returns = self.calculator.calculate_factor_returns(
            &factor_ids,
            request.start_date,
            request.end_date,
        ).await?;
        
        // Calculate total return (this would come from a portfolio service in a real implementation)
        // This is a placeholder that sums factor contributions
        let total_return: f64 = factor_contributions.values().sum();
        
        // Calculate benchmark return if requested
        let (benchmark_return, active_return) = if let Some(benchmark_id) = &request.benchmark_id {
            // This would come from a portfolio service in a real implementation
            // This is a placeholder
            let benchmark_return = 0.0;
            let active_return = total_return - benchmark_return;
            
            (Some(benchmark_return), Some(active_return))
        } else {
            (None, None)
        };
        
        // Calculate attribution effects
        let mut factor_attribution = HashMap::new();
        for (factor_id, contribution) in &factor_contributions {
            let exposure = exposures.get(factor_id).unwrap_or(&0.0);
            let factor_return = factor_returns.get(factor_id).unwrap_or(&0.0);
            
            factor_attribution.insert(factor_id.clone(), AttributionEffect {
                exposure: *exposure,
                factor_return: *factor_return,
                contribution: *contribution,
                percentage: if total_return != 0.0 { contribution / total_return * 100.0 } else { 0.0 },
            });
        }
        
        // Calculate specific return (unexplained by factors)
        // This is a placeholder
        let specific_return = 0.0;
        
        Ok(PerformanceAttributionResponse {
            total_return,
            benchmark_return,
            active_return,
            factor_attribution,
            specific_return,
            start_date: request.start_date,
            end_date: request.end_date,
        })
    }
    
    async fn optimize_portfolio(
        &self,
        request: PortfolioOptimizationRequest,
    ) -> Result<PortfolioOptimizationResponse> {
        // This would be implemented with a quadratic programming solver
        // For now, we'll return a placeholder response
        
        // Get current portfolio weights
        // This would come from a portfolio service in a real implementation
        let current_weights = HashMap::new();
        
        // Create optimized weights (placeholder)
        let optimized_weights = HashMap::new();
        
        // Calculate suggested trades
        let suggested_trades = HashMap::new();
        
        // Calculate expected risk and return (placeholders)
        let expected_risk = 0.0;
        let expected_return = 0.0;
        
        // Calculate factor exposures of optimized portfolio
        let factor_exposures = HashMap::new();
        
        Ok(PortfolioOptimizationResponse {
            optimized_weights,
            current_weights,
            suggested_trades,
            expected_risk,
            expected_return,
            factor_exposures,
            as_of_date: request.as_of_date,
        })
    }
    
    async fn forecast_factor_returns(
        &self,
        request: FactorForecastRequest,
    ) -> Result<FactorForecastResponse> {
        // This would be implemented with a time series forecasting model
        // For now, we'll return a placeholder response
        
        // Create forecasts
        let mut forecasts = HashMap::new();
        
        for factor_id in &request.factor_ids {
            let mut factor_forecasts = Vec::new();
            
            // Generate forecast for each date in the horizon
            let days = (request.end_date - request.start_date).num_days() as usize;
            for i in 0..days {
                let date = request.start_date + chrono::Duration::days(i as i64);
                
                factor_forecasts.push(FactorForecast {
                    date,
                    forecasted_return: 0.0, // Placeholder
                    lower_bound: 0.0,       // Placeholder
                    upper_bound: 0.0,       // Placeholder
                });
            }
            
            forecasts.insert(factor_id.clone(), factor_forecasts);
        }
        
        Ok(FactorForecastResponse {
            forecasts,
            start_date: request.start_date,
            end_date: request.end_date,
            horizon: request.horizon,
            confidence_level: request.confidence_level,
        })
    }
} 