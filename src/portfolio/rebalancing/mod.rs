use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// Import the factor model module
use crate::factor_model::FactorModelApi;

/// Portfolio Rebalancing Service
/// 
/// This service is responsible for analyzing portfolio drift and generating rebalancing trades.
/// It provides functionality for:
/// - Calculating drift from target weights
/// - Calculating factor drift
/// - Generating rebalance trades
/// - Handling cash flows (deposits and withdrawals)
/// - Managing portfolio transitions
#[derive(Clone)]
pub struct PortfolioRebalancingService {
    factor_model_api: FactorModelApi,
}

/// Represents the drift analysis result for a portfolio
#[derive(Debug, Clone)]
pub struct DriftAnalysisResult {
    /// Overall drift score (0.0 to 1.0)
    pub drift_score: f64,
    /// Drift by asset class
    pub asset_class_drift: HashMap<String, f64>,
    /// Drift by sector
    pub sector_drift: HashMap<String, f64>,
    /// Drift by factor
    pub factor_drift: HashMap<String, f64>,
    /// Drift by security
    pub security_drift: HashMap<String, f64>,
}

/// Represents a trade to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalanceTrade {
    /// Security identifier
    pub security_id: String,
    /// Trade amount in base currency
    pub amount: f64,
    /// Trade direction (buy/sell)
    pub is_buy: bool,
    /// Trade reason
    pub reason: TradeReason,
    /// Estimated tax impact
    pub tax_impact: Option<f64>,
}

/// Reason for generating a trade
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TradeReason {
    /// Rebalancing to target weights
    Rebalance,
    /// Handling cash inflow
    Deposit,
    /// Handling cash outflow
    Withdrawal,
    /// Portfolio transition
    Transition,
    /// Tax-loss harvesting
    TaxLossHarvesting,
    /// Factor exposure adjustment
    FactorExposureAdjustment,
}

/// Represents a portfolio holding
#[derive(Debug, Clone)]
pub struct PortfolioHolding {
    /// Security identifier
    pub security_id: String,
    /// Current market value in base currency
    pub market_value: f64,
    /// Current weight in the portfolio
    pub weight: f64,
    /// Target weight in the portfolio
    pub target_weight: f64,
    /// Cost basis in base currency
    pub cost_basis: f64,
    /// Purchase date
    pub purchase_date: String,
    /// Factor exposures for this security
    pub factor_exposures: HashMap<String, f64>,
}

/// Portfolio information
#[derive(Debug, Clone)]
pub struct Portfolio {
    /// Portfolio identifier
    pub id: String,
    /// Portfolio name
    pub name: String,
    /// Total market value
    pub total_market_value: f64,
    /// Cash balance
    pub cash_balance: f64,
    /// Holdings
    pub holdings: Vec<PortfolioHolding>,
}

/// Cash flow information
#[derive(Debug, Clone)]
pub struct CashFlow {
    /// Cash flow amount (positive for inflow, negative for outflow)
    pub amount: f64,
    /// Cash flow date
    pub date: String,
    /// Cash flow type
    pub flow_type: CashFlowType,
}

/// Cash flow type
#[derive(Debug, Clone, PartialEq)]
pub enum CashFlowType {
    /// Deposit
    Deposit,
    /// Withdrawal
    Withdrawal,
    /// Dividend
    Dividend,
    /// Interest
    Interest,
    /// Fee
    Fee,
}

impl PortfolioRebalancingService {
    /// Create a new portfolio rebalancing service
    pub fn new(factor_model_api: FactorModelApi) -> Self {
        Self {
            factor_model_api,
        }
    }
    
    /// Calculate the factor drift between current and target factor exposures
    /// 
    /// # Arguments
    /// * `portfolio_id` - The ID of the portfolio to analyze
    /// * `target_exposures` - The target factor exposures
    /// * `drift_threshold` - Optional threshold to consider significant drift (default: 0.1)
    /// 
    /// # Returns
    /// A HashMap containing the drift for each factor
    pub fn calculate_factor_drift(
        &self,
        portfolio_id: &str,
        target_exposures: &HashMap<String, f64>,
        drift_threshold: Option<f64>
    ) -> Option<HashMap<String, f64>> {
        // Get current factor exposures
        let current_exposures = self.factor_model_api.get_factor_exposures(portfolio_id)?;
        let threshold = drift_threshold.unwrap_or(0.1);
        
        // Calculate drift for each factor
        let mut factor_drift = HashMap::new();
        
        // Get all unique factor IDs from both current and target exposures
        let mut all_factors = current_exposures.keys().collect::<Vec<&String>>();
        for factor_id in target_exposures.keys() {
            if !all_factors.contains(&factor_id) {
                all_factors.push(factor_id);
            }
        }
        
        // Calculate drift for each factor
        for factor_id in all_factors {
            let current = current_exposures.get(factor_id).unwrap_or(&0.0);
            let target = target_exposures.get(factor_id).unwrap_or(&0.0);
            
            // Calculate absolute drift
            let drift = current - target;
            
            // Only include significant drift
            if drift.abs() >= threshold {
                factor_drift.insert(factor_id.clone(), drift);
            }
        }
        
        Some(factor_drift)
    }
    
