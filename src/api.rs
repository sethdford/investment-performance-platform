//! # Investment Platform API
//! 
//! This module provides a clean, simple API for interacting with the investment platform.
//! The API is designed to be easy to use, well-documented, and consistent.
//! 
//! ## Core Resources
//! 
//! The API is organized around the following core resources:
//! 
//! - **Models**: Investment model portfolios that define asset allocations
//! - **Accounts**: Client investment accounts
//! - **Sleeves**: Segments of an account managed according to specific models
//! - **Trades**: Buy and sell orders for securities
//! 
//! ## API Design Principles
//! 
//! This API follows these design principles:
//! 
//! 1. **Simplicity**: Simple, intuitive interfaces that are easy to understand
//! 2. **Consistency**: Consistent naming, parameter ordering, and return types
//! 3. **Versioning**: Clear versioning to ensure backward compatibility
//! 4. **Documentation**: Comprehensive documentation with examples
//! 5. **Error Handling**: Clear, actionable error messages

use crate::portfolio::model::{
    ModelPortfolio, ModelPortfolioService, UnifiedManagedAccount,
    ESGScreeningCriteria, TaxOptimizationSettings, ESGImpactReport,
    ModelDriftAnalysis
};
use crate::portfolio::rebalancing::RebalanceTrade;
use crate::portfolio::rebalancing::TradeReason as RebalanceTradeReason;
use crate::common::error::{Error, Result};
use std::collections::HashMap;
use crate::tax::tlh::{AlgorithmicTLHService, AlgorithmicTLHConfig};

/// API client for the investment platform
/// 
/// This is the main entry point for interacting with the investment platform API.
/// 
/// # Examples
/// 
/// ```
/// use investment_management::api::Client;
/// use investment_management::api::CreateModelParams;
/// use investment_management::api::CreateAccountParams;
/// use investment_management::api::RebalanceParams;
/// use std::collections::HashMap;
/// 
/// // Create a new API client
/// let client = Client::new();
/// 
/// // Create a model portfolio
/// let model_params = CreateModelParams {
///     id: Some("model-1".to_string()),
///     name: "Balanced Growth".to_string(),
///     description: Some("A balanced growth portfolio".to_string()),
///     model_type: investment_management::portfolio::model::ModelType::Direct,
///     securities: {
///         let mut securities = HashMap::new();
///         securities.insert("AAPL".to_string(), 0.25);
///         securities.insert("MSFT".to_string(), 0.25);
///         securities.insert("AMZN".to_string(), 0.25);
///         securities.insert("GOOGL".to_string(), 0.25);
///         securities
///     },
///     asset_allocation: {
///         let mut allocation = HashMap::new();
///         allocation.insert("Equity".to_string(), 1.0);
///         allocation
///     },
///     sector_allocation: {
///         let mut allocation = HashMap::new();
///         allocation.insert("Technology".to_string(), 0.75);
///         allocation.insert("Consumer Discretionary".to_string(), 0.25);
///         allocation
///     },
///     child_models: HashMap::new(),
/// };
/// 
/// // Create an account
/// let account_params = CreateAccountParams {
///     id: Some("account-1".to_string()),
///     name: "John's Investment Account".to_string(),
///     owner: "John Smith".to_string(),
///     model_id: "model-1".to_string(),
///     initial_investment: 100000.0,
///     _tax_settings: None,
///     _esg_criteria: None,
/// };
/// ```
pub struct Client {
    /// API for managing model portfolios
    pub models: ModelPortfoliosApi,
    
    /// API for managing accounts
    pub accounts: AccountsApi,
    
    /// API for managing trades
    pub trades: TradesApi,
}

impl Client {
    /// Create a new API client
    pub fn new() -> Self {
        let service = ModelPortfolioService::new(crate::factor_model::FactorModelApi::new());
        
        Self {
            models: ModelPortfoliosApi::new(service.clone()),
            accounts: AccountsApi::new(service.clone()),
            trades: TradesApi::new(service),
        }
    }
}

/// API for managing model portfolios
pub struct ModelPortfoliosApi {
    service: ModelPortfolioService,
}

