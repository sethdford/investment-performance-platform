use crate::error::AppError;
use crate::models::TimeSeries;
use crate::repository::timestream::TimestreamRepository;
use crate::repository::dynamodb::DynamoDbRepository;
use crate::repository::Repository;
use chrono::NaiveDate;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

/// Supported chart formats
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChartFormat {
    Png,
    Jpg,
    Svg,
    Pdf,
}

impl Default for ChartFormat {
    fn default() -> Self {
        ChartFormat::Png
    }
}

impl ChartFormat {
    /// Get the MIME type for the chart format
    pub fn mime_type(&self) -> &'static str {
        match self {
            ChartFormat::Png => "image/png",
            ChartFormat::Jpg => "image/jpeg",
            ChartFormat::Svg => "image/svg+xml",
            ChartFormat::Pdf => "application/pdf",
        }
    }

    /// Get the file extension for the chart format
    pub fn extension(&self) -> &'static str {
        match self {
            ChartFormat::Png => "png",
            ChartFormat::Jpg => "jpg",
            ChartFormat::Svg => "svg",
            ChartFormat::Pdf => "pdf",
        }
    }
}

/// Time interval for data points
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TimeInterval {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

impl Default for TimeInterval {
    fn default() -> Self {
        TimeInterval::Monthly
    }
}

/// Chart type for allocation charts
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AllocationChartType {
    Pie,
    Donut,
    Bar,
    Treemap,
}

impl Default for AllocationChartType {
    fn default() -> Self {
        AllocationChartType::Pie
    }
}

/// Chart type for attribution charts
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AttributionChartType {
    Bar,
    StackedBar,
    Waterfall,
}

impl Default for AttributionChartType {
    fn default() -> Self {
        AttributionChartType::Bar
    }
}

/// Parameters for generating a performance chart
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerformanceChartParams {
    pub portfolio_id: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub benchmark_id: Option<String>,
    #[serde(default = "default_metrics")]
    pub metrics: Vec<String>,
    #[serde(default)]
    pub interval: TimeInterval,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default)]
    pub format: ChartFormat,
}

/// Parameters for generating a comparison chart
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ComparisonChartParams {
    pub portfolio_ids: Vec<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub benchmark_id: Option<String>,
    #[serde(default = "default_metric")]
    pub metric: String,
    #[serde(default)]
    pub interval: TimeInterval,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default)]
    pub format: ChartFormat,
}

/// Parameters for generating a risk-return chart
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RiskReturnChartParams {
    pub portfolio_ids: Vec<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub benchmark_id: Option<String>,
    #[serde(default = "default_return_metric")]
    pub return_metric: String,
    #[serde(default = "default_risk_metric")]
    pub risk_metric: String,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default)]
    pub format: ChartFormat,
}

/// Parameters for generating an allocation chart
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AllocationChartParams {
    pub portfolio_id: String,
    #[serde(default = "default_date")]
    pub date: NaiveDate,
    #[serde(default = "default_group_by")]
    pub group_by: String,
    #[serde(default)]
    pub chart_type: AllocationChartType,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default)]
    pub format: ChartFormat,
}

/// Parameters for generating an attribution chart
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AttributionChartParams {
    pub portfolio_id: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    #[serde(default = "default_group_by")]
    pub group_by: String,
    #[serde(default)]
    pub chart_type: AttributionChartType,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default)]
    pub format: ChartFormat,
}

// Default functions for parameter defaults
fn default_metrics() -> Vec<String> {
    vec!["twr".to_string()]
}

fn default_metric() -> String {
    "twr".to_string()
}

fn default_return_metric() -> String {
    "twr".to_string()
}

fn default_risk_metric() -> String {
    "volatility".to_string()
}

fn default_group_by() -> String {
    "type".to_string()
}

fn default_width() -> u32 {
    800
}

fn default_height() -> u32 {
    400
}

fn default_date() -> NaiveDate {
    chrono::Utc::now().date_naive()
}

/// Visualization service for generating charts
pub struct VisualizationService {
    timestream_repo: Arc<TimestreamRepository>,
    dynamodb_repo: Arc<DynamoDbRepository>,
}

impl VisualizationService {
    /// Create a new visualization service
    pub fn new(
        timestream_repo: Arc<TimestreamRepository>,
        dynamodb_repo: Arc<DynamoDbRepository>,
    ) -> Self {
        Self {
            timestream_repo,
            dynamodb_repo,
        }
    }

