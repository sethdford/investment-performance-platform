use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use std::collections::HashMap;
use rust_decimal::Decimal;

/// Time series data point for performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeries {
    /// Date of the data point
    pub date: NaiveDate,
    
    /// Time-weighted return
    pub twr: f64,
    
    /// Money-weighted return
    pub mwr: f64,
    
    /// Volatility
    pub volatility: f64,
    
    /// Benchmark return
    pub benchmark_return: Option<f64>,
}

/// Time series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    /// Date of the data point
    pub date: NaiveDate,
    
    /// Value of the data point
    pub value: f64,
    
    /// Metric name
    pub metric: String,
}

/// Represents an item in the system
///
/// This is the core data model for the application. Items are stored in DynamoDB
/// and can be created, retrieved, and deleted through the API.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Item {
    /// Unique identifier for the item
    /// 
    /// If not provided when creating an item, a UUID will be automatically generated.
    #[serde(default = "generate_id")]
    pub id: String,
    
    /// Name of the item (required)
    pub name: String,
    
    /// Optional description of the item
    pub description: Option<String>,
    
    /// Timestamp when the item was created
    /// 
    /// If not provided when creating an item, the current time will be used.
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
    
    /// Classification level of this item
    /// Options: PUBLIC, INTERNAL, CONFIDENTIAL, RESTRICTED
    #[serde(default = "default_classification")]
    pub classification: String,
}

/// Generates a new UUID string for item IDs
fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// Returns the current UTC time for item creation timestamps
fn default_created_at() -> DateTime<Utc> {
    Utc::now()
}

/// Returns the default classification level for items
fn default_classification() -> String {
    "INTERNAL".to_string()
}

/// Generic API response wrapper
///
/// This struct is used to wrap API responses with a status code and body.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    /// HTTP status code
    pub status_code: u16,
    
    /// Response body
    pub body: T,
}

/// Error response for API errors
///
/// This struct is used to return error messages to API clients.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error message
    pub message: String,
}

/// Represents an event related to an item
///
/// Events are sent to SQS when items are created, updated, or deleted.
/// The event processor Lambda consumes these events and performs additional processing.
#[derive(Debug, Serialize, Deserialize)]
pub struct ItemEvent {
    /// Type of event (Created, Updated, Deleted)
    pub event_type: ItemEventType,
    
    /// The item associated with the event
    pub item: Item,
    
    /// Timestamp when the event occurred
    pub timestamp: DateTime<Utc>,
}

/// Types of events that can occur for items
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ItemEventType {
    /// Item was created
    Created,
    
    /// Item was updated
    Updated,
    
    /// Item was deleted
    Deleted,
}

/// Audit record for tracking changes to items
///
/// This struct is used to maintain an audit trail of all changes to items.
/// It includes information about who made the change, what was changed, and when.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditRecord {
    /// Unique identifier for the audit record
    pub event_id: String,
    
    /// ID of the user who performed the action
    pub user_id: String,
    
    /// Action that was performed (create, update, delete)
    pub action: String,
    
    /// ID of the resource that was affected
    pub resource_id: String,
    
    /// Type of resource that was affected
    pub resource_type: String,
    
    /// Timestamp when the action occurred
    pub timestamp: DateTime<Utc>,
    
    /// Previous state of the resource (for updates and deletes)
    pub previous_state: Option<String>,
    
    /// New state of the resource (for creates and updates)
    pub new_state: Option<String>,
    
    /// ID of the request that triggered the action
    pub request_id: String,
    
    /// Hash of the original request for non-repudiation
    pub hash: Option<String>,
}

/// Represents a client in the system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Client {
    /// Unique identifier for the client
    #[serde(default = "generate_id")]
    pub id: String,
    
    /// Client name
    pub name: String,
    
    /// Client type (Individual, Institution, etc.)
    pub client_type: ClientType,
    
    /// Client contact information
    pub contact: ContactInfo,
    
    /// Client classification/segmentation
    pub classification: String,
    
    /// Client creation timestamp
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
    
    /// Client last updated timestamp
    #[serde(default = "default_created_at")]
    pub updated_at: DateTime<Utc>,
    
    /// Client status
    #[serde(default = "default_status")]
    pub status: Status,
    
    /// Additional client metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Client type enumeration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ClientType {
    Individual,
    Joint,
    Institution,
    Trust,
    Other(String),
}

