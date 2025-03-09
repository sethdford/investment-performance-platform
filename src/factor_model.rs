use ndarray::{Array1, Array2};
use std::collections::HashMap;

// Import the visualization module
use crate::visualization::{
    FactorExposureHeatmapRequest, 
    VisualizationFormat, VisualizationService
};

// Factor model types
/// Factor information
#[derive(Debug, Clone)]
pub struct Factor {
    /// Factor identifier
    pub id: String,
    /// Factor name
    pub name: String,
    /// Factor category
    pub category: String,
}

pub struct FactorExposure {
    pub factor_id: String,
    pub exposure: f64,
}

pub struct FactorReturn {
    pub factor_id: String,
    pub return_value: f64,
}

pub struct Portfolio {
    pub id: String,
    pub name: String,
}

/// Factor Model API
/// 
/// This API provides access to factor model functionality.
#[derive(Debug, Clone)]
pub struct FactorModelApi {
    factors: Vec<Factor>,
    factor_returns: HashMap<String, f64>,
    factor_exposures: HashMap<String, HashMap<String, f64>>,
    factor_covariance: Array2<f64>,
}

impl FactorModelApi {
    pub fn new() -> Self {
        // Create some sample factors
        let factors = vec![
            Factor { id: "momentum".to_string(), name: "Momentum".to_string(), category: "Style".to_string() },
            Factor { id: "value".to_string(), name: "Value".to_string(), category: "Style".to_string() },
            Factor { id: "size".to_string(), name: "Size".to_string(), category: "Style".to_string() },
            Factor { id: "quality".to_string(), name: "Quality".to_string(), category: "Style".to_string() },
            Factor { id: "volatility".to_string(), name: "Volatility".to_string(), category: "Risk".to_string() },
            Factor { id: "growth".to_string(), name: "Growth".to_string(), category: "Style".to_string() },
        ];
        
        // Create sample factor returns
        let mut factor_returns = HashMap::new();
        factor_returns.insert("momentum".to_string(), 0.02);
        factor_returns.insert("value".to_string(), 0.01);
        factor_returns.insert("size".to_string(), -0.01);
        factor_returns.insert("quality".to_string(), 0.03);
        factor_returns.insert("volatility".to_string(), -0.02);
        factor_returns.insert("growth".to_string(), 0.015);
        
        // Create sample factor exposures for a portfolio
        let mut portfolio_exposures = HashMap::new();
        portfolio_exposures.insert("momentum".to_string(), 0.3);
        portfolio_exposures.insert("value".to_string(), 0.5);
        portfolio_exposures.insert("size".to_string(), -0.2);
        portfolio_exposures.insert("quality".to_string(), 0.7);
        portfolio_exposures.insert("volatility".to_string(), -0.4);
        portfolio_exposures.insert("growth".to_string(), 0.1);
        
        // Create sample factor exposures for a benchmark
        let mut benchmark_exposures = HashMap::new();
        benchmark_exposures.insert("momentum".to_string(), 0.1);
        benchmark_exposures.insert("value".to_string(), 0.0);
        benchmark_exposures.insert("size".to_string(), 0.0);
        benchmark_exposures.insert("quality".to_string(), 0.2);
        benchmark_exposures.insert("volatility".to_string(), 0.0);
        benchmark_exposures.insert("growth".to_string(), 0.3);
        
        // Store factor exposures
        let mut factor_exposures = HashMap::new();
        factor_exposures.insert("portfolio-123".to_string(), portfolio_exposures);
        factor_exposures.insert("benchmark-sp500".to_string(), benchmark_exposures);
        
        // Create a factor covariance matrix (symmetric positive definite)
        let factor_covariance = Array2::from_shape_vec((6, 6), vec![
            0.04, 0.01, 0.00, 0.01, -0.02, 0.01,
            0.01, 0.02, 0.00, 0.00, -0.01, 0.00,
            0.00, 0.00, 0.03, 0.00, 0.01, 0.00,
            0.01, 0.00, 0.00, 0.01, 0.00, 0.01,
            -0.02, -0.01, 0.01, 0.00, 0.05, -0.01,
            0.01, 0.00, 0.00, 0.01, -0.01, 0.02,
        ]).unwrap();
        
        Self {
            factors,
            factor_returns,
            factor_exposures,
            factor_covariance,
        }
    }
    
    pub fn get_factors(&self) -> &Vec<Factor> {
        &self.factors
    }
    
    pub fn get_factor_exposures(&self, portfolio_id: &str) -> Option<&HashMap<String, f64>> {
        self.factor_exposures.get(portfolio_id)
    }
    
    pub fn calculate_factor_contribution(&self, portfolio_id: &str) -> Option<HashMap<String, f64>> {
        let exposures = self.get_factor_exposures(portfolio_id)?;
        
        let mut contributions = HashMap::new();
        for (factor_id, exposure) in exposures {
            let factor_return = self.factor_returns.get(factor_id).unwrap_or(&0.0);
            let contribution = exposure * factor_return;
            contributions.insert(factor_id.clone(), contribution);
        }
        
        Some(contributions)
    }
    
    pub fn calculate_active_exposures(&self, portfolio_id: &str, benchmark_id: &str) -> Option<HashMap<String, f64>> {
        let portfolio_exposures = self.get_factor_exposures(portfolio_id)?;
        let benchmark_exposures = self.get_factor_exposures(benchmark_id)?;
        
        let mut active_exposures = HashMap::new();
        for (factor_id, portfolio_exposure) in portfolio_exposures {
            let benchmark_exposure = benchmark_exposures.get(factor_id).unwrap_or(&0.0);
            let active_exposure = portfolio_exposure - benchmark_exposure;
            active_exposures.insert(factor_id.clone(), active_exposure);
        }
        
        Some(active_exposures)
    }
    
