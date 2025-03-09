//! Analytics and visualization for the Investment Management Platform
//!
//! This module provides analytics and visualization capabilities for the platform.

use ndarray::Array2;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub enum VisualizationFormat {
    Svg,
    Json,
    Html,
}

pub struct FactorExposureHeatmapRequest {
    pub portfolio_exposures: HashMap<String, f64>,
    pub benchmark_exposures: Option<HashMap<String, f64>>,
    pub active_exposures: Option<HashMap<String, f64>>,
    pub format: VisualizationFormat,
    pub title: Option<String>,
}

pub struct FactorExposureHeatmapResponse {
    pub data: String,
    pub format: VisualizationFormat,
    pub width: u32,
    pub height: u32,
}

pub struct VisualizationService;

impl VisualizationService {
    pub fn new() -> Self {
        Self
    }
    
    pub fn generate_factor_exposure_heatmap(&self, request: FactorExposureHeatmapRequest) -> FactorExposureHeatmapResponse {
        // Generate a simple SVG heatmap
        let title = request.title.unwrap_or_else(|| "Factor Exposure Heatmap".to_string());
        
        let mut svg = format!(
            r#"<svg width="800" height="600" xmlns="http://www.w3.org/2000/svg">
                <style>
                    .title {{ font-size: 24px; font-family: Arial; text-anchor: middle; }}
                    .factor {{ font-size: 14px; font-family: Arial; text-anchor: end; }}
                    .value {{ font-size: 14px; font-family: Arial; text-anchor: middle; }}
                    .label {{ font-size: 16px; font-family: Arial; text-anchor: middle; }}
                </style>
                <text x="400" y="30" class="title">{}</text>
                <text x="230" y="60" class="label">Portfolio</text>
            "#,
            title
        );
        
        // Add benchmark label if benchmark exposures are provided
        if request.benchmark_exposures.is_some() {
            svg.push_str(r#"<text x="380" y="60" class="label">Benchmark</text>"#);
            svg.push_str(r#"<text x="530" y="60" class="label">Active</text>"#);
        }
        
        // Add factor exposures to the heatmap
        let factors: Vec<&String> = request.portfolio_exposures.keys().collect();
        
        for (i, factor) in factors.iter().enumerate() {
            let y = 100 + i * 40;
            let portfolio_exposure = request.portfolio_exposures.get(*factor).unwrap();
            
            // Calculate color based on exposure value
            let portfolio_color = if *portfolio_exposure > 0.0 {
                let intensity = (*portfolio_exposure * 255.0).min(255.0) as u8;
                format!("rgb(255,{},{})", 255 - intensity, 255 - intensity)
            } else {
                let intensity = (portfolio_exposure.abs() * 255.0).min(255.0) as u8;
                format!("rgb({},{},255)", 255 - intensity, 255 - intensity)
            };
            
            // Add factor name and portfolio rectangle
            svg.push_str(&format!(
                r#"<text x="150" y="{}" class="factor">{}</text>
                   <rect x="180" y="{}" width="100" height="30" fill="{}" stroke="black" />
                   <text x="230" y="{}" class="value">{:.2}</text>
                "#,
                y + 20, factor, y, portfolio_color, y + 20, portfolio_exposure
            ));
            
            // Add benchmark and active exposures if provided
            if let Some(benchmark_exposures) = &request.benchmark_exposures {
                if let Some(benchmark_exposure) = benchmark_exposures.get(*factor) {
                    // Calculate color for benchmark exposure
                    let benchmark_color = if *benchmark_exposure > 0.0 {
                        let intensity = (*benchmark_exposure * 255.0).min(255.0) as u8;
                        format!("rgb(255,{},{})", 255 - intensity, 255 - intensity)
                    } else {
                        let intensity = (benchmark_exposure.abs() * 255.0).min(255.0) as u8;
                        format!("rgb({},{},255)", 255 - intensity, 255 - intensity)
                    };
                    
                    // Add benchmark rectangle
                    svg.push_str(&format!(
                        r#"<rect x="330" y="{}" width="100" height="30" fill="{}" stroke="black" />
                           <text x="380" y="{}" class="value">{:.2}</text>
                        "#,
                        y, benchmark_color, y + 20, benchmark_exposure
                    ));
                    
                    // Add active exposure if provided
                    if let Some(active_exposures) = &request.active_exposures {
                        if let Some(active_exposure) = active_exposures.get(*factor) {
                            // Calculate color for active exposure
                            let active_color = if *active_exposure > 0.0 {
                                let intensity = (*active_exposure * 255.0).min(255.0) as u8;
                                format!("rgb(255,{},{})", 255 - intensity, 255 - intensity)
                            } else {
                                let intensity = (active_exposure.abs() * 255.0).min(255.0) as u8;
                                format!("rgb({},{},255)", 255 - intensity, 255 - intensity)
                            };
                            
                            // Add active rectangle
                            svg.push_str(&format!(
                                r#"<rect x="480" y="{}" width="100" height="30" fill="{}" stroke="black" />
                                   <text x="530" y="{}" class="value">{:.2}</text>
                                "#,
                                y, active_color, y + 20, active_exposure
                            ));
                        }
                    }
                }
            }
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
    
    pub fn save_visualization(&self, response: &FactorExposureHeatmapResponse, filename: &str) -> Result<(), std::io::Error> {
        let path = Path::new(filename);
        let mut file = File::create(path)?;
        file.write_all(response.data.as_bytes())?;
        Ok(())
    }
}

/// Generate a heatmap SVG for factor exposures
///
/// # Arguments
///
/// * `factor_exposures` - A 2D array of factor exposures
/// * `factor_names` - Names of the factors
/// * `security_names` - Names of the securities
/// * `output_file` - Path to the output SVG file
///
/// # Returns
///
/// * `Result<(), std::io::Error>` - Result of the operation
pub fn generate_factor_exposure_heatmap(
    factor_exposures: &Array2<f64>,
    factor_names: &[String],
    security_names: &[String],
    output_file: &str,
) -> Result<(), std::io::Error> {
    let (n_securities, n_factors) = factor_exposures.dim();
    
    // Validate dimensions
    if factor_names.len() != n_factors {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Number of factor names does not match number of factors",
        ));
    }
    
    if security_names.len() != n_securities {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Number of security names does not match number of securities",
        ));
    }
    
