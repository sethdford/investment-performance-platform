//! Core data structures for the factor model

use chrono::{DateTime, NaiveDate, Utc};
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Factor category representing the type of factor
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactorCategory {
    /// Market factor (e.g., market beta)
    Market,
    
    /// Style factor (e.g., value, growth, size)
    Style,
    
    /// Industry factor (e.g., technology, healthcare)
    Industry,
    
    /// Country factor (e.g., US, Japan)
    Country,
    
    /// Currency factor (e.g., USD, EUR)
    Currency,
    
    /// Macroeconomic factor (e.g., inflation, interest rates)
    Macro,
    
    /// Custom factor defined by the user
    Custom(String),
}

/// Factor representing a risk factor in a multi-factor model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Factor {
    /// Unique identifier for the factor
    pub id: String,
    
    /// Name of the factor
    pub name: String,
    
    /// Description of the factor
    pub description: String,
    
    /// Category of the factor
    pub category: FactorCategory,
    
    /// Creation date of the factor
    pub created_at: DateTime<Utc>,
    
    /// Last update date of the factor
    pub updated_at: DateTime<Utc>,
    
    /// Additional metadata for the factor
    pub metadata: HashMap<String, String>,
}

impl Factor {
    /// Create a new factor
    pub fn new(
        id: String,
        name: String,
        description: String,
        category: FactorCategory,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description,
            category,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the factor
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Factor exposure representing a security's exposure to a factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorExposure {
    /// Security identifier
    pub security_id: String,
    
    /// Factor identifier
    pub factor_id: String,
    
    /// Exposure value
    pub exposure: f64,
    
    /// Date of the exposure
    pub as_of_date: NaiveDate,
    
    /// Creation date of the exposure record
    pub created_at: DateTime<Utc>,
}

impl FactorExposure {
    /// Create a new factor exposure
    pub fn new(
        security_id: String,
        factor_id: String,
        exposure: f64,
        as_of_date: NaiveDate,
    ) -> Self {
        Self {
            security_id,
            factor_id,
            exposure,
            as_of_date,
            created_at: Utc::now(),
        }
    }
}

/// Factor return representing the return of a factor for a period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorReturn {
    /// Factor identifier
    pub factor_id: String,
    
    /// Return value
    pub return_value: f64,
    
    /// Start date of the period
    pub start_date: NaiveDate,
    
    /// End date of the period
    pub end_date: NaiveDate,
    
    /// Creation date of the return record
    pub created_at: DateTime<Utc>,
}

impl FactorReturn {
    /// Create a new factor return
    pub fn new(
        factor_id: String,
        return_value: f64,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Self {
        Self {
            factor_id,
            return_value,
            start_date,
            end_date,
            created_at: Utc::now(),
        }
    }
}

/// Factor covariance matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorCovariance {
    /// Factor identifiers in the order they appear in the matrix
    pub factor_ids: Vec<String>,
    
    /// Covariance matrix
    pub covariance_matrix: Vec<Vec<f64>>,
    
    /// As of date for the covariance matrix
    pub as_of_date: NaiveDate,
    
    /// Creation date of the covariance matrix
    pub created_at: DateTime<Utc>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl FactorCovariance {
    /// Create a new factor covariance matrix
    pub fn new(
        factor_ids: Vec<String>,
        covariance_matrix: Vec<Vec<f64>>,
        as_of_date: NaiveDate,
    ) -> Self {
        Self {
            factor_ids,
            covariance_matrix,
            as_of_date,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }
    
    /// Convert to ndarray format
    pub fn to_ndarray(&self) -> Result<Array2<f64>, &'static str> {
        let n = self.factor_ids.len();
        if self.covariance_matrix.len() != n {
            return Err("Covariance matrix rows don't match factor count");
        }
        
        for row in &self.covariance_matrix {
            if row.len() != n {
                return Err("Covariance matrix is not square");
            }
        }
        
        let flat_data: Vec<f64> = self.covariance_matrix
            .iter()
            .flat_map(|row| row.iter().cloned())
            .collect();
        
        Ok(Array2::from_shape_vec((n, n), flat_data)
            .map_err(|_| "Failed to create ndarray")?)
    }
    
    /// Create from ndarray format
    pub fn from_ndarray(
        factor_ids: Vec<String>,
        matrix: Array2<f64>,
        as_of_date: NaiveDate,
    ) -> Result<Self, &'static str> {
        let n = factor_ids.len();
        let shape = matrix.shape();
        
        if shape[0] != n || shape[1] != n {
            return Err("Matrix dimensions don't match factor count");
        }
        
        let covariance_matrix = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| matrix[[i, j]])
                    .collect()
            })
            .collect();
        
        Ok(Self::new(factor_ids, covariance_matrix, as_of_date))
    }
}

/// Status of a factor model version
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactorModelStatus {
    /// Model is in development
    Development,
    
    /// Model is being tested
    Testing,
    
    /// Model is active and in production
    Active,
    
    /// Model is archived and no longer in use
    Archived,
}

/// Factor model version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorModelVersion {
    /// Unique identifier for the model version
    pub id: String,
    
    /// Name of the model version
    pub name: String,
    
    /// Description of the model version
    pub description: String,
    
    /// Effective date of the model version
    pub effective_date: NaiveDate,
    
    /// Expiration date of the model version (if any)
    pub expiration_date: Option<NaiveDate>,
    
    /// Creation date of the model version
    pub created_at: DateTime<Utc>,
    
    /// Last update date of the model version
    pub updated_at: DateTime<Utc>,
    
    /// Status of the model version
    pub status: FactorModelStatus,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl FactorModelVersion {
    /// Create a new factor model version
    pub fn new(
        id: String,
        name: String,
        description: String,
        effective_date: NaiveDate,
        expiration_date: Option<NaiveDate>,
        status: FactorModelStatus,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description,
            effective_date,
            expiration_date,
            created_at: now,
            updated_at: now,
            status,
            metadata: HashMap::new(),
        }
    }
    
    /// Set the status of the model version
    pub fn with_status(mut self, status: FactorModelStatus) -> Self {
        self.status = status;
        self
    }
    
    /// Set the expiration date of the model version
    pub fn with_expiration_date(mut self, expiration_date: NaiveDate) -> Self {
        self.expiration_date = Some(expiration_date);
        self
    }
    
    /// Add metadata to the model version
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Check if the model version is active on a given date
    pub fn is_active_on(&self, date: NaiveDate) -> bool {
        if self.status != FactorModelStatus::Active {
            return false;
        }
        
        if date < self.effective_date {
            return false;
        }
        
        if let Some(expiration_date) = self.expiration_date {
            if date > expiration_date {
                return false;
            }
        }
        
        true
    }
} 