/// Contact information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContactInfo {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<Address>,
}

/// Physical address
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Address {
    pub street1: String,
    pub street2: Option<String>,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
}

/// Status enumeration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Status {
    Active,
    Inactive,
    Pending,
    Closed,
}

/// Returns the default status
fn default_status() -> Status {
    Status::Active
}

/// Represents a portfolio in the system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Portfolio {
    /// Unique identifier for the portfolio
    #[serde(default = "generate_id")]
    pub id: String,
    
    /// Portfolio name
    pub name: String,
    
    /// Client ID that owns this portfolio
    pub client_id: String,
    
    /// Portfolio inception date
    pub inception_date: NaiveDate,
    
    /// Portfolio benchmark ID
    pub benchmark_id: Option<String>,
    
    /// Portfolio creation timestamp
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
    
    /// Portfolio last updated timestamp
    #[serde(default = "default_created_at")]
    pub updated_at: DateTime<Utc>,
    
    /// Portfolio status
    #[serde(default = "default_status")]
    pub status: Status,
    
    /// Additional portfolio metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Portfolio transactions
    #[serde(default)]
    pub transactions: Vec<Transaction>,

    /// Portfolio holdings
    #[serde(default)]
    pub holdings: Vec<Holding>,
}

impl Portfolio {
    /// Add a transaction to the portfolio
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    /// Add a holding to the portfolio
    pub fn add_holding(&mut self, holding: Holding) {
        self.holdings.push(holding);
    }
}

/// Represents a holding in a portfolio
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Holding {
    /// Security symbol
    pub symbol: String,
    
    /// Holding quantity
    pub quantity: Decimal,
    
    /// Cost basis of the holding
    pub cost_basis: Option<Decimal>,
    
    /// Currency of the holding
    pub currency: String,
}

/// Represents an account in the system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    /// Unique identifier for the account
    #[serde(default = "generate_id")]
    pub id: String,
    
    /// Account number
    pub account_number: String,
    
    /// Account name
    pub name: String,
    
    /// Portfolio ID this account belongs to
    pub portfolio_id: String,
    
    /// Account type
    pub account_type: AccountType,
    
    /// Account inception date
    pub inception_date: NaiveDate,
    
    /// Account tax status
    pub tax_status: TaxStatus,
    
    /// Account creation timestamp
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
    
    /// Account last updated timestamp
    #[serde(default = "default_created_at")]
    pub updated_at: DateTime<Utc>,
    
    /// Account status
    #[serde(default = "default_status")]
    pub status: Status,
    
    /// Additional account metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Account type enumeration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AccountType {
    Individual,
    Joint,
    IRA,
    Roth,
    Trust,
    CorporateRetirement,
    Custodial,
    Other(String),
}

/// Tax status enumeration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TaxStatus {
    Taxable,
    TaxDeferred,
    TaxExempt,
}

/// Represents a security in the system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Security {
    /// Unique identifier for the security
    #[serde(default = "generate_id")]
    pub id: String,
    
    /// Security symbol
    pub symbol: String,
    
    /// Security name
    pub name: String,
    
    /// Security type
    pub security_type: SecurityType,
    
    /// Security asset class
    pub asset_class: AssetClass,
    
    /// CUSIP identifier
    pub cusip: Option<String>,
    
    /// ISIN identifier
    pub isin: Option<String>,
    
    /// SEDOL identifier
    pub sedol: Option<String>,
    
    /// Security creation timestamp
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
    
    /// Security last updated timestamp
    #[serde(default = "default_created_at")]
    pub updated_at: DateTime<Utc>,
    
    /// Additional security metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Security type enumeration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SecurityType {
    Equity,
    FixedIncome,
    MutualFund,
    ETF,
    Option,
    Future,
    Cash,
    RealEstate,
    Alternative,
    Other(String),
}