    // SVG parameters
    let cell_width = 60;
    let cell_height = 30;
    let header_height = 40;
    let header_width = 150;
    let svg_width = header_width + n_factors * cell_width;
    let svg_height = header_height + n_securities * cell_height;
    
    // Create SVG content
    let mut svg = format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
        svg_width, svg_height
    );
    
    // Add factor names (column headers)
    for (i, factor_name) in factor_names.iter().enumerate() {
        let x = header_width + i * cell_width + cell_width / 2;
        let y = header_height / 2;
        svg.push_str(&format!(
            r#"<text x="{}" y="{}" font-family="Arial" font-size="12" text-anchor="middle">{}</text>"#,
            x, y, factor_name
        ));
    }
    
    // Add security names (row headers)
    for (i, security_name) in security_names.iter().enumerate() {
        let x = header_width / 2;
        let y = header_height + i * cell_height + cell_height / 2;
        svg.push_str(&format!(
            r#"<text x="{}" y="{}" font-family="Arial" font-size="12" text-anchor="middle">{}</text>"#,
            x, y, security_name
        ));
    }
    
    // Add heatmap cells
    for i in 0..n_securities {
        for j in 0..n_factors {
            let value = factor_exposures[[i, j]];
            let color = get_color_for_value(value);
            let text_color = if value.abs() > 0.5 { "white" } else { "black" };
            
            let x = header_width + j * cell_width;
            let y = header_height + i * cell_height;
            
            svg.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" />"#,
                x, y, cell_width, cell_height, color
            ));
            
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" font-family="Arial" font-size="12" text-anchor="middle" fill="{}">{:.2}</text>"#,
                x + cell_width / 2,
                y + cell_height / 2,
                text_color,
                value
            ));
        }
    }
    
    svg.push_str("</svg>");
    
    // Write to file
    let mut file = File::create(output_file)?;
    file.write_all(svg.as_bytes())?;
    
    Ok(())
}

/// Get a color for a value in the heatmap
///
/// # Arguments
///
/// * `value` - The value to get a color for
///
/// # Returns
///
/// * `String` - The color as a hex string
fn get_color_for_value(value: f64) -> String {
    let normalized = (value + 1.0) / 2.0; // Map from [-1, 1] to [0, 1]
    let normalized = normalized.max(0.0).min(1.0); // Clamp to [0, 1]
    
    if normalized < 0.5 {
        // Blue to white gradient for negative values
        let intensity = (normalized * 2.0 * 255.0) as u8;
        format!("#{:02x}{:02x}ff", intensity, intensity)
    } else {
        // White to red gradient for positive values
        let intensity = ((1.0 - (normalized - 0.5) * 2.0) * 255.0) as u8;
        format!("ff{:02x}{:02x}", intensity, intensity)
    }
} 