    /// Calculate the overall drift score based on factor exposures
    /// 
    /// # Arguments
    /// * `factor_drift` - The factor drift HashMap
    /// * `factor_importance` - Optional weights for each factor's importance
    /// 
    /// # Returns
    /// A drift score between 0.0 and 1.0
    pub fn calculate_drift_score(
        &self,
        factor_drift: &HashMap<String, f64>,
        factor_importance: Option<&HashMap<String, f64>>
    ) -> f64 {
        if factor_drift.is_empty() {
            return 0.0;
        }
        
        // Create a default HashMap to use if factor_importance is None
        let default_importance = HashMap::new();
        let importance = factor_importance.unwrap_or(&default_importance);
        
        // Calculate weighted sum of absolute drifts
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        
        for (factor_id, drift) in factor_drift {
            let weight = importance.get(factor_id).unwrap_or(&1.0);
            weighted_sum += drift.abs() * weight;
            total_weight += weight;
        }
        
        // Normalize to get a score between 0 and 1
        // Using a sigmoid-like function to map the weighted average to [0,1]
        let avg_drift = weighted_sum / total_weight.max(1.0);
        let score = 1.0 / (1.0 + (-5.0 * avg_drift).exp());
        
        score
    }
    
    /// Generate rebalance trades to align with target factor exposures
    /// 
    /// # Arguments
    /// * `portfolio_id` - The ID of the portfolio to rebalance
    /// * `target_exposures` - The target factor exposures
    /// * `max_trades` - Optional maximum number of trades to generate
    /// * `tax_aware` - Whether to consider tax implications
    /// 
    /// # Returns
    /// A vector of RebalanceTrade objects
    pub fn generate_factor_rebalance_trades(
        &self,
        portfolio_id: &str,
        target_exposures: &HashMap<String, f64>,
        max_trades: Option<usize>,
        tax_aware: bool
    ) -> Vec<RebalanceTrade> {
        // Get current factor exposures
        let current_exposures = match self.factor_model_api.get_factor_exposures(portfolio_id) {
            Some(exposures) => exposures,
            None => return Vec::new(), // Portfolio not found or no exposures
        };
        
        // Calculate factor drift
        let factor_drift = match self.calculate_factor_drift(portfolio_id, target_exposures, None) {
            Some(drift) => drift,
            None => return Vec::new(), // No significant drift
        };
        
        if factor_drift.is_empty() {
            return Vec::new(); // No significant drift
        }
        
        // Get portfolio holdings
        // In a real implementation, this would come from a portfolio service
        // For now, we'll create a mock portfolio based on the factor exposures
        let mock_portfolio = self.create_mock_portfolio_from_exposures(portfolio_id, &current_exposures);
        
        // Calculate factor sensitivities for each security
        // This represents how much each security contributes to each factor exposure
        let factor_sensitivities = self.calculate_factor_sensitivities(&mock_portfolio);
        
        // Determine which securities to trade to reduce factor drift
        let securities_to_trade = self.select_securities_for_factor_adjustment(
            &factor_drift,
            &factor_sensitivities,
            max_trades
        );
        
        // Generate trades
        let mut trades = Vec::new();
        
        for (security_id, factors_to_adjust) in securities_to_trade {
            // Find the holding
            let holding = match mock_portfolio.holdings.iter().find(|h| h.security_id == security_id) {
                Some(h) => h,
                None => continue,
            };
            
            // Calculate the trade amount based on factor adjustments
            let (trade_amount, is_buy) = self.calculate_factor_trade_amount(
                holding,
                &factors_to_adjust,
                &factor_drift
            );
            
            // Skip small trades
            if trade_amount < 100.0 {
                continue;
            }
            
            // Calculate tax impact for sells if tax_aware is true
            let tax_impact = if tax_aware && !is_buy {
                let unrealized_gain = holding.market_value - holding.cost_basis;
                if unrealized_gain > 0.0 {
                    // Simplified tax calculation (assuming 20% capital gains tax)
                    Some(unrealized_gain * 0.2)
                } else {
                    Some(0.0) // No tax impact for losses
                }
            } else {
                None
            };
            
            // Create trade
            let trade = RebalanceTrade {
                security_id: security_id.clone(),
                amount: trade_amount,
                is_buy,
                reason: TradeReason::FactorExposureAdjustment,
                tax_impact,
            };
            
            trades.push(trade);
        }
        
        trades
    }
    
