use crate::portfolio::rebalancing::{PortfolioRebalancingService, RebalanceTrade, Portfolio, CashFlow};
use crate::factor_model::FactorModelApi;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc, NaiveDate, Datelike};
use serde::{Serialize, Deserialize};
use tracing::info;
use uuid::Uuid;

/// Model provider for the financial advisor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelProvider {
    /// AWS Bedrock
    Bedrock,
    
    /// Mock provider for testing
    Mock,
}

/// Financial advisor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialAdvisorConfig {
    /// Model provider
    pub model_provider: ModelProvider,
    
    /// Model ID
    pub model_id: Option<String>,
    
    /// Use streaming responses
    pub use_streaming: bool,
    
    /// Maximum tokens to generate
    pub max_tokens: u32,
    
    /// Temperature (0.0-1.0)
    pub temperature: f64,
}

impl Default for FinancialAdvisorConfig {
    fn default() -> Self {
        Self {
            model_provider: ModelProvider::Bedrock,
            model_id: Some("anthropic.claude-3-sonnet-20240229-v1:0".to_string()),
            use_streaming: true,
            max_tokens: 4096,
            temperature: 0.2,
        }
    }
}

// Export the streaming handler module
pub mod streaming_handler;

// Export the examples module
pub mod examples;

// Export the client profiling module
pub mod client_profiling;

// Export the risk profiling module
pub mod risk_profiling;

// Export the goal templates module
pub mod goal_templates;

// Export the natural language processing module
pub mod nlp;

// Add the CFP knowledge module
pub mod cfp_knowledge;

/// Financial advisor event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FinancialAdvisorEventType {
    /// Portfolio drift detected
    PortfolioDrift,
    
    /// Tax loss harvesting opportunity
    TaxLossHarvesting,
    
    /// Cash flow received
    CashFlow,
    
    /// Market volatility alert
    MarketVolatility,
    
    /// Rebalancing recommendation
    RebalancingRecommendation,
    
    /// Goal progress update
    GoalProgress,
}

/// Financial advisor recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialAdvisorRecommendation {
    /// Unique identifier
    pub id: String,
    
    /// Recommendation type
    pub recommendation_type: FinancialAdvisorEventType,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Portfolio ID
    pub portfolio_id: String,
    
    /// Recommendation title
    pub title: String,
    
    /// Recommendation description
    pub description: String,
    
    /// Recommendation priority (1-5, with 1 being highest)
    pub priority: u8,
    
    /// Recommended trades
    pub recommended_trades: Option<Vec<RebalanceTrade>>,
    
    /// Additional data
    pub additional_data: Option<serde_json::Value>,
}

/// Financial advisor notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    /// Email notifications enabled
    pub email_enabled: bool,
    
    /// Email address
    pub email_address: Option<String>,
    
    /// Push notifications enabled
    pub push_enabled: bool,
    
    /// Device token for push notifications
    pub device_token: Option<String>,
    
    /// SMS notifications enabled
    pub sms_enabled: bool,
    
    /// Phone number for SMS
    pub phone_number: Option<String>,
    
    /// Minimum priority for notifications (1-5)
    pub min_priority: u8,
    
    /// Event types to notify about
    pub event_types: Vec<FinancialAdvisorEventType>,
}

/// Financial advisor service
#[derive(Clone)]
pub struct FinancialAdvisorService {
    /// Portfolio rebalancing service
    rebalancing_service: PortfolioRebalancingService,
    
    /// Factor model API
    factor_model_api: FactorModelApi,
    
    /// Notification preferences by user ID
    notification_preferences: Arc<Mutex<HashMap<String, NotificationPreferences>>>,
    
    /// Recent recommendations by portfolio ID
    recent_recommendations: Arc<Mutex<HashMap<String, Vec<FinancialAdvisorRecommendation>>>>,
    
    /// Risk profiling service
    risk_profiling_service: risk_profiling::RiskProfilingService,
    
    /// Configuration
    config: FinancialAdvisorConfig,
}

impl FinancialAdvisorService {
    /// Create a new financial advisor service
    pub async fn new(config: FinancialAdvisorConfig, rebalancing_service: Option<PortfolioRebalancingService>) -> Result<Self> {
        // Create a new FactorModelApi
        let factor_model_api = FactorModelApi::new();
        
        // Create a new PortfolioRebalancingService if one wasn't provided
        let rebalancing_service = rebalancing_service.unwrap_or_else(|| PortfolioRebalancingService::new(factor_model_api.clone()));
        
        Ok(Self {
            rebalancing_service,
            factor_model_api,
            notification_preferences: Arc::new(Mutex::new(HashMap::new())),
            recent_recommendations: Arc::new(Mutex::new(HashMap::new())),
            risk_profiling_service: risk_profiling::RiskProfilingService::new(),
            config,
        })
    }
    
    /// Set notification preferences for a user
    pub async fn set_notification_preferences(&self, user_id: &str, preferences: NotificationPreferences) -> Result<()> {
        let mut prefs = self.notification_preferences.lock().await;
        prefs.insert(user_id.to_string(), preferences);
        Ok(())
    }
    
    /// Get notification preferences for a user
    pub async fn get_notification_preferences(&self, user_id: &str) -> Result<Option<NotificationPreferences>> {
        let prefs = self.notification_preferences.lock().await;
        Ok(prefs.get(user_id).cloned())
    }
    
    /// Check for portfolio drift and generate recommendations
    pub async fn check_portfolio_drift(&self, portfolio: &Portfolio, _user_id: &str) -> Result<Option<FinancialAdvisorRecommendation>> {
        // Get target factor exposures (in a real implementation, this would come from the user's risk profile)
        let target_exposures = self.factor_model_api.get_factor_exposures(portfolio.id.as_str())
            .ok_or_else(|| anyhow!("Failed to get factor exposures for portfolio {}", portfolio.id))?;
        
        // Calculate factor drift
        if let Some(factor_drift) = self.rebalancing_service.calculate_factor_drift(&portfolio.id, &target_exposures, None) {
            // Calculate drift score
            let drift_score = self.rebalancing_service.calculate_drift_score(&factor_drift, None);
            
            // If drift score is above threshold, generate a recommendation
            if drift_score > 0.15 {
                // Generate rebalance trades
                let trades = self.rebalancing_service.generate_factor_rebalance_trades(
                    &portfolio.id,
                    &target_exposures,
                    Some(5),
                    true
                );
                
                // Create recommendation
                let recommendation = FinancialAdvisorRecommendation {
                    id: Uuid::new_v4().to_string(),
                    recommendation_type: FinancialAdvisorEventType::PortfolioDrift,
                    timestamp: Utc::now(),
                    portfolio_id: portfolio.id.clone(),
                    title: "Portfolio Drift Detected".to_string(),
                    description: format!(
                        "Your portfolio has drifted from its target allocations with a drift score of {:.2}. \
                        Consider rebalancing to maintain your desired risk and return profile.",
                        drift_score
                    ),
                    priority: if drift_score > 0.3 { 2 } else { 3 },
                    recommended_trades: if !trades.is_empty() { Some(trades) } else { None },
                    additional_data: Some(serde_json::json!({
                        "drift_score": drift_score,
                        "factor_drift": factor_drift,
                    })),
                };
                
                // Store the recommendation
                self.store_recommendation(&portfolio.id, recommendation.clone()).await?;
                
                return Ok(Some(recommendation));
            }
        }
        
        Ok(None)
    }
    
