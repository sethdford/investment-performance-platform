use performance_calculator::calculations::factor_model::{
    api::{
        DefaultFactorModelApiService, FactorExposureRequest, FactorExposureResponse,
        FactorModelApiService,
    },
    repository::{FactorRepository, InMemoryFactorRepository},
    types::{Factor, FactorCategory, FactorExposure, FactorModelVersion, FactorReturn, FactorModelStatus},
    visualization::{
        ColorScheme, DefaultFactorModelVisualizationService, FactorExposureHeatmapRequest,
        FactorModelVisualizationService, VisualizationFormat,
    },
    calculator::DefaultFactorModelCalculator,
};
use std::{collections::HashMap, fs::File, io::Write, path::Path, sync::Arc};
use chrono::{NaiveDate, Utc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Factor Model API Example");

    // Create a simple repository with sample data
    let repository = create_sample_repository().await?;
    
    // Create calculator
    let calculator = Arc::new(DefaultFactorModelCalculator::new(repository.clone()));
    
    // Create API service
    let api_service = DefaultFactorModelApiService::new(calculator, repository.clone());
    
    // Create visualization service
    let viz_service = DefaultFactorModelVisualizationService::new();

    // Define portfolio and benchmark for analysis
    let portfolio_id = "portfolio-123";
    let benchmark_id = "benchmark-sp500";
    let as_of_date = NaiveDate::from_ymd_opt(2023, 6, 30).unwrap();

    println!("\n=== Factor Exposure Analysis ===");
    // Create request for factor exposure analysis
    let exposure_request = FactorExposureRequest {
        portfolio_id: portfolio_id.to_string(),
        benchmark_id: Some(benchmark_id.to_string()),
        as_of_date,
        factor_categories: None,
    };

    // Call API to analyze factor exposures
    let exposure_result = api_service.analyze_factor_exposures(exposure_request).await?;
    
    // Print results
    println!("Portfolio Factor Exposures:");
    for (factor, exposure) in &exposure_result.portfolio_exposures {
        println!("  {} = {}", factor, exposure);
    }
    
    println!("\nActive Exposures vs Benchmark:");
    if let Some(active_exposures) = &exposure_result.active_exposures {
        for (factor, exposure) in active_exposures {
            println!("  {} = {}", factor, exposure);
        }
    }

    println!("\n=== Generate Visualizations ===");
    // Create request for factor exposure heatmap
    let heatmap_request = FactorExposureHeatmapRequest {
        exposure_data: exposure_result,
        format: VisualizationFormat::Svg,
        color_scheme: ColorScheme::RedWhiteBlue,
        show_absolute_values: false,
        group_by_category: true,
        include_legend: true,
        title: Some("Factor Exposure Analysis".to_string()),
    };
    
    // Generate visualization
    let heatmap_result = viz_service.generate_factor_exposure_heatmap(heatmap_request)?;
    
    // Save visualization to file
    let output_path = Path::new("factor_exposure_heatmap.svg");
    let mut file = File::create(output_path)?;
    file.write_all(&heatmap_result.data)?;
    
    println!("Saved factor exposure heatmap to {}", output_path.display());
    println!("\nExample completed successfully!");
    
    Ok(())
}