    pub fn calculate_portfolio_risk(&self, portfolio_id: &str) -> Option<f64> {
        let exposures = self.get_factor_exposures(portfolio_id)?;
        
        // Convert exposures to a vector in the same order as the covariance matrix
        let mut exposure_vector = Array1::zeros(self.factors.len());
        for (i, factor) in self.factors.iter().enumerate() {
            let exposure = exposures.get(&factor.id).unwrap_or(&0.0);
            exposure_vector[i] = *exposure;
        }
        
        // Calculate portfolio risk: sqrt(e^T * V * e)
        let ve = self.factor_covariance.dot(&exposure_vector);
        let risk = (exposure_vector.dot(&ve)).sqrt();
        
        Some(risk)
    }
    
    pub fn calculate_risk_decomposition(&self, portfolio_id: &str) -> Option<HashMap<String, f64>> {
        let exposures = self.get_factor_exposures(portfolio_id)?;
        let portfolio_risk = self.calculate_portfolio_risk(portfolio_id)?;
        
        // Convert exposures to a vector in the same order as the covariance matrix
        let mut exposure_vector = Array1::zeros(self.factors.len());
        for (i, factor) in self.factors.iter().enumerate() {
            let exposure = exposures.get(&factor.id).unwrap_or(&0.0);
            exposure_vector[i] = *exposure;
        }
        
        // Calculate risk contribution for each factor
        let ve = self.factor_covariance.dot(&exposure_vector);
        
        let mut risk_contributions = HashMap::new();
        for (i, factor) in self.factors.iter().enumerate() {
            let exposure = exposure_vector[i];
            let marginal_contribution = ve[i];
            let contribution = exposure * marginal_contribution / portfolio_risk;
            risk_contributions.insert(factor.id.clone(), contribution);
        }
        
        Some(risk_contributions)
    }
}

// Example usage
pub fn run_factor_model_example() {
    println!("Running Factor Model Example with BLAS and LAPACK integration");
    
    // Create the factor model API
    let api = FactorModelApi::new();
    
    // Define portfolio and benchmark IDs
    let portfolio_id = "portfolio-123";
    let benchmark_id = "benchmark-sp500";
    
    // Get factors
    println!("\n=== Factors ===");
    for factor in api.get_factors() {
        println!("Factor: {} ({}), Category: {}", factor.name, factor.id, factor.category);
    }
    
    // Get factor exposures
    println!("\n=== Factor Exposures ===");
    if let Some(exposures) = api.get_factor_exposures(portfolio_id) {
        println!("Portfolio Factor Exposures:");
        for (factor_id, exposure) in exposures {
            println!("  {} = {:.2}", factor_id, exposure);
        }
    }
    
    // Calculate factor contribution
    println!("\n=== Factor Contribution ===");
    if let Some(contributions) = api.calculate_factor_contribution(portfolio_id) {
        println!("Portfolio Factor Contributions:");
        for (factor_id, contribution) in contributions {
            println!("  {} = {:.4}", factor_id, contribution);
        }
    }
    
    // Calculate active exposures
    println!("\n=== Active Exposures ===");
    if let Some(active_exposures) = api.calculate_active_exposures(portfolio_id, benchmark_id) {
        println!("Active Exposures vs Benchmark:");
        for (factor_id, active_exposure) in active_exposures {
            println!("  {} = {:.2}", factor_id, active_exposure);
        }
    }
    
    // Calculate portfolio risk
    println!("\n=== Portfolio Risk ===");
    if let Some(risk) = api.calculate_portfolio_risk(portfolio_id) {
        println!("Portfolio Risk: {:.4}", risk);
    }
    
    // Calculate risk decomposition
    println!("\n=== Risk Decomposition ===");
    if let Some(risk_contributions) = api.calculate_risk_decomposition(portfolio_id) {
        println!("Risk Contributions:");
        for (factor_id, contribution) in risk_contributions {
            println!("  {} = {:.4}", factor_id, contribution);
        }
    }
    
    // Generate visualizations
    println!("\n=== Generate Visualizations ===");
    
    // Create visualization service
    let viz_service = VisualizationService::new();
    
    // Get the data for visualization
    if let (Some(portfolio_exposures), Some(benchmark_exposures), Some(active_exposures)) = (
        api.get_factor_exposures(portfolio_id).cloned(),
        api.get_factor_exposures(benchmark_id).cloned(),
        api.calculate_active_exposures(portfolio_id, benchmark_id)
    ) {
        // Create request for factor exposure heatmap
        let heatmap_request = FactorExposureHeatmapRequest {
            portfolio_exposures,
            benchmark_exposures: Some(benchmark_exposures),
            active_exposures: Some(active_exposures),
            format: VisualizationFormat::Svg,
            title: Some("Factor Exposure Analysis".to_string()),
        };
        
        // Generate visualization
        let heatmap_response = viz_service.generate_factor_exposure_heatmap(heatmap_request);
        
        // Save visualization to file
        let filename = "factor_exposure_heatmap.svg";
        if let Err(e) = viz_service.save_visualization(&heatmap_response, filename) {
            println!("Error saving visualization: {}", e);
        } else {
            println!("Saved factor exposure heatmap to {}", filename);
        }
    }
    
    println!("\nFactor Model Example completed successfully!");
} 