    /// Handle cash flow and generate recommendations
    pub async fn handle_cash_flow(&self, portfolio: &Portfolio, cash_flow: &CashFlow, _user_id: &str) -> Result<Option<FinancialAdvisorRecommendation>> {
        // Generate trades to handle the cash flow
        let trades = self.rebalancing_service.generate_cash_flow_trades(
            portfolio,
            cash_flow,
            true, // Maintain target weights
            None, // Use default minimum trade amount
        );
        
        // Only create a recommendation if there are trades to make
        if !trades.is_empty() {
            // Create recommendation
            let recommendation = FinancialAdvisorRecommendation {
                id: uuid::Uuid::new_v4().to_string(),
                recommendation_type: FinancialAdvisorEventType::CashFlow,
                timestamp: Utc::now(),
                portfolio_id: portfolio.id.clone(),
                title: if cash_flow.amount > 0.0 {
                    "Investment Opportunity from Deposit".to_string()
                } else {
                    "Withdrawal Strategy".to_string()
                },
                description: if cash_flow.amount > 0.0 {
                    format!(
                        "You've received a deposit of ${:.2}. We recommend investing it according to your target allocation.",
                        cash_flow.amount
                    )
                } else {
                    format!(
                        "You've requested a withdrawal of ${:.2}. We recommend selling from these positions to maintain your target allocation.",
                        cash_flow.amount.abs()
                    )
                },
                priority: 2, // Higher priority for cash flow
                recommended_trades: Some(trades),
                additional_data: Some(serde_json::json!({
                    "cash_flow_amount": cash_flow.amount,
                    "cash_flow_type": format!("{:?}", cash_flow.flow_type),
                })),
            };
            
            // Store recommendation
            self.store_recommendation(portfolio.id.as_str(), recommendation.clone()).await?;
            
            // Return recommendation
            return Ok(Some(recommendation));
        }
        
        Ok(None)
    }
    
    /// Check for tax loss harvesting opportunities
    pub async fn check_tax_loss_harvesting(&self, portfolio: &Portfolio, _user_id: &str) -> Result<Option<FinancialAdvisorRecommendation>> {
        // In a real implementation, this would use the portfolio rebalancing service to identify tax loss harvesting opportunities
        // For now, we'll create a simplified implementation
        
        let mut tlh_opportunities = Vec::new();
        let mut total_tax_savings = 0.0;
        
        // Look for holdings with unrealized losses
        for holding in &portfolio.holdings {
            let unrealized_gain_loss = holding.market_value - holding.cost_basis;
            
            // If there's a loss and it's significant (more than 5% of cost basis)
            if unrealized_gain_loss < 0.0 && unrealized_gain_loss.abs() > (holding.cost_basis * 0.05) {
                // Assume a 20% tax rate for short-term capital gains
                let potential_tax_savings = unrealized_gain_loss.abs() * 0.2;
                
                // Only consider if potential tax savings is significant (e.g., more than $100)
                if potential_tax_savings > 100.0 {
                    tlh_opportunities.push((holding, potential_tax_savings));
                    total_tax_savings += potential_tax_savings;
                }
            }
        }
        
        // If there are opportunities, create a recommendation
        if !tlh_opportunities.is_empty() {
            // Generate trades for tax loss harvesting
            let mut trades = Vec::new();
            
            for (holding, _) in &tlh_opportunities {
                // Create sell trade
                let sell_trade = RebalanceTrade {
                    security_id: holding.security_id.clone(),
                    amount: holding.market_value,
                    is_buy: false,
                    reason: crate::portfolio::rebalancing::TradeReason::TaxLossHarvesting,
                    tax_impact: Some(-holding.market_value + holding.cost_basis),
                };
                
                trades.push(sell_trade);
                
                // In a real implementation, we would also create a buy trade for a similar but not substantially identical security
                // For simplicity, we'll skip that here
            }
            
            // Create recommendation
            let recommendation = FinancialAdvisorRecommendation {
                id: uuid::Uuid::new_v4().to_string(),
                recommendation_type: FinancialAdvisorEventType::TaxLossHarvesting,
                timestamp: Utc::now(),
                portfolio_id: portfolio.id.clone(),
                title: "Tax Loss Harvesting Opportunity".to_string(),
                description: format!(
                    "We've identified tax loss harvesting opportunities that could save you approximately ${:.2} in taxes.",
                    total_tax_savings
                ),
                priority: 2, // Higher priority for tax savings
                recommended_trades: Some(trades),
                additional_data: Some(serde_json::json!({
                    "potential_tax_savings": total_tax_savings,
                    "opportunities": tlh_opportunities.len(),
                })),
            };
            
            // Store recommendation
            self.store_recommendation(portfolio.id.as_str(), recommendation.clone()).await?;
            
            // Return recommendation
            return Ok(Some(recommendation));
        }
        
        Ok(None)
    }
    
    /// Store a recommendation
    async fn store_recommendation(&self, portfolio_id: &str, recommendation: FinancialAdvisorRecommendation) -> Result<()> {
        let mut recommendations = self.recent_recommendations.lock().await;
        
        // Get or create the vector for this portfolio
        let portfolio_recommendations = recommendations
            .entry(portfolio_id.to_string())
            .or_insert_with(Vec::new);
        
        // Add the recommendation
        portfolio_recommendations.push(recommendation);
        
        // Keep only the 10 most recent recommendations
        if portfolio_recommendations.len() > 10 {
            portfolio_recommendations.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            portfolio_recommendations.truncate(10);
        }
        
        Ok(())
    }
    
    /// Get recent recommendations for a portfolio
    pub async fn get_recent_recommendations(&self, portfolio_id: &str) -> Result<Vec<FinancialAdvisorRecommendation>> {
        let recommendations = self.recent_recommendations.lock().await;
        
        // Get recommendations for this portfolio
        let portfolio_recommendations = recommendations
            .get(portfolio_id)
            .cloned()
            .unwrap_or_default();
        
        Ok(portfolio_recommendations)
    }
    
    /// Get risk profiling service
    pub fn get_risk_profiling_service(&self) -> &risk_profiling::RiskProfilingService {
        &self.risk_profiling_service
    }
}

