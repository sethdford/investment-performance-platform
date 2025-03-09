use chrono::NaiveDate;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

// Mock types to demonstrate the API structure
#[derive(Debug, Clone)]
struct FactorExposureRequest {
    portfolio_id: String,
    benchmark_id: Option<String>,
    as_of_date: NaiveDate,
}

#[derive(Debug, Clone)]
struct FactorExposureResponse {
    portfolio_exposures: HashMap<String, f64>,
    benchmark_exposures: Option<HashMap<String, f64>>,
    active_exposures: Option<HashMap<String, f64>>,
    as_of_date: NaiveDate,
}

#[derive(Debug, Clone)]
struct FactorExposureHeatmapRequest {
    exposure_data: FactorExposureResponse,
    format: VisualizationFormat,
    title: Option<String>,
}

#[derive(Debug, Clone)]
struct FactorExposureHeatmapResponse {
    data: String,
    format: VisualizationFormat,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Copy)]
enum VisualizationFormat {
    Svg,
    Json,
    Html,
}

// Mock API service
struct MockFactorModelApiService;

impl MockFactorModelApiService {
    fn new() -> Self {
        Self
    }

    fn analyze_factor_exposures(&self, request: FactorExposureRequest) -> FactorExposureResponse {
        // Create mock portfolio exposures
        let mut portfolio_exposures = HashMap::new();
        portfolio_exposures.insert("value".to_string(), 0.5);
        portfolio_exposures.insert("momentum".to_string(), 0.3);
        portfolio_exposures.insert("quality".to_string(), 0.7);
        portfolio_exposures.insert("size".to_string(), -0.2);
        portfolio_exposures.insert("volatility".to_string(), -0.4);
        portfolio_exposures.insert("growth".to_string(), 0.1);

        // Create mock benchmark exposures if requested
        let benchmark_exposures = if request.benchmark_id.is_some() {
            let mut exposures = HashMap::new();
            exposures.insert("value".to_string(), 0.0);
            exposures.insert("momentum".to_string(), 0.1);
            exposures.insert("quality".to_string(), 0.2);
            exposures.insert("size".to_string(), 0.0);
            exposures.insert("volatility".to_string(), 0.0);
            exposures.insert("growth".to_string(), 0.3);
            Some(exposures)
        } else {
            None
        };

        // Calculate active exposures if benchmark is provided
        let active_exposures = if let Some(benchmark) = &benchmark_exposures {
            let mut active = HashMap::new();
            for (factor, exposure) in &portfolio_exposures {
                let benchmark_exposure = benchmark.get(factor).unwrap_or(&0.0);
                active.insert(factor.clone(), exposure - benchmark_exposure);
            }
            Some(active)
        } else {
            None
        };

        FactorExposureResponse {
            portfolio_exposures,
            benchmark_exposures,
            active_exposures,
            as_of_date: request.as_of_date,
        }
    }
}

// Mock visualization service
struct MockFactorModelVisualizationService;

impl MockFactorModelVisualizationService {
    fn new() -> Self {
        Self
    }

    fn generate_factor_exposure_heatmap(&self, request: FactorExposureHeatmapRequest) -> FactorExposureHeatmapResponse {
        // Generate a simple SVG heatmap
        let title = request.title.unwrap_or_else(|| "Factor Exposure Heatmap".to_string());
        
        let mut svg = format!(
            r#"<svg width="800" height="600" xmlns="http://www.w3.org/2000/svg">
                <style>
                    .title {{ font-size: 24px; font-family: Arial; text-anchor: middle; }}
                    .factor {{ font-size: 14px; font-family: Arial; text-anchor: end; }}
                    .value {{ font-size: 14px; font-family: Arial; text-anchor: middle; }}
                </style>
                <text x="400" y="30" class="title">{}</text>
            "#,
            title
        );

        // Add factor exposures to the heatmap
        let factors: Vec<&String> = request.exposure_data.portfolio_exposures.keys().collect();
        let factor_count = factors.len();
        
        for (i, factor) in factors.iter().enumerate() {
            let y = 80 + i * 40;
            let exposure = request.exposure_data.portfolio_exposures.get(*factor).unwrap();
            
            // Calculate color based on exposure value
            let color = if *exposure > 0.0 {
                let intensity = (*exposure * 255.0).min(255.0) as u8;
                format!("rgb(255,{},{})", 255 - intensity, 255 - intensity)
            } else {
                let intensity = (exposure.abs() * 255.0).min(255.0) as u8;
                format!("rgb({},{},255)", 255 - intensity, 255 - intensity)
            };
            
            // Add factor name and rectangle
            svg.push_str(&format!(
                r#"<text x="150" y="{}" class="factor">{}</text>
                   <rect x="180" y="{}" width="100" height="30" fill="{}" stroke="black" />
                   <text x="230" y="{}" class="value">{:.2}</text>
                "#,
                y + 20, factor, y, color, y + 20, exposure
            ));
        }
        
        // Close SVG
        svg.push_str("</svg>");
        
        FactorExposureHeatmapResponse {
            data: svg,
            format: request.format,
            width: 800,
            height: 600,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Factor Model API Mock Example");

    // Create API service
    let api_service = MockFactorModelApiService::new();
    
    // Create visualization service
    let viz_service = MockFactorModelVisualizationService::new();

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
    };

    // Call API to analyze factor exposures
    let exposure_result = api_service.analyze_factor_exposures(exposure_request);
    
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
        title: Some("Factor Exposure Analysis".to_string()),
    };
    
    // Generate visualization
    let heatmap_result = viz_service.generate_factor_exposure_heatmap(heatmap_request);
    
    // Save visualization to file
    let output_path = Path::new("factor_exposure_heatmap.svg");
    let mut file = File::create(output_path)?;
    file.write_all(heatmap_result.data.as_bytes())?;
    
    println!("Saved factor exposure heatmap to {}", output_path.display());
    println!("\nExample completed successfully!");
    
    Ok(())
} 