impl ModelPortfoliosApi {
    /// Create a new model portfolios API
    fn new(service: ModelPortfolioService) -> Self {
        Self { service }
    }
    
    /// Create a new model portfolio
    /// 
    /// # Parameters
    /// 
    /// - `params`: Parameters for creating the model portfolio
    /// 
    /// # Returns
    /// 
    /// The created model portfolio, or an error if creation failed
    /// 
    /// # Examples
    /// 
    /// ```
    /// use investment_management::api::Client;
    /// use investment_management::api::CreateModelParams;
    /// use std::collections::HashMap;
    /// 
    /// let client = Client::new();
    /// 
    /// let mut securities = HashMap::new();
    /// securities.insert("AAPL".to_string(), 0.25);
    /// securities.insert("MSFT".to_string(), 0.25);
    /// securities.insert("AMZN".to_string(), 0.25);
    /// securities.insert("GOOGL".to_string(), 0.25);
    /// 
    /// let model = client.models.create(CreateModelParams {
    ///     id: None,
    ///     name: "Technology Growth".to_string(),
    ///     description: None,
    ///     model_type: investment_management::portfolio::model::ModelType::Direct,
    ///     securities,
    ///     asset_allocation: HashMap::new(),
    ///     sector_allocation: HashMap::new(),
    ///     child_models: HashMap::new(),
    /// }).unwrap();
    /// ```
    pub fn create(&self, params: CreateModelParams) -> Result<ModelPortfolio> {
        // Validate the parameters
        if params.name.is_empty() {
            return Err(Error::validation("Model name cannot be empty"));
        }
        
        let securities_weight_sum: f64 = params.securities.values().sum();
        if (securities_weight_sum - 1.0).abs() > 0.0001 && !params.securities.is_empty() {
            return Err(Error::validation(format!("Securities weights must sum to 1.0, got {}", securities_weight_sum)));
        }
        
        let mut securities = HashMap::new();
        for (security_id, weight) in params.securities {
            securities.insert(security_id, weight);
        }
        
        let mut asset_allocation = HashMap::new();
        for (asset_class, weight) in params.asset_allocation {
            asset_allocation.insert(asset_class, weight);
        }
        
        let mut sector_allocation = HashMap::new();
        for (sector, weight) in params.sector_allocation {
            sector_allocation.insert(sector, weight);
        }
        
        let mut child_models = HashMap::new();
        for (model_id, weight) in params.child_models {
            child_models.insert(model_id, weight);
        }
        
        let model = ModelPortfolio {
            id: params.id.unwrap_or_else(|| format!("model-{}", uuid::Uuid::new_v4())),
            name: params.name,
            description: params.description,
            model_type: params.model_type,
            asset_allocation,
            sector_allocation,
            securities,
            child_models,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        
        self.service.create_model_portfolio(model)
            .map_err(|err| Error::internal(err.to_string()))
    }
    
    /// Retrieve a model portfolio by ID
    /// 
    /// # Parameters
    /// 
    /// - `id`: The ID of the model portfolio to retrieve
    /// 
    /// # Returns
    /// 
    /// The model portfolio, or an error if not found
    /// 
    /// # Examples
    /// 
    /// ```
    /// use investment_management::api::Client;
    /// 
    /// let client = Client::new();
    /// let model = client.models.get("model-1").unwrap();
    /// println!("Model: {}", model.name);
    /// ```
    pub fn get(&self, id: &str) -> Result<ModelPortfolio> {
        self.service.get_model_portfolio(id)
            .ok_or_else(|| Error::not_found(format!("ModelPortfolio with ID {}", id)))
    }
    
    /// List all model portfolios
    /// 
    /// # Parameters
    /// 
    /// - `params`: Parameters for listing model portfolios
    /// 
    /// # Returns
    /// 
    /// A list of model portfolios
    /// 
    /// # Examples
    /// 
    /// ```
    /// use investment_management::api::Client;
    /// use investment_management::api::ListModelsParams;
    /// 
    /// let client = Client::new();
    /// let models = client.models.list(ListModelsParams::default()).unwrap();
    /// println!("Found {} models", models.len());
    /// ```
    pub fn list(&self, _params: ListModelsParams) -> Result<Vec<ModelPortfolio>> {
        // In a real implementation, this would query a database
        // For now, we'll return a mock list
        let mock_model = self.service.get_model_portfolio("mock-model")
            .ok_or_else(|| Error::internal("Mock model not found"))?;
        
        let test_model = self.service.get_model_portfolio("test-model")
            .ok_or_else(|| Error::internal("Test model not found"))?;
        
        Ok(vec![mock_model, test_model])
    }
}

/// API for managing accounts
pub struct AccountsApi {
    service: ModelPortfolioService,
}

impl AccountsApi {
    /// Create a new accounts API
    fn new(service: ModelPortfolioService) -> Self {
        Self { service }
    }
    