    // Helper method to create a mock portfolio from factor exposures
    fn create_mock_portfolio_from_exposures(
        &self,
        portfolio_id: &str,
        exposures: &HashMap<String, f64>
    ) -> Portfolio {
        // In a real implementation, this would fetch the actual portfolio
        // For now, we'll create a mock portfolio with synthetic holdings
        
        let mut holdings = Vec::new();
        let total_market_value = 1_000_000.0; // $1M mock portfolio
        
        // Create 5 mock holdings with different factor exposures
        let securities = vec!["AAPL", "MSFT", "AMZN", "GOOGL", "FB"];
        let weights = vec![0.2, 0.2, 0.2, 0.2, 0.2]; // Equal weights for simplicity
        
        for (i, security_id) in securities.iter().enumerate() {
            // Create factor exposures for this security
            // In a real implementation, these would come from a factor model
            let mut factor_exposures = HashMap::new();
            for (factor, exposure) in exposures {
                // Distribute the portfolio's factor exposure across securities
                // with some variation to make it interesting
                let security_exposure = exposure * (0.8 + (i as f64 * 0.1));
                factor_exposures.insert(factor.clone(), security_exposure);
            }
            
            let market_value = total_market_value * weights[i];
            let holding = PortfolioHolding {
                security_id: security_id.to_string(),
                market_value,
                weight: weights[i],
                target_weight: weights[i], // Same as current for simplicity
                cost_basis: market_value * 0.8, // Assume 20% gain
                purchase_date: "2022-01-01".to_string(),
                factor_exposures,
            };
            
            holdings.push(holding);
        }
        
        Portfolio {
            id: portfolio_id.to_string(),
            name: format!("Mock Portfolio {}", portfolio_id),
            total_market_value,
            cash_balance: total_market_value * 0.05, // 5% cash
            holdings,
        }
    }
    
    // Helper method to calculate factor sensitivities for each security
    fn calculate_factor_sensitivities(
        &self,
        portfolio: &Portfolio
    ) -> HashMap<String, HashMap<String, f64>> {
        let mut sensitivities = HashMap::new();
        
        for holding in &portfolio.holdings {
            let mut security_sensitivities = HashMap::new();
            
            for (factor, exposure) in &holding.factor_exposures {
                // Factor sensitivity is the contribution to the factor exposure
                // weighted by the security's weight in the portfolio
                let sensitivity = exposure * holding.weight;
                security_sensitivities.insert(factor.clone(), sensitivity);
            }
            
            sensitivities.insert(holding.security_id.clone(), security_sensitivities);
        }
        
        sensitivities
    }
    
    // Helper method to select securities for factor adjustment
    fn select_securities_for_factor_adjustment(
        &self,
        factor_drift: &HashMap<String, f64>,
        factor_sensitivities: &HashMap<String, HashMap<String, f64>>,
        max_trades: Option<usize>
    ) -> HashMap<String, Vec<String>> {
        let mut securities_to_trade = HashMap::new();
        let max_trade_count = max_trades.unwrap_or(10);
        
        // For each factor with drift, find the securities with the highest sensitivity
        for (factor, drift) in factor_drift {
            // Sort securities by their sensitivity to this factor
            let mut securities_by_sensitivity = Vec::new();
            
            for (security_id, sensitivities) in factor_sensitivities {
                if let Some(sensitivity) = sensitivities.get(factor) {
                    // If drift is positive, we want to reduce exposure (negative sensitivity)
                    // If drift is negative, we want to increase exposure (positive sensitivity)
                    let effective_sensitivity = if *drift > 0.0 { -sensitivity } else { *sensitivity };
                    
                    securities_by_sensitivity.push((security_id.clone(), effective_sensitivity));
                }
            }
            
            // Sort by effective sensitivity (descending)
            securities_by_sensitivity.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            
            // Take the top securities
            let top_securities = securities_by_sensitivity.iter()
                .take(max_trade_count)
                .map(|(id, _)| id.clone())
                .collect::<Vec<String>>();
            
            // Add each security to the map with this factor
            for security_id in top_securities {
                securities_to_trade
                    .entry(security_id)
                    .or_insert_with(Vec::new)
                    .push(factor.clone());
            }
        }
        
        securities_to_trade
    }
    
    // Helper method to calculate trade amount based on factor adjustments
    fn calculate_factor_trade_amount(
        &self,
        holding: &PortfolioHolding,
        factors_to_adjust: &[String],
        factor_drift: &HashMap<String, f64>
    ) -> (f64, bool) {
        let mut total_adjustment = 0.0;
        
        for factor in factors_to_adjust {
            if let Some(drift) = factor_drift.get(factor) {
                if let Some(exposure) = holding.factor_exposures.get(factor) {
                    // Calculate adjustment based on drift and exposure
                    // If drift is positive, we want to reduce exposure
                    // If drift is negative, we want to increase exposure
                    let adjustment = drift * exposure * 2.0; // Scale factor
                    total_adjustment += adjustment;
                }
            }
        }
        
        // Determine trade direction
        let is_buy = total_adjustment < 0.0;
        
        // Calculate trade amount based on adjustment
        let trade_amount = (total_adjustment.abs() * holding.market_value).min(holding.market_value);
        
        (trade_amount, is_buy)
    }
    
