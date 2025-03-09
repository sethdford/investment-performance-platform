use chrono::NaiveDate;
use performance_calculator::calculations::factor_model::{
    Factor, FactorCategory, FactorCovariance, FactorExposure, FactorReturn,
    FactorModelCalculator, FactorRepository, FactorModelVersion, FactorModelStatus,
    CovarianceEstimatorFactory,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Import the in-memory repository implementation
use performance_calculator::calculations::factor_model::repository::InMemoryFactorRepository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Factor Model Example");
    println!("===================");

    // Create a repository
    let repository = Arc::new(InMemoryFactorRepository::new());
    
    // Create some factors
    let market_factor = Factor::new(
        "MKT".to_string(),
        "Market".to_string(),
        "Market factor representing broad market movements".to_string(),
        FactorCategory::Market,
    );
    
    let size_factor = Factor::new(
        "SIZE".to_string(),
        "Size".to_string(),
        "Size factor representing the size premium".to_string(),
        FactorCategory::Style,
    );
    
    let value_factor = Factor::new(
        "VAL".to_string(),
        "Value".to_string(),
        "Value factor representing the value premium".to_string(),
        FactorCategory::Style,
    );
    
    // Add factors to repository
    repository.create_factor(market_factor.clone()).await?;
    repository.create_factor(size_factor.clone()).await?;
    repository.create_factor(value_factor.clone()).await?;
    
    println!("Created factors: Market, Size, Value");
    
    // Create factor returns
    let start_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    let dates = (0..10).map(|i| start_date.checked_add_days(chrono::Days::new(i)).unwrap()).collect::<Vec<_>>();
    
    // Market factor returns
    for (i, date) in dates.iter().enumerate() {
        let return_value = 0.001 * (i as f64) + 0.005;
        let factor_return = FactorReturn::new(
            "MKT".to_string(),
            return_value,
            date.checked_sub_days(chrono::Days::new(1)).unwrap(),
            *date,
        );
        repository.create_factor_return(factor_return).await?;
    }
    
    // Size factor returns
    for (i, date) in dates.iter().enumerate() {
        let return_value = -0.0005 * (i as f64) + 0.002;
        let factor_return = FactorReturn::new(
            "SIZE".to_string(),
            return_value,
            date.checked_sub_days(chrono::Days::new(1)).unwrap(),
            *date,
        );
        repository.create_factor_return(factor_return).await?;
    }
    
    // Value factor returns
    for (i, date) in dates.iter().enumerate() {
        let return_value = 0.0008 * (i as f64) - 0.001;
        let factor_return = FactorReturn::new(
            "VAL".to_string(),
            return_value,
            date.checked_sub_days(chrono::Days::new(1)).unwrap(),
            *date,
        );
        repository.create_factor_return(factor_return).await?;
    }
    
    println!("Created factor returns for 10 days");
    
    // Create factor exposures for a security
    let security_id = "AAPL";
    let as_of_date = dates.last().unwrap();
    
    let market_exposure = FactorExposure::new(
        security_id.to_string(),
        "MKT".to_string(),
        1.05,
        *as_of_date,
    );
    
    let size_exposure = FactorExposure::new(
        security_id.to_string(),
        "SIZE".to_string(),
        -0.8,
        *as_of_date,
    );
    
    let value_exposure = FactorExposure::new(
        security_id.to_string(),
        "VAL".to_string(),
        0.3,
        *as_of_date,
    );
    
    repository.create_factor_exposure(market_exposure).await?;
    repository.create_factor_exposure(size_exposure).await?;
    repository.create_factor_exposure(value_exposure).await?;
    
    println!("Created factor exposures for security {}", security_id);
    
    // Create a factor model version
    let model_version = FactorModelVersion::new(
        "V1".to_string(),
        "First Version".to_string(),
        "Initial factor model version".to_string(),
        start_date,
        None,
        FactorModelStatus::Active,
    );
    
    repository.create_factor_model_version(model_version).await?;
    
    println!("Created factor model version V1");
    
    // Retrieve factor returns for covariance calculation
    let factor_ids = vec!["MKT".to_string(), "SIZE".to_string(), "VAL".to_string()];
    let mut factor_returns_map = HashMap::new();
    
    for factor_id in &factor_ids {
        let returns = repository.get_factor_returns(
            &[factor_id.clone()],
            start_date,
            *as_of_date,
        ).await?;
        
        factor_returns_map.insert(factor_id.clone(), returns);
    }
    
    // Calculate covariance matrix using sample estimator
    let sample_estimator = CovarianceEstimatorFactory::create_sample_estimator();
    let sample_covariance = sample_estimator.estimate_covariance(
        &factor_returns_map,
        *as_of_date,
    )?;
    
    println!("\nSample Covariance Matrix:");
    print_covariance_matrix(&sample_covariance);
    
    // Calculate covariance matrix using Ledoit-Wolf estimator
    let ledoit_wolf_estimator = CovarianceEstimatorFactory::create_ledoit_wolf_estimator();
    let ledoit_wolf_covariance = ledoit_wolf_estimator.estimate_covariance(
        &factor_returns_map,
        *as_of_date,
    )?;
    
    println!("\nLedoit-Wolf Covariance Matrix:");
    print_covariance_matrix(&ledoit_wolf_covariance);
    
    // Store the covariance matrix
    repository.create_factor_covariance(ledoit_wolf_covariance.clone()).await?;
    
    println!("\nStored factor covariance matrix in repository");
    
    // Retrieve the covariance matrix
    let retrieved_covariance = repository.get_factor_covariance(
        &factor_ids,
        *as_of_date,
    ).await?;
    
    println!("\nRetrieved Covariance Matrix:");
    print_covariance_matrix(&retrieved_covariance);
    
    println!("\nFactor Model Example completed successfully!");
    
    Ok(())
}

// Helper function to print a covariance matrix
fn print_covariance_matrix(covariance: &FactorCovariance) {
    let n = covariance.factor_ids.len();
    
    // Print header
    print!("{:>10}", "");
    for factor_id in &covariance.factor_ids {
        print!("{:>10}", factor_id);
    }
    println!();
    
    // Print rows
    for i in 0..n {
        print!("{:>10}", covariance.factor_ids[i]);
        for j in 0..n {
            print!("{:>10.6}", covariance.covariance_matrix[i][j]);
        }
        println!();
    }
} 