//! Visualization Module
//!
//! This module provides data visualization capabilities for the Performance Calculator,
//! including chart generation, report creation, and data export.

use anyhow::{Result, Context, anyhow};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::calculations::audit::AuditTrail;
use crate::calculations::distributed_cache::Cache;

/// Configuration for the Visualization module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    /// Whether visualization features are enabled
    pub enabled: bool,
    /// Maximum number of data points in a chart
    pub max_data_points: usize,
    /// Default chart width in pixels
    pub default_chart_width: u32,
    /// Default chart height in pixels
    pub default_chart_height: u32,
    /// Whether to cache visualization results
    pub enable_caching: bool,
    /// TTL for cached visualizations in seconds
    pub cache_ttl_seconds: u64,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_data_points: 1000,
            default_chart_width: 800,
            default_chart_height: 600,
            enable_caching: true,
            cache_ttl_seconds: 3600,
        }
    }
}

/// Chart type enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ChartType {
    /// Line chart
    Line,
    /// Bar chart
    Bar,
    /// Pie chart
    Pie,
    /// Area chart
    Area,
    /// Scatter plot
    Scatter,
    /// Candlestick chart
    Candlestick,
    /// Heatmap
    Heatmap,
}

/// Chart data series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSeries {
    /// Series name
    pub name: String,
    /// Series data (x-value -> y-value)
    pub data: Vec<(String, Decimal)>,
    /// Series color (hex code)
    pub color: Option<String>,
    /// Series type (overrides chart type if specified)
    pub series_type: Option<ChartType>,
}

/// Chart options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartOptions {
    /// Chart title
    pub title: String,
    /// Chart subtitle
    pub subtitle: Option<String>,
    /// Chart type
    pub chart_type: ChartType,
    /// Chart width in pixels
    pub width: u32,
    /// Chart height in pixels
    pub height: u32,
    /// X-axis title
    pub x_axis_title: Option<String>,
    /// Y-axis title
    pub y_axis_title: Option<String>,
    /// Whether to show legend
    pub show_legend: bool,
    /// Whether to show tooltips
    pub show_tooltips: bool,
    /// Whether to enable zooming
    pub enable_zoom: bool,
    /// Whether to stack series (for bar and area charts)
    pub stacked: bool,
    /// Custom colors for the chart
    pub colors: Option<Vec<String>>,
}

impl Default for ChartOptions {
    fn default() -> Self {
        Self {
            title: "Chart".to_string(),
            subtitle: None,
            chart_type: ChartType::Line,
            width: 800,
            height: 600,
            x_axis_title: None,
            y_axis_title: None,
            show_legend: true,
            show_tooltips: true,
            enable_zoom: false,
            stacked: false,
            colors: None,
        }
    }
}

/// Chart definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDefinition {
    /// Chart options
    pub options: ChartOptions,
    /// Chart series
    pub series: Vec<ChartSeries>,
}

/// Chart output format
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ChartFormat {
    /// SVG format
    SVG,
    /// PNG format
    PNG,
    /// JSON format (for client-side rendering)
    JSON,
}

/// Chart result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartResult {
    /// Chart ID
    pub id: String,
    /// Chart definition
    pub definition: ChartDefinition,
    /// Chart data in the specified format
    pub data: String,
    /// Chart format
    pub format: ChartFormat,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Report template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplate {
    /// Template ID
    pub id: String,
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Template content (HTML/Markdown with placeholders)
    pub content: String,
    /// Charts included in the template
    pub charts: Vec<ChartDefinition>,
    /// Tables included in the template
    pub tables: Vec<TableDefinition>,
}

/// Table column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    /// Column ID
    pub id: String,
    /// Column title
    pub title: String,
    /// Column data type
    pub data_type: String,
    /// Column format (e.g., "0.00%", "MM/DD/YYYY")
    pub format: Option<String>,
    /// Column width
    pub width: Option<String>,
    /// Whether the column is sortable
    pub sortable: bool,
    /// Whether the column is filterable
    pub filterable: bool,
}

