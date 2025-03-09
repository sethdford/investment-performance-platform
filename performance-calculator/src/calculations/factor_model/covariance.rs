//! Covariance estimation and regularization for factor models

use chrono::NaiveDate;
use ndarray::{Array1, Array2};
use std::collections::HashMap;

use super::error::FactorModelError;
use super::types::{FactorCovariance, FactorReturn};
use super::Result;

/// Covariance estimator for factor returns
pub trait CovarianceEstimator {
    /// Estimate covariance matrix from factor returns
    fn estimate_covariance(
        &self,
        factor_returns: &HashMap<String, Vec<FactorReturn>>,
        as_of_date: NaiveDate,
    ) -> Result<FactorCovariance>;
}

/// Sample covariance estimator
pub struct SampleCovarianceEstimator;

impl SampleCovarianceEstimator {
    /// Create a new sample covariance estimator
    pub fn new() -> Self {
        Self
    }
}

impl CovarianceEstimator for SampleCovarianceEstimator {
    fn estimate_covariance(
        &self,
        factor_returns: &HashMap<String, Vec<FactorReturn>>,
        as_of_date: NaiveDate,
    ) -> Result<FactorCovariance> {
        // Get factor IDs
        let factor_ids: Vec<String> = factor_returns.keys().cloned().collect();
        let n = factor_ids.len();
        
        if n == 0 {
            return Err(FactorModelError::ValidationError("No factor returns provided".to_string()));
        }
        
        // Create return series for each factor
        let mut return_series: HashMap<String, Vec<f64>> = HashMap::new();
        let mut dates: Vec<NaiveDate> = Vec::new();
        
        // Collect all dates from all factor returns
        for (factor_id, returns) in factor_returns {
            for factor_return in returns {
                if !dates.contains(&factor_return.end_date) {
                    dates.push(factor_return.end_date);
                }
            }
        }
        
        // Sort dates
        dates.sort();
        
        // Create return series for each factor
        for factor_id in &factor_ids {
            let mut series = Vec::new();
            
            for date in &dates {
                let return_value = factor_returns
                    .get(factor_id)
                    .and_then(|returns| {
                        returns.iter()
                            .find(|r| r.end_date == *date)
                            .map(|r| r.return_value)
                    })
                    .unwrap_or(0.0);
                
                series.push(return_value);
            }
            
            return_series.insert(factor_id.clone(), series);
        }
        
        // Calculate mean returns
        let mut mean_returns = HashMap::new();
        
        for (factor_id, series) in &return_series {
            let mean = series.iter().sum::<f64>() / series.len() as f64;
            mean_returns.insert(factor_id.clone(), mean);
        }
        
        // Calculate covariance matrix
        let mut covariance_matrix = vec![vec![0.0; n]; n];
        
        for i in 0..n {
            for j in 0..n {
                let factor_i = &factor_ids[i];
                let factor_j = &factor_ids[j];
                
                let series_i = return_series.get(factor_i).unwrap();
                let series_j = return_series.get(factor_j).unwrap();
                
                let mean_i = mean_returns.get(factor_i).unwrap();
                let mean_j = mean_returns.get(factor_j).unwrap();
                
                let mut cov = 0.0;
                
                for k in 0..series_i.len() {
                    cov += (series_i[k] - mean_i) * (series_j[k] - mean_j);
                }
                
                cov /= (series_i.len() - 1) as f64;
                covariance_matrix[i][j] = cov;
            }
        }
        
        // Create FactorCovariance
        Ok(FactorCovariance::new(
            factor_ids,
            covariance_matrix,
            as_of_date,
        ))
    }
}

/// Shrinkage covariance estimator using Ledoit-Wolf method
pub struct LedoitWolfEstimator {
    /// Target matrix for shrinkage (identity or constant correlation)
    target_type: ShrinkageTarget,
}

/// Shrinkage target type
pub enum ShrinkageTarget {
    /// Identity matrix (diagonal of sample covariance)
    Identity,
    
    /// Constant correlation matrix
    ConstantCorrelation,
}

impl LedoitWolfEstimator {
    /// Create a new Ledoit-Wolf estimator with identity target
    pub fn new() -> Self {
        Self {
            target_type: ShrinkageTarget::Identity,
        }
    }
    
    /// Create a new Ledoit-Wolf estimator with constant correlation target
    pub fn with_constant_correlation() -> Self {
        Self {
            target_type: ShrinkageTarget::ConstantCorrelation,
        }
    }
    