    /// Generate a performance chart for a portfolio
    pub async fn generate_performance_chart(
        &self,
        params: PerformanceChartParams,
    ) -> Result<(Vec<u8>, String), AppError> {
        info!(
            "Generating performance chart for portfolio {} from {} to {}",
            params.portfolio_id, params.start_date, params.end_date
        );
        
        // Get the portfolio to verify it exists and get the name
        let portfolio = match self.dynamodb_repo.get_portfolio(&params.portfolio_id).await? {
            Some(p) => p,
            None => return Err(AppError::NotFound(format!("Portfolio not found: {}", params.portfolio_id))),
        };
        
        // Get time series data from Timestream
        let time_series = self.timestream_repo
            .get_performance_time_series(
                &params.portfolio_id,
                params.start_date,
                params.end_date,
                params.interval,
                params.benchmark_id.as_deref(),
            )
            .await?;
        
        // Create the chart
        let chart_data = self.create_performance_chart(
            &portfolio.name,
            &time_series,
            &params.metrics,
            params.benchmark_id.as_deref(),
            params.width,
            params.height,
            params.format,
        )?;
        
        Ok((chart_data, params.format.mime_type().to_string()))
    }

    /// Generate a comparison chart for multiple portfolios
    pub async fn generate_comparison_chart(
        &self,
        params: ComparisonChartParams,
    ) -> Result<(Vec<u8>, String), AppError> {
        info!(
            "Generating comparison chart for portfolios {:?} from {} to {}",
            params.portfolio_ids, params.start_date, params.end_date
        );
        
        // Get all portfolios to verify they exist and get their names
        let mut portfolio_names = HashMap::new();
        let mut all_time_series = Vec::new();
        
        for portfolio_id in &params.portfolio_ids {
            match self.dynamodb_repo.get_portfolio(portfolio_id).await? {
                Some(portfolio) => {
                    portfolio_names.insert(portfolio_id.clone(), portfolio.name.clone());
                    
                    // Get time series data for this portfolio
                    let time_series = self.timestream_repo
                        .get_performance_time_series(
                            portfolio_id,
                            params.start_date,
                            params.end_date,
                            params.interval,
                            params.benchmark_id.as_deref(),
                        )
                        .await?;
                    
                    all_time_series.push((portfolio.name, time_series));
                },
                None => return Err(AppError::NotFound(format!("Portfolio not found: {}", portfolio_id))),
            }
        }
        
        // Create the chart
        let chart_data = self.create_comparison_chart(
            &all_time_series,
            &params.metric,
            params.benchmark_id.as_deref(),
            params.width,
            params.height,
            params.format,
        )?;
        
        Ok((chart_data, params.format.mime_type().to_string()))
    }

    /// Generate a risk-return chart for multiple portfolios
    pub async fn generate_risk_return_chart(
        &self,
        params: RiskReturnChartParams,
    ) -> Result<(Vec<u8>, String), AppError> {
        info!(
            "Generating risk-return chart for {} portfolios from {} to {}",
            params.portfolio_ids.len(),
            params.start_date,
            params.end_date
        );

        if params.portfolio_ids.is_empty() {
            return Err(AppError::Validation("No portfolio IDs provided".to_string()));
        }

        // For now, return a mock implementation
        // In a real implementation, this would generate a risk-return chart
        let chart_data = vec![0u8; 100]; // Mock data
        
        Ok((chart_data, params.format.mime_type().to_string()))
    }

    /// Generate an allocation chart for a portfolio
    pub async fn generate_allocation_chart(
        &self,
        params: AllocationChartParams,
    ) -> Result<(Vec<u8>, String), AppError> {
        info!(
            "Generating allocation chart for portfolio {} as of {}",
            params.portfolio_id, params.date
        );

        // For now, return a mock implementation
        // In a real implementation, this would generate an allocation chart
        let chart_data = vec![0u8; 100]; // Mock data
        
        Ok((chart_data, params.format.mime_type().to_string()))
    }

    /// Generate an attribution chart for a portfolio
    pub async fn generate_attribution_chart(
        &self,
        params: AttributionChartParams,
    ) -> Result<(Vec<u8>, String), AppError> {
        info!(
            "Generating attribution chart for portfolio {} from {} to {}",
            params.portfolio_id, params.start_date, params.end_date
        );

        // For now, return a mock implementation
        // In a real implementation, this would generate an attribution chart
        let chart_data = vec![0u8; 100]; // Mock data
        
        Ok((chart_data, params.format.mime_type().to_string()))
    }

    // Private methods for chart creation

    /// Create a performance chart
    fn create_performance_chart(
        &self,
        _portfolio_name: &str,
        _time_series: &[TimeSeries],
        _metrics: &[String],
        _benchmark_id: Option<&str>,
        _width: u32,
        _height: u32,
        _format: ChartFormat,
    ) -> Result<Vec<u8>, AppError> {
        // Mock implementation
        let buffer = Vec::new();
        Ok(buffer)
    }

    /// Create a comparison chart
    fn create_comparison_chart(
        &self,
        _all_time_series: &[(String, Vec<TimeSeries>)],
        _metric: &str,
        _benchmark_id: Option<&str>,
        _width: u32,
        _height: u32,
        _format: ChartFormat,
    ) -> Result<Vec<u8>, AppError> {
        // Mock implementation
        let buffer = Vec::new();
        Ok(buffer)
    }
} 