// Example usage
pub fn run_financial_advisor_example() {
    // This would be implemented in a real application
    println!("Financial Advisor Example");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Test the financial advisor service
    #[tokio::test]
    async fn test_financial_advisor_service() {
        // Create dependencies
        let factor_model_api = FactorModelApi::new();
        let rebalancing_service = PortfolioRebalancingService::new(factor_model_api.clone());
        
        // Create financial advisor service
        let advisor = FinancialAdvisorService::new(FinancialAdvisorConfig::default(), Some(rebalancing_service)).await.unwrap();
        
        // Set notification preferences
        let preferences = NotificationPreferences {
            email_enabled: true,
            email_address: Some("user@example.com".to_string()),
            push_enabled: false,
            device_token: None,
            sms_enabled: false,
            phone_number: None,
            min_priority: 3,
            event_types: vec![
                FinancialAdvisorEventType::PortfolioDrift,
                FinancialAdvisorEventType::TaxLossHarvesting,
                FinancialAdvisorEventType::CashFlow,
            ],
        };
        
        advisor.set_notification_preferences("user-123", preferences).await.unwrap();
        
        // Get notification preferences
        let retrieved_preferences = advisor.get_notification_preferences("user-123").await.unwrap();
        assert!(retrieved_preferences.is_some());
        assert_eq!(retrieved_preferences.unwrap().email_address, Some("user@example.com".to_string()));
        
        // Create a test portfolio
        let portfolio = create_test_portfolio();
        
        // Check for portfolio drift
        let drift_recommendation = advisor.check_portfolio_drift(&portfolio, "user-123").await.unwrap();
        
        // In a real test, we would verify the recommendation details
        // For now, we'll just check that a recommendation was generated
        assert!(drift_recommendation.is_some());
        
        // Check for tax loss harvesting opportunities
        let tlh_recommendation = advisor.check_tax_loss_harvesting(&portfolio, "user-123").await.unwrap();
        
        // In a real test, we would verify the recommendation details
        // For now, we'll just check that a recommendation was generated
        assert!(tlh_recommendation.is_some());
        
        // Handle a cash flow
        let cash_flow = CashFlow {
            amount: 10000.0,
            date: "2023-01-01".to_string(),
            flow_type: crate::portfolio::rebalancing::CashFlowType::Deposit,
        };
        
        let cash_flow_recommendation = advisor.handle_cash_flow(&portfolio, &cash_flow, "user-123").await.unwrap();
        
        // In a real test, we would verify the recommendation details
        // For now, we'll just check that a recommendation was generated
        assert!(cash_flow_recommendation.is_some());
        
        // Get recent recommendations
        let recommendations = advisor.get_recent_recommendations(&portfolio.id).await.unwrap();
        
        // We should have 3 recommendations
        assert_eq!(recommendations.len(), 3);
    }
    
    // Create a test portfolio
    fn create_test_portfolio() -> Portfolio {
        Portfolio {
            id: "portfolio-123".to_string(),
            name: "Test Portfolio".to_string(),
            total_market_value: 100000.0,
            cash_balance: 5000.0,
            holdings: vec![
                crate::portfolio::rebalancing::PortfolioHolding {
                    security_id: "VTI".to_string(),
                    market_value: 30000.0,
                    weight: 0.3,
                    target_weight: 0.4,
                    cost_basis: 25000.0,
                    purchase_date: "2022-01-01".to_string(),
                    factor_exposures: HashMap::new(),
                },
                crate::portfolio::rebalancing::PortfolioHolding {
                    security_id: "BND".to_string(),
                    market_value: 20000.0,
                    weight: 0.2,
                    target_weight: 0.3,
                    cost_basis: 22000.0,
                    purchase_date: "2022-01-01".to_string(),
                    factor_exposures: HashMap::new(),
                },
                crate::portfolio::rebalancing::PortfolioHolding {
                    security_id: "VEA".to_string(),
                    market_value: 25000.0,
                    weight: 0.25,
                    target_weight: 0.2,
                    cost_basis: 28000.0,
                    purchase_date: "2022-01-01".to_string(),
                    factor_exposures: HashMap::new(),
                },
                crate::portfolio::rebalancing::PortfolioHolding {
                    security_id: "VWO".to_string(),
                    market_value: 20000.0,
                    weight: 0.2,
                    target_weight: 0.1,
                    cost_basis: 18000.0,
                    purchase_date: "2022-01-01".to_string(),
                    factor_exposures: HashMap::new(),
                },
            ],
        }
    }
}

/// Risk tolerance level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskToleranceLevel {
    /// Very conservative - Minimal risk, focus on capital preservation
    VeryConservative,
    
    /// Conservative - Low risk, emphasis on stability
    Conservative,
    
    /// Moderate - Balanced approach to risk and return
    Moderate,
    
    /// Aggressive - Higher risk for higher potential returns
    Aggressive,
    
    /// Very aggressive - Maximum risk tolerance for maximum potential returns
    VeryAggressive,
}

/// Time horizon for financial goals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeHorizon {
    /// Short term (0-2 years)
    ShortTerm,
    
    /// Medium term (3-5 years)
    MediumTerm,
    
    /// Long term (6-10 years)
    LongTerm,
    
    /// Very long term (10+ years)
    VeryLongTerm,
}

/// Financial goal type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalType {
    /// Retirement
    Retirement,
    
    /// Education funding
    Education {
        /// Beneficiary name
        beneficiary: String,
        
        /// Education level (e.g., "undergraduate", "graduate")
        education_level: String,
    },
    
    /// Home purchase
    HomePurchase {
        /// Property type (e.g., "primary residence", "vacation home")
        property_type: String,
        
        /// Location
        location: Option<String>,
    },
    
    /// Major purchase
    MajorPurchase {
        /// Purchase description
        description: String,
    },
    
    /// Emergency fund
    EmergencyFund,
    
    /// Debt repayment
    DebtRepayment {
        /// Debt type (e.g., "student loan", "mortgage", "credit card")
        debt_type: String,
    },
    
    /// Wealth accumulation
    WealthAccumulation,
    
    /// Legacy/estate planning
    Legacy {
        /// Beneficiary information
        beneficiary: Option<String>,
    },
    
    /// Custom goal
    Custom {
        /// Goal name
        name: String,
        
        /// Goal description
        description: String,
    },
}

/// Goal priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum GoalPriority {
    /// Essential - Must achieve
    Essential = 1,
    
    /// Important - Strong desire to achieve
    Important = 2,
    
    /// Aspirational - Would like to achieve if possible
    Aspirational = 3,
}

/// Goal status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalStatus {
    /// Not started
    NotStarted,
    
    /// In progress
    InProgress,
    
    /// On track
    OnTrack,
    
    /// At risk
    AtRisk,
    
    /// Achieved
    Achieved,
    
    /// Deferred
    Deferred,
}

/// Financial goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialGoal {
    /// Unique identifier
    pub id: String,
    
    /// Goal name
    pub name: String,
    
    /// Goal type
    pub goal_type: GoalType,
    
    /// Goal description
    pub description: String,
    
    /// Target amount
    pub target_amount: f64,
    
    /// Current amount saved
    pub current_amount: f64,
    
    /// Target date
    pub target_date: NaiveDate,
    
    /// Time horizon
    pub time_horizon: TimeHorizon,
    
    /// Priority level
    pub priority: GoalPriority,
    
    /// Current status
    pub status: GoalStatus,
    
    /// Monthly contribution
    pub monthly_contribution: f64,
    
    /// Required rate of return to achieve goal
    pub required_return_rate: Option<f64>,
    
    /// Probability of success (0-100)
    pub success_probability: Option<f64>,
    
    /// Associated accounts
    pub associated_accounts: Vec<String>,
    
    /// Goal-specific risk tolerance (may differ from overall profile)
    pub risk_tolerance: Option<RiskToleranceLevel>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    
    /// Creation date
    pub created_at: DateTime<Utc>,
    
    /// Last updated date
    pub updated_at: DateTime<Utc>,
}

/// Income source type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncomeSourceType {
    /// Employment income
    Employment,
    
    /// Self-employment income
    SelfEmployment,
    
    /// Rental income
    Rental,
    
    /// Investment income
    Investment,
    
    /// Pension income
    Pension,
    
    /// Social Security income
    SocialSecurity,
    
    /// Other income
    Other(String),
}