    /// Calculate optimal shrinkage intensity
    fn calculate_shrinkage_intensity(
        &self,
        returns: &Array2<f64>,
        sample_cov: &Array2<f64>,
        target: &Array2<f64>,
    ) -> f64 {
        let (t, n) = returns.dim();
        
        // Calculate mean returns
        let mean_returns = returns.mean_axis(ndarray::Axis(0)).unwrap();
        
        // Calculate demeaned returns
        let mut demeaned_returns = returns.clone();
        for i in 0..t {
            for j in 0..n {
                demeaned_returns[[i, j]] -= mean_returns[j];
            }
        }
        
        // Calculate pi_hat (sum of asymptotic variances of entries of sample covariance matrix)
        let mut pi_hat = 0.0;
        
        for k in 0..t {
            let r_k = demeaned_returns.row(k);
            let m = Array2::from_shape_fn((n, n), |(i, j)| r_k[i] * r_k[j]);
            
            for i in 0..n {
                for j in 0..n {
                    pi_hat += (m[[i, j]] - sample_cov[[i, j]]).powi(2);
                }
            }
        }
        
        pi_hat /= t as f64;
        
        // Calculate rho_hat (Frobenius norm of difference between target and sample covariance)
        let mut rho_hat = 0.0;
        
        for i in 0..n {
            for j in 0..n {
                rho_hat += (target[[i, j]] - sample_cov[[i, j]]).powi(2);
            }
        }
        
        // Calculate optimal shrinkage intensity
        let intensity = pi_hat / rho_hat;
        
        // Clamp to [0, 1]
        intensity.max(0.0).min(1.0)
    }
}

impl CovarianceEstimator for LedoitWolfEstimator {
    fn estimate_covariance(
        &self,
        factor_returns: &HashMap<String, Vec<FactorReturn>>,
        as_of_date: NaiveDate,
    ) -> Result<FactorCovariance> {
        // First, calculate sample covariance
        let sample_estimator = SampleCovarianceEstimator::new();
        let sample_covariance = sample_estimator.estimate_covariance(factor_returns, as_of_date)?;
        
        // Get factor IDs
        let factor_ids = sample_covariance.factor_ids.clone();
        let n = factor_ids.len();
        
        // Convert to ndarray
        let sample_cov_matrix = sample_covariance.to_ndarray()
            .map_err(|e| FactorModelError::CalculationError(e.to_string()))?;
        
        // Create target matrix based on target type
        let target_matrix = match self.target_type {
            ShrinkageTarget::Identity => {
                // Use diagonal of sample covariance
                Array2::from_shape_fn((n, n), |(i, j)| {
                    if i == j {
                        sample_cov_matrix[[i, j]]
                    } else {
                        0.0
                    }
                })
            },
            ShrinkageTarget::ConstantCorrelation => {
                // Calculate average correlation
                let mut sum_corr = 0.0;
                let mut count = 0;
                
                for i in 0..n {
                    for j in 0..n {
                        if i != j {
                            let corr = sample_cov_matrix[[i, j]] / 
                                (sample_cov_matrix[[i, i]] * sample_cov_matrix[[j, j]]).sqrt();
                            sum_corr += corr;
                            count += 1;
                        }
                    }
                }
                
                let avg_corr = if count > 0 { sum_corr / count as f64 } else { 0.0 };
                
                // Create constant correlation matrix
                Array2::from_shape_fn((n, n), |(i, j)| {
                    if i == j {
                        sample_cov_matrix[[i, j]]
                    } else {
                        avg_corr * (sample_cov_matrix[[i, i]] * sample_cov_matrix[[j, j]]).sqrt()
                    }
                })
            },
        };
        
        // Create returns matrix for shrinkage intensity calculation
        // This is a placeholder - in a real implementation, we would use the actual returns
        // For now, we'll use a fixed shrinkage intensity
        let shrinkage_intensity = 0.2;
        
        // Combine sample covariance and target with shrinkage intensity
        let shrunk_matrix = &sample_cov_matrix * (1.0 - shrinkage_intensity) + &target_matrix * shrinkage_intensity;
        
        // Convert back to Vec<Vec<f64>>
        let shrunk_covariance = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| shrunk_matrix[[i, j]])
                    .collect()
            })
            .collect();
        
        // Create FactorCovariance
        Ok(FactorCovariance::new(
            factor_ids,
            shrunk_covariance,
            as_of_date,
        ))
    }
}

/// Factory for creating covariance estimators
pub struct CovarianceEstimatorFactory;

impl CovarianceEstimatorFactory {
    /// Create a sample covariance estimator
    pub fn create_sample_estimator() -> Box<dyn CovarianceEstimator> {
        Box::new(SampleCovarianceEstimator::new())
    }
    
    /// Create a Ledoit-Wolf estimator with identity target
    pub fn create_ledoit_wolf_estimator() -> Box<dyn CovarianceEstimator> {
        Box::new(LedoitWolfEstimator::new())
    }
    
    /// Create a Ledoit-Wolf estimator with constant correlation target
    pub fn create_ledoit_wolf_constant_correlation_estimator() -> Box<dyn CovarianceEstimator> {
        Box::new(LedoitWolfEstimator::with_constant_correlation())
    }
} 