async fn create_sample_repository() -> Result<Arc<dyn FactorRepository>, Box<dyn std::error::Error>> {
    let repository = Arc::new(InMemoryFactorRepository::new());
    
    // Create factors
    let value_factor = Factor::new(
        "value".to_string(),
        "Value".to_string(),
        "Measures relative cheapness".to_string(),
        FactorCategory::Style,
    );
    
    let momentum_factor = Factor::new(
        "momentum".to_string(),
        "Momentum".to_string(),
        "Measures price trend".to_string(),
        FactorCategory::Style,
    );
    
    let quality_factor = Factor::new(
        "quality".to_string(),
        "Quality".to_string(),
        "Measures business profitability".to_string(),
        FactorCategory::Style,
    );
    
    let size_factor = Factor::new(
        "size".to_string(),
        "Size".to_string(),
        "Measures market capitalization".to_string(),
        FactorCategory::Style,
    );
    
    let volatility_factor = Factor::new(
        "volatility".to_string(),
        "Volatility".to_string(),
        "Measures price volatility".to_string(),
        FactorCategory::Custom("Risk".to_string()),
    );
    
    let growth_factor = Factor::new(
        "growth".to_string(),
        "Growth".to_string(),
        "Measures earnings growth".to_string(),
        FactorCategory::Style,
    );
    
    // Add factors to repository
    repository.create_factor(value_factor).await?;
    repository.create_factor(momentum_factor).await?;
    repository.create_factor(quality_factor).await?;
    repository.create_factor(size_factor).await?;
    repository.create_factor(volatility_factor).await?;
    repository.create_factor(growth_factor).await?;
    
    // Create factor model version
    let model_version = FactorModelVersion::new(
        "model-v1".to_string(),
        "US Equity Factor Model v1".to_string(),
        "Basic 6-factor model for US equities".to_string(),
        NaiveDate::from_ymd_opt(2023, 6, 30).unwrap(),
        None,
        FactorModelStatus::Active,
    );
    
    repository.create_factor_model_version(model_version).await?;
    
    // Create factor returns
    let as_of_date = NaiveDate::from_ymd_opt(2023, 6, 30).unwrap();
    
    let value_return = FactorReturn::new(
        "value".to_string(),
        0.02,
        as_of_date,
        as_of_date,
    );
    
    let momentum_return = FactorReturn::new(
        "momentum".to_string(),
        0.03,
        as_of_date,
        as_of_date,
    );
    
    let quality_return = FactorReturn::new(
        "quality".to_string(),
        0.01,
        as_of_date,
        as_of_date,
    );
    
    let size_return = FactorReturn::new(
        "size".to_string(),
        -0.01,
        as_of_date,
        as_of_date,
    );
    
    let volatility_return = FactorReturn::new(
        "volatility".to_string(),
        -0.02,
        as_of_date,
        as_of_date,
    );
    
    let growth_return = FactorReturn::new(
        "growth".to_string(),
        0.015,
        as_of_date,
        as_of_date,
    );
    
    repository.create_factor_return(value_return).await?;
    repository.create_factor_return(momentum_return).await?;
    repository.create_factor_return(quality_return).await?;
    repository.create_factor_return(size_return).await?;
    repository.create_factor_return(volatility_return).await?;
    repository.create_factor_return(growth_return).await?;
    
    // Create factor exposures for portfolio
    let portfolio_value_exposure = FactorExposure::new(
        "portfolio-123".to_string(),
        "value".to_string(),
        0.5,
        as_of_date,
    );
    
    let portfolio_momentum_exposure = FactorExposure::new(
        "portfolio-123".to_string(),
        "momentum".to_string(),
        0.3,
        as_of_date,
    );
    
    let portfolio_quality_exposure = FactorExposure::new(
        "portfolio-123".to_string(),
        "quality".to_string(),
        0.7,
        as_of_date,
    );
    
    let portfolio_size_exposure = FactorExposure::new(
        "portfolio-123".to_string(),
        "size".to_string(),
        -0.2,
        as_of_date,
    );
    
    let portfolio_volatility_exposure = FactorExposure::new(
        "portfolio-123".to_string(),
        "volatility".to_string(),
        -0.4,
        as_of_date,
    );
    
    let portfolio_growth_exposure = FactorExposure::new(
        "portfolio-123".to_string(),
        "growth".to_string(),
        0.1,
        as_of_date,
    );
    
    repository.create_factor_exposure(portfolio_value_exposure).await?;
    repository.create_factor_exposure(portfolio_momentum_exposure).await?;
    repository.create_factor_exposure(portfolio_quality_exposure).await?;
    repository.create_factor_exposure(portfolio_size_exposure).await?;
    repository.create_factor_exposure(portfolio_volatility_exposure).await?;
    repository.create_factor_exposure(portfolio_growth_exposure).await?;
    
    // Create factor exposures for benchmark
    let benchmark_value_exposure = FactorExposure::new(
        "benchmark-sp500".to_string(),
        "value".to_string(),
        0.0,
        as_of_date,
    );
    
    let benchmark_momentum_exposure = FactorExposure::new(
        "benchmark-sp500".to_string(),
        "momentum".to_string(),
        0.1,
        as_of_date,
    );
    
    let benchmark_quality_exposure = FactorExposure::new(
        "benchmark-sp500".to_string(),
        "quality".to_string(),
        0.2,
        as_of_date,
    );
    
    let benchmark_size_exposure = FactorExposure::new(
        "benchmark-sp500".to_string(),
        "size".to_string(),
        0.0,
        as_of_date,
    );
    
    let benchmark_volatility_exposure = FactorExposure::new(
        "benchmark-sp500".to_string(),
        "volatility".to_string(),
        0.0,
        as_of_date,
    );
    
    let benchmark_growth_exposure = FactorExposure::new(
        "benchmark-sp500".to_string(),
        "growth".to_string(),
        0.3,
        as_of_date,
    );
    
    repository.create_factor_exposure(benchmark_value_exposure).await?;
    repository.create_factor_exposure(benchmark_momentum_exposure).await?;
    repository.create_factor_exposure(benchmark_quality_exposure).await?;
    repository.create_factor_exposure(benchmark_size_exposure).await?;
    repository.create_factor_exposure(benchmark_volatility_exposure).await?;
    repository.create_factor_exposure(benchmark_growth_exposure).await?;
    
    Ok(repository)
} 