/// Income source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeSource {
    /// Unique identifier
    pub id: String,
    
    /// Source name
    pub name: String,
    
    /// Income type
    pub income_type: IncomeSourceType,
    
    /// Annual amount
    pub annual_amount: f64,
    
    /// Is this income taxable?
    pub is_taxable: bool,
    
    /// Frequency (e.g., "monthly", "bi-weekly", "annual")
    pub frequency: String,
    
    /// Expected growth rate (annual)
    pub growth_rate: Option<f64>,
    
    /// Start date
    pub start_date: Option<NaiveDate>,
    
    /// End date (if applicable)
    pub end_date: Option<NaiveDate>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Expense category
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpenseCategory {
    /// Housing (mortgage, rent, property taxes)
    Housing,
    
    /// Utilities (electricity, water, gas, internet)
    Utilities,
    
    /// Food (groceries, dining out)
    Food,
    
    /// Transportation (car payment, gas, public transit)
    Transportation,
    
    /// Healthcare (insurance, medications, doctor visits)
    Healthcare,
    
    /// Insurance (life, disability, property)
    Insurance,
    
    /// Debt payments (credit cards, student loans)
    DebtPayments,
    
    /// Entertainment and recreation
    Entertainment,
    
    /// Personal care
    PersonalCare,
    
    /// Education
    Education,
    
    /// Childcare
    Childcare,
    
    /// Gifts and donations
    GiftsAndDonations,
    
    /// Savings and investments
    SavingsAndInvestments,
    
    /// Taxes
    Taxes,
    
    /// Other expenses
    Other(String),
}

/// Expense
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expense {
    /// Unique identifier
    pub id: String,
    
    /// Expense name
    pub name: String,
    
    /// Expense category
    pub category: ExpenseCategory,
    
    /// Monthly amount
    pub monthly_amount: f64,
    
    /// Is this expense essential?
    pub is_essential: bool,
    
    /// Is this expense tax-deductible?
    pub is_tax_deductible: bool,
    
    /// Expected growth rate (annual)
    pub growth_rate: Option<f64>,
    
    /// Start date
    pub start_date: Option<NaiveDate>,
    
    /// End date (if applicable)
    pub end_date: Option<NaiveDate>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Asset type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    /// Cash and cash equivalents
    Cash,
    
    /// Investment accounts
    Investment,
    
    /// Retirement accounts
    Retirement,
    
    /// Real estate
    RealEstate,
    
    /// Business ownership
    Business,
    
    /// Personal property (vehicles, collectibles)
    PersonalProperty,
    
    /// Other assets
    Other(String),
}

/// Asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    /// Unique identifier
    pub id: String,
    
    /// Asset name
    pub name: String,
    
    /// Asset type
    pub asset_type: AssetType,
    
    /// Current value
    pub current_value: f64,
    
    /// Cost basis (for tax purposes)
    pub cost_basis: Option<f64>,
    
    /// Expected growth rate (annual)
    pub growth_rate: Option<f64>,
    
    /// Is this asset liquid?
    pub is_liquid: bool,
    
    /// Associated account ID (if applicable)
    pub account_id: Option<String>,
    
    /// Acquisition date
    pub acquisition_date: Option<NaiveDate>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Liability type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LiabilityType {
    /// Mortgage
    Mortgage,
    
    /// Auto loan
    AutoLoan,
    
    /// Student loan
    StudentLoan,
    
    /// Credit card debt
    CreditCard,
    
    /// Personal loan
    PersonalLoan,
    
    /// Business loan
    BusinessLoan,
    
    /// Tax liability
    TaxLiability,
    
    /// Other liability
    Other(String),
}

/// Liability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Liability {
    /// Unique identifier
    pub id: String,
    
    /// Liability name
    pub name: String,
    
    /// Liability type
    pub liability_type: LiabilityType,
    
    /// Current balance
    pub current_balance: f64,
    
    /// Interest rate
    pub interest_rate: f64,
    
    /// Minimum monthly payment
    pub minimum_payment: f64,
    
    /// Is this interest tax-deductible?
    pub is_tax_deductible: bool,
    
    /// Original loan amount
    pub original_amount: Option<f64>,
    
    /// Loan term in months
    pub term_months: Option<u32>,
    
    /// Origination date
    pub origination_date: Option<NaiveDate>,
    
    /// Maturity date
    pub maturity_date: Option<NaiveDate>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Insurance policy type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InsurancePolicyType {
    /// Life insurance
    Life,
    
    /// Health insurance
    Health,
    
    /// Disability insurance
    Disability,
    
    /// Long-term care insurance
    LongTermCare,
    
    /// Property insurance
    Property,
    
    /// Auto insurance
    Auto,
    
    /// Umbrella liability insurance
    Umbrella,
    
    /// Other insurance
    Other(String),
}

/// Insurance policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsurancePolicy {
    /// Unique identifier
    pub id: String,
    
    /// Policy name
    pub name: String,
    
    /// Policy type
    pub policy_type: InsurancePolicyType,
    
    /// Provider name
    pub provider: String,
    
    /// Policy number
    pub policy_number: String,
    
    /// Coverage amount
    pub coverage_amount: f64,
    
    /// Annual premium
    pub annual_premium: f64,
    
    /// Beneficiaries
    pub beneficiaries: Option<Vec<String>>,
    
    /// Effective date
    pub effective_date: NaiveDate,
    
    /// Expiration date
    pub expiration_date: Option<NaiveDate>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Types of behavioral biases
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BehavioralBiasType {
    LossAversion,
    Overconfidence,
    MentalAccounting,
    Anchoring,
    HerdMentality,
    RecencyBias,
    ConfirmationBias,
    StatusQuoBias,
    EndowmentEffect,
    SelfServingBias,
    AvailabilityBias,
    Other(String),
}

/// Risk profile questionnaire response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskProfileResponse {
    /// Question identifier
    pub question_id: String,
    
    /// Response value (typically 1-5 or similar scale)
    pub response_value: i32,
    
    /// Additional comments
    pub comments: Option<String>,
    
    /// Timestamp of response
    pub timestamp: DateTime<Utc>,
}

/// Client profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientProfile {
    /// Unique identifier
    pub id: String,
    
    /// Client name
    pub name: String,
    
    /// Email address
    pub email: String,
    
    /// Date of birth
    pub date_of_birth: NaiveDate,
    
    /// Retirement age
    pub retirement_age: Option<u8>,
    
    /// Life expectancy
    pub life_expectancy: Option<u8>,
    
    /// Tax filing status
    pub tax_filing_status: Option<String>,
    
    /// Tax bracket (federal)
    pub federal_tax_bracket: Option<f64>,
    
    /// State of residence
    pub state: Option<String>,
    
    /// State tax bracket
    pub state_tax_bracket: Option<f64>,
    
    /// Overall risk tolerance
    pub risk_tolerance: RiskToleranceLevel,
    
    /// Financial goals
    pub goals: Vec<FinancialGoal>,
    
    /// Income sources
    pub income_sources: Vec<IncomeSource>,
    
    /// Expenses
    pub expenses: Vec<Expense>,
    
    /// Assets
    pub assets: Vec<Asset>,
    
    /// Liabilities
    pub liabilities: Vec<Liability>,
    
    /// Insurance policies
    pub insurance_policies: Vec<InsurancePolicy>,
    
    /// Risk profile questionnaire responses
    pub risk_profile_responses: Vec<RiskProfileResponse>,
    
    /// Identified behavioral biases
    pub behavioral_biases: Vec<BehavioralBiasType>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    
    /// Creation date
    pub created_at: DateTime<Utc>,
    
    /// Last updated date
    pub updated_at: DateTime<Utc>,
}

/// Client profile service
pub struct ClientProfileService {
    /// Client profiles (in a real implementation, this would be a database)
    profiles: HashMap<String, ClientProfile>,
}

