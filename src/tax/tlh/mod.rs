use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime};
use crate::model_portfolio::{UnifiedManagedAccount, HoldingPeriod, TaxLot};
use crate::portfolio::rebalancing::{RebalanceTrade, TradeReason};
use crate::common::error::{Error, Result};

/// Configuration for algorithmic tax-loss harvesting
#[derive(Debug, Clone)]
pub struct AlgorithmicTLHConfig {
    /// Minimum market value for a position to be considered for TLH
    pub min_position_value: f64,
    /// Minimum loss percentage to trigger harvesting (e.g., 0.05 for 5%)
    pub min_loss_percentage: f64,
    /// Minimum loss amount to trigger harvesting
    pub min_loss_amount: f64,
    /// Maximum percentage of portfolio to harvest in a single day
    pub max_daily_harvest_percentage: f64,
    /// Minimum days between harvests for the same security
    pub harvest_frequency_days: u32,
    /// Whether to automatically reinvest in correlated securities
    pub auto_reinvest: bool,
    /// Correlation threshold for replacement securities (0.0 to 1.0)
    pub correlation_threshold: f64,
    /// Whether to use market volatility to adjust harvesting thresholds
    pub volatility_adjusted_thresholds: bool,
    /// Whether to consider tax-rate arbitrage (short-term vs long-term)
    pub tax_rate_arbitrage: bool,
    /// Whether to enable wash sale prevention
    pub wash_sale_prevention: bool,
    /// Number of days to avoid repurchasing the same security (default 30)
    pub wash_sale_window_days: u32,
}

impl Default for AlgorithmicTLHConfig {
    fn default() -> Self {
        Self {
            min_position_value: 5000.0,
            min_loss_percentage: 0.05,
            min_loss_amount: 500.0,
            max_daily_harvest_percentage: 0.10,
            harvest_frequency_days: 7,
            auto_reinvest: true,
            correlation_threshold: 0.90,
            volatility_adjusted_thresholds: true,
            tax_rate_arbitrage: true,
            wash_sale_prevention: true,
            wash_sale_window_days: 30,
        }
    }
}

/// Status of a security for tax-loss harvesting
#[derive(Debug, Clone)]
pub struct SecurityTLHStatus {
    /// Security identifier
    pub security_id: String,
    /// Last time this security was harvested
    pub last_harvested: Option<SystemTime>,
    /// Current market value
    pub current_value: f64,
    /// Cost basis
    pub cost_basis: f64,
    /// Unrealized gain/loss
    pub unrealized_gain_loss: f64,
    /// Unrealized gain/loss percentage
    pub unrealized_gain_loss_percentage: f64,
    /// Whether this security is currently on the wash sale list
    pub in_wash_sale_window: bool,
    /// Potential tax savings if harvested
    pub potential_tax_savings: f64,
    /// Recommended replacement securities
    pub replacement_candidates: Vec<ReplacementSecurity>,
}

/// Replacement security for tax-loss harvesting
#[derive(Debug, Clone)]
pub struct ReplacementSecurity {
    /// Security identifier
    pub security_id: String,
    /// Correlation with the original security
    pub correlation: f64,
    /// Tracking error relative to the original security
    pub tracking_error: f64,
    /// Factor exposure similarity score (0.0 to 1.0)
    pub factor_similarity: f64,
    /// Whether this security has been recently sold (wash sale concern)
    pub recently_sold: bool,
}

/// Real-time market data for a security
#[derive(Debug, Clone)]
pub struct MarketData {
    /// Security identifier
    pub security_id: String,
    /// Current price
    pub current_price: f64,
    /// Daily volatility
    pub daily_volatility: f64,
    /// 30-day volatility
    pub volatility_30d: f64,
    /// Trading volume
    pub volume: u64,
    /// Bid-ask spread
    pub bid_ask_spread: f64,
    /// Timestamp of the data
    pub timestamp: SystemTime,
}