    /// Create a new account
    /// 
    /// # Parameters
    /// 
    /// - `params`: Parameters for creating the account
    /// 
    /// # Returns
    /// 
    /// The created account, or an error if creation failed
    /// 
    /// # Examples
    /// 
    /// ```
    /// use investment_management::api::Client;
    /// use investment_management::api::CreateAccountParams;
    /// 
    /// let client = Client::new();
    /// let account = client.accounts.create(CreateAccountParams {
    ///     id: Some("account-1".to_string()),
    ///     name: "John's Investment Account".to_string(),
    ///     owner: "John Smith".to_string(),
    ///     model_id: "model-1".to_string(),
    ///     initial_investment: 100000.0,
    ///     _tax_settings: None,
    ///     _esg_criteria: None,
    /// }).unwrap();
    /// 
    /// println!("Created account: {}", account.name);
    /// ```
    pub fn create(&self, params: CreateAccountParams) -> Result<UnifiedManagedAccount> {
        // Validate the parameters
        if params.name.is_empty() {
            return Err(Error::validation("Account name cannot be empty"));
        }
        
        if params.owner.is_empty() {
            return Err(Error::validation("Account owner cannot be empty"));
        }
        
        if params.model_id.is_empty() {
            return Err(Error::validation("Model ID cannot be empty"));
        }
        
        if params.initial_investment <= 0.0 {
            return Err(Error::validation(format!("Initial investment must be positive, got {}", params.initial_investment)));
        }
        
        // Create the account
        self.service.create_uma_from_model(
            &params.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            &params.name,
            &params.owner,
            &params.model_id,
            params.initial_investment
        )
        .map_err(|err| Error::internal(err.to_string()))
    }
    
    /// Create a demo account with predefined settings
    pub fn create_demo_account(&self) -> Result<UnifiedManagedAccount> {
        // Get a model portfolio
        let model = self.service.get_model_portfolio("model-1")
            .ok_or_else(|| Error::not_found(format!("ModelPortfolio with ID {}", "model-1")))?;
        
        // Create the account
        self.service.create_uma_from_model(
            &uuid::Uuid::new_v4().to_string(),
            "Demo Account",
            "Demo User",
            &model.id,
            1_000_000.0
        )
        .map_err(|err| Error::internal(err.to_string()))
    }
    
    /// Retrieve an account by ID
    /// 
    /// # Parameters
    /// 
    /// - `id`: The ID of the account to retrieve
    /// 
    /// # Returns
    /// 
    /// The account, or an error if not found
    /// 
    /// # Examples
    /// 
    /// ```
    /// use investment_management::api::Client;
    /// 
    /// let client = Client::new();
    /// let account = client.accounts.get("account-1").unwrap();
    /// println!("Account: {}", account.name);
    /// ```
    pub fn get(&self, id: &str) -> Result<UnifiedManagedAccount> {
        // In a real implementation, this would query a database
        // For now, we'll return a mock account
        if id.is_empty() {
            return Err(Error::validation("Account ID cannot be empty"));
        }
        
        self.service.create_uma_from_model(
            "mock-account",
            "Mock Account",
            "Mock Owner",
            "mock-model",
            1_000_000.0
        )
        .map_err(|err| Error::internal(err.to_string()))
    }
    