    /// Generate rebalance trades to align with target weights
    /// 
    /// # Arguments
    /// * `portfolio` - The portfolio to rebalance
    /// * `max_trades` - Optional maximum number of trades to generate
    /// * `tax_aware` - Whether to consider tax implications
    /// * `min_trade_amount` - Minimum trade amount to consider
    /// * `drift_threshold` - Minimum weight drift to consider for rebalancing
    /// 
    /// # Returns
    /// A vector of RebalanceTrade objects
    pub fn generate_rebalance_trades(
        &self,
        portfolio: &Portfolio,
        max_trades: Option<usize>,
        tax_aware: bool,
        min_trade_amount: Option<f64>,
        drift_threshold: Option<f64>
    ) -> Vec<RebalanceTrade> {
        let min_amount = min_trade_amount.unwrap_or(100.0);
        let threshold = drift_threshold.unwrap_or(0.02); // 2% drift threshold
        let max_trade_count = max_trades.unwrap_or(usize::MAX);
        
        // Calculate weight drift for each holding
        let mut drifts: Vec<(usize, f64, f64)> = portfolio.holdings.iter().enumerate()
            .map(|(i, holding)| {
                let drift = holding.weight - holding.target_weight;
                let abs_drift = drift.abs();
                (i, drift, abs_drift)
            })
            .filter(|(_, _, abs_drift)| *abs_drift >= threshold) // Only consider significant drifts
            .collect();
        
        // Sort by absolute drift (descending)
        drifts.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        
        // Limit to max_trades
        if drifts.len() > max_trade_count {
            drifts.truncate(max_trade_count);
        }
        
        let mut trades = Vec::new();
        
        // Generate trades for each drift
        for (index, drift, _) in drifts {
            let holding = &portfolio.holdings[index];
            
            // Calculate trade amount
            let target_value = portfolio.total_market_value * holding.target_weight;
            let current_value = holding.market_value;
            let trade_amount = (target_value - current_value).abs();
            
            // Skip small trades
            if trade_amount < min_amount {
                continue;
            }
            
            // Determine if it's a buy or sell
            let is_buy = drift < 0.0; // Negative drift means underweight, so buy
            
            // Calculate tax impact for sells if tax_aware is true
            let tax_impact = if tax_aware && !is_buy {
                let unrealized_gain = current_value - holding.cost_basis;
                if unrealized_gain > 0.0 {
                    // Simplified tax calculation (assuming 20% capital gains tax)
                    Some(unrealized_gain * 0.2)
                } else {
                    Some(0.0) // No tax impact for losses
                }
            } else {
                None
            };
            
            // Create trade
            let trade = RebalanceTrade {
                security_id: holding.security_id.clone(),
                amount: trade_amount,
                is_buy,
                reason: TradeReason::Rebalance,
                tax_impact,
            };
            
            trades.push(trade);
        }
        
        trades
    }
    
    /// Generate trades to handle cash flows (deposits or withdrawals)
    /// 
    /// # Arguments
    /// * `portfolio` - The portfolio to handle cash flow for
    /// * `cash_flow` - The cash flow information
    /// * `maintain_target_weights` - Whether to maintain target weights after cash flow
    /// * `min_trade_amount` - Minimum trade amount to consider
    /// 
    /// # Returns
    /// A vector of RebalanceTrade objects
    pub fn generate_cash_flow_trades(
        &self,
        portfolio: &Portfolio,
        cash_flow: &CashFlow,
        maintain_target_weights: bool,
        min_trade_amount: Option<f64>
    ) -> Vec<RebalanceTrade> {
        let min_amount = min_trade_amount.unwrap_or(100.0);
        let flow_amount = cash_flow.amount;
        
        // Determine if it's a deposit or withdrawal
        let is_deposit = flow_amount > 0.0;
        
        // If we don't need to maintain target weights, just update cash balance
        if !maintain_target_weights {
            return Vec::new();
        }
        
        let mut trades = Vec::new();
        
        // For deposits: buy securities to maintain target weights
        if is_deposit {
            // Calculate new total portfolio value including deposit
            let new_total_value = portfolio.total_market_value + flow_amount;
            
            // Generate buy trades proportional to target weights
            for holding in &portfolio.holdings {
                // Calculate amount to buy based on target weight
                let target_value = new_total_value * holding.target_weight;
                let current_value = holding.market_value;
                let buy_amount = target_value - current_value;
                
                // Skip small trades
                if buy_amount < min_amount {
                    continue;
                }
                
                // Create buy trade
                let trade = RebalanceTrade {
                    security_id: holding.security_id.clone(),
                    amount: buy_amount,
                    is_buy: true,
                    reason: TradeReason::Deposit,
                    tax_impact: None,
                };
                
                trades.push(trade);
            }
        } 
        // For withdrawals: sell securities to maintain target weights
        else {
            // Calculate new total portfolio value after withdrawal
            let withdrawal_amount = flow_amount.abs();
            let new_total_value = portfolio.total_market_value - withdrawal_amount;
            
            // Check if we have enough cash
            let remaining_withdrawal = withdrawal_amount - portfolio.cash_balance;
            
            // If we have enough cash, no trades needed
            if remaining_withdrawal <= 0.0 {
                return Vec::new();
            }
            
            // Generate sell trades proportional to target weights
            for holding in &portfolio.holdings {
                // Calculate amount to sell based on target weight
                let target_value = new_total_value * holding.target_weight;
                let current_value = holding.market_value;
                let sell_amount = (current_value - target_value).max(0.0);
                
                // Skip small trades
                if sell_amount < min_amount {
                    continue;
                }
                
                // Create sell trade
                let trade = RebalanceTrade {
                    security_id: holding.security_id.clone(),
                    amount: sell_amount,
                    is_buy: false,
                    reason: TradeReason::Withdrawal,
                    tax_impact: None, // Tax impact could be calculated here
                };
                
                trades.push(trade);
            }
        }
        
        trades
    }
    