/// Table definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDefinition {
    /// Table ID
    pub id: String,
    /// Table title
    pub title: String,
    /// Table columns
    pub columns: Vec<TableColumn>,
    /// Table data (rows)
    pub data: Vec<HashMap<String, serde_json::Value>>,
    /// Default sort column
    pub default_sort: Option<String>,
    /// Default sort direction (true for ascending, false for descending)
    pub default_sort_ascending: Option<bool>,
    /// Whether to show pagination
    pub paginated: bool,
    /// Page size if paginated
    pub page_size: Option<usize>,
}

/// Report output format
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ReportFormat {
    /// HTML format
    HTML,
    /// PDF format
    PDF,
    /// Excel format
    Excel,
    /// CSV format
    CSV,
    /// JSON format
    JSON,
}

/// Report result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportResult {
    /// Report ID
    pub id: String,
    /// Report name
    pub name: String,
    /// Report data in the specified format
    pub data: Vec<u8>,
    /// Report format
    pub format: ReportFormat,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Visualization engine for generating charts and reports
pub struct VisualizationEngine {
    /// Configuration
    config: VisualizationConfig,
    /// Cache for visualization results
    cache: Arc<dyn Cache + Send + Sync>,
    /// Audit trail
    audit_trail: Arc<dyn AuditTrail + Send + Sync>,
    /// Report templates
    templates: HashMap<String, ReportTemplate>,
}

impl VisualizationEngine {
    /// Create a new visualization engine
    pub fn new(
        config: VisualizationConfig,
        cache: Arc<dyn Cache + Send + Sync>,
        audit_trail: Arc<dyn AuditTrail + Send + Sync>,
    ) -> Self {
        Self {
            config,
            cache,
            audit_trail,
            templates: HashMap::new(),
        }
    }
    
    /// Register a report template
    pub fn register_template(&mut self, template: ReportTemplate) -> Result<()> {
        self.templates.insert(template.id.clone(), template);
        Ok(())
    }
    
    /// Get available report templates
    pub fn get_templates(&self) -> Vec<ReportTemplate> {
        self.templates.values().cloned().collect()
    }
    
    /// Generate a chart
    pub async fn generate_chart(
        &self,
        definition: ChartDefinition,
        format: ChartFormat,
        request_id: &str,
    ) -> Result<ChartResult> {
        // Check if visualization is enabled
        if !self.config.enabled {
            return Err(anyhow!("Visualization module is not enabled"));
        }
        
        // Validate chart definition
        if definition.series.is_empty() {
            return Err(anyhow!("Chart must have at least one data series"));
        }
        
        // Limit data points if necessary
        let mut limited_definition = definition.clone();
        for series in &mut limited_definition.series {
            if series.data.len() > self.config.max_data_points {
                // Downsample data points
                series.data = downsample_data(&series.data, self.config.max_data_points);
            }
        }
        
        // Create cache key
        let cache_key = format!(
            "visualization:chart:{}:{}",
            serde_json::to_string(&limited_definition)?,
            format!("{:?}", format)
        );
        
        // Try to get from cache
        if self.config.enable_caching {
            if let Some(cached_result) = self.cache.get::<ChartResult>(&cache_key).await? {
                // Record cache hit in audit trail
                self.audit_trail.record(crate::calculations::audit::AuditRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    entity_id: "chart".to_string(),
                    entity_type: "visualization".to_string(),
                    action: "chart_generation_cache_hit".to_string(),
                    user_id: "system".to_string(),
                    parameters: format!("chart_type={:?},format={:?}", limited_definition.options.chart_type, format),
                    result: format!("cached_result_found"),
                }).await?;
                
                return Ok(cached_result);
            }
        }
        
        // Generate chart data based on format
        let chart_data = match format {
            ChartFormat::SVG => generate_svg_chart(&limited_definition)?,
            ChartFormat::PNG => generate_png_chart(&limited_definition)?,
            ChartFormat::JSON => generate_json_chart(&limited_definition)?,
        };
        
        // Cache the result
        if self.config.enable_caching {
            self.cache.set(&cache_key, &chart_data, Some(self.config.cache_ttl_seconds)).await?;
        }
        
        // Record in audit trail
        self.audit_trail.record(crate::calculations::audit::AuditRecord {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            entity_id: "chart".to_string(),
            entity_type: "visualization".to_string(),
            action: "chart_generation".to_string(),
            user_id: "system".to_string(),
            parameters: format!(
                "chart_type={:?},format={:?}",
                limited_definition.options.chart_type,
                format
            ),
            result: format!("chart_generated"),
        }).await?;
        
