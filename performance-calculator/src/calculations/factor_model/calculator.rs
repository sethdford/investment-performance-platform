//! Calculator for factor model operations

use async_trait::async_trait;
use chrono::NaiveDate;
use ndarray::{Array1, Array2};
use std::collections::HashMap;
use std::sync::Arc;

use super::error::FactorModelError;
use super::repository::FactorRepository;
use super::types::{Factor, FactorCategory, FactorCovariance, FactorExposure, FactorReturn};
use super::Result;

/// Calculator for factor model operations
#[async_trait]
pub trait FactorModelCalculator: Send + Sync {
    /// Calculate factor exposures for a portfolio
    async fn calculate_portfolio_factor_exposures(
        &self,
        portfolio_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<HashMap<String, f64>>;
    
    /// Calculate factor returns for a period
    async fn calculate_factor_returns(
        &self,
        factor_ids: &[String],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<String, f64>>;
    
    /// Calculate factor contributions to portfolio return
    async fn calculate_factor_contributions(
        &self,
        portfolio_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<String, f64>>;
    
    /// Calculate factor risk (systematic risk) for a portfolio
    async fn calculate_factor_risk(
        &self,
        portfolio_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<f64>;
    
    /// Calculate specific risk (idiosyncratic risk) for a portfolio
    async fn calculate_specific_risk(
        &self,
        portfolio_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<f64>;
    
    /// Calculate total risk for a portfolio
    async fn calculate_total_risk(
        &self,
        portfolio_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<f64>;
    
    /// Calculate marginal contribution to risk for each factor
    async fn calculate_marginal_contribution_to_risk(
        &self,
        portfolio_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<HashMap<String, f64>>;
    
    /// Calculate tracking error against a benchmark
    async fn calculate_tracking_error(
        &self,
        portfolio_id: &str,
        benchmark_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<f64>;
    
    /// Calculate active factor exposures against a benchmark
    async fn calculate_active_factor_exposures(
        &self,
        portfolio_id: &str,
        benchmark_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<HashMap<String, f64>>;
}

/// Default implementation of the factor model calculator
pub struct DefaultFactorModelCalculator {
    repository: Arc<dyn FactorRepository>,
}

impl DefaultFactorModelCalculator {
    /// Create a new default factor model calculator
    pub fn new(repository: Arc<dyn FactorRepository>) -> Self {
        Self { repository }
    }
    
    /// Get portfolio holdings
    async fn get_portfolio_holdings(
        &self,
        portfolio_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<HashMap<String, f64>> {
        // This is a placeholder. In a real implementation, this would retrieve
        // the portfolio holdings from a portfolio repository.
        // For now, we'll return a mock portfolio with some securities.
        
        // Mock securities with weights
        let holdings = vec![
            ("AAPL".to_string(), 0.05),
            ("MSFT".to_string(), 0.05),
            ("AMZN".to_string(), 0.05),
            ("GOOGL".to_string(), 0.05),
            ("FB".to_string(), 0.05),
            ("TSLA".to_string(), 0.05),
            ("BRK.A".to_string(), 0.05),
            ("JPM".to_string(), 0.05),
            ("JNJ".to_string(), 0.05),
            ("V".to_string(), 0.05),
            ("PG".to_string(), 0.05),
            ("UNH".to_string(), 0.05),
            ("HD".to_string(), 0.05),
            ("BAC".to_string(), 0.05),
            ("MA".to_string(), 0.05),
            ("DIS".to_string(), 0.05),
            ("NVDA".to_string(), 0.05),
            ("PYPL".to_string(), 0.05),
            ("ADBE".to_string(), 0.05),
            ("CMCSA".to_string(), 0.05),
        ];
        
        Ok(holdings.into_iter().collect())
    }
    
    /// Get security returns
    async fn get_security_returns(
        &self,
        security_ids: &[String],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<String, f64>> {
        // This is a placeholder. In a real implementation, this would retrieve
        // the security returns from a data repository.
        // For now, we'll return mock returns for the securities.
        
        // Mock returns
        let returns = vec![
            ("AAPL".to_string(), 0.15),
            ("MSFT".to_string(), 0.12),
            ("AMZN".to_string(), 0.10),
            ("GOOGL".to_string(), 0.08),
            ("FB".to_string(), 0.05),
            ("TSLA".to_string(), 0.25),
            ("BRK.A".to_string(), 0.07),
            ("JPM".to_string(), 0.09),
            ("JNJ".to_string(), 0.06),
            ("V".to_string(), 0.11),
            ("PG".to_string(), 0.04),
            ("UNH".to_string(), 0.08),
            ("HD".to_string(), 0.09),
            ("BAC".to_string(), 0.07),
            ("MA".to_string(), 0.10),
            ("DIS".to_string(), 0.05),
            ("NVDA".to_string(), 0.20),
            ("PYPL".to_string(), 0.15),
            ("ADBE".to_string(), 0.12),
            ("CMCSA".to_string(), 0.03),
        ];
        
        let return_map: HashMap<String, f64> = returns.into_iter().collect();
        
        // Filter to only include the requested securities
        Ok(security_ids
            .iter()
            .filter_map(|id| {
                return_map.get(id).map(|&r| (id.clone(), r))
            })
            .collect())
    }
    
    /// Get portfolio return
    async fn get_portfolio_return(
        &self,
        portfolio_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<f64> {
        // This is a placeholder. In a real implementation, this would retrieve
        // the portfolio return from a performance repository.
        // For now, we'll calculate a mock return based on the holdings and security returns.
        
        let holdings = self.get_portfolio_holdings(portfolio_id, end_date).await?;
        let security_ids: Vec<String> = holdings.keys().cloned().collect();
        let security_returns = self.get_security_returns(&security_ids, start_date, end_date).await?;
        
        let portfolio_return = holdings.iter()
            .map(|(id, weight)| {
                security_returns.get(id).unwrap_or(&0.0) * weight
            })
            .sum();
        
        Ok(portfolio_return)
    }
}

#[async_trait]
impl FactorModelCalculator for DefaultFactorModelCalculator {
    async fn calculate_portfolio_factor_exposures(
        &self,
        portfolio_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<HashMap<String, f64>> {
        // Get portfolio holdings
        let holdings = self.get_portfolio_holdings(portfolio_id, as_of_date).await?;
        let security_ids: Vec<String> = holdings.keys().cloned().collect();
        
        // Get factor exposures for all securities in the portfolio
        let security_exposures = self.repository
            .get_factor_exposures_for_securities(&security_ids, as_of_date)
            .await?;
        
        // Calculate portfolio factor exposures as weighted average of security exposures
        let mut portfolio_exposures = HashMap::new();
        
        for (security_id, exposures) in security_exposures {
            let weight = holdings.get(&security_id).cloned().unwrap_or(0.0);
            
            for exposure in exposures {
                let factor_exposure = portfolio_exposures
                    .entry(exposure.factor_id.clone())
                    .or_insert(0.0);
                
                *factor_exposure += exposure.exposure * weight;
            }
        }
        
        Ok(portfolio_exposures)
    }
    
    async fn calculate_factor_returns(
        &self,
        factor_ids: &[String],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<String, f64>> {
        // Get factor returns from the repository
        let factor_returns = self.repository
            .get_factor_returns(factor_ids, start_date, end_date)
            .await?;
        
        // Convert to HashMap
        let returns_map: HashMap<String, f64> = factor_returns
            .into_iter()
            .map(|r| (r.factor_id, r.return_value))
            .collect();
        
        Ok(returns_map)
    }
    
    async fn calculate_factor_contributions(
        &self,
        portfolio_id: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<String, f64>> {
        // Get portfolio factor exposures at the start of the period
        let factor_exposures = self.calculate_portfolio_factor_exposures(portfolio_id, start_date).await?;
        
        // Get factor returns for the period
        let factor_ids: Vec<String> = factor_exposures.keys().cloned().collect();
        let factor_returns = self.calculate_factor_returns(&factor_ids, start_date, end_date).await?;
        
        // Calculate factor contributions as exposure * return
        let mut factor_contributions = HashMap::new();
        
        for (factor_id, exposure) in factor_exposures {
            if let Some(&factor_return) = factor_returns.get(&factor_id) {
                factor_contributions.insert(factor_id, exposure * factor_return);
            }
        }
        
        Ok(factor_contributions)
    }
    
    async fn calculate_factor_risk(
        &self,
        portfolio_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<f64> {
        // Get portfolio factor exposures
        let factor_exposures = self.calculate_portfolio_factor_exposures(portfolio_id, as_of_date).await?;
        
        // Get factor covariance matrix
        let factor_ids: Vec<String> = factor_exposures.keys().cloned().collect();
        let factor_covariance = self.repository
            .get_factor_covariance(&factor_ids, as_of_date)
            .await?;
        
        // Convert exposures to ndarray
        let mut exposures_array = Array1::zeros(factor_ids.len());
        for (i, factor_id) in factor_ids.iter().enumerate() {
            exposures_array[i] = *factor_exposures.get(factor_id).unwrap_or(&0.0);
        }
        
        // Convert covariance matrix to ndarray
        let covariance_matrix = factor_covariance.to_ndarray()
            .map_err(|e| FactorModelError::CalculationError(e.to_string()))?;
        
        // Calculate factor risk as sqrt(x^T * V * x)
        let factor_variance = exposures_array.dot(&covariance_matrix.dot(&exposures_array));
        let factor_risk = factor_variance.sqrt();
        
        Ok(factor_risk)
    }
    
    async fn calculate_specific_risk(
        &self,
        portfolio_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<f64> {
        // This is a placeholder. In a real implementation, this would calculate
        // the specific risk based on security-specific variances.
        // For now, we'll return a mock value.
        
        Ok(0.02) // 2% specific risk
    }
    
    async fn calculate_total_risk(
        &self,
        portfolio_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<f64> {
        // Calculate factor risk
        let factor_risk = self.calculate_factor_risk(portfolio_id, as_of_date).await?;
        
        // Calculate specific risk
        let specific_risk = self.calculate_specific_risk(portfolio_id, as_of_date).await?;
        
        // Calculate total risk as sqrt(factor_risk^2 + specific_risk^2)
        let total_risk = (factor_risk.powi(2) + specific_risk.powi(2)).sqrt();
        
        Ok(total_risk)
    }
    
    async fn calculate_marginal_contribution_to_risk(
        &self,
        portfolio_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<HashMap<String, f64>> {
        // Get portfolio factor exposures
        let factor_exposures = self.calculate_portfolio_factor_exposures(portfolio_id, as_of_date).await?;
        
        // Get factor covariance matrix
        let factor_ids: Vec<String> = factor_exposures.keys().cloned().collect();
        let factor_covariance = self.repository
            .get_factor_covariance(&factor_ids, as_of_date)
            .await?;
        
        // Convert exposures to ndarray
        let mut exposures_array = Array1::zeros(factor_ids.len());
        for (i, factor_id) in factor_ids.iter().enumerate() {
            exposures_array[i] = *factor_exposures.get(factor_id).unwrap_or(&0.0);
        }
        
        // Convert covariance matrix to ndarray
        let covariance_matrix = factor_covariance.to_ndarray()
            .map_err(|e| FactorModelError::CalculationError(e.to_string()))?;
        
        // Calculate total risk
        let total_risk = self.calculate_total_risk(portfolio_id, as_of_date).await?;
        
        // Calculate marginal contribution to risk for each factor
        let mut marginal_contributions = HashMap::new();
        
        for (i, factor_id) in factor_ids.iter().enumerate() {
            // Calculate marginal contribution as (V * x)_i / sqrt(x^T * V * x)
            let factor_contribution = covariance_matrix.row(i).dot(&exposures_array) / total_risk;
            marginal_contributions.insert(factor_id.clone(), factor_contribution);
        }
        
        Ok(marginal_contributions)
    }
    
    async fn calculate_tracking_error(
        &self,
        portfolio_id: &str,
        benchmark_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<f64> {
        // Get active factor exposures
        let active_exposures = self.calculate_active_factor_exposures(
            portfolio_id,
            benchmark_id,
            as_of_date,
        ).await?;
        
        // Get factor covariance matrix
        let factor_ids: Vec<String> = active_exposures.keys().cloned().collect();
        let factor_covariance = self.repository
            .get_factor_covariance(&factor_ids, as_of_date)
            .await?;
        
        // Convert active exposures to ndarray
        let mut active_exposures_array = Array1::zeros(factor_ids.len());
        for (i, factor_id) in factor_ids.iter().enumerate() {
            active_exposures_array[i] = *active_exposures.get(factor_id).unwrap_or(&0.0);
        }
        
        // Convert covariance matrix to ndarray
        let covariance_matrix = factor_covariance.to_ndarray()
            .map_err(|e| FactorModelError::CalculationError(e.to_string()))?;
        
        // Calculate tracking error as sqrt(x_a^T * V * x_a)
        let tracking_variance = active_exposures_array.dot(&covariance_matrix.dot(&active_exposures_array));
        let tracking_error = tracking_variance.sqrt();
        
        Ok(tracking_error)
    }
    
    async fn calculate_active_factor_exposures(
        &self,
        portfolio_id: &str,
        benchmark_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<HashMap<String, f64>> {
        // Get portfolio factor exposures
        let portfolio_exposures = self.calculate_portfolio_factor_exposures(portfolio_id, as_of_date).await?;
        
        // Get benchmark factor exposures
        let benchmark_exposures = self.calculate_portfolio_factor_exposures(benchmark_id, as_of_date).await?;
        
        // Calculate active exposures as portfolio - benchmark
        let mut active_exposures = HashMap::new();
        
        // Add all portfolio exposures
        for (factor_id, exposure) in &portfolio_exposures {
            active_exposures.insert(factor_id.clone(), *exposure);
        }
        
        // Subtract benchmark exposures
        for (factor_id, exposure) in &benchmark_exposures {
            let active_exposure = active_exposures.entry(factor_id.clone()).or_insert(0.0);
            *active_exposure -= exposure;
        }
        
        Ok(active_exposures)
    }
}

/// Factory for creating factor model calculators
pub struct FactorModelCalculatorFactory;

impl FactorModelCalculatorFactory {
    /// Create a new default factor model calculator
    pub fn create_default_calculator(repository: Arc<dyn FactorRepository>) -> Arc<dyn FactorModelCalculator> {
        Arc::new(DefaultFactorModelCalculator::new(repository))
    }
} 