/// Algorithmic tax-loss harvesting service
pub struct AlgorithmicTLHService {
    /// Configuration for the service
    config: AlgorithmicTLHConfig,
    /// Market data provider
    market_data: HashMap<String, MarketData>,
    /// Security correlation matrix
    correlation_matrix: HashMap<(String, String), f64>,
    /// Harvest history
    harvest_history: HashMap<String, Vec<SystemTime>>,
    /// Wash sale tracking
    wash_sale_list: HashMap<String, SystemTime>,
    /// Replacement security database
    replacement_database: HashMap<String, Vec<ReplacementSecurity>>,
}

impl AlgorithmicTLHService {
    /// Create a new algorithmic TLH service with the given configuration
    pub fn new(config: AlgorithmicTLHConfig) -> Self {
        Self {
            config,
            market_data: HashMap::new(),
            correlation_matrix: HashMap::new(),
            harvest_history: HashMap::new(),
            wash_sale_list: HashMap::new(),
            replacement_database: HashMap::new(),
        }
    }

    /// Update market data for a security
    pub fn update_market_data(&mut self, data: MarketData) {
        self.market_data.insert(data.security_id.clone(), data);
    }

    /// Update correlation between two securities
    pub fn update_correlation(&mut self, security1: &str, security2: &str, correlation: f64) {
        self.correlation_matrix.insert((security1.to_string(), security2.to_string()), correlation);
        self.correlation_matrix.insert((security2.to_string(), security1.to_string()), correlation);
    }

    /// Record a harvest for a security
    pub fn record_harvest(&mut self, security_id: &str) {
        let now = SystemTime::now();
        let history = self.harvest_history.entry(security_id.to_string()).or_insert_with(Vec::new);
        history.push(now);
        
        // Add to wash sale list if wash sale prevention is enabled
        if self.config.wash_sale_prevention {
            self.wash_sale_list.insert(security_id.to_string(), now);
        }
    }

    /// Clean up expired wash sale entries
    pub fn clean_wash_sale_list(&mut self) {
        let now = SystemTime::now();
        let window = Duration::from_secs(self.config.wash_sale_window_days as u64 * 24 * 60 * 60);
        
        self.wash_sale_list.retain(|_, timestamp| {
            match now.duration_since(*timestamp) {
                Ok(duration) => duration < window,
                Err(_) => true, // Keep if there's an error calculating duration
            }
        });
    }

    /// Check if a security is eligible for harvesting
    pub fn is_eligible_for_harvest(&self, security_id: &str, tax_lot: &TaxLot) -> bool {
        // Check if the security is in the wash sale window
        if self.config.wash_sale_prevention {
            if let Some(timestamp) = self.wash_sale_list.get(security_id) {
                let now = SystemTime::now();
                let window = Duration::from_secs(self.config.wash_sale_window_days as u64 * 24 * 60 * 60);
                
                if let Ok(duration) = now.duration_since(*timestamp) {
                    if duration < window {
                        return false;
                    }
                }
            }
        }
        
        // Check harvest frequency
        if let Some(history) = self.harvest_history.get(security_id) {
            if let Some(last_harvest) = history.last() {
                let now = SystemTime::now();
                let min_duration = Duration::from_secs(self.config.harvest_frequency_days as u64 * 24 * 60 * 60);
                
                if let Ok(duration) = now.duration_since(*last_harvest) {
                    if duration < min_duration {
                        return false;
                    }
                }
            }
        }
        
        // Check minimum position value
        let position_value = tax_lot.quantity * tax_lot.market_value_per_share;
        if position_value < self.config.min_position_value {
            return false;
        }
        
        // Check minimum loss percentage and amount
        let loss_percentage = tax_lot.unrealized_gain_loss / (tax_lot.quantity * tax_lot.cost_basis_per_share);
        let loss_amount = tax_lot.unrealized_gain_loss;
        
        if loss_percentage > -self.config.min_loss_percentage {
            return false;
        }
        
        if loss_amount > -self.config.min_loss_amount {
            return false;
        }
        
        true
    }