impl ClientProfileService {
    /// Create a new client profile service
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
        }
    }
    
    /// Create a new client profile
    pub fn create_profile(&mut self, name: &str, email: &str, date_of_birth: NaiveDate, risk_tolerance: RiskToleranceLevel) -> Result<String> {
        // Validate inputs
        if name.is_empty() {
            return Err(anyhow!("Name cannot be empty"));
        }
        
        if email.is_empty() || !email.contains('@') {
            return Err(anyhow!("Invalid email address"));
        }
        
        // Check if email is already in use
        for profile in self.profiles.values() {
            if profile.email == email {
                return Err(anyhow!("Email address is already in use"));
            }
        }
        
        // Create a new profile
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let profile = ClientProfile {
            id: id.clone(),
            name: name.to_string(),
            email: email.to_string(),
            date_of_birth,
            retirement_age: None,
            life_expectancy: None,
            tax_filing_status: None,
            federal_tax_bracket: None,
            state: None,
            state_tax_bracket: None,
            risk_tolerance,
            goals: Vec::new(),
            income_sources: Vec::new(),
            expenses: Vec::new(),
            assets: Vec::new(),
            liabilities: Vec::new(),
            insurance_policies: Vec::new(),
            risk_profile_responses: Vec::new(),
            behavioral_biases: Vec::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        };
        
        // Store the profile
        self.profiles.insert(id.clone(), profile);
        
        info!(client_id = %id, name = %name, "Created new client profile");
        
        Ok(id)
    }
    
    /// Get a client profile by ID
    pub fn get_profile(&self, id: &str) -> Result<&ClientProfile> {
        self.profiles.get(id).ok_or_else(|| anyhow!("Client profile not found"))
    }
    
    /// Update a client profile
    pub fn update_profile(&mut self, id: &str, update_fn: impl FnOnce(&mut ClientProfile)) -> Result<()> {
        let profile = self.profiles.get_mut(id).ok_or_else(|| anyhow!("Client profile not found"))?;
        
        // Apply the update
        update_fn(profile);
        
        // Update the last modified timestamp
        profile.updated_at = Utc::now();
        
        info!(client_id = %id, "Updated client profile");
        
        Ok(())
    }
    
    /// Delete a client profile
    pub fn delete_profile(&mut self, id: &str) -> Result<()> {
        if self.profiles.remove(id).is_none() {
            return Err(anyhow!("Client profile not found"));
        }
        
        info!(client_id = %id, "Deleted client profile");
        
        Ok(())
    }
    
    /// Add a financial goal to a client profile
    pub fn add_goal(&mut self, client_id: &str, name: &str, goal_type: GoalType, description: &str, 
                    target_amount: f64, target_date: NaiveDate, priority: GoalPriority) -> Result<String> {
        // Validate inputs
        if name.is_empty() {
            return Err(anyhow!("Goal name cannot be empty"));
        }
        
        if description.is_empty() {
            return Err(anyhow!("Goal description cannot be empty"));
        }
        
        if target_amount <= 0.0 {
            return Err(anyhow!("Target amount must be positive"));
        }
        
        // Calculate time horizon based on target date
        let today = Utc::now().date_naive();
        let years_to_goal = (target_date.year_ce().1 as i32 - today.year_ce().1 as i32) as f64;
        
        let time_horizon = if years_to_goal <= 2.0 {
            TimeHorizon::ShortTerm
        } else if years_to_goal <= 5.0 {
            TimeHorizon::MediumTerm
        } else if years_to_goal <= 10.0 {
            TimeHorizon::LongTerm
        } else {
            TimeHorizon::VeryLongTerm
        };
        
        // Create a new goal
        let goal_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let goal = FinancialGoal {
            id: goal_id.clone(),
            name: name.to_string(),
            goal_type,
            description: description.to_string(),
            target_amount,
            current_amount: 0.0,
            target_date,
            time_horizon,
            priority,
            status: GoalStatus::NotStarted,
            monthly_contribution: 0.0,
            required_return_rate: None,
            success_probability: None,
            associated_accounts: Vec::new(),
            risk_tolerance: None,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        };
        
        // Add the goal to the profile
        let profile = self.profiles.get_mut(client_id).ok_or_else(|| anyhow!("Client profile not found"))?;
        profile.goals.push(goal);
        profile.updated_at = now;
        
        info!(client_id = %client_id, goal_id = %goal_id, goal_name = %name, "Added financial goal to client profile");
        
        Ok(goal_id)
    }
    
    /// Update a financial goal
    pub fn update_goal(&mut self, client_id: &str, goal_id: &str, update_fn: impl FnOnce(&mut FinancialGoal)) -> Result<()> {
        let profile = self.profiles.get_mut(client_id).ok_or_else(|| anyhow!("Client profile not found"))?;
        
        // Find the goal
        let goal = profile.goals.iter_mut()
            .find(|g| g.id == goal_id)
            .ok_or_else(|| anyhow!("Financial goal not found"))?;
        
        // Apply the update
        update_fn(goal);
        
        // Update timestamps
        goal.updated_at = Utc::now();
        profile.updated_at = Utc::now();
        
        info!(client_id = %client_id, goal_id = %goal_id, "Updated financial goal");
        
        Ok(())
    }
    
    /// Delete a financial goal
    pub fn delete_goal(&mut self, client_id: &str, goal_id: &str) -> Result<()> {
        let profile = self.profiles.get_mut(client_id).ok_or_else(|| anyhow!("Client profile not found"))?;
        
        // Find the goal index
        let goal_index = profile.goals.iter()
            .position(|g| g.id == goal_id)
            .ok_or_else(|| anyhow!("Financial goal not found"))?;
        
        // Remove the goal
        profile.goals.remove(goal_index);
        profile.updated_at = Utc::now();
        
        info!(client_id = %client_id, goal_id = %goal_id, "Deleted financial goal");
        
        Ok(())
    }
    
    /// Calculate net worth for a client
    pub fn calculate_net_worth(&self, client_id: &str) -> Result<f64> {
        let profile = self.get_profile(client_id)?;
        
        // Sum all assets
        let total_assets = profile.assets.iter()
            .map(|asset| asset.current_value)
            .sum::<f64>();
        
        // Sum all liabilities
        let total_liabilities = profile.liabilities.iter()
            .map(|liability| liability.current_balance)
            .sum::<f64>();
        
        // Calculate net worth
        let net_worth = total_assets - total_liabilities;
        
        Ok(net_worth)
    }
    
    /// Calculate monthly cash flow for a client
    pub fn calculate_monthly_cash_flow(&self, client_id: &str) -> Result<f64> {
        let profile = self.get_profile(client_id)?;
        
        // Calculate monthly income
        let monthly_income = profile.income_sources.iter()
            .map(|income| {
                match income.frequency.to_lowercase().as_str() {
                    "monthly" => income.annual_amount / 12.0,
                    "bi-weekly" => income.annual_amount / 26.0 * 2.0,
                    "weekly" => income.annual_amount / 52.0 * 4.0,
                    "annual" | "yearly" => income.annual_amount / 12.0,
                    "semi-monthly" => income.annual_amount / 24.0 * 2.0,
                    "quarterly" => income.annual_amount / 4.0 / 3.0,
                    _ => income.annual_amount / 12.0, // Default to monthly
                }
            })
            .sum::<f64>();
        
        // Calculate monthly expenses
        let monthly_expenses = profile.expenses.iter()
            .map(|expense| expense.monthly_amount)
            .sum::<f64>();
        
        // Calculate net cash flow
        let net_cash_flow = monthly_income - monthly_expenses;
        
        Ok(net_cash_flow)
    }
    
    /// Calculate goal funding status
    pub fn calculate_goal_funding_status(&self, client_id: &str, goal_id: &str) -> Result<f64> {
        let profile = self.get_profile(client_id)?;
        
        // Find the goal
        let goal = profile.goals.iter()
            .find(|g| g.id == goal_id)
            .ok_or_else(|| anyhow!("Financial goal not found"))?;
        
        // Calculate goal funding status
        let funding_status = goal.current_amount / goal.target_amount;
        
        Ok(funding_status)
    }
}