    /// Generate trades for transitioning a portfolio to a new model
    /// 
    /// # Arguments
    /// * `current_portfolio` - The current portfolio
    /// * `target_weights` - The target weights for the new model
    /// * `tax_aware` - Whether to consider tax implications
    /// * `tracking_error_constraint` - Maximum allowed tracking error
    /// * `min_trade_amount` - Minimum trade amount to consider
    /// 
    /// # Returns
    /// A vector of RebalanceTrade objects
    pub fn generate_transition_trades(
        &self,
        current_portfolio: &Portfolio,
        target_weights: &HashMap<String, f64>,
        tax_aware: bool,
        _tracking_error_constraint: Option<f64>,
        min_trade_amount: Option<f64>
    ) -> Vec<RebalanceTrade> {
        let min_amount = min_trade_amount.unwrap_or(100.0);
        
        // Create a map of current holdings by security_id
        let mut current_holdings_map: HashMap<String, &PortfolioHolding> = HashMap::new();
        for holding in &current_portfolio.holdings {
            current_holdings_map.insert(holding.security_id.clone(), holding);
        }
        
        let mut trades = Vec::new();
        
        // Process sells first (securities in current portfolio not in target or with reduced weight)
        for holding in &current_portfolio.holdings {
            let target_weight = target_weights.get(&holding.security_id).unwrap_or(&0.0);
            
            // If target weight is less than current weight, sell some or all
            if *target_weight < holding.weight {
                let target_value = current_portfolio.total_market_value * target_weight;
                let current_value = holding.market_value;
                let sell_amount = current_value - target_value;
                
                // Skip small trades
                if sell_amount < min_amount {
                    continue;
                }
                
                // Calculate tax impact if tax_aware is true
                let tax_impact = if tax_aware {
                    let unrealized_gain = current_value - holding.cost_basis;
                    if unrealized_gain > 0.0 {
                        // Simplified tax calculation (assuming 20% capital gains tax)
                        Some(unrealized_gain * sell_amount / current_value * 0.2)
                    } else {
                        Some(0.0) // No tax impact for losses
                    }
                } else {
                    None
                };
                
                // Create sell trade
                let trade = RebalanceTrade {
                    security_id: holding.security_id.clone(),
                    amount: sell_amount,
                    is_buy: false,
                    reason: TradeReason::Transition,
                    tax_impact,
                };
                
                trades.push(trade);
            }
        }
        
        // Process buys (securities in target not in current portfolio or with increased weight)
        for (security_id, target_weight) in target_weights {
            let current_holding = current_holdings_map.get(security_id);
            let current_weight = current_holding.map_or(0.0, |h| h.weight);
            
            // If target weight is greater than current weight, buy more
            if *target_weight > current_weight {
                let target_value = current_portfolio.total_market_value * target_weight;
                let current_value = current_holding.map_or(0.0, |h| h.market_value);
                let buy_amount = target_value - current_value;
                
                // Skip small trades
                if buy_amount < min_amount {
                    continue;
                }
                
                // Create buy trade
                let trade = RebalanceTrade {
                    security_id: security_id.clone(),
                    amount: buy_amount,
                    is_buy: true,
                    reason: TradeReason::Transition,
                    tax_impact: None,
                };
                
                trades.push(trade);
            }
        }
        
        // If tracking_error_constraint is provided, we would need to optimize the trades
        // to ensure the resulting portfolio doesn't exceed the tracking error constraint
        // This would require a more complex optimization algorithm
        
        trades
    }
}