    /// Find replacement securities for a given security
    pub fn find_replacement_securities(&self, security_id: &str) -> Vec<ReplacementSecurity> {
        let mut replacements = Vec::new();
        
        // Check if we have pre-computed replacements
        if let Some(candidates) = self.replacement_database.get(security_id) {
            return candidates.clone();
        }
        
        // Otherwise, find replacements based on correlation
        for ((sec1, sec2), correlation) in &self.correlation_matrix {
            if sec1 == security_id && *correlation >= self.config.correlation_threshold {
                // Check if the replacement is not in the wash sale window
                let is_in_wash_window = if let Some(timestamp) = self.wash_sale_list.get(sec2) {
                    let now = SystemTime::now();
                    let window = Duration::from_secs(self.config.wash_sale_window_days as u64 * 24 * 60 * 60);
                    
                    match now.duration_since(*timestamp) {
                        Ok(duration) => duration < window,
                        Err(_) => false,
                    }
                } else {
                    false
                };
                
                replacements.push(ReplacementSecurity {
                    security_id: sec2.clone(),
                    correlation: *correlation,
                    tracking_error: 1.0 - *correlation, // Simple approximation
                    factor_similarity: *correlation,    // Simple approximation
                    recently_sold: is_in_wash_window,
                });
            }
        }
        
        // Sort by correlation (highest first)
        replacements.sort_by(|a, b| b.correlation.partial_cmp(&a.correlation).unwrap());
        
        replacements
    }

    /// Generate real-time tax-loss harvesting trades for an account
    pub fn generate_real_time_tlh_trades(
        &mut self,
        uma: &UnifiedManagedAccount,
        market_volatility: Option<f64>
    ) -> Result<Vec<RebalanceTrade>> {
        // Ensure the account has tax settings
        let tax_settings = match &uma.tax_settings {
            Some(settings) => settings,
            None => return Err(Error::validation("Account does not have tax optimization settings")),
        };
        
        let mut harvest_trades = Vec::new();
        let mut reinvest_trades = Vec::new();
        let mut total_harvest_value = 0.0;
        let max_harvest_value = uma.total_market_value * self.config.max_daily_harvest_percentage;
        
        // Clean up expired wash sale entries
        self.clean_wash_sale_list();
        
        // Get all tax lots
        let all_lots = uma.all_tax_lots();
        
        // Adjust loss percentage based on market volatility
        let mut _adjusted_min_loss_percentage = self.config.min_loss_percentage;
        
        // If market volatility is provided, adjust the loss percentage
        if let Some(volatility) = market_volatility {
            if volatility > 0.0 {
                _adjusted_min_loss_percentage = self.config.min_loss_percentage * (1.0 + volatility);
            }
        }
        
        // Track securities to avoid wash sales
        let mut harvested_securities = HashSet::new();
        
        // First pass: identify all eligible lots and sort by tax savings potential
        let mut eligible_lots: Vec<_> = all_lots.iter()
            .filter(|(_, lot)| {
                lot.unrealized_gain_loss < 0.0 && 
                self.is_eligible_for_harvest(&lot.security_id, lot)
            })
            .collect();
        
        // Sort by tax savings potential (largest first)
        eligible_lots.sort_by(|(_, a), (_, b)| {
            let a_tax_rate = if a.holding_period == HoldingPeriod::ShortTerm {
                tax_settings.short_term_tax_rate
            } else {
                tax_settings.long_term_tax_rate
            };
            
            let b_tax_rate = if b.holding_period == HoldingPeriod::ShortTerm {
                tax_settings.short_term_tax_rate
            } else {
                tax_settings.long_term_tax_rate
            };
            
            let a_savings = -a.unrealized_gain_loss * a_tax_rate;
            let b_savings = -b.unrealized_gain_loss * b_tax_rate;
            
            b_savings.partial_cmp(&a_savings).unwrap()
        });
        
        // Second pass: generate trades up to the maximum daily harvest value
        for (_sleeve_id, lot) in eligible_lots {
            // Skip if we've reached the maximum harvest value
            if total_harvest_value >= max_harvest_value {
                break;
            }
            
            // Calculate the trade amount and tax impact
            let trade_amount = lot.quantity * lot.market_value_per_share;
            let tax_impact = lot.unrealized_gain_loss;
            
            // Skip small trades
            if trade_amount < self.config.min_position_value {
                continue;
            }
            
            // Create the sell trade
            let sell_trade = RebalanceTrade {
                security_id: lot.security_id.clone(),
                amount: trade_amount,
                is_buy: false,
                reason: TradeReason::TaxLossHarvesting,
                tax_impact: Some(tax_impact),
            };
            
            // Add to harvest trades
            harvest_trades.push(sell_trade);
            total_harvest_value += trade_amount;
            
            // Record the harvest
            self.record_harvest(&lot.security_id);
            harvested_securities.insert(lot.security_id.clone());
            
            // Find replacement securities if auto-reinvest is enabled
            if self.config.auto_reinvest {
                let replacements = self.find_replacement_securities(&lot.security_id);
                
                // Choose the best replacement that's not in the wash sale window
                for replacement in replacements {
                    if !replacement.recently_sold && !harvested_securities.contains(&replacement.security_id) {
                        // Create the buy trade for the replacement
                        let buy_trade = RebalanceTrade {
                            security_id: replacement.security_id.clone(),
                            amount: trade_amount,
                            is_buy: true,
                            reason: TradeReason::TaxLossHarvesting,
                            tax_impact: None,
                        };
                        
                        reinvest_trades.push(buy_trade);
                        break;
                    }
                }
            }
        }
        
        // Combine all trades
        let mut all_trades = Vec::new();
        all_trades.extend(harvest_trades);
        all_trades.extend(reinvest_trades);
        
        Ok(all_trades)
    }