        Ok(ChartResult {
            id: uuid::Uuid::new_v4().to_string(),
            definition: limited_definition,
            data: chart_data,
            format,
            created_at: chrono::Utc::now(),
        })
    }

    /// Generate a report from a template
    pub async fn generate_report(
        &self,
        template_id: &str,
        parameters: HashMap<String, serde_json::Value>,
        format: ReportFormat,
        request_id: &str,
    ) -> Result<ReportResult> {
        // Check if visualization is enabled
        if !self.config.enabled {
            return Err(anyhow!("Visualization module is not enabled"));
        }
        
        // Get template
        let template = self.templates.get(template_id)
            .ok_or_else(|| anyhow!("Template not found: {}", template_id))?
            .clone();
        
        // Create cache key
        let cache_key = format!(
            "visualization:report:{}:{}:{}",
            template_id,
            serde_json::to_string(&parameters)?,
            format!("{:?}", format)
        );
        
        // Try to get from cache
        if self.config.enable_caching {
            if let Some(cached_result) = self.cache.get::<ReportResult>(&cache_key).await? {
                // Record cache hit in audit trail
                self.audit_trail.record(crate::calculations::audit::AuditRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    entity_id: template_id.to_string(),
                    entity_type: "report_template".to_string(),
                    action: "report_generation_cache_hit".to_string(),
                    user_id: "system".to_string(),
                    parameters: format!("template_id={},format={:?}", template_id, format),
                    result: format!("cached_result_found"),
                }).await?;
                
                return Ok(cached_result);
            }
        }
        
        // Generate charts for the report
        let mut charts = HashMap::new();
        for chart_def in &template.charts {
            // Apply parameters to chart definition
            let processed_def = apply_parameters_to_chart(chart_def, &parameters)?;
            
            // Generate chart
            let chart = self.generate_chart(
                processed_def,
                ChartFormat::SVG, // Use SVG for reports
                request_id,
            ).await?;
            
            charts.insert(chart.id.clone(), chart);
        }
        
        // Generate tables for the report
        let tables = template.tables.clone();
        
        // Apply parameters to template content
        let content = apply_parameters_to_content(&template.content, &parameters, &charts, &tables)?;
        
        // Generate report in the requested format
        let report_data = match format {
            ReportFormat::HTML => generate_html_report(&content, &charts, &tables)?,
            ReportFormat::PDF => generate_pdf_report(&content, &charts, &tables)?,
            ReportFormat::Excel => generate_excel_report(&tables)?,
            ReportFormat::CSV => generate_csv_report(&tables)?,
            ReportFormat::JSON => generate_json_report(&parameters, &charts, &tables)?,
        };
        
        let result = ReportResult {
            id: uuid::Uuid::new_v4().to_string(),
            name: template.name.clone(),
            data: report_data,
            format,
            created_at: chrono::Utc::now(),
        };
        
        // Cache the result
        if self.config.enable_caching {
            self.cache.set(&cache_key, &result, Some(self.config.cache_ttl_seconds)).await?;
        }
        
        // Record in audit trail
        self.audit_trail.record(crate::calculations::audit::AuditRecord {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            entity_id: template_id.to_string(),
            entity_type: "report_template".to_string(),
            action: "report_generation".to_string(),
            user_id: "system".to_string(),
            parameters: format!(
                "template_id={},format={:?}",
                template_id,
                format
            ),
            result: format!("report_generated"),
        }).await?;
        
        Ok(result)
    }
    
    /// Export data in various formats
    pub async fn export_data(
        &self,
        data: Vec<HashMap<String, serde_json::Value>>,
        format: ReportFormat,
        request_id: &str,
    ) -> Result<Vec<u8>> {
        // Check if visualization is enabled
        if !self.config.enabled {
            return Err(anyhow!("Visualization module is not enabled"));
        }
        
        // Generate export in the requested format
        let export_data = match format {
            ReportFormat::Excel => generate_excel_export(&data)?,
            ReportFormat::CSV => generate_csv_export(&data)?,
            ReportFormat::JSON => serde_json::to_vec(&data)
                .map_err(|e| anyhow!("Failed to serialize data: {}", e))?,
            _ => return Err(anyhow!("Unsupported export format: {:?}", format)),
        };
        
        // Record in audit trail
        self.audit_trail.record(crate::calculations::audit::AuditRecord {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            entity_id: "data_export".to_string(),
            entity_type: "export".to_string(),
            action: "data_export".to_string(),
            user_id: "system".to_string(),
            parameters: format!("format={:?},row_count={}", format, data.len()),
            result: format!("export_generated"),
        }).await?;
        
        Ok(export_data)
    }
}