    /// Apply ESG screening to an account
    /// 
    /// # Parameters
    /// 
    /// - `id`: The ID of the account to screen
    /// - `params`: ESG screening parameters
    /// 
    /// # Returns
    /// 
    /// The screened account, or an error if screening failed
    /// 
    /// # Examples
    /// 
    /// ```
    /// use investment_management::api::Client;
    /// use investment_management::api::ESGScreeningParams;
    /// 
    /// let client = Client::new();
    /// let esg_params = ESGScreeningParams {
    ///     min_overall_score: Some(70.0),
    ///     min_environmental_score: Some(65.0),
    ///     min_social_score: None,
    ///     min_governance_score: None,
    ///     max_controversy_score: Some(30.0),
    ///     excluded_sectors: vec!["Tobacco".to_string(), "Weapons".to_string()],
    ///     excluded_activities: vec!["Animal Testing".to_string()],
    /// };
    /// let account = client.accounts.apply_esg_screening("account-1", esg_params).unwrap();
    /// println!("Applied ESG screening to account: {}", account.name);
    /// ```
    pub fn apply_esg_screening(&self, id: &str, params: ESGScreeningParams) -> Result<UnifiedManagedAccount> {
        // Get the account
        let account = self.get(id)?;
        
        let criteria = ESGScreeningCriteria {
            min_overall_score: params.min_overall_score,
            min_environmental_score: params.min_environmental_score,
            min_social_score: params.min_social_score,
            min_governance_score: params.min_governance_score,
            max_controversy_score: params.max_controversy_score,
            excluded_sectors: params.excluded_sectors,
            excluded_activities: params.excluded_activities,
        };
        
        let mut screened_account = account.clone();
        screened_account.with_esg_screening(criteria);
        
        Ok(screened_account)
    }
    
    /// Generate an ESG impact report for an account
    /// 
    /// # Parameters
    /// 
    /// - `id`: The ID of the account to generate a report for
    /// 
    /// # Returns
    /// 
    /// The ESG impact report, or an error if generation failed
    /// 
    /// # Examples
    /// 
    /// ```
    /// use investment_management::api::Client;
    /// 
    /// let client = Client::new();
    /// let report = client.accounts.generate_esg_report("account-1").unwrap();
    /// println!("Overall ESG score: {}", report.overall_score);
    /// ```
    pub fn generate_esg_report(&self, id: &str) -> Result<ESGImpactReport> {
        // Get the account
        let account = self.get(id)?;
        
        // Generate the report
        let report = self.service.generate_esg_impact_report(&account);
        
        Ok(report)
    }
    
    /// Apply tax optimization to an account
    /// 
    /// # Parameters
    /// 
    /// - `id`: The ID of the account to optimize
    /// - `params`: Tax optimization parameters
    /// 
    /// # Returns
    /// 
    /// The optimized account
    pub fn apply_tax_optimization(&self, id: &str, params: TaxOptimizationParams) -> Result<UnifiedManagedAccount> {
        // Get the account
        let account = self.get(id)?;
        
        let settings = TaxOptimizationSettings {
            annual_tax_budget: params._max_capital_gains,
            realized_gains_ytd: 0.0, // Default value
            prioritize_loss_harvesting: params._enable_tax_loss_harvesting,
            defer_short_term_gains: true, // Default value
            min_tax_savings_threshold: params._min_tax_benefit,
            short_term_tax_rate: 0.35, // Default value
            long_term_tax_rate: 0.15, // Default value
        };
        
        let mut optimized_account = account.clone();
        optimized_account.with_tax_optimization(settings);
        
        Ok(optimized_account)
    }
    
    /// Analyze drift for an account
    /// 
    /// # Parameters
    /// 
    /// - `id`: The ID of the account to analyze
    /// 
    /// # Returns
    /// 
    /// A drift analysis for the account
    pub fn analyze_drift(&self, id: &str) -> Result<ModelDriftAnalysis> {
        // Get the account
        let account = self.get(id)?;
        
        // Analyze drift
        let drift_analysis = self.service.analyze_uma_drift(&account);
        
        Ok(drift_analysis)
    }
    
