use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Core events - these are essential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    Transaction(TransactionEvent),
    PriceUpdate(PriceUpdateEvent),
    PortfolioValuation(PortfolioValuationEvent),
    PerformanceCalculation(PerformanceCalculationEvent),
    // Removing redundant events:
    // - StreamingNotification (can be handled by Transaction/PriceUpdate)
    // - DataImport (can be handled by Transaction batch)
    // - ReportGeneration (can be handled by PerformanceCalculation)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionEvent {
    pub id: String,
    pub portfolio_id: String,
    pub transaction_date: NaiveDate,
    pub settlement_date: Option<NaiveDate>,
    pub transaction_type: TransactionType,
    pub symbol: Option<String>,
    pub quantity: Option<Decimal>,
    pub price: Option<Decimal>,
    pub amount: Decimal,
    pub currency: String,
    pub fees: Option<Decimal>,
    pub taxes: Option<Decimal>,
    pub notes: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Buy,
    Sell,
    Dividend,
    Interest,
    Deposit,
    Withdrawal,
    Fee,
    Tax,
    Split,
    Transfer,
    // Removing redundant transaction types:
    // - CorporateAction (can be handled by specific types)
    // - Adjustment (can be handled by specific types)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdateEvent {
    pub symbol: String,
    pub price_date: NaiveDate,
    pub price: Decimal,
    pub currency: String,
    pub source: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioValuationEvent {
    pub portfolio_id: String,
    pub valuation_date: NaiveDate,
    pub total_value: Decimal,
    pub currency: String,
    pub holdings: Vec<HoldingValuation>,
    pub cash_balances: Vec<CashBalance>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldingValuation {
    pub symbol: String,
    pub quantity: Decimal,
    pub price: Decimal,
    pub value: Decimal,
    pub currency: String,
    pub unrealized_pl: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashBalance {
    pub currency: String,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceCalculationEvent {
    pub portfolio_id: String,
    pub calculation_type: CalculationType,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub base_currency: String,
    pub result: PerformanceResult,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CalculationType {
    TWR,
    MWR,
    ModifiedDietz,
    // Removing redundant calculation types:
    // - CustomCalculation (can be handled by specific types)
    // - RiskMetrics (should be separate)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceResult {
    pub value: Decimal,
    pub annualized: Option<Decimal>,
    pub sub_period_returns: Option<HashMap<NaiveDate, Decimal>>,
    pub metadata: Option<HashMap<String, String>>,
}

// Audit events - essential for compliance and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub entity_id: String,
    pub entity_type: String,
    pub action: String,
    pub user_id: String,
    pub parameters: String,
    pub result: String,
}

// Scheduler events - essential for Phase 2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerEvent {
    pub job_id: String,
    pub job_type: String,
    pub scheduled_time: DateTime<Utc>,
    pub execution_time: Option<DateTime<Utc>>,
    pub status: JobStatus,
    pub parameters: HashMap<String, String>,
    pub result: Option<String>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Scheduled,
    Running,
    Completed,
    Failed,
    Cancelled,
}

// Removed redundant events:
// - NotificationEvent (handled by integration module)
// - AnalyticsEvent (handled by analytics module)
// - VisualizationEvent (handled by visualization module)
// - IntegrationEvent (handled by integration module)
// - DataImportEvent (handled by integration module)
// - ReportGenerationEvent (handled by visualization module) 