/// Risk profile question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskProfileQuestion {
    /// Question ID
    pub id: String,
    
    /// Question text
    pub text: String,
    
    /// Answer options
    pub options: Vec<RiskProfileAnswerOption>,
    
    /// Category (e.g., "risk_capacity", "risk_willingness", "time_horizon")
    pub category: String,
    
    /// Weight in overall risk score calculation
    pub weight: f64,
}

/// Risk profile answer option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskProfileAnswerOption {
    /// Option value (typically 1-5)
    pub value: i32,
    
    /// Option text
    pub text: String,
    
    /// Risk score contribution
    pub risk_score: f64,
}

/// Risk profile questionnaire
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskProfileQuestionnaire {
    /// Questionnaire ID
    pub id: String,
    
    /// Questionnaire name
    pub name: String,
    
    /// Questionnaire description
    pub description: String,
    
    /// Questions
    pub questions: Vec<RiskProfileQuestion>,
}

/// Goal template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalTemplate {
    /// Template ID
    pub id: String,
    
    /// Template name
    pub name: String,
    
    /// Goal type
    pub goal_type: GoalType,
    
    /// Template description
    pub description: String,
    
    /// Default time horizon
    pub default_time_horizon: TimeHorizon,
    
    /// Default priority
    pub default_priority: GoalPriority,
    
    /// Suggested target amount formula (e.g., "income * 0.5" for emergency fund)
    pub target_amount_formula: Option<String>,
    
    /// Suggested monthly contribution formula
    pub monthly_contribution_formula: Option<String>,
    
    /// Life stage applicability (e.g., "young_adult", "family_formation", "pre_retirement", "retirement")
    pub life_stages: Vec<String>,
    
    /// Recommended risk tolerance
    pub recommended_risk_tolerance: Option<RiskToleranceLevel>,
}

/// Life stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifeStage {
    /// Young adult (20-35)
    YoungAdult,
    
    /// Family formation (30-45)
    FamilyFormation,
    
    /// Peak earning years (40-55)
    PeakEarnings,
    
    /// Pre-retirement (55-65)
    PreRetirement,
    
    /// Retirement (65+)
    Retirement,
}

/// Risk profiling service
pub struct RiskProfilingService {
    /// Risk profile questionnaires
    questionnaires: HashMap<String, RiskProfileQuestionnaire>,
}

impl RiskProfilingService {
    /// Create a new risk profiling service
    pub fn new() -> Self {
        let mut service = Self {
            questionnaires: HashMap::new(),
        };
        
        // Initialize with default questionnaire
        service.initialize_default_questionnaire();
        
        service
    }
    
    /// Initialize default risk profile questionnaire
    fn initialize_default_questionnaire(&mut self) {
        let questionnaire = RiskProfileQuestionnaire {
            id: "default".to_string(),
            name: "Standard Risk Profile Assessment".to_string(),
            description: "Standard questionnaire to assess risk tolerance and capacity".to_string(),
            questions: vec![
                RiskProfileQuestion {
                    id: "time_horizon".to_string(),
                    text: "When do you expect to need the money you're investing?".to_string(),
                    options: vec![
                        RiskProfileAnswerOption {
                            value: 1,
                            text: "Within the next 3 years".to_string(),
                            risk_score: 1.0,
                        },
                        RiskProfileAnswerOption {
                            value: 2,
                            text: "3-5 years".to_string(),
                            risk_score: 2.0,
                        },
                        RiskProfileAnswerOption {
                            value: 3,
                            text: "6-10 years".to_string(),
                            risk_score: 3.0,
                        },
                        RiskProfileAnswerOption {
                            value: 4,
                            text: "11-20 years".to_string(),
                            risk_score: 4.0,
                        },
                        RiskProfileAnswerOption {
                            value: 5,
                            text: "More than 20 years".to_string(),
                            risk_score: 5.0,
                        },
                    ],
                    category: "time_horizon".to_string(),
                    weight: 1.0,
                },
                RiskProfileQuestion {
                    id: "market_decline".to_string(),
                    text: "If your investments suddenly declined 20% in value, what would you do?".to_string(),
                    options: vec![
                        RiskProfileAnswerOption {
                            value: 1,
                            text: "Sell all remaining investments to prevent further losses".to_string(),
                            risk_score: 1.0,
                        },
                        RiskProfileAnswerOption {
                            value: 2,
                            text: "Sell some investments to reduce exposure".to_string(),
                            risk_score: 2.0,
                        },
                        RiskProfileAnswerOption {
                            value: 3,
                            text: "Do nothing and wait for recovery".to_string(),
                            risk_score: 3.0,
                        },
                        RiskProfileAnswerOption {
                            value: 4,
                            text: "Invest a little more to take advantage of lower prices".to_string(),
                            risk_score: 4.0,
                        },
                        RiskProfileAnswerOption {
                            value: 5,
                            text: "Invest significantly more to take advantage of the opportunity".to_string(),
                            risk_score: 5.0,
                        },
                    ],
                    category: "risk_willingness".to_string(),
                    weight: 1.5,
                },
                RiskProfileQuestion {
                    id: "income_stability".to_string(),
                    text: "How stable is your current and future income?".to_string(),
                    options: vec![
                        RiskProfileAnswerOption {
                            value: 1,
                            text: "Very unstable - I could lose my job or income source at any time".to_string(),
                            risk_score: 1.0,
                        },
                        RiskProfileAnswerOption {
                            value: 2,
                            text: "Somewhat unstable - My income fluctuates significantly".to_string(),
                            risk_score: 2.0,
                        },
                        RiskProfileAnswerOption {
                            value: 3,
                            text: "Moderately stable - My income is generally reliable but not guaranteed".to_string(),
                            risk_score: 3.0,
                        },
                        RiskProfileAnswerOption {
                            value: 4,
                            text: "Stable - My income is very reliable".to_string(),
                            risk_score: 4.0,
                        },
                        RiskProfileAnswerOption {
                            value: 5,
                            text: "Very stable - My income is guaranteed or I have multiple reliable sources".to_string(),
                            risk_score: 5.0,
                        },
                    ],
                    category: "risk_capacity".to_string(),
                    weight: 1.2,
                },
                RiskProfileQuestion {
                    id: "emergency_fund".to_string(),
                    text: "How many months of expenses do you have saved in an emergency fund?".to_string(),
                    options: vec![
                        RiskProfileAnswerOption {
                            value: 1,
                            text: "None".to_string(),
                            risk_score: 1.0,
                        },
                        RiskProfileAnswerOption {
                            value: 2,
                            text: "1-2 months".to_string(),
                            risk_score: 2.0,
                        },
                        RiskProfileAnswerOption {
                            value: 3,
                            text: "3-5 months".to_string(),
                            risk_score: 3.0,
                        },
                        RiskProfileAnswerOption {
                            value: 4,
                            text: "6-12 months".to_string(),
                            risk_score: 4.0,
                        },
                        RiskProfileAnswerOption {
                            value: 5,
                            text: "More than 12 months".to_string(),
                            risk_score: 5.0,
                        },
                    ],
                    category: "risk_capacity".to_string(),
                    weight: 1.0,
                },
                RiskProfileQuestion {
                    id: "investment_knowledge".to_string(),
                    text: "How would you rate your investment knowledge?".to_string(),
                    options: vec![
                        RiskProfileAnswerOption {
                            value: 1,
                            text: "None - I'm a beginner".to_string(),
                            risk_score: 1.0,
                        },
                        RiskProfileAnswerOption {
                            value: 2,
                            text: "Limited - I understand basic concepts".to_string(),
                            risk_score: 2.0,
                        },
                        RiskProfileAnswerOption {
                            value: 3,
                            text: "Moderate - I understand diversification and asset allocation".to_string(),
                            risk_score: 3.0,
                        },
                        RiskProfileAnswerOption {
                            value: 4,
                            text: "Good - I understand various investment vehicles and strategies".to_string(),
                            risk_score: 4.0,
                        },
                        RiskProfileAnswerOption {
                            value: 5,
                            text: "Excellent - I have professional investment knowledge".to_string(),
                            risk_score: 5.0,
                        },
                    ],
                    category: "risk_willingness".to_string(),
                    weight: 0.8,
                },
            ],
        };
        
        self.questionnaires.insert(questionnaire.id.clone(), questionnaire);
    }
    