/// Downsample data points to reduce chart size
fn downsample_data(data: &[(String, Decimal)], max_points: usize) -> Vec<(String, Decimal)> {
    if data.len() <= max_points {
        return data.to_vec();
    }
    
    // Simple downsampling by taking evenly spaced points
    let step = data.len() / max_points;
    let mut result = Vec::with_capacity(max_points);
    
    // Always include first and last points
    result.push(data[0].clone());
    
    for i in (step..data.len() - step).step_by(step) {
        result.push(data[i].clone());
    }
    
    result.push(data[data.len() - 1].clone());
    
    result
}

/// Generate SVG chart
fn generate_svg_chart(definition: &ChartDefinition) -> Result<String> {
    // In a real implementation, this would use a charting library
    // For demonstration, we'll create a simple SVG
    
    let width = definition.options.width;
    let height = definition.options.height;
    
    let mut svg = format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
        width, height
    );
    
    // Add title
    svg.push_str(&format!(
        r#"<text x="{}" y="20" text-anchor="middle" font-family="Arial" font-size="16" font-weight="bold">{}</text>"#,
        width / 2, definition.options.title
    ));
    
    // Add subtitle if present
    if let Some(subtitle) = &definition.options.subtitle {
        svg.push_str(&format!(
            r#"<text x="{}" y="40" text-anchor="middle" font-family="Arial" font-size="12">{}</text>"#,
            width / 2, subtitle
        ));
    }
    
    // In a real implementation, we would draw axes, data series, etc.
    svg.push_str("<!-- Chart content would be generated here -->");
    
    // Close SVG
    svg.push_str("</svg>");
    
    Ok(svg)
}

/// Generate PNG chart
fn generate_png_chart(definition: &ChartDefinition) -> Result<String> {
    // In a real implementation, this would use a charting library to generate PNG
    // For demonstration, we'll return a placeholder
    
    // Generate SVG first
    let svg = generate_svg_chart(definition)?;
    
    // In a real implementation, we would convert SVG to PNG
    // For demonstration, we'll return base64-encoded placeholder
    Ok("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==".to_string())
}

/// Generate JSON chart data for client-side rendering
fn generate_json_chart(definition: &ChartDefinition) -> Result<String> {
    // Convert chart definition to JSON for client-side rendering
    serde_json::to_string(definition).map_err(|e| anyhow!("Failed to serialize chart: {}", e))
}

/// Apply parameters to chart definition
fn apply_parameters_to_chart(
    chart: &ChartDefinition,
    parameters: &HashMap<String, serde_json::Value>,
) -> Result<ChartDefinition> {
    // In a real implementation, this would replace placeholders in the chart definition
    // For demonstration, we'll return the original chart
    Ok(chart.clone())
}

/// Apply parameters to template content
fn apply_parameters_to_content(
    content: &str,
    parameters: &HashMap<String, serde_json::Value>,
    charts: &HashMap<String, ChartResult>,
    tables: &[TableDefinition],
) -> Result<String> {
    // In a real implementation, this would replace placeholders in the content
    // For demonstration, we'll return the original content
    Ok(content.to_string())
}