/// Asset class enumeration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AssetClass {
    DomesticEquity,
    InternationalEquity,
    EmergingMarkets,
    GovernmentBond,
    CorporateBond,
    MunicipalBond,
    HighYield,
    Cash,
    RealEstate,
    Commodity,
    Hedge,
    PrivateEquity,
    Other(String),
}

/// Represents a price point for a security
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Price {
    /// Security ID
    pub security_id: String,
    
    /// Price date
    pub date: NaiveDate,
    
    /// Price value
    pub price: f64,
    
    /// Currency code (ISO 4217)
    pub currency: String,
    
    /// Price source
    pub source: String,
    
    /// Price timestamp
    #[serde(default = "default_created_at")]
    pub timestamp: DateTime<Utc>,
}

/// Represents a transaction in the system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    /// Unique identifier for the transaction
    #[serde(default = "generate_id")]
    pub id: String,
    
    /// Account ID this transaction belongs to
    pub account_id: String,
    
    /// Security ID this transaction involves
    pub security_id: Option<String>,
    
    /// Transaction date
    pub transaction_date: NaiveDate,
    
    /// Settlement date
    pub settlement_date: Option<NaiveDate>,
    
    /// Transaction type
    pub transaction_type: TransactionType,
    
    /// Transaction amount
    pub amount: f64,
    
    /// Transaction quantity
    pub quantity: Option<f64>,
    
    /// Transaction price
    pub price: Option<f64>,
    
    /// Transaction fees
    pub fees: Option<f64>,
    
    /// Transaction currency
    pub currency: String,
    
    /// Transaction creation timestamp
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
    
    /// Transaction last updated timestamp
    #[serde(default = "default_created_at")]
    pub updated_at: DateTime<Utc>,
    
    /// Additional transaction metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Transaction type enumeration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TransactionType {
    Buy,
    Sell,
    Deposit,
    Withdrawal,
    Dividend,
    Interest,
    Fee,
    Transfer,
    Split,
    Other(String),
}

/// Represents a position in the system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Position {
    /// Account ID this position belongs to
    pub account_id: String,
    
    /// Security ID this position involves
    pub security_id: String,
    
    /// Position date
    pub date: NaiveDate,
    
    /// Position quantity
    pub quantity: f64,
    
    /// Position market value
    pub market_value: f64,
    
    /// Position cost basis
    pub cost_basis: Option<f64>,
    
    /// Position currency
    pub currency: String,
    
    /// Position creation timestamp
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
    
    /// Position last updated timestamp
    #[serde(default = "default_created_at")]
    pub updated_at: DateTime<Utc>,
}

/// Represents a performance metric in the system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PerformanceMetric {
    /// Entity ID (Portfolio ID or Account ID)
    pub entity_id: String,
    
    /// Entity type (Portfolio or Account)
    pub entity_type: EntityType,
    
    /// Metric date
    pub date: NaiveDate,
    
    /// Metric period
    pub period: PerformancePeriod,
    
    /// Time-weighted return
    pub time_weighted_return: Option<f64>,
    
    /// Money-weighted return (IRR)
    pub money_weighted_return: Option<f64>,
    
    /// Benchmark return
    pub benchmark_return: Option<f64>,
    
    /// Alpha
    pub alpha: Option<f64>,
    
    /// Beta
    pub beta: Option<f64>,
    
    /// Sharpe ratio
    pub sharpe_ratio: Option<f64>,
    
    /// Sortino ratio
    pub sortino_ratio: Option<f64>,
    
    /// Information ratio
    pub information_ratio: Option<f64>,
    
    /// Tracking error
    pub tracking_error: Option<f64>,
    
    /// Maximum drawdown
    pub max_drawdown: Option<f64>,
    
    /// Metric creation timestamp
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
    
    /// Metric last updated timestamp
    #[serde(default = "default_created_at")]
    pub updated_at: DateTime<Utc>,
}

/// Entity type enumeration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum EntityType {
    Portfolio,
    Account,
}

/// Performance period enumeration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PerformancePeriod {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    YearToDate,
    OneYear,
    ThreeYear,
    FiveYear,
    TenYear,
    SinceInception,
    Custom(String),
}