    /// Monitor market conditions and automatically trigger TLH when appropriate
    pub fn monitor_and_harvest(
        &mut self,
        uma: &UnifiedManagedAccount,
        market_volatility: Option<f64>,
        execution_callback: impl Fn(Vec<RebalanceTrade>) -> Result<()>
    ) -> Result<()> {
        // Generate trades based on current market conditions
        let trades = self.generate_real_time_tlh_trades(uma, market_volatility)?;
        
        // If there are trades to execute, call the execution callback
        if !trades.is_empty() {
            execution_callback(trades)?;
        }
        
        Ok(())
    }

    /// Analyze the performance of tax-loss harvesting
    pub fn analyze_tlh_performance(&self, _uma: &UnifiedManagedAccount) -> TLHPerformanceReport {
        // In a real implementation, this would analyze historical harvesting data
        // and calculate metrics like total tax savings, harvest frequency, etc.
        
        TLHPerformanceReport {
            total_tax_savings: 0.0,
            total_harvested_losses: 0.0,
            harvest_count: 0,
            average_harvest_amount: 0.0,
            largest_harvest: 0.0,
            harvest_efficiency: 0.0,
        }
    }
}

/// Performance report for tax-loss harvesting
#[derive(Debug, Clone)]
pub struct TLHPerformanceReport {
    /// Total tax savings from harvesting
    pub total_tax_savings: f64,
    /// Total harvested losses
    pub total_harvested_losses: f64,
    /// Number of harvests
    pub harvest_count: usize,
    /// Average harvest amount
    pub average_harvest_amount: f64,
    /// Largest single harvest
    pub largest_harvest: f64,
    /// Harvest efficiency (tax savings / potential tax savings)
    pub harvest_efficiency: f64,
} 