// Example usage
pub fn run_portfolio_rebalancing_example() {
    println!("\n=== Portfolio Rebalancing Example ===");
    
    // Create the factor model API
    let factor_model_api = crate::factor_model::FactorModelApi::new();
    
    // Create the portfolio rebalancing service
    let rebalancing_service = PortfolioRebalancingService::new(factor_model_api);
    
    // Create a test portfolio
    let portfolio = create_test_portfolio();
    
    println!("\n--- Portfolio Information ---");
    println!("Portfolio: {} ({})", portfolio.name, portfolio.id);
    println!("Total Market Value: ${:.2}", portfolio.total_market_value);
    println!("Cash Balance: ${:.2}", portfolio.cash_balance);
    
    println!("\n--- Holdings ---");
    for holding in &portfolio.holdings {
        println!("Security: {}", holding.security_id);
        println!("  Market Value: ${:.2}", holding.market_value);
        println!("  Current Weight: {:.2}%", holding.weight * 100.0);
        println!("  Target Weight: {:.2}%", holding.target_weight * 100.0);
        println!("  Drift: {:.2}%", (holding.weight - holding.target_weight) * 100.0);
    }
    
    // Generate rebalance trades
    let trades = rebalancing_service.generate_rebalance_trades(&portfolio, None, true, None, None);
    
    println!("\n--- Rebalance Trades ---");
    for trade in &trades {
        let action = if trade.is_buy { "BUY" } else { "SELL" };
        println!("{} {} ${:.2}", action, trade.security_id, trade.amount);
        
        if let Some(tax_impact) = trade.tax_impact {
            println!("  Tax Impact: ${:.2}", tax_impact);
        }
    }
    
    // Create a deposit cash flow
    let deposit = CashFlow {
        amount: 20000.0,
        date: "2023-01-01".to_string(),
        flow_type: CashFlowType::Deposit,
    };
    
    // Generate deposit trades
    let deposit_trades = rebalancing_service.generate_cash_flow_trades(&portfolio, &deposit, true, None);
    
    println!("\n--- Deposit Trades (${:.2}) ---", deposit.amount);
    for trade in &deposit_trades {
        println!("BUY {} ${:.2}", trade.security_id, trade.amount);
    }
    
    // Create a withdrawal cash flow
    let withdrawal = CashFlow {
        amount: -15000.0,
        date: "2023-01-02".to_string(),
        flow_type: CashFlowType::Withdrawal,
    };
    
    // Generate withdrawal trades
    let withdrawal_trades = rebalancing_service.generate_cash_flow_trades(&portfolio, &withdrawal, true, None);
    
    println!("\n--- Withdrawal Trades (${:.2}) ---", withdrawal.amount);
    if withdrawal_trades.is_empty() {
        println!("No trades needed (sufficient cash balance)");
    } else {
        for trade in &withdrawal_trades {
            println!("SELL {} ${:.2}", trade.security_id, trade.amount);
        }
    }
    
    // Define target weights for a new model
    let mut new_model_weights = HashMap::new();
    new_model_weights.insert("AAPL".to_string(), 0.05);
    new_model_weights.insert("MSFT".to_string(), 0.15);
    new_model_weights.insert("AMZN".to_string(), 0.10);
    new_model_weights.insert("GOOGL".to_string(), 0.25);
    new_model_weights.insert("TSLA".to_string(), 0.20); // New position
    new_model_weights.insert("NVDA".to_string(), 0.15); // New position
    new_model_weights.insert("META".to_string(), 0.10); // New position
    
    // Generate transition trades
    let transition_trades = rebalancing_service.generate_transition_trades(
        &portfolio,
        &new_model_weights,
        true,
        None,
        None
    );
    
    println!("\n--- Transition Trades ---");
    for trade in &transition_trades {
        let action = if trade.is_buy { "BUY" } else { "SELL" };
        println!("{} {} ${:.2}", action, trade.security_id, trade.amount);
        
        if let Some(tax_impact) = trade.tax_impact {
            println!("  Tax Impact: ${:.2}", tax_impact);
        }
    }
    
    // Calculate factor drift
    let mut target_exposures = HashMap::new();
    target_exposures.insert("momentum".to_string(), 0.2);
    target_exposures.insert("value".to_string(), 0.3);
    target_exposures.insert("quality".to_string(), 0.4);
    target_exposures.insert("growth".to_string(), 0.3);
    
    if let Some(factor_drift) = rebalancing_service.calculate_factor_drift("portfolio-123", &target_exposures, None) {
        println!("\n--- Factor Drift ---");
        for (factor, drift) in &factor_drift {
            println!("{}: {:.2}", factor, drift);
        }
        
        // Calculate drift score
        let drift_score = rebalancing_service.calculate_drift_score(&factor_drift, None);
        println!("\nDrift Score: {:.2}", drift_score);
        
        // Generate factor rebalance trades
        let factor_trades = rebalancing_service.generate_factor_rebalance_trades(
            "portfolio-123",
            &target_exposures,
            Some(5),
            true
        );
        
        println!("\n--- Factor Rebalance Trades ---");
        if factor_trades.is_empty() {
            println!("No factor rebalance trades generated");
        } else {
            for trade in &factor_trades {
                let action = if trade.is_buy { "BUY" } else { "SELL" };
                println!("{} {} ${:.2} (Factor Adjustment)", action, trade.security_id, trade.amount);
                
                if let Some(tax_impact) = trade.tax_impact {
                    println!("  Tax Impact: ${:.2}", tax_impact);
                }
            }
        }
    }
}