/// Represents a benchmark in the system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Benchmark {
    /// Unique identifier for the benchmark
    #[serde(default = "generate_id")]
    pub id: String,
    
    /// Benchmark name
    pub name: String,
    
    /// Benchmark symbol
    pub symbol: Option<String>,
    
    /// Benchmark description
    pub description: Option<String>,
    
    /// Benchmark creation timestamp
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
    
    /// Benchmark last updated timestamp
    #[serde(default = "default_created_at")]
    pub updated_at: DateTime<Utc>,
}

/// Represents a benchmark return in the system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BenchmarkReturn {
    /// Benchmark ID
    pub benchmark_id: String,
    
    /// Return date
    pub date: NaiveDate,
    
    /// Return period
    pub period: PerformancePeriod,
    
    /// Return value
    pub return_value: f64,
    
    /// Return creation timestamp
    #[serde(default = "default_created_at")]
    pub created_at: DateTime<Utc>,
}

/// Performance metrics for a portfolio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Portfolio ID
    pub portfolio_id: String,
    
    /// Start date for the performance period
    pub start_date: String,
    
    /// End date for the performance period
    pub end_date: String,
    
    /// Time-weighted return
    pub twr: f64,
    
    /// Money-weighted return
    pub mwr: f64,
    
    /// Volatility (standard deviation of returns)
    pub volatility: Option<f64>,
    
    /// Sharpe ratio
    pub sharpe_ratio: Option<f64>,
    
    /// Maximum drawdown
    pub max_drawdown: Option<f64>,
    
    /// Benchmark ID
    pub benchmark_id: Option<String>,
    
    /// Benchmark return
    pub benchmark_return: Option<f64>,
    
    /// Tracking error
    pub tracking_error: Option<f64>,
    
    /// Information ratio
    pub information_ratio: Option<f64>,
    
    /// When the metrics were calculated
    pub calculated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_item_serialization() {
        let item = Item {
            id: "test-id".to_string(),
            name: "Test Item".to_string(),
            description: Some("Test Description".to_string()),
            created_at: DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            classification: "INTERNAL".to_string(),
        };

        let json = serde_json::to_string(&item).unwrap();
        let expected = json!({
            "id": "test-id",
            "name": "Test Item",
            "description": "Test Description",
            "created_at": "2023-01-01T00:00:00Z",
            "classification": "INTERNAL"
        });

        assert_eq!(serde_json::from_str::<serde_json::Value>(&json).unwrap(), expected);
    }

    #[test]
    fn test_item_deserialization() {
        let json = r#"{
            "id": "test-id",
            "name": "Test Item",
            "description": "Test Description",
            "created_at": "2023-01-01T00:00:00Z",
            "classification": "INTERNAL"
        }"#;

        let item: Item = serde_json::from_str(json).unwrap();
        
        assert_eq!(item.id, "test-id");
        assert_eq!(item.name, "Test Item");
        assert_eq!(item.description, Some("Test Description".to_string()));
        assert_eq!(
            item.created_at,
            DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        );
        assert_eq!(item.classification, "INTERNAL");
    }

    #[test]
    fn test_item_default_values() {
        let json = r#"{
            "name": "Test Item"
        }"#;

        let item: Item = serde_json::from_str(json).unwrap();
        
        assert!(!item.id.is_empty()); // ID should be auto-generated
        assert_eq!(item.name, "Test Item");
        assert_eq!(item.description, None);
        // created_at should be auto-generated and close to now
        let now = Utc::now();
        let diff = now.signed_duration_since(item.created_at);
        assert!(diff.num_seconds() < 10); // Should be within 10 seconds
        assert_eq!(item.classification, "INTERNAL");
    }

    #[test]
    fn test_item_event_serialization() {
        let item = Item {
            id: "test-id".to_string(),
            name: "Test Item".to_string(),
            description: Some("Test Description".to_string()),
            created_at: Utc::now(),
            classification: "PUBLIC".to_string(),
        };
        
        let event = ItemEvent {
            event_type: ItemEventType::Created,
            item,
            timestamp: Utc::now(),
        };
        
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("Test Item"));
        assert!(json.contains("Created"));
    }
} 