/// Generate HTML report
fn generate_html_report(
    content: &str,
    charts: &HashMap<String, ChartResult>,
    tables: &[TableDefinition],
) -> Result<Vec<u8>> {
    // In a real implementation, this would generate a complete HTML document
    // For demonstration, we'll create a simple HTML structure
    
    let mut html = String::from("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str("<title>Performance Report</title>\n");
    html.push_str("<style>\n");
    html.push_str("body { font-family: Arial, sans-serif; margin: 20px; }\n");
    html.push_str("table { border-collapse: collapse; width: 100%; }\n");
    html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
    html.push_str("th { background-color: #f2f2f2; }\n");
    html.push_str("</style>\n");
    html.push_str("</head>\n<body>\n");
    
    // Add content
    html.push_str(content);
    
    // Add charts
    for (id, chart) in charts {
        if chart.format == ChartFormat::SVG {
            html.push_str(&format!("<div id=\"chart-{}\">\n", id));
            html.push_str(&chart.data);
            html.push_str("</div>\n");
        }
    }
    
    // Add tables
    for table in tables {
        html.push_str(&format!("<h3>{}</h3>\n", table.title));
        html.push_str("<table>\n<thead>\n<tr>\n");
        
        // Table headers
        for column in &table.columns {
            html.push_str(&format!("<th>{}</th>\n", column.title));
        }
        
        html.push_str("</tr>\n</thead>\n<tbody>\n");
        
        // Table rows
        for row in &table.data {
            html.push_str("<tr>\n");
            
            for column in &table.columns {
                let value = row.get(&column.id)
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                
                html.push_str(&format!("<td>{}</td>\n", value));
            }
            
            html.push_str("</tr>\n");
        }
        
        html.push_str("</tbody>\n</table>\n");
    }
    
    html.push_str("</body>\n</html>");
    
    Ok(html.into_bytes())
}

/// Generate PDF report
fn generate_pdf_report(
    content: &str,
    charts: &HashMap<String, ChartResult>,
    tables: &[TableDefinition],
) -> Result<Vec<u8>> {
    // In a real implementation, this would use a PDF generation library
    // For demonstration, we'll return a placeholder
    
    // Generate HTML first
    let html = generate_html_report(content, charts, tables)?;
    
    // In a real implementation, we would convert HTML to PDF
    // For demonstration, we'll return the HTML bytes
    Ok(html)
}

/// Generate Excel report
fn generate_excel_report(tables: &[TableDefinition]) -> Result<Vec<u8>> {
    // In a real implementation, this would use an Excel generation library
    // For demonstration, we'll return a placeholder
    
    // Create a simple CSV as a placeholder
    let mut csv = String::new();
    
    for table in tables {
        csv.push_str(&format!("{}\n", table.title));
        
        // Headers
        let headers: Vec<String> = table.columns.iter()
            .map(|c| c.title.clone())
            .collect();
        
        csv.push_str(&headers.join(","));
        csv.push('\n');
        
        // Rows
        for row in &table.data {
            let values: Vec<String> = table.columns.iter()
                .map(|c| row.get(&c.id)
                    .map(|v| v.to_string())
                    .unwrap_or_default())
                .collect();
            
            csv.push_str(&values.join(","));
            csv.push('\n');
        }
        
        csv.push('\n');
    }
    
    Ok(csv.into_bytes())
}

/// Generate CSV report
fn generate_csv_report(tables: &[TableDefinition]) -> Result<Vec<u8>> {
    // Similar to Excel but simpler
    generate_excel_report(tables)
}

/// Generate JSON report
fn generate_json_report(
    parameters: &HashMap<String, serde_json::Value>,
    charts: &HashMap<String, ChartResult>,
    tables: &[TableDefinition],
) -> Result<Vec<u8>> {
    // Create a JSON structure with all report components
    let report = serde_json::json!({
        "parameters": parameters,
        "charts": charts,
        "tables": tables,
    });
    
    serde_json::to_vec(&report)
        .map_err(|e| anyhow!("Failed to serialize report: {}", e))
}

/// Generate Excel export
fn generate_excel_export(data: &[HashMap<String, serde_json::Value>]) -> Result<Vec<u8>> {
    // In a real implementation, this would use an Excel generation library
    // For demonstration, we'll create a simple CSV
    
    if data.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut csv = String::new();
    
    // Headers
    let headers: Vec<String> = data[0].keys().cloned().collect();
    csv.push_str(&headers.join(","));
    csv.push('\n');
    
    // Rows
    for row in data {
        let values: Vec<String> = headers.iter()
            .map(|h| row.get(h)
                .map(|v| v.to_string())
                .unwrap_or_default())
            .collect();
        
        csv.push_str(&values.join(","));
        csv.push('\n');
    }
    
    Ok(csv.into_bytes())
}

/// Generate CSV export
fn generate_csv_export(data: &[HashMap<String, serde_json::Value>]) -> Result<Vec<u8>> {
    // Similar to Excel but simpler
    generate_excel_export(data)
} 