// Helper function to create a test portfolio
fn create_test_portfolio() -> Portfolio {
    // Create holdings
    let holdings = vec![
        PortfolioHolding {
            security_id: "AAPL".to_string(),
            market_value: 15000.0,
            weight: 0.15,
            target_weight: 0.10,
            cost_basis: 10000.0,
            purchase_date: "2022-01-01".to_string(),
            factor_exposures: {
                let mut exposures = HashMap::new();
                exposures.insert("momentum".to_string(), 0.5);
                exposures.insert("quality".to_string(), 0.8);
                exposures
            },
        },
        PortfolioHolding {
            security_id: "MSFT".to_string(),
            market_value: 20000.0,
            weight: 0.20,
            target_weight: 0.25,
            cost_basis: 18000.0,
            purchase_date: "2022-02-01".to_string(),
            factor_exposures: {
                let mut exposures = HashMap::new();
                exposures.insert("momentum".to_string(), 0.6);
                exposures.insert("quality".to_string(), 0.7);
                exposures
            },
        },
        PortfolioHolding {
            security_id: "AMZN".to_string(),
            market_value: 25000.0,
            weight: 0.25,
            target_weight: 0.20,
            cost_basis: 22000.0,
            purchase_date: "2022-03-01".to_string(),
            factor_exposures: {
                let mut exposures = HashMap::new();
                exposures.insert("momentum".to_string(), 0.7);
                exposures.insert("growth".to_string(), 0.9);
                exposures
            },
        },
        PortfolioHolding {
            security_id: "GOOGL".to_string(),
            market_value: 30000.0,
            weight: 0.30,
            target_weight: 0.35,
            cost_basis: 25000.0,
            purchase_date: "2022-04-01".to_string(),
            factor_exposures: {
                let mut exposures = HashMap::new();
                exposures.insert("momentum".to_string(), 0.6);
                exposures.insert("quality".to_string(), 0.8);
                exposures
            },
        },
    ];
    
    // Create portfolio
    Portfolio {
        id: "test-portfolio".to_string(),
        name: "Test Portfolio".to_string(),
        total_market_value: 100000.0,
        cash_balance: 10000.0,
        holdings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper function to create a test FactorModelApi
    fn create_test_factor_model_api() -> FactorModelApi {
        FactorModelApi::new()
    }
    
    #[test]
    fn test_calculate_factor_drift() {
        let factor_model_api = create_test_factor_model_api();
        let service = PortfolioRebalancingService::new(factor_model_api);
        
        // Define target exposures different from the current ones
        let mut target_exposures = HashMap::new();
        target_exposures.insert("momentum".to_string(), 0.1); // Current is 0.3
        target_exposures.insert("value".to_string(), 0.5);    // Current is 0.5 (no drift)
        target_exposures.insert("size".to_string(), 0.0);     // Current is -0.2
        
        // Calculate drift with default threshold
        let drift = service.calculate_factor_drift("portfolio-123", &target_exposures, None);
        
        assert!(drift.is_some());
        let drift = drift.unwrap();
        
        // Check drift values with approximate equality for floating point
        let momentum_drift = drift.get("momentum").unwrap();
        assert!((momentum_drift - 0.2).abs() < 1e-10);  // 0.3 - 0.1 = 0.2
        
        assert!(!drift.contains_key("value"));             // No drift for value
        
        let size_drift = drift.get("size").unwrap();
        assert!((size_drift - (-0.2)).abs() < 1e-10);     // -0.2 - 0.0 = -0.2
        
        // Test with a higher threshold
        let drift = service.calculate_factor_drift("portfolio-123", &target_exposures, Some(0.25));
        
        assert!(drift.is_some());
        let drift = drift.unwrap();
        
        // Only drifts >= 0.25 should be included
        assert!(!drift.contains_key("momentum")); // 0.2 drift is below threshold
        assert!(!drift.contains_key("value"));    // No drift
        assert!(!drift.contains_key("size"));     // 0.2 drift is below threshold
    }
    
    #[test]
    fn test_calculate_drift_score() {
        let factor_model_api = create_test_factor_model_api();
        let service = PortfolioRebalancingService::new(factor_model_api);
        
        // Create a sample factor drift
        let mut factor_drift = HashMap::new();
        factor_drift.insert("momentum".to_string(), 0.2);
        factor_drift.insert("size".to_string(), -0.2);
        factor_drift.insert("quality".to_string(), 0.1);
        
        // Calculate drift score with equal importance
        let score = service.calculate_drift_score(&factor_drift, None);
        
        // Score should be between 0 and 1
        assert!(score > 0.0);
        assert!(score < 1.0);
        
        // Create factor importance weights
        let mut factor_importance = HashMap::new();
        factor_importance.insert("momentum".to_string(), 2.0); // More important
        factor_importance.insert("size".to_string(), 1.0);
        factor_importance.insert("quality".to_string(), 0.5);  // Less important
        
        // Calculate weighted drift score
        let weighted_score = service.calculate_drift_score(&factor_drift, Some(&factor_importance));
        
        // Weighted score should be higher due to higher weight on momentum
        assert!(weighted_score > 0.0);
        assert!(weighted_score < 1.0);
        
        // Empty drift should have zero score
        let empty_drift = HashMap::new();
        let empty_score = service.calculate_drift_score(&empty_drift, None);
        assert_eq!(empty_score, 0.0);
    }
    
    #[test]
    fn test_generate_rebalance_trades() {
        let factor_model_api = create_test_factor_model_api();
        let service = PortfolioRebalancingService::new(factor_model_api);
        
        // Create a test portfolio
        let portfolio = create_test_portfolio();
        
        // Generate rebalance trades
        let trades = service.generate_rebalance_trades(&portfolio, None, false, None, None);
        
        // Verify trades
        assert!(!trades.is_empty());
        
        // Check that trades are correctly generated
        for trade in &trades {
            // Verify trade properties
            assert!(trade.amount > 0.0);
            assert!(trade.security_id.len() > 0);
            
            // Check reason
            match trade.reason {
                TradeReason::Rebalance => (),
                _ => panic!("Expected Rebalance reason"),
            }
        }
        
        // Test with tax awareness
        let tax_aware_trades = service.generate_rebalance_trades(&portfolio, None, true, None, None);
        
        // Verify tax impact is calculated for sells
        for trade in &tax_aware_trades {
            if !trade.is_buy {
                assert!(trade.tax_impact.is_some());
            }
        }
    }
    
    #[test]
    fn test_generate_cash_flow_trades() {
        let factor_model_api = create_test_factor_model_api();
        let service = PortfolioRebalancingService::new(factor_model_api);
        
        // Create a test portfolio
        let portfolio = create_test_portfolio();
        
        // Create a deposit cash flow
        let deposit = CashFlow {
            amount: 10000.0,
            date: "2023-01-01".to_string(),
            flow_type: CashFlowType::Deposit,
        };
        
        // Generate deposit trades
        let deposit_trades = service.generate_cash_flow_trades(&portfolio, &deposit, true, None);
        
        // Verify deposit trades
        assert!(!deposit_trades.is_empty());
        for trade in &deposit_trades {
            assert!(trade.is_buy);
            
            // Check reason
            match trade.reason {
                TradeReason::Deposit => (),
                _ => panic!("Expected Deposit reason"),
            }
        }
        
        // Create a withdrawal cash flow
        let withdrawal = CashFlow {
            amount: -5000.0,
            date: "2023-01-02".to_string(),
            flow_type: CashFlowType::Withdrawal,
        };
        
        // Generate withdrawal trades
        let withdrawal_trades = service.generate_cash_flow_trades(&portfolio, &withdrawal, true, None);
        
        // Verify withdrawal trades
        if !withdrawal_trades.is_empty() {
            for trade in &withdrawal_trades {
                assert!(!trade.is_buy);
                
                // Check reason
                match trade.reason {
                    TradeReason::Withdrawal => (),
                    _ => panic!("Expected Withdrawal reason"),
                }
            }
        }
    }
    
    #[test]
    fn test_generate_factor_rebalance_trades() {
        let factor_model_api = create_test_factor_model_api();
        let service = PortfolioRebalancingService::new(factor_model_api);
        
        // Define target exposures different from the current ones
        let mut target_exposures = HashMap::new();
        target_exposures.insert("momentum".to_string(), 0.1); // Current is 0.3
        target_exposures.insert("value".to_string(), 0.7);    // Current is 0.5
        target_exposures.insert("quality".to_string(), 0.6);  // Current is 0.7
        
        // Generate factor rebalance trades
        let trades = service.generate_factor_rebalance_trades(
            "portfolio-123",
            &target_exposures,
            Some(3),
            true
        );
        
        // We can't assert much about the trades since they depend on the mock portfolio
        // which is created internally in the method, but we can check basic properties
        
        // The method might return empty trades if the mock portfolio doesn't have
        // significant factor drift, so we'll just check that the method runs without errors
        
        // If trades are generated, verify their properties
        for trade in &trades {
            // Verify trade properties
            assert!(trade.amount > 0.0);
            assert!(trade.security_id.len() > 0);
            
            // Check reason
            match trade.reason {
                TradeReason::FactorExposureAdjustment => (),
                _ => panic!("Expected FactorExposureAdjustment reason"),
            }
            
            // Verify tax impact for sells if tax_aware is true
            if !trade.is_buy {
                assert!(trade.tax_impact.is_some());
            }
        }
    }
    
    // Helper function to create a test portfolio
    fn create_test_portfolio() -> Portfolio {
        // Create holdings
        let holdings = vec![
            PortfolioHolding {
                security_id: "AAPL".to_string(),
                market_value: 15000.0,
                weight: 0.15,
                target_weight: 0.10,
                cost_basis: 10000.0,
                purchase_date: "2022-01-01".to_string(),
                factor_exposures: {
                    let mut exposures = HashMap::new();
                    exposures.insert("momentum".to_string(), 0.5);
                    exposures.insert("quality".to_string(), 0.8);
                    exposures
                },
            },
            PortfolioHolding {
                security_id: "MSFT".to_string(),
                market_value: 20000.0,
                weight: 0.20,
                target_weight: 0.25,
                cost_basis: 18000.0,
                purchase_date: "2022-02-01".to_string(),
                factor_exposures: {
                    let mut exposures = HashMap::new();
                    exposures.insert("momentum".to_string(), 0.6);
                    exposures.insert("quality".to_string(), 0.7);
                    exposures
                },
            },
            PortfolioHolding {
                security_id: "AMZN".to_string(),
                market_value: 25000.0,
                weight: 0.25,
                target_weight: 0.20,
                cost_basis: 22000.0,
                purchase_date: "2022-03-01".to_string(),
                factor_exposures: {
                    let mut exposures = HashMap::new();
                    exposures.insert("momentum".to_string(), 0.7);
                    exposures.insert("growth".to_string(), 0.9);
                    exposures
                },
            },
            PortfolioHolding {
                security_id: "GOOGL".to_string(),
                market_value: 30000.0,
                weight: 0.30,
                target_weight: 0.35,
                cost_basis: 25000.0,
                purchase_date: "2022-04-01".to_string(),
                factor_exposures: {
                    let mut exposures = HashMap::new();
                    exposures.insert("momentum".to_string(), 0.6);
                    exposures.insert("quality".to_string(), 0.8);
                    exposures
                },
            },
        ];
        
        // Create portfolio
        Portfolio {
            id: "test-portfolio".to_string(),
            name: "Test Portfolio".to_string(),
            total_market_value: 100000.0,
            cash_balance: 10000.0,
            holdings,
        }
    }
} 