    /// Get a questionnaire by ID
    pub fn get_questionnaire(&self, id: &str) -> Option<&RiskProfileQuestionnaire> {
        self.questionnaires.get(id)
    }
    
    /// Calculate risk tolerance based on questionnaire responses
    pub fn calculate_risk_tolerance(&self, responses: &[RiskProfileResponse], questionnaire_id: &str) -> Result<RiskToleranceLevel> {
        let questionnaire = self.get_questionnaire(questionnaire_id)
            .ok_or_else(|| anyhow!("Questionnaire not found"))?;
        
        // Create a map of question IDs to questions for easy lookup
        let questions_map: HashMap<String, &RiskProfileQuestion> = questionnaire.questions.iter()
            .map(|q| (q.id.clone(), q))
            .collect();
        
        let mut total_weighted_score = 0.0;
        let mut total_weight = 0.0;
        
        // Calculate weighted score
        for response in responses {
            if let Some(question) = questions_map.get(&response.question_id) {
                // Find the option that matches the response value
                if let Some(option) = question.options.iter().find(|o| o.value == response.response_value) {
                    total_weighted_score += option.risk_score * question.weight;
                    total_weight += question.weight;
                }
            }
        }
        
        // Calculate average score
        let average_score = if total_weight > 0.0 {
            total_weighted_score / total_weight
        } else {
            0.0
        };
        
        // Map average score to risk tolerance level
        let risk_tolerance = match average_score {
            score if score < 1.5 => RiskToleranceLevel::VeryConservative,
            score if score < 2.5 => RiskToleranceLevel::Conservative,
            score if score < 3.5 => RiskToleranceLevel::Moderate,
            score if score < 4.5 => RiskToleranceLevel::Aggressive,
            _ => RiskToleranceLevel::VeryAggressive,
        };
        
        Ok(risk_tolerance)
    }
    
    /// Detect behavioral biases based on questionnaire responses
    pub fn detect_behavioral_biases(&self, responses: &[RiskProfileResponse]) -> Vec<BehavioralBiasType> {
        let mut biases = Vec::new();
        
        // This is a simplified implementation
        // In a real system, this would involve more sophisticated analysis
        
        // Check for loss aversion
        if let Some(response) = responses.iter().find(|r| r.question_id == "market_decline") {
            if response.response_value <= 2 {
                biases.push(BehavioralBiasType::LossAversion);
            }
        }
        
        // Check for overconfidence
        if let Some(response) = responses.iter().find(|r| r.question_id == "investment_knowledge") {
            if response.response_value >= 4 {
                biases.push(BehavioralBiasType::Overconfidence);
            }
        }
        
        biases
    }
}

/// Goal template service
pub struct GoalTemplateService {
    /// Goal templates
    templates: HashMap<String, GoalTemplate>,
}

impl GoalTemplateService {
    /// Create a new goal template service
    pub fn new() -> Self {
        let mut service = Self {
            templates: HashMap::new(),
        };
        
        // Initialize with default templates
        service.initialize_default_templates();
        
        service
    }
    
    /// Initialize default goal templates
    fn initialize_default_templates(&mut self) {
        let templates = vec![
            GoalTemplate {
                id: "retirement".to_string(),
                name: "Retirement".to_string(),
                goal_type: GoalType::Retirement,
                description: "Save for a comfortable retirement".to_string(),
                default_time_horizon: TimeHorizon::VeryLongTerm,
                default_priority: GoalPriority::Essential,
                target_amount_formula: Some("annual_income * 25".to_string()),
                monthly_contribution_formula: Some("annual_income * 0.15 / 12".to_string()),
                life_stages: vec![
                    "YoungAdult".to_string(),
                    "FamilyFormation".to_string(),
                    "PeakEarnings".to_string(),
                    "PreRetirement".to_string(),
                ],
                recommended_risk_tolerance: Some(RiskToleranceLevel::Moderate),
            },
            GoalTemplate {
                id: "emergency_fund".to_string(),
                name: "Emergency Fund".to_string(),
                goal_type: GoalType::EmergencyFund,
                description: "Build an emergency fund for unexpected expenses".to_string(),
                default_time_horizon: TimeHorizon::ShortTerm,
                default_priority: GoalPriority::Essential,
                target_amount_formula: Some("monthly_expenses * 6".to_string()),
                monthly_contribution_formula: Some("monthly_income * 0.1".to_string()),
                life_stages: vec![
                    "YoungAdult".to_string(),
                    "FamilyFormation".to_string(),
                    "PeakEarnings".to_string(),
                    "PreRetirement".to_string(),
                    "Retirement".to_string(),
                ],
                recommended_risk_tolerance: Some(RiskToleranceLevel::VeryConservative),
            },
            GoalTemplate {
                id: "home_purchase".to_string(),
                name: "Home Purchase".to_string(),
                goal_type: GoalType::HomePurchase {
                    property_type: "primary residence".to_string(),
                    location: None,
                },
                description: "Save for a down payment on a home".to_string(),
                default_time_horizon: TimeHorizon::MediumTerm,
                default_priority: GoalPriority::Important,
                target_amount_formula: Some("annual_income * 1.5".to_string()),
                monthly_contribution_formula: Some("monthly_income * 0.2".to_string()),
                life_stages: vec![
                    "YoungAdult".to_string(),
                    "FamilyFormation".to_string(),
                ],
                recommended_risk_tolerance: Some(RiskToleranceLevel::Conservative),
            },
            GoalTemplate {
                id: "education".to_string(),
                name: "Education Funding".to_string(),
                goal_type: GoalType::Education {
                    beneficiary: "Child".to_string(),
                    education_level: "undergraduate".to_string(),
                },
                description: "Save for education expenses".to_string(),
                default_time_horizon: TimeHorizon::LongTerm,
                default_priority: GoalPriority::Important,
                target_amount_formula: Some("100000".to_string()),
                monthly_contribution_formula: Some("target_amount / (years_to_goal * 12)".to_string()),
                life_stages: vec![
                    "FamilyFormation".to_string(),
                    "PeakEarnings".to_string(),
                ],
                recommended_risk_tolerance: Some(RiskToleranceLevel::Moderate),
            },
        ];
        
        for template in templates {
            self.templates.insert(template.id.clone(), template);
        }
    }
    
    /// Get a template by ID
    pub fn get_template(&self, id: &str) -> Option<&GoalTemplate> {
        self.templates.get(id)
    }
    