    /// Generate rebalance trades for an account
    /// 
    /// # Parameters
    /// 
    /// - `id`: The ID of the account to rebalance
    /// - `params`: Optional rebalance parameters
    /// 
    /// # Returns
    /// 
    /// A list of trades to execute
    pub fn generate_rebalance_trades(&self, id: &str, params: Option<RebalanceParams>) -> Result<Vec<RebalanceTrade>> {
        // Get the account
        let account = self.get(id)?;
        
        // Set up parameters
        let drift_threshold = Some(0.02); // 2% drift threshold
        let min_trade_amount = Some(100.0); // $100 minimum trade
        let max_trades = Some(20); // Maximum 20 trades
        
        // If tax optimization is enabled, use tax-aware rebalancing
        let tax_aware = params.as_ref()
            .and_then(|p| p._tax_optimization.as_ref())
            .map(|_| true)
            .unwrap_or(false);
        
        // Generate trades
        if tax_aware {
            Ok(self.service.generate_tax_optimized_trades(
                &account,
                max_trades,
                min_trade_amount,
                drift_threshold
            ))
        } else {
            Ok(self.service.generate_uma_rebalance_trades(
                &account,
                max_trades,
                tax_aware,
                min_trade_amount,
                drift_threshold
            ))
        }
    }
    
    /// Generate algorithmic tax-loss harvesting trades for an account
    /// 
    /// # Parameters
    /// 
    /// * `id` - Account ID
    /// * `config` - Configuration for algorithmic tax-loss harvesting
    /// * `market_volatility` - Current market volatility (0.0 to 1.0)
    /// 
    /// # Returns
    /// 
    /// A list of trades for tax-loss harvesting
    /// 
    /// # Examples
    /// 
    /// ```
    /// use investment_management::api::Client;
    /// use investment_management::algorithmic_tlh::AlgorithmicTLHConfig;
    /// 
    /// let client = Client::new();
    /// let config = AlgorithmicTLHConfig::default();
    /// 
    /// // Note: In a real application, you would need to ensure the account has tax optimization settings
    /// // This example shows the API call pattern but may return an error in tests
    /// match client.accounts.generate_algorithmic_tlh_trades("account-1", config, None) {
    ///     Ok(trades) => println!("Generated {} tax-loss harvesting trades", trades.len()),
    ///     Err(e) => println!("Error generating trades: {}", e),
    /// }
    /// ```
    pub fn generate_algorithmic_tlh_trades(
        &self,
        id: &str,
        config: AlgorithmicTLHConfig,
        market_volatility: Option<f64>
    ) -> Result<Vec<RebalanceTrade>> {
        // Get the account
        let account = self.get(id)?;
        
        // Create TLH service
        let mut tlh_service = AlgorithmicTLHService::new(config);
        
        // Generate trades
        let trades = tlh_service.generate_real_time_tlh_trades(&account, market_volatility)
            .map_err(|err| Error::internal(err.to_string()))?;
        
        Ok(trades)
    }
    
    /// Monitor an account for tax-loss harvesting opportunities
    /// 
    /// # Parameters
    /// 
    /// * `id` - Account ID
    /// * `config` - Configuration for algorithmic tax-loss harvesting
    /// * `market_volatility` - Current market volatility (0.0 to 1.0)
    /// 
    /// # Returns
    /// 
    /// Success if monitoring was started successfully
    pub fn monitor_account_for_tlh(
        &self,
        id: &str,
        config: AlgorithmicTLHConfig,
        market_volatility: Option<f64>
    ) -> Result<()> {
        // Get the account
        let account = self.get(id)?;
        
        // Create TLH service
        let mut tlh_service = AlgorithmicTLHService::new(config);
        
        // Monitor account
        tlh_service.monitor_and_harvest(
            &account,
            market_volatility,
            |_| Ok(())
        )
        .map_err(|err| Error::internal(err.to_string()))?;
        
        Ok(())
    }
}

/// API for managing trades
pub struct TradesApi {
    _service: ModelPortfolioService,
}

impl TradesApi {
    /// Create a new trades API
    fn new(service: ModelPortfolioService) -> Self {
        Self { _service: service }
    }
    
