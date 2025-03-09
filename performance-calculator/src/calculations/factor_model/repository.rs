//! Repository interfaces and implementations for factor model data

use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::error::FactorModelError;
use super::types::{Factor, FactorCategory, FactorCovariance, FactorExposure, FactorReturn, FactorModelVersion, FactorModelStatus};
use super::Result;

/// Repository for factor model data
#[async_trait]
pub trait FactorRepository: Send + Sync {
    /// Get a factor by ID
    async fn get_factor(&self, factor_id: &str) -> Result<Factor>;
    
    /// Get all factors
    async fn get_all_factors(&self) -> Result<Vec<Factor>>;
    
    /// Get factors by category
    async fn get_factors_by_category(&self, category: &FactorCategory) -> Result<Vec<Factor>>;
    
    /// Create a new factor
    async fn create_factor(&self, factor: Factor) -> Result<Factor>;
    
    /// Update a factor
    async fn update_factor(&self, factor: Factor) -> Result<Factor>;
    
    /// Delete a factor
    async fn delete_factor(&self, factor_id: &str) -> Result<()>;
    
    /// Get factor exposures for a security on a specific date
    async fn get_factor_exposures_for_security(
        &self,
        security_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<Vec<FactorExposure>>;
    
    /// Get factor exposures for multiple securities on a specific date
    async fn get_factor_exposures_for_securities(
        &self,
        security_ids: &[String],
        as_of_date: NaiveDate,
    ) -> Result<HashMap<String, Vec<FactorExposure>>>;
    
    /// Get factor exposures for a specific factor on a specific date
    async fn get_factor_exposures_for_factor(
        &self,
        factor_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<Vec<FactorExposure>>;
    
    /// Create a new factor exposure
    async fn create_factor_exposure(&self, exposure: FactorExposure) -> Result<FactorExposure>;
    
    /// Create multiple factor exposures
    async fn create_factor_exposures(&self, exposures: Vec<FactorExposure>) -> Result<Vec<FactorExposure>>;
    
    /// Get factor returns for a specific period
    async fn get_factor_returns(
        &self,
        factor_ids: &[String],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<FactorReturn>>;
    
    /// Create a new factor return
    async fn create_factor_return(&self, factor_return: FactorReturn) -> Result<FactorReturn>;
    
    /// Create multiple factor returns
    async fn create_factor_returns(&self, factor_returns: Vec<FactorReturn>) -> Result<Vec<FactorReturn>>;
    
    /// Get factor covariance matrix for a specific date
    async fn get_factor_covariance(
        &self,
        factor_ids: &[String],
        as_of_date: NaiveDate,
    ) -> Result<FactorCovariance>;
    
    /// Create a new factor covariance matrix
    async fn create_factor_covariance(&self, covariance: FactorCovariance) -> Result<FactorCovariance>;
    
    /// Get factor model version by ID
    async fn get_factor_model_version(&self, version_id: &str) -> Result<FactorModelVersion>;
    
    /// Get active factor model version for a specific date
    async fn get_active_factor_model_version(&self, as_of_date: NaiveDate) -> Result<FactorModelVersion>;
    
    /// Create a new factor model version
    async fn create_factor_model_version(&self, version: FactorModelVersion) -> Result<FactorModelVersion>;
    
    /// Update a factor model version
    async fn update_factor_model_version(&self, version: FactorModelVersion) -> Result<FactorModelVersion>;
}

/// In-memory implementation of the factor repository
pub struct InMemoryFactorRepository {
    factors: Mutex<HashMap<String, Factor>>,
    exposures: Mutex<Vec<FactorExposure>>,
    returns: Mutex<Vec<FactorReturn>>,
    covariances: Mutex<Vec<FactorCovariance>>,
    model_versions: Mutex<HashMap<String, FactorModelVersion>>,
}

impl InMemoryFactorRepository {
    /// Create a new in-memory factor repository
    pub fn new() -> Self {
        Self {
            factors: Mutex::new(HashMap::new()),
            exposures: Mutex::new(Vec::new()),
            returns: Mutex::new(Vec::new()),
            covariances: Mutex::new(Vec::new()),
            model_versions: Mutex::new(HashMap::new()),
        }
    }
    
    /// Create a new in-memory factor repository with initial data
    pub fn with_data(
        factors: Vec<Factor>,
        exposures: Vec<FactorExposure>,
        returns: Vec<FactorReturn>,
        covariances: Vec<FactorCovariance>,
        model_versions: Vec<FactorModelVersion>,
    ) -> Self {
        let factors_map = factors.into_iter().map(|f| (f.id.clone(), f)).collect();
        let model_versions_map = model_versions.into_iter().map(|v| (v.id.clone(), v)).collect();
        
        Self {
            factors: Mutex::new(factors_map),
            exposures: Mutex::new(exposures),
            returns: Mutex::new(returns),
            covariances: Mutex::new(covariances),
            model_versions: Mutex::new(model_versions_map),
        }
    }
}

#[async_trait]
impl FactorRepository for InMemoryFactorRepository {
    async fn get_factor(&self, factor_id: &str) -> Result<Factor> {
        let factors = self.factors.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        
        factors.get(factor_id)
            .cloned()
            .ok_or_else(|| FactorModelError::FactorNotFound(factor_id.to_string()))
    }
    
    async fn get_all_factors(&self) -> Result<Vec<Factor>> {
        let factors = self.factors.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        
        Ok(factors.values().cloned().collect())
    }
    
    async fn get_factors_by_category(&self, category: &FactorCategory) -> Result<Vec<Factor>> {
        let factors = self.factors.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        
        Ok(factors.values()
            .filter(|f| &f.category == category)
            .cloned()
            .collect())
    }
    
    async fn create_factor(&self, factor: Factor) -> Result<Factor> {
        let mut factors = self.factors.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        
        if factors.contains_key(&factor.id) {
            return Err(FactorModelError::ValidationError(format!("Factor with ID {} already exists", factor.id)));
        }
        
        let factor_clone = factor.clone();
        factors.insert(factor.id.clone(), factor);
        
        Ok(factor_clone)
    }
    
    async fn update_factor(&self, factor: Factor) -> Result<Factor> {
        let mut factors = self.factors.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        
        if !factors.contains_key(&factor.id) {
            return Err(FactorModelError::FactorNotFound(factor.id.clone()));
        }
        
        let factor_clone = factor.clone();
        factors.insert(factor.id.clone(), factor);
        
        Ok(factor_clone)
    }
    
    async fn delete_factor(&self, factor_id: &str) -> Result<()> {
        let mut factors = self.factors.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        
        if !factors.contains_key(factor_id) {
            return Err(FactorModelError::FactorNotFound(factor_id.to_string()));
        }
        
        factors.remove(factor_id);
        
        Ok(())
    }
    
    async fn get_factor_exposures_for_security(
        &self,
        security_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<Vec<FactorExposure>> {
        let exposures = self.exposures.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        
        Ok(exposures.iter()
            .filter(|e| e.security_id == security_id && e.as_of_date == as_of_date)
            .cloned()
            .collect())
    }
    
    async fn get_factor_exposures_for_securities(
        &self,
        security_ids: &[String],
        as_of_date: NaiveDate,
    ) -> Result<HashMap<String, Vec<FactorExposure>>> {
        let exposures = self.exposures.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        
        let mut result = HashMap::new();
        
        for security_id in security_ids {
            let security_exposures: Vec<FactorExposure> = exposures.iter()
                .filter(|e| e.security_id == *security_id && e.as_of_date == as_of_date)
                .cloned()
                .collect();
            
            result.insert(security_id.clone(), security_exposures);
        }
        
        Ok(result)
    }
    
    async fn get_factor_exposures_for_factor(
        &self,
        factor_id: &str,
        as_of_date: NaiveDate,
    ) -> Result<Vec<FactorExposure>> {
        let exposures = self.exposures.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        
        Ok(exposures.iter()
            .filter(|e| e.factor_id == factor_id && e.as_of_date == as_of_date)
            .cloned()
            .collect())
    }
    
    async fn create_factor_exposure(&self, exposure: FactorExposure) -> Result<FactorExposure> {
        let mut exposures = self.exposures.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        
        // Check if the factor exists
        let factors = self.factors.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        if !factors.contains_key(&exposure.factor_id) {
            return Err(FactorModelError::FactorNotFound(exposure.factor_id.clone()));
        }
        
        let exposure_clone = exposure.clone();
        exposures.push(exposure);
        
        Ok(exposure_clone)
    }
    
    async fn create_factor_exposures(&self, exposures: Vec<FactorExposure>) -> Result<Vec<FactorExposure>> {
        let mut result = Vec::new();
        
        for exposure in exposures {
            let created = self.create_factor_exposure(exposure).await?;
            result.push(created);
        }
        
        Ok(result)
    }
    
    async fn get_factor_returns(
        &self,
        factor_ids: &[String],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<FactorReturn>> {
        let returns = self.returns.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        
        Ok(returns.iter()
            .filter(|r| {
                factor_ids.contains(&r.factor_id) &&
                r.start_date >= start_date &&
                r.end_date <= end_date
            })
            .cloned()
            .collect())
    }
    
    async fn create_factor_return(&self, factor_return: FactorReturn) -> Result<FactorReturn> {
        let mut returns = self.returns.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        
        // Check if the factor exists
        let factors = self.factors.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        if !factors.contains_key(&factor_return.factor_id) {
            return Err(FactorModelError::FactorNotFound(factor_return.factor_id.clone()));
        }
        
        let return_clone = factor_return.clone();
        returns.push(factor_return);
        
        Ok(return_clone)
    }
    
    async fn create_factor_returns(&self, factor_returns: Vec<FactorReturn>) -> Result<Vec<FactorReturn>> {
        let mut result = Vec::new();
        
        for factor_return in factor_returns {
            let created = self.create_factor_return(factor_return).await?;
            result.push(created);
        }
        
        Ok(result)
    }
    
    async fn get_factor_covariance(
        &self,
        factor_ids: &[String],
        as_of_date: NaiveDate,
    ) -> Result<FactorCovariance> {
        let covariances = self.covariances.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        
        // Find the most recent covariance matrix that includes all the requested factors
        // and is not after the as_of_date
        let covariance = covariances.iter()
            .filter(|c| {
                c.as_of_date <= as_of_date &&
                factor_ids.iter().all(|id| c.factor_ids.contains(id))
            })
            .max_by_key(|c| c.as_of_date)
            .cloned()
            .ok_or_else(|| FactorModelError::DataNotAvailable(format!("No covariance matrix available for date {}", as_of_date)))?;
        
        // If the covariance matrix contains more factors than requested, extract the subset
        if covariance.factor_ids.len() > factor_ids.len() {
            // Create a mapping from factor ID to index in the original matrix
            let factor_indices: HashMap<String, usize> = covariance.factor_ids.iter()
                .enumerate()
                .map(|(i, id)| (id.clone(), i))
                .collect();
            
            // Create a new matrix with only the requested factors
            let mut new_factor_ids = Vec::new();
            let mut new_covariance_matrix = Vec::new();
            
            for factor_id in factor_ids {
                if let Some(&i) = factor_indices.get(factor_id) {
                    new_factor_ids.push(factor_id.clone());
                    
                    let mut row = Vec::new();
                    for other_factor_id in factor_ids {
                        if let Some(&j) = factor_indices.get(other_factor_id) {
                            row.push(covariance.covariance_matrix[i][j]);
                        }
                    }
                    
                    new_covariance_matrix.push(row);
                }
            }
            
            return Ok(FactorCovariance {
                factor_ids: new_factor_ids,
                covariance_matrix: new_covariance_matrix,
                as_of_date: covariance.as_of_date,
                created_at: Utc::now(),
                metadata: covariance.metadata.clone(),
            });
        }
        
        Ok(covariance)
    }
    
    async fn create_factor_covariance(&self, covariance: FactorCovariance) -> Result<FactorCovariance> {
        let mut covariances = self.covariances.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        
        // Check if all factors exist
        let factors = self.factors.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        for factor_id in &covariance.factor_ids {
            if !factors.contains_key(factor_id) {
                return Err(FactorModelError::FactorNotFound(factor_id.clone()));
            }
        }
        
        // Validate the covariance matrix
        let n = covariance.factor_ids.len();
        if covariance.covariance_matrix.len() != n {
            return Err(FactorModelError::ValidationError("Covariance matrix rows don't match factor count".to_string()));
        }
        
        for row in &covariance.covariance_matrix {
            if row.len() != n {
                return Err(FactorModelError::ValidationError("Covariance matrix is not square".to_string()));
            }
        }
        
        let covariance_clone = covariance.clone();
        covariances.push(covariance);
        
        Ok(covariance_clone)
    }
    
    async fn get_factor_model_version(&self, version_id: &str) -> Result<FactorModelVersion> {
        let model_versions = self.model_versions.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        
        model_versions.get(version_id)
            .cloned()
            .ok_or_else(|| FactorModelError::ValidationError(format!("Factor model version with ID {} not found", version_id)))
    }
    
    async fn get_active_factor_model_version(&self, as_of_date: NaiveDate) -> Result<FactorModelVersion> {
        let model_versions = self.model_versions.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        
        model_versions.values()
            .filter(|v| v.is_active_on(as_of_date))
            .max_by_key(|v| v.effective_date)
            .cloned()
            .ok_or_else(|| FactorModelError::DataNotAvailable(format!("No active factor model version for date {}", as_of_date)))
    }
    
    async fn create_factor_model_version(&self, version: FactorModelVersion) -> Result<FactorModelVersion> {
        let mut model_versions = self.model_versions.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        
        if model_versions.contains_key(&version.id) {
            return Err(FactorModelError::ValidationError(format!("Factor model version with ID {} already exists", version.id)));
        }
        
        let version_clone = version.clone();
        model_versions.insert(version.id.clone(), version);
        
        Ok(version_clone)
    }
    
    async fn update_factor_model_version(&self, version: FactorModelVersion) -> Result<FactorModelVersion> {
        let mut model_versions = self.model_versions.lock().map_err(|e| FactorModelError::RepositoryError(e.to_string()))?;
        
        if !model_versions.contains_key(&version.id) {
            return Err(FactorModelError::ValidationError(format!("Factor model version with ID {} not found", version.id)));
        }
        
        let version_clone = version.clone();
        model_versions.insert(version.id.clone(), version);
        
        Ok(version_clone)
    }
}

/// Factory for creating factor repositories
pub struct FactorRepositoryFactory;

impl FactorRepositoryFactory {
    /// Create a new in-memory factor repository
    pub fn create_in_memory_repository() -> Arc<dyn FactorRepository> {
        Arc::new(InMemoryFactorRepository::new())
    }
    
    /// Create a new in-memory factor repository with sample data
    pub fn create_sample_repository() -> Arc<dyn FactorRepository> {
        // Create sample factors
        let market_factor = Factor::new(
            "MKT".to_string(),
            "Market".to_string(),
            "Market factor representing systematic market risk".to_string(),
            FactorCategory::Market,
        );
        
        let size_factor = Factor::new(
            "SIZE".to_string(),
            "Size".to_string(),
            "Size factor representing the risk premium of small cap stocks".to_string(),
            FactorCategory::Style,
        );
        
        let value_factor = Factor::new(
            "VAL".to_string(),
            "Value".to_string(),
            "Value factor representing the risk premium of value stocks".to_string(),
            FactorCategory::Style,
        );
        
        let momentum_factor = Factor::new(
            "MOM".to_string(),
            "Momentum".to_string(),
            "Momentum factor representing the risk premium of stocks with positive momentum".to_string(),
            FactorCategory::Style,
        );
        
        let tech_factor = Factor::new(
            "TECH".to_string(),
            "Technology".to_string(),
            "Technology sector factor".to_string(),
            FactorCategory::Industry,
        );
        
        let factors = vec![
            market_factor,
            size_factor,
            value_factor,
            momentum_factor,
            tech_factor,
        ];
        
        // Create a sample factor model version
        let model_version = FactorModelVersion::new(
            "V1".to_string(),
            "Sample Model v1.0".to_string(),
            "Sample factor model for testing".to_string(),
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            None,
            FactorModelStatus::Active,
        );
        
        let model_versions = vec![model_version];
        
        // Create the repository
        Arc::new(InMemoryFactorRepository::with_data(
            factors,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            model_versions,
        ))
    }
} 