    /// Get templates by life stage
    pub fn get_templates_by_life_stage(&self, life_stage: &str) -> Vec<&GoalTemplate> {
        self.templates.values()
            .filter(|t| t.life_stages.contains(&life_stage.to_string()))
            .collect()
    }
    
    /// Create a goal from a template
    pub fn create_goal_from_template(&self, template_id: &str, client_profile: &ClientProfile, 
                                     target_date: NaiveDate, custom_name: Option<&str>) -> Result<FinancialGoal> {
        let template = self.get_template(template_id)
            .ok_or_else(|| anyhow!("Goal template not found"))?;
        
        // Calculate target amount based on formula if available
        let target_amount = match &template.target_amount_formula {
            Some(formula) => self.calculate_formula_value(formula, client_profile)?,
            None => 0.0, // Default value, should be overridden by caller
        };
        
        // Calculate monthly contribution based on formula if available
        let monthly_contribution = match &template.monthly_contribution_formula {
            Some(formula) => self.calculate_formula_value(formula, client_profile)?,
            None => 0.0, // Default value, should be overridden by caller
        };
        
        // Calculate time horizon based on target date
        let today = Utc::now().date_naive();
        let years_to_goal = (target_date.year() - today.year()) as f64 + 
                           (target_date.ordinal() as f64 - today.ordinal() as f64) / 365.0;
        
        let time_horizon = if years_to_goal <= 2.0 {
            TimeHorizon::ShortTerm
        } else if years_to_goal <= 5.0 {
            TimeHorizon::MediumTerm
        } else if years_to_goal <= 10.0 {
            TimeHorizon::LongTerm
        } else {
            TimeHorizon::VeryLongTerm
        };
        
        // Create the goal
        let goal_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let goal = FinancialGoal {
            id: goal_id,
            name: custom_name.unwrap_or(&template.name).to_string(),
            goal_type: template.goal_type.clone(),
            description: template.description.clone(),
            target_amount,
            current_amount: 0.0,
            target_date,
            time_horizon,
            priority: template.default_priority,
            status: GoalStatus::NotStarted,
            monthly_contribution,
            required_return_rate: None,
            success_probability: None,
            associated_accounts: Vec::new(),
            risk_tolerance: template.recommended_risk_tolerance,
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        };
        
        Ok(goal)
    }
    
    /// Calculate a value based on a formula and client profile
    fn calculate_formula_value(&self, formula: &str, client_profile: &ClientProfile) -> Result<f64> {
        // This is a simplified implementation
        // In a real system, this would involve a proper formula parser
        
        // Get some basic values from the profile
        let annual_income: f64 = client_profile.income_sources.iter()
            .map(|source| source.annual_amount)
            .sum();
        
        let monthly_income = annual_income / 12.0;
        
        let monthly_expenses: f64 = client_profile.expenses.iter()
            .map(|expense| expense.monthly_amount)
            .sum();
        
        // Very simple formula evaluation
        match formula {
            "annual_income * 25" => Ok(annual_income * 25.0),
            "annual_income * 1.5" => Ok(annual_income * 1.5),
            "monthly_expenses * 6" => Ok(monthly_expenses * 6.0),
            "monthly_income * 0.1" => Ok(monthly_income * 0.1),
            "monthly_income * 0.2" => Ok(monthly_income * 0.2),
            "annual_income * 0.15 / 12" => Ok(annual_income * 0.15 / 12.0),
            "100000" => Ok(100000.0),
            _ => Err(anyhow!("Unsupported formula: {}", formula)),
        }
    }
}

/// Financial analysis service
pub struct FinancialAnalysisService;

impl FinancialAnalysisService {
    /// Create a new financial analysis service
    pub fn new() -> Self {
        Self
    }
    
    /// Calculate net worth for a client
    pub fn calculate_net_worth(&self, client_profile: &ClientProfile) -> f64 {
        // Sum all assets
        let total_assets = client_profile.assets.iter()
            .map(|asset| asset.current_value)
            .sum::<f64>();
        
        // Sum all liabilities
        let total_liabilities = client_profile.liabilities.iter()
            .map(|liability| liability.current_balance)
            .sum::<f64>();
        
        // Calculate net worth
        total_assets - total_liabilities
    }
    
    /// Calculate monthly cash flow for a client
    pub fn calculate_monthly_cash_flow(&self, client_profile: &ClientProfile) -> f64 {
        // Calculate monthly income
        let monthly_income = client_profile.income_sources.iter()
            .map(|income| {
                match income.frequency.to_lowercase().as_str() {
                    "monthly" => income.annual_amount / 12.0,
                    "bi-weekly" => income.annual_amount / 26.0 * 2.0,
                    "weekly" => income.annual_amount / 52.0 * 4.0,
                    "annual" | "yearly" => income.annual_amount / 12.0,
                    "semi-monthly" => income.annual_amount / 24.0 * 2.0,
                    "quarterly" => income.annual_amount / 4.0 / 3.0,
                    _ => income.annual_amount / 12.0, // Default to monthly
                }
            })
            .sum::<f64>();
        
        // Calculate monthly expenses
        let monthly_expenses = client_profile.expenses.iter()
            .map(|expense| expense.monthly_amount)
            .sum::<f64>();
        
        // Calculate net cash flow
        monthly_income - monthly_expenses
    }
    
    /// Calculate debt-to-income ratio
    pub fn calculate_debt_to_income_ratio(&self, client_profile: &ClientProfile) -> f64 {
        // Calculate monthly income
        let monthly_income = client_profile.income_sources.iter()
            .map(|income| income.annual_amount / 12.0)
            .sum::<f64>();
        
        if monthly_income == 0.0 {
            return 0.0;
        }
        
        // Calculate monthly debt payments
        let monthly_debt_payments = client_profile.liabilities.iter()
            .map(|liability| liability.minimum_payment)
            .sum::<f64>();
        
        // Calculate debt-to-income ratio
        monthly_debt_payments / monthly_income
    }
    
    /// Calculate savings rate
    pub fn calculate_savings_rate(&self, client_profile: &ClientProfile) -> f64 {
        // Calculate monthly income
        let monthly_income = client_profile.income_sources.iter()
            .map(|income| income.annual_amount / 12.0)
            .sum::<f64>();
        
        if monthly_income == 0.0 {
            return 0.0;
        }
        
        // Calculate monthly savings
        let monthly_savings = client_profile.expenses.iter()
            .filter(|expense| expense.category == ExpenseCategory::SavingsAndInvestments)
            .map(|expense| expense.monthly_amount)
            .sum::<f64>();
        
        // Calculate savings rate
        monthly_savings / monthly_income
    }
    
    /// Calculate emergency fund coverage
    pub fn calculate_emergency_fund_coverage(&self, client_profile: &ClientProfile) -> f64 {
        // Calculate monthly essential expenses
        let monthly_essential_expenses = client_profile.expenses.iter()
            .filter(|expense| expense.is_essential)
            .map(|expense| expense.monthly_amount)
            .sum::<f64>();
        
        if monthly_essential_expenses == 0.0 {
            return 0.0;
        }
        
        // Calculate liquid assets
        let liquid_assets = client_profile.assets.iter()
            .filter(|asset| asset.is_liquid && asset.asset_type == AssetType::Cash)
            .map(|asset| asset.current_value)
            .sum::<f64>();
        
        // Calculate emergency fund coverage in months
        liquid_assets / monthly_essential_expenses
    }
    
    /// Calculate goal funding status
    pub fn calculate_goal_funding_status(&self, goal: &FinancialGoal) -> f64 {
        goal.current_amount / goal.target_amount
    }
} 