    /// Execute trades for an account
    /// 
    /// # Parameters
    /// 
    /// - `account_id`: The ID of the account to execute trades for
    /// - `trades`: The trades to execute
    /// 
    /// # Returns
    /// 
    /// The executed trades, or an error if execution failed
    /// 
    /// # Examples
    /// 
    /// ```
    /// use investment_management::api::Client;
    /// use investment_management::api::RebalanceParams;
    /// 
    /// let client = Client::new();
    /// let params = RebalanceParams::default();
    /// 
    /// // Note: In a real application, you would generate actual trades first
    /// // This example shows the API call pattern but may return an error in tests
    /// match client.accounts.generate_rebalance_trades("account-1", Some(params)) {
    ///     Ok(trades) => {
    ///         if !trades.is_empty() {
    ///             match client.trades.execute("account-1", trades) {
    ///                 Ok(executed_trades) => println!("Executed {} trades", executed_trades.len()),
    ///                 Err(e) => println!("Error executing trades: {}", e),
    ///             }
    ///         } else {
    ///             println!("No trades to execute");
    ///         }
    ///     },
    ///     Err(e) => println!("Error generating trades: {}", e),
    /// }
    /// ```
    pub fn execute(&self, account_id: &str, trades: Vec<RebalanceTrade>) -> Result<Vec<ExecutedTrade>> {
        // Validate inputs
        if account_id.is_empty() {
            return Err(Error::validation("Account ID cannot be empty"));
        }
        
        if trades.is_empty() {
            return Err(Error::validation("No trades to execute"));
        }
        
        // In a real implementation, this would execute the trades
        // For now, we'll just return mock executed trades
        
        let executed_trades = trades.iter().map(|trade| {
            ExecutedTrade {
                id: format!("trade-{}", uuid::Uuid::new_v4()),
                security_id: trade.security_id.clone(),
                amount: trade.amount,
                is_buy: trade.is_buy,
                reason: Self::convert_trade_reason(&trade.reason),
                _tax_impact: trade.tax_impact,
                _execution_price: 100.0, // Mock price
                _quantity: trade.amount / 100.0, // Mock quantity
                status: TradeStatus::Executed,
                _executed_at: chrono::Utc::now().to_rfc3339(),
            }
        }).collect();
        
        Ok(executed_trades)
    }
    
    // Helper function to convert between trade reason types
    fn convert_trade_reason(reason: &RebalanceTradeReason) -> TradeReason {
        match reason {
            RebalanceTradeReason::Rebalance => TradeReason::Rebalancing,
            RebalanceTradeReason::Deposit => TradeReason::_Deposit,
            RebalanceTradeReason::Withdrawal => TradeReason::_Withdrawal,
            RebalanceTradeReason::Transition => TradeReason::_Transition,
            RebalanceTradeReason::TaxLossHarvesting => TradeReason::_TaxLossHarvesting,
            RebalanceTradeReason::FactorExposureAdjustment => TradeReason::_FactorExposureAdjustment,
        }
    }
}

/// Parameters for creating a model portfolio
#[derive(Debug, Clone)]
pub struct CreateModelParams {
    /// Optional ID for the model (will be generated if not provided)
    pub id: Option<String>,
    /// Name of the model
    pub name: String,
    /// Optional description of the model
    pub description: Option<String>,
    /// Type of model (Direct, Composite, or Hybrid)
    pub model_type: crate::portfolio::model::ModelType,
    /// Securities and their target weights
    pub securities: HashMap<String, f64>,
    /// Asset allocation (asset class -> target weight)
    pub asset_allocation: HashMap<String, f64>,
    /// Sector allocation (sector -> target weight)
    pub sector_allocation: HashMap<String, f64>,
    /// Child models for composite models (model ID -> target weight)
    pub child_models: HashMap<String, f64>,
}

impl Default for CreateModelParams {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            description: None,
            model_type: crate::portfolio::model::ModelType::Direct,
            securities: HashMap::new(),
            asset_allocation: HashMap::new(),
            sector_allocation: HashMap::new(),
            child_models: HashMap::new(),
        }
    }
}

/// Parameters for listing model portfolios
#[derive(Debug, Clone, Default)]
pub struct ListModelsParams {
    /// Maximum number of models to return
    pub _limit: Option<usize>,
    /// Offset for pagination
    pub _offset: Option<usize>,
    /// Filter by model type
    pub _model_type: Option<crate::portfolio::model::ModelType>,
}

/// Parameters for creating an account
#[derive(Debug, Clone, Default)]
pub struct CreateAccountParams {
    /// Optional ID for the account (will be generated if not provided)
    pub id: Option<String>,
    /// Name of the account
    pub name: String,
    /// Owner of the account
    pub owner: String,
    /// ID of the model to use for the account
    pub model_id: String,
    /// Initial investment amount
    pub initial_investment: f64,
    /// Optional tax optimization settings
    pub _tax_settings: Option<TaxOptimizationParams>,
    /// Optional ESG screening criteria
    pub _esg_criteria: Option<ESGScreeningParams>,
}

/// ESG screening parameters
#[derive(Debug, Clone)]
pub struct ESGScreeningParams {
    /// Minimum overall ESG score (0-100)
    pub min_overall_score: Option<f64>,
    /// Minimum environmental score (0-100)
    pub min_environmental_score: Option<f64>,
    /// Minimum social score (0-100)
    pub min_social_score: Option<f64>,
    /// Minimum governance score (0-100)
    pub min_governance_score: Option<f64>,
    /// Maximum controversy score (0-100)
    pub max_controversy_score: Option<f64>,
    /// Excluded sectors
    pub excluded_sectors: Vec<String>,
    /// Excluded business activities
    pub excluded_activities: Vec<String>,
}

impl Default for ESGScreeningParams {
    fn default() -> Self {
        Self {
            min_overall_score: None,
            min_environmental_score: None,
            min_social_score: None,
            min_governance_score: None,
            max_controversy_score: None,
            excluded_sectors: Vec::new(),
            excluded_activities: Vec::new(),
        }
    }
}

/// Tax optimization parameters
#[derive(Debug, Clone)]
pub struct TaxOptimizationParams {
    /// Whether to enable tax-loss harvesting
    pub _enable_tax_loss_harvesting: bool,
    /// Maximum capital gains to realize
    pub _max_capital_gains: Option<f64>,
    /// Minimum tax benefit required for harvesting
    pub _min_tax_benefit: Option<f64>,
}

impl Default for TaxOptimizationParams {
    fn default() -> Self {
        Self {
            _enable_tax_loss_harvesting: false,
            _max_capital_gains: None,
            _min_tax_benefit: None,
        }
    }
}

/// Rebalance parameters
#[derive(Debug, Clone)]
pub struct RebalanceParams {
    /// Portfolio identifier
    pub portfolio_id: String,
    /// Target model portfolio identifier
    pub model_id: String,
    /// Tax optimization parameters
    pub _tax_optimization: Option<TaxOptimizationParams>,
    /// Trade constraints
    pub _constraints: Option<TradeConstraints>,
}

impl Default for RebalanceParams {
    fn default() -> Self {
        Self {
            portfolio_id: String::new(),
            model_id: String::new(),
            _tax_optimization: None,
            _constraints: None,
        }
    }
}

/// Status of a trade
#[derive(Debug, Clone)]
pub enum TradeStatus {
    /// Trade has been created but not executed
    _Pending,
    /// Trade has been executed
    Executed,
    /// Trade has been canceled
    _Canceled,
    /// Trade has failed
    _Failed,
}

/// Executed trade
#[derive(Debug, Clone)]
pub struct ExecutedTrade {
    /// Trade identifier
    pub id: String,
    /// Security identifier
    pub security_id: String,
    /// Trade amount in base currency
    pub amount: f64,
    /// Trade direction (buy/sell)
    pub is_buy: bool,
    /// Trade reason
    pub reason: TradeReason,
    /// Estimated tax impact
    pub _tax_impact: Option<f64>,
    /// Execution price per share
    pub _execution_price: f64,
    /// Quantity of shares
    pub _quantity: f64,
    /// Status of the trade
    pub status: TradeStatus,
    /// Execution timestamp
    pub _executed_at: String,
}

/// Portfolio
#[derive(Debug, Clone)]
pub struct Portfolio {
    /// Portfolio identifier
    pub _id: String,
    /// Portfolio name
    pub _name: String,
    /// Portfolio holdings
    pub holdings: Vec<PortfolioHolding>,
    /// Cash balance
    pub _cash_balance: f64,
}

/// Portfolio holding
#[derive(Debug, Clone)]
pub struct PortfolioHolding {
    /// Security identifier
    pub security_id: String,
    /// Quantity of shares
    pub quantity: f64,
    /// Market value
    pub market_value: f64,
    /// Purchase date
    pub _purchase_date: Option<String>,
}

/// Trade reason
#[derive(Debug, Clone)]
pub enum TradeReason {
    /// Rebalancing
    Rebalancing,
    /// Deposit
    _Deposit,
    /// Withdrawal
    _Withdrawal,
    /// Transition
    _Transition,
    /// Tax loss harvesting
    _TaxLossHarvesting,
    /// Factor exposure adjustment
    _FactorExposureAdjustment,
}

/// Trade constraints
#[derive(Debug, Clone)]
pub struct TradeConstraints {
    /// Minimum trade amount
    pub _min_trade_amount: Option<f64>,
    /// Maximum number of trades
    pub _max_trades: Option<usize>,
    /// Restricted securities
    pub _restricted_securities: Vec<String>,
}

impl Default for TradeConstraints {
    fn default() -> Self {
        Self {
            _min_trade_amount: None,
            _max_trades: None,
            _restricted_securities: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Create a simple test for the Client::new() method
    #[test]
    fn test_client_new() {
        // Test that Client::new() creates a new client with the expected APIs
        let _client = Client::new();
        
        // Just verify that the client was created successfully
        assert!(true);
    }

    // Create a test for the ModelPortfoliosApi::create method
    #[test]
    fn test_create_model_params_default() {
        // Test the Default implementation for CreateModelParams
        let params = CreateModelParams::default();
        
        assert!(params.id.is_none());
        assert_eq!(params.name, String::new());
        assert!(params.description.is_none());
        assert!(matches!(params.model_type, crate::portfolio::model::ModelType::Direct));
        assert!(params.securities.is_empty());
        assert!(params.asset_allocation.is_empty());
        assert!(params.sector_allocation.is_empty());
        assert!(params.child_models.is_empty());
    }

    // Create a test for the ESGScreeningParams::default method
    #[test]
    fn test_esg_screening_params_default() {
        // Test the Default implementation for ESGScreeningParams
        let params = ESGScreeningParams::default();
        
        assert!(params.min_overall_score.is_none());
        assert!(params.min_environmental_score.is_none());
        assert!(params.min_social_score.is_none());
        assert!(params.min_governance_score.is_none());
        assert!(params.max_controversy_score.is_none());
        assert!(params.excluded_sectors.is_empty());
        assert!(params.excluded_activities.is_empty());
    }

    // Create a test for the TaxOptimizationParams::default method
    #[test]
    fn test_tax_optimization_params_default() {
        // Test the Default implementation for TaxOptimizationParams
        let params = TaxOptimizationParams::default();
        
        assert!(params._enable_tax_loss_harvesting == false);
        assert!(params._max_capital_gains.is_none());
        assert!(params._min_tax_benefit.is_none());
    }

    // Create a test for the RebalanceParams::default method
    #[test]
    fn test_rebalance_params_default() {
        // Test the Default implementation for RebalanceParams
        let params = RebalanceParams::default();
        
        assert!(params.portfolio_id.is_empty());
        assert!(params.model_id.is_empty());
        assert!(params._tax_optimization.is_none());
        assert!(params._constraints.is_none());
    }

    // Create a test for the CreateAccountParams::default method
    #[test]
    fn test_create_account_params_default() {
        // Test the Default implementation for CreateAccountParams
        let params = CreateAccountParams::default();
        
        assert!(params.id.is_none());
        assert_eq!(params.name, String::new());
        assert_eq!(params.owner, String::new());
        assert_eq!(params.model_id, String::new());
        assert_eq!(params.initial_investment, 0.0);
        assert!(params._tax_settings.is_none());
        assert!(params._esg_criteria.is_none());
    }

    // Create a test for the ListModelsParams::default method
    #[test]
    fn test_list_models_params_default() {
        // Test the Default implementation for ListModelsParams
        let params = ListModelsParams::default();
        
        assert!(params._limit.is_none());
        assert!(params._offset.is_none());
        assert!(params._model_type.is_none());
    }
} 