use std::collections::HashMap;
use chrono::{Utc, NaiveDate, Duration, Datelike};
use uuid::Uuid;
use crate::portfolio::model::{UnifiedManagedAccount, TaxOptimizationSettings, ESGScreeningCriteria};
use crate::portfolio::rebalancing::{RebalanceTrade, TradeReason, PortfolioHolding};
use crate::common::error::Result;
use super::household_types::*;

/// Represents a household member
#[derive(Debug, Clone)]
pub struct HouseholdMember {
    /// Member identifier
    pub id: String,
    /// Member name
    pub name: String,
    /// Relationship to primary member
    pub relationship: MemberRelationship,
    /// Tax filing status
    pub tax_filing_status: Option<TaxFilingStatus>,
    /// Member-specific tax settings
    pub tax_settings: Option<TaxOptimizationSettings>,
    /// Member-specific ESG preferences
    pub esg_preferences: Option<ESGScreeningCriteria>,
}

/// Relationship to primary household member
#[derive(Debug, Clone, PartialEq)]
pub enum MemberRelationship {
    /// Primary household member
    Primary,
    /// Spouse
    Spouse,
    /// Child
    Child,
    /// Dependent
    Dependent,
    /// Other relationship
    Other(String),
}

/// Tax filing status
#[derive(Debug, Clone, PartialEq)]
pub enum TaxFilingStatus {
    /// Single
    Single,
    /// Married filing jointly
    MarriedJoint,
    /// Married filing separately
    MarriedSeparate,
    /// Head of household
    HeadOfHousehold,
    /// Qualifying widow(er)
    QualifyingWidow,
}

/// Account type for tax treatment
#[derive(Debug, Clone, PartialEq)]
pub enum AccountTaxType {
    /// Taxable account
    Taxable,
    /// Tax-deferred account (e.g., Traditional IRA, 401(k))
    TaxDeferred,
    /// Tax-exempt account (e.g., Roth IRA)
    TaxExempt,
    /// Trust account
    Trust,
    /// Other account type
    Other(String),
}

/// Represents a unified managed household
#[derive(Debug, Clone)]
pub struct UnifiedManagedHousehold {
    /// Household identifier
    pub id: String,
    /// Household name
    pub name: String,
    /// Household members
    pub members: Vec<HouseholdMember>,
    /// Accounts in the household
    pub accounts: HashMap<String, UnifiedManagedAccount>,
    /// Account ownership mapping (account_id -> member_ids)
    pub account_ownership: HashMap<String, Vec<String>>,
    /// Account tax types (account_id -> tax_type)
    pub account_tax_types: HashMap<String, AccountTaxType>,
    /// Household-level tax settings
    pub household_tax_settings: Option<TaxOptimizationSettings>,
    /// Household-level ESG criteria
    pub household_esg_criteria: Option<ESGScreeningCriteria>,
    /// Created date
    pub created_at: String,
    /// Last updated date
    pub updated_at: String,
    /// Financial goals
    pub financial_goals: Vec<FinancialGoal>,
    /// Goal contributions
    pub goal_contributions: HashMap<String, Vec<GoalContribution>>,
    /// Estate plans
    pub estate_plans: Vec<EstatePlan>,
    /// Beneficiary designations
    pub beneficiary_designations: Vec<BeneficiaryDesignation>,
    /// Charitable vehicles
    pub charitable_vehicles: Vec<CharitableVehicle>,
    /// Charities
    pub charities: Vec<Charity>,
    /// Donations
    pub donations: Vec<CharitableDonation>,
}

impl UnifiedManagedHousehold {
    /// Create a new household with a primary member
    pub fn new(
        id: String,
        name: String,
        primary_member_name: String,
    ) -> Self {
        let primary_member = HouseholdMember {
            id: format!("member-{}", uuid::Uuid::new_v4()),
            name: primary_member_name,
            relationship: MemberRelationship::Primary,
            tax_filing_status: None,
            tax_settings: None,
            esg_preferences: None,
        };
        
        let now = chrono::Utc::now().to_rfc3339();
        
        Self {
            id,
            name,
            members: vec![primary_member],
            accounts: HashMap::new(),
            account_ownership: HashMap::new(),
            account_tax_types: HashMap::new(),
            household_tax_settings: None,
            household_esg_criteria: None,
            created_at: now.clone(),
            updated_at: now,
            financial_goals: Vec::new(),
            goal_contributions: HashMap::new(),
            estate_plans: Vec::new(),
            beneficiary_designations: Vec::new(),
            charitable_vehicles: Vec::new(),
            charities: Vec::new(),
            donations: Vec::new(),
        }
    }
    
    /// Add a member to the household
    pub fn add_member(&mut self, name: String, relationship: MemberRelationship) -> &HouseholdMember {
        let member = HouseholdMember {
            id: format!("member-{}", uuid::Uuid::new_v4()),
            name,
            relationship,
            tax_filing_status: None,
            tax_settings: None,
            esg_preferences: None,
        };
        
        self.members.push(member);
        self.updated_at = chrono::Utc::now().to_rfc3339();
        
        self.members.last().unwrap()
    }
    
    /// Add an account to the household
    pub fn add_account(
        &mut self,
        account: UnifiedManagedAccount,
        member_ids: Vec<String>,
        tax_type: AccountTaxType,
    ) -> Result<()> {
        // Validate that all member IDs exist
        for member_id in &member_ids {
            if !self.members.iter().any(|m| &m.id == member_id) {
                return Err(format!("Member with ID {} not found in household", member_id).into());
            }
        }
        
        let account_id = account.id.clone();
        self.accounts.insert(account_id.clone(), account);
        self.account_ownership.insert(account_id.clone(), member_ids);
        self.account_tax_types.insert(account_id, tax_type);
        self.updated_at = chrono::Utc::now().to_rfc3339();
        
        Ok(())
    }
    
    /// Get the total household market value
    pub fn total_market_value(&self) -> f64 {
        self.accounts.values().map(|a| a.total_market_value).sum()
    }
    
    /// Get the total household cash balance
    pub fn total_cash_balance(&self) -> f64 {
        self.accounts.values().map(|a| a.cash_balance).sum()
    }
    
    /// Get accounts owned by a specific member
    pub fn get_member_accounts(&self, member_id: &str) -> Vec<&UnifiedManagedAccount> {
        let account_ids: Vec<&String> = self.account_ownership.iter()
            .filter(|(_, member_ids)| member_ids.contains(&member_id.to_string()))
            .map(|(account_id, _)| account_id)
            .collect();
            
        account_ids.iter()
            .filter_map(|id| self.accounts.get(*id))
            .collect()
    }
    
    /// Get accounts by tax type
    pub fn get_accounts_by_tax_type(&self, tax_type: &AccountTaxType) -> Vec<&UnifiedManagedAccount> {
        let account_ids: Vec<&String> = self.account_tax_types.iter()
            .filter(|(_, account_tax_type)| *account_tax_type == tax_type)
            .map(|(account_id, _)| account_id)
            .collect();
            
        account_ids.iter()
            .filter_map(|id| self.accounts.get(*id))
            .collect()
    }
    
    /// Apply household-level tax optimization
    pub fn apply_household_tax_optimization(&mut self, settings: TaxOptimizationSettings) {
        self.household_tax_settings = Some(settings);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
    
    /// Apply household-level ESG screening
    pub fn apply_household_esg_screening(&mut self, criteria: ESGScreeningCriteria) {
        self.household_esg_criteria = Some(criteria);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
    
    pub fn add_estate_plan(&mut self, plan: EstatePlan) {
        self.estate_plans.push(plan);
    }
    
    pub fn update_estate_plan(&mut self, plan_id: &str, updated_plan: EstatePlan) -> Result<()> {
        if let Some(index) = self.estate_plans.iter().position(|p| p.id == plan_id) {
            self.estate_plans[index] = updated_plan;
            Ok(())
        } else {
            Err("Estate plan not found".into())
        }
    }
    
    pub fn add_beneficiary_designation(&mut self, designation: BeneficiaryDesignation) {
        self.beneficiary_designations.push(designation);
    }
    
    pub fn update_beneficiary_designation(&mut self, account_id: &str, updated_designation: BeneficiaryDesignation) -> Result<()> {
        if let Some(index) = self.beneficiary_designations.iter().position(|d| d.account_id == account_id) {
            self.beneficiary_designations[index] = updated_designation;
            Ok(())
        } else {
            Err("Beneficiary designation not found".into())
        }
    }
    
    pub fn add_charity(&mut self, charity: Charity) {
        self.charities.push(charity);
    }
    
    pub fn add_charitable_vehicle(&mut self, vehicle: CharitableVehicle) {
        self.charitable_vehicles.push(vehicle);
    }
    
    pub fn add_donation(&mut self, donation: CharitableDonation) {
        self.donations.push(donation);
    }
    
    pub fn get_charity(&self, charity_id: &str) -> Option<&Charity> {
        self.charities.iter().find(|c| c.id == charity_id)
    }
    
    pub fn get_charitable_vehicle(&self, vehicle_id: &str) -> Option<&CharitableVehicle> {
        self.charitable_vehicles.iter().find(|v| v.id == vehicle_id)
    }
    
    pub fn update_charity(&mut self, charity_id: &str, updated_charity: Charity) -> Result<()> {
        if let Some(index) = self.charities.iter().position(|c| c.id == charity_id) {
            self.charities[index] = updated_charity;
            Ok(())
        } else {
            Err("Charity not found".into())
        }
    }
    
    pub fn update_charitable_vehicle(&mut self, vehicle_id: &str, updated_vehicle: CharitableVehicle) -> Result<()> {
        if let Some(index) = self.charitable_vehicles.iter().position(|v| v.id == vehicle_id) {
            self.charitable_vehicles[index] = updated_vehicle;
            Ok(())
        } else {
            Err("Charitable vehicle not found".into())
        }
    }
}

/// Service for managing unified managed households
pub struct HouseholdService {
    // Dependencies would go here
}

impl HouseholdService {
    /// Creates a new instance of the HouseholdService.
    ///
    /// This service provides functionality for managing households, including:
    /// - Creating and managing household members
    /// - Managing accounts within the household
    /// - Analyzing household risk and asset allocation
    /// - Generating tax-optimized trades
    /// - Creating withdrawal plans
    /// - Managing charitable giving
    /// - Planning for estate distribution
    ///
    /// # Returns
    ///
    /// A new instance of `HouseholdService`.
    ///
    /// # Examples
    ///
    /// ```
    /// use investment_management::portfolio::model::household::HouseholdService;
    ///
    /// let service = HouseholdService::new();
    /// ```
    pub fn new() -> Self {
        Self {}
    }
    
    /// Creates a new household with a primary member.
    ///
    /// This method initializes a new household with the given name and creates
    /// a primary member with the provided name. The household is assigned a unique
    /// identifier.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the household
    /// * `primary_member_name` - The name of the primary household member
    ///
    /// # Returns
    ///
    /// A new `UnifiedManagedHousehold` instance with the primary member.
    ///
    /// # Examples
    ///
    /// ```
    /// use investment_management::portfolio::model::household::HouseholdService;
    ///
    /// let service = HouseholdService::new();
    /// let household = service.create_household("Smith Family", "John Smith");
    ///
    /// assert_eq!(household.name, "Smith Family");
    /// assert_eq!(household.members.len(), 1);
    /// assert_eq!(household.members[0].name, "John Smith");
    /// ```
    pub fn create_household(
        &self,
        name: &str,
        primary_member_name: &str,
    ) -> UnifiedManagedHousehold {
        UnifiedManagedHousehold::new(
            format!("household-{}", uuid::Uuid::new_v4()),
            name.to_string(),
            primary_member_name.to_string(),
        )
    }
    
    /// Generates tax-optimized trades across all accounts in the household.
    ///
    /// This method analyzes the household's accounts and generates a set of trades
    /// that optimize for tax efficiency while maintaining the desired asset allocation.
    /// It considers:
    /// - Tax-loss harvesting opportunities
    /// - Asset location efficiency (placing tax-efficient assets in taxable accounts)
    /// - Tax-aware rebalancing
    /// - Household-level tax budget constraints
    ///
    /// # Parameters
    ///
    /// * `household` - The household for which to generate trades
    ///
    /// # Returns
    ///
    /// A vector of tuples, where each tuple contains an account ID and a vector of
    /// trades for that account. The trades are optimized to minimize tax impact
    /// while maintaining the desired asset allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use investment_management::portfolio::model::household::HouseholdService;
    ///
    /// let service = HouseholdService::new();
    /// let household = service.create_household("Smith Family", "John Smith");
    /// // Add accounts and assets to the household...
    ///
    /// let trades = service.generate_household_tax_optimized_trades(&household);
    /// for (account_id, account_trades) in trades {
    ///     println!("Trades for account {}: {} trades", account_id, account_trades.len());
    /// }
    /// ```
    pub fn generate_household_tax_optimized_trades(
        &self,
        household: &UnifiedManagedHousehold,
    ) -> Vec<(String, Vec<RebalanceTrade>)> {
        let mut all_trades = Vec::new();
        
        // Step 1: Identify tax-loss harvesting opportunities in taxable accounts
        let taxable_accounts = household.get_accounts_by_tax_type(&AccountTaxType::Taxable);
        for account in taxable_accounts {
            let tlh_trades = self.identify_tax_loss_harvesting_opportunities(account);
            if !tlh_trades.is_empty() {
                all_trades.push((account.id.clone(), tlh_trades));
            }
        }
        
        // Step 2: Identify asset location optimization opportunities
        let asset_location_trades = self.generate_asset_location_trades(household);
        for (account_id, trades) in asset_location_trades {
            // Check if we already have trades for this account
            if let Some(pos) = all_trades.iter().position(|(id, _)| id == &account_id) {
                // Append to existing trades
                all_trades[pos].1.extend(trades);
            } else {
                // Add new entry
                all_trades.push((account_id, trades));
            }
        }
        
        // Step 3: Apply household-level tax budget constraints
        if let Some(tax_settings) = &household.household_tax_settings {
            all_trades = self.optimize_household_trades_for_tax_budget(
                household,
                all_trades,
                tax_settings.annual_tax_budget.unwrap_or(f64::MAX),
                tax_settings.realized_gains_ytd,
            );
        }
        
        all_trades
    }
    
    /// Identify tax-loss harvesting opportunities in an account
    fn identify_tax_loss_harvesting_opportunities(
        &self,
        account: &UnifiedManagedAccount,
    ) -> Vec<RebalanceTrade> {
        let mut tlh_trades = Vec::new();
        
        // Only consider accounts with tax settings
        if account.tax_settings.is_none() {
            return tlh_trades;
        }
        
        let tax_settings = account.tax_settings.as_ref().unwrap();
        
        // Check if tax-loss harvesting is enabled
        if !tax_settings.prioritize_loss_harvesting {
            return tlh_trades;
        }
        
        // Get the minimum tax savings threshold
        let min_tax_savings = tax_settings.min_tax_savings_threshold.unwrap_or(100.0);
        
        // Iterate through all sleeves and holdings
        for sleeve in &account.sleeves {
            for holding in &sleeve.holdings {
                // Calculate unrealized loss
                let unrealized_gain_loss = holding.market_value - holding.cost_basis;
                
                // Only consider positions with losses
                if unrealized_gain_loss >= 0.0 {
                    continue;
                }
                
                // Calculate potential tax savings
                let potential_tax_savings = -unrealized_gain_loss * tax_settings.short_term_tax_rate;
                
                // Check if the potential tax savings meets the threshold
                if potential_tax_savings < min_tax_savings {
                    continue;
                }
                
                // Create a sell trade
                let sell_trade = RebalanceTrade {
                    security_id: holding.security_id.clone(),
                    amount: holding.market_value,
                    is_buy: false,
                    reason: TradeReason::TaxLossHarvesting,
                    tax_impact: Some(-potential_tax_savings), // Negative tax impact means tax savings
                };
                
                tlh_trades.push(sell_trade);
                
                // Find a suitable replacement security
                if let Some(replacement_id) = self.find_tax_loss_harvesting_replacement(&holding.security_id) {
                    // Create a buy trade for the replacement
                    let buy_trade = RebalanceTrade {
                        security_id: replacement_id,
                        amount: holding.market_value,
                        is_buy: true,
                        reason: TradeReason::TaxLossHarvesting,
                        tax_impact: None,
                    };
                    
                    tlh_trades.push(buy_trade);
                }
            }
        }
        
        tlh_trades
    }
    
    /// Find a suitable replacement security for tax-loss harvesting
    fn find_tax_loss_harvesting_replacement(&self, security_id: &str) -> Option<String> {
        // In a real implementation, this would find a security with similar characteristics
        // but not substantially identical to avoid wash sale rules
        
        // For now, use a simple mapping
        match security_id {
            "AAPL" => Some("MSFT".to_string()),
            "MSFT" => Some("AAPL".to_string()),
            "AMZN" => Some("GOOGL".to_string()),
            "GOOGL" => Some("AMZN".to_string()),
            "JPM" => Some("BAC".to_string()),
            "BAC" => Some("JPM".to_string()),
            _ => None,
        }
    }
    
    /// Generate trades to optimize asset location across accounts
    fn generate_asset_location_trades(
        &self,
        household: &UnifiedManagedHousehold,
    ) -> Vec<(String, Vec<RebalanceTrade>)> {
        let mut account_trades = Vec::new();
        
        // Get asset location recommendations
        let recommendations = self.generate_asset_location_recommendations(household);
        
        // Convert recommendations to trades
        for recommendation in recommendations {
            // Create sell trade in source account
            let sell_trade = RebalanceTrade {
                security_id: recommendation.security_id.clone(),
                amount: recommendation.market_value,
                is_buy: false,
                reason: TradeReason::Rebalance, // Use Rebalance as a substitute
                tax_impact: None, // Would be calculated in a real implementation
            };
            
            // Create buy trade in target account
            let buy_trade = RebalanceTrade {
                security_id: recommendation.security_id.clone(),
                amount: recommendation.market_value,
                is_buy: true,
                reason: TradeReason::Rebalance, // Use Rebalance as a substitute
                tax_impact: None,
            };
            
            // Add sell trade to source account
            let source_id = recommendation.source_account_id.clone();
            if let Some(pos) = account_trades.iter().position(|(id, _): &(String, Vec<RebalanceTrade>)| id == &source_id) {
                account_trades[pos].1.push(sell_trade);
            } else {
                account_trades.push((source_id, vec![sell_trade]));
            }
            
            // Add buy trade to target account
            let target_id = recommendation.target_account_id.clone();
            if let Some(pos) = account_trades.iter().position(|(id, _): &(String, Vec<RebalanceTrade>)| id == &target_id) {
                account_trades[pos].1.push(buy_trade);
            } else {
                account_trades.push((target_id, vec![buy_trade]));
            }
        }
        
        account_trades
    }
    
    /// Optimize trades across the household to stay within tax budget
    fn optimize_household_trades_for_tax_budget(
        &self,
        _household: &UnifiedManagedHousehold,
        account_trades: Vec<(String, Vec<RebalanceTrade>)>,
        tax_budget: f64,
        realized_gains_ytd: f64,
    ) -> Vec<(String, Vec<RebalanceTrade>)> {
        // Calculate remaining tax budget
        let remaining_budget = tax_budget - realized_gains_ytd;
        
        // If no budget left, prioritize only tax-loss harvesting trades
        if remaining_budget <= 0.0 {
            return account_trades.into_iter()
                .map(|(account_id, trades)| {
                    let filtered_trades = trades.into_iter()
                        .filter(|t| {
                            t.is_buy || // Keep all buys
                            t.reason == TradeReason::TaxLossHarvesting || // Keep TLH sells
                            t.tax_impact.unwrap_or(0.0) <= 0.0 // Keep trades with no tax impact or tax savings
                        })
                        .collect::<Vec<_>>();
                    (account_id, filtered_trades)
                })
                .filter(|(_, trades)| !trades.is_empty())
                .collect();
        }
        
        // Calculate total tax impact of all sell trades
        let mut total_tax_impact = 0.0;
        for (_, trades) in &account_trades {
            for trade in trades {
                if !trade.is_buy && trade.tax_impact.is_some() {
                    total_tax_impact += trade.tax_impact.unwrap();
                }
            }
        }
        
        // If within budget, return all trades
        if total_tax_impact <= remaining_budget {
            return account_trades;
        }
        
        // Need to prioritize trades to stay within budget
        let mut prioritized_trades = Vec::new();
        
        // First, add all tax-loss harvesting trades (they reduce tax burden)
        for (account_id, trades) in &account_trades {
            let tlh_trades: Vec<_> = trades.iter()
                .filter(|t| t.reason == TradeReason::TaxLossHarvesting || t.is_buy)
                .cloned()
                .collect();
            
            if !tlh_trades.is_empty() {
                prioritized_trades.push((account_id.clone(), tlh_trades));
            }
        }
        
        // Then, add other trades until we hit the budget
        let mut current_tax_impact = 0.0;
        
        // Calculate current tax impact from TLH trades
        for (_, trades) in &prioritized_trades {
            for trade in trades {
                if !trade.is_buy && trade.tax_impact.is_some() {
                    current_tax_impact += trade.tax_impact.unwrap();
                }
            }
        }
        
        // Sort remaining trades by tax impact (lowest first)
        let mut remaining_trades = Vec::new();
        for (account_id, trades) in &account_trades {
            for trade in trades {
                if trade.reason != TradeReason::TaxLossHarvesting && !trade.is_buy {
                    remaining_trades.push((account_id.clone(), trade.clone()));
                }
            }
        }
        
        // Sort by tax impact (lowest first)
        remaining_trades.sort_by(|a, b| {
            let a_impact = a.1.tax_impact.unwrap_or(0.0);
            let b_impact = b.1.tax_impact.unwrap_or(0.0);
            a_impact.partial_cmp(&b_impact).unwrap()
        });
        
        // Add trades until we hit the budget
        for (account_id, trade) in remaining_trades {
            let trade_impact = trade.tax_impact.unwrap_or(0.0);
            if current_tax_impact + trade_impact <= remaining_budget {
                // Add this trade
                current_tax_impact += trade_impact;
                
                // Find the account in prioritized_trades
                if let Some(pos) = prioritized_trades.iter().position(|(id, _)| id == &account_id) {
                    prioritized_trades[pos].1.push(trade.clone());
                } else {
                    prioritized_trades.push((account_id.clone(), vec![trade.clone()]));
                }
                
                // Also add the corresponding buy trade if it exists
                for (id, trades) in &account_trades {
                    if id == &account_id {
                        for buy_trade in trades {
                            if buy_trade.is_buy && 
                               buy_trade.security_id == trade.security_id && 
                               buy_trade.amount == trade.amount {
                                // Find the account in prioritized_trades
                                if let Some(pos) = prioritized_trades.iter().position(|(pid, _)| pid == id) {
                                    prioritized_trades[pos].1.push(buy_trade.clone());
                                } else {
                                    prioritized_trades.push((id.clone(), vec![buy_trade.clone()]));
                                }
                                break;
                            }
                        }
                        break;
                    }
                }
            }
        }
        
        prioritized_trades
    }
    
    /// Analyzes the asset allocation across the entire household.
    ///
    /// This method aggregates all holdings across all accounts in the household
    /// and calculates the allocation by asset class, sector, and individual security.
    /// It also evaluates the efficiency of asset location (whether tax-efficient assets
    /// are held in taxable accounts and tax-inefficient assets in tax-advantaged accounts).
    ///
    /// # Parameters
    ///
    /// * `household` - The household to analyze
    ///
    /// # Returns
    ///
    /// A `HouseholdAssetAllocation` struct containing:
    /// - Asset class allocation (e.g., equities, fixed income, alternatives)
    /// - Sector allocation (e.g., technology, healthcare, financials)
    /// - Security allocation (individual securities and their weights)
    /// - Asset location score (measure of tax efficiency of asset placement)
    /// - Recommendations for improving asset location
    ///
    /// # Examples
    ///
    /// ```
    /// use investment_management::portfolio::model::household::HouseholdService;
    ///
    /// let service = HouseholdService::new();
    /// let household = service.create_household("Smith Family", "John Smith");
    /// // Add accounts and assets to the household...
    ///
    /// let allocation = service.analyze_household_asset_allocation(&household);
    /// println!("Asset Location Efficiency Score: {:.2}", allocation.asset_location_score);
    /// for (asset_class, weight) in &allocation.asset_class_allocation {
    ///     println!("{}: {:.2}%", asset_class, weight * 100.0);
    /// }
    /// ```
    pub fn analyze_household_asset_allocation(
        &self,
        household: &UnifiedManagedHousehold,
    ) -> HouseholdAssetAllocation {
        let mut asset_classes = HashMap::new();
        let mut sectors = HashMap::new();
        let mut securities = HashMap::new();
        let total_value = household.total_market_value();
        
        // Aggregate holdings across all accounts
        for account in household.accounts.values() {
            for sleeve in &account.sleeves {
                for holding in &sleeve.holdings {
                    // Aggregate by security
                    *securities.entry(holding.security_id.clone())
                        .or_insert(0.0) += holding.market_value;
                    
                    // Get security metadata (would come from a security master in real implementation)
                    let asset_class = self.get_security_asset_class(&holding.security_id);
                    let sector = self.get_security_sector(&holding.security_id);
                    
                    // Aggregate by asset class
                    *asset_classes.entry(asset_class)
                        .or_insert(0.0) += holding.market_value;
                    
                    // Aggregate by sector
                    *sectors.entry(sector)
                        .or_insert(0.0) += holding.market_value;
                }
            }
        }
        
        // Convert to percentages
        let asset_class_allocation = asset_classes.iter()
            .map(|(class, value)| (class.clone(), value / total_value))
            .collect();
            
        let sector_allocation = sectors.iter()
            .map(|(sector, value)| (sector.clone(), value / total_value))
            .collect();
            
        let security_allocation = securities.iter()
            .map(|(security, value)| (security.clone(), value / total_value))
            .collect();
        
        // Analyze asset location efficiency
        let asset_location_score = self.calculate_asset_location_efficiency(household);
        
        // Analyze tax efficiency
        let tax_efficiency_score = self.calculate_tax_efficiency_score(household);
        
        HouseholdAssetAllocation {
            household_id: household.id.clone(),
            household_name: household.name.clone(),
            total_market_value: total_value,
            asset_class_allocation,
            sector_allocation,
            security_allocation,
            asset_location_score,
            tax_efficiency_score,
            asset_location_recommendations: self.generate_asset_location_recommendations(household),
        }
    }
    
    /// Get asset class for a security (mock implementation)
    fn get_security_asset_class(&self, security_id: &str) -> String {
        // In a real implementation, this would look up the asset class from a security master
        // For now, use a simple heuristic based on the security ID
        if security_id.starts_with("B") {
            "Fixed Income".to_string()
        } else if security_id.starts_with("R") {
            "Real Estate".to_string()
        } else {
            "Equity".to_string()
        }
    }
    
    /// Get sector for a security (mock implementation)
    fn get_security_sector(&self, security_id: &str) -> String {
        // In a real implementation, this would look up the sector from a security master
        // For now, use a simple mapping based on common stock tickers
        match security_id {
            "AAPL" | "MSFT" | "GOOGL" => "Technology".to_string(),
            "JPM" | "BAC" | "WFC" => "Financials".to_string(),
            "JNJ" | "PFE" | "MRK" => "Healthcare".to_string(),
            "XOM" | "CVX" | "COP" => "Energy".to_string(),
            "PG" | "KO" | "PEP" => "Consumer Staples".to_string(),
            "AMZN" | "TSLA" | "HD" => "Consumer Discretionary".to_string(),
            _ => "Other".to_string(),
        }
    }
    
    /// Calculate asset location efficiency score
    fn calculate_asset_location_efficiency(&self, household: &UnifiedManagedHousehold) -> f64 {
        // This would implement a sophisticated algorithm to determine how efficiently
        // assets are located across taxable and tax-advantaged accounts
        
        // Get taxable and tax-advantaged accounts
        let taxable_accounts = household.get_accounts_by_tax_type(&AccountTaxType::Taxable);
        let tax_deferred_accounts = household.get_accounts_by_tax_type(&AccountTaxType::TaxDeferred);
        let tax_exempt_accounts = household.get_accounts_by_tax_type(&AccountTaxType::TaxExempt);
        
        // If there's only one type of account, asset location is not applicable
        if taxable_accounts.is_empty() || (tax_deferred_accounts.is_empty() && tax_exempt_accounts.is_empty()) {
            return 1.0; // Not applicable
        }
        
        // Calculate the percentage of tax-inefficient assets in tax-advantaged accounts
        // and tax-efficient assets in taxable accounts
        let mut tax_inefficient_assets_in_tax_advantaged = 0.0;
        let mut total_tax_inefficient_assets = 0.0;
        let mut tax_efficient_assets_in_taxable = 0.0;
        let mut total_tax_efficient_assets = 0.0;
        
        // Analyze taxable accounts
        for account in &taxable_accounts {
            for sleeve in &account.sleeves {
                for holding in &sleeve.holdings {
                    let asset_class = self.get_security_asset_class(&holding.security_id);
                    let tax_efficiency = self.get_asset_tax_efficiency(&asset_class);
                    
                    if tax_efficiency > 0.7 { // Tax-efficient asset
                        tax_efficient_assets_in_taxable += holding.market_value;
                        total_tax_efficient_assets += holding.market_value;
                    } else { // Tax-inefficient asset
                        total_tax_inefficient_assets += holding.market_value;
                    }
                }
            }
        }
        
        // Analyze tax-advantaged accounts
        let tax_advantaged_accounts: Vec<&UnifiedManagedAccount> = [&tax_deferred_accounts[..], &tax_exempt_accounts[..]].concat();
        for account in tax_advantaged_accounts {
            for sleeve in &account.sleeves {
                for holding in &sleeve.holdings {
                    let asset_class = self.get_security_asset_class(&holding.security_id);
                    let tax_efficiency = self.get_asset_tax_efficiency(&asset_class);
                    
                    if tax_efficiency <= 0.7 { // Tax-inefficient asset
                        tax_inefficient_assets_in_tax_advantaged += holding.market_value;
                        total_tax_inefficient_assets += holding.market_value;
                    } else { // Tax-efficient asset
                        total_tax_efficient_assets += holding.market_value;
                    }
                }
            }
        }
        
        // Calculate efficiency ratios
        let tax_inefficient_ratio = if total_tax_inefficient_assets > 0.0 {
            tax_inefficient_assets_in_tax_advantaged / total_tax_inefficient_assets
        } else {
            1.0
        };
        
        let tax_efficient_ratio = if total_tax_efficient_assets > 0.0 {
            tax_efficient_assets_in_taxable / total_tax_efficient_assets
        } else {
            1.0
        };
        
        // Combine the ratios (weighted average)
        0.6 * tax_inefficient_ratio + 0.4 * tax_efficient_ratio
    }
    
    /// Get tax efficiency score for an asset class (mock implementation)
    fn get_asset_tax_efficiency(&self, asset_class: &str) -> f64 {
        // In a real implementation, this would be based on more sophisticated analysis
        // For now, use a simple mapping
        match asset_class {
            "Fixed Income" => 0.3, // Tax-inefficient
            "Real Estate" => 0.4, // Tax-inefficient
            "Equity" => 0.8, // Tax-efficient
            _ => 0.5, // Neutral
        }
    }
    
    /// Generate asset location recommendations
    fn generate_asset_location_recommendations(&self, household: &UnifiedManagedHousehold) -> Vec<AssetLocationRecommendation> {
        let mut recommendations = Vec::new();
        
        // Get taxable and tax-advantaged accounts
        let taxable_accounts = household.get_accounts_by_tax_type(&AccountTaxType::Taxable);
        let tax_deferred_accounts = household.get_accounts_by_tax_type(&AccountTaxType::TaxDeferred);
        let tax_exempt_accounts = household.get_accounts_by_tax_type(&AccountTaxType::TaxExempt);
        
        // If there's only one type of account, asset location is not applicable
        if taxable_accounts.is_empty() || (tax_deferred_accounts.is_empty() && tax_exempt_accounts.is_empty()) {
            return recommendations;
        }
        
        // Find tax-inefficient assets in taxable accounts
        for account in &taxable_accounts {
            for sleeve in &account.sleeves {
                for holding in &sleeve.holdings {
                    let asset_class = self.get_security_asset_class(&holding.security_id);
                    let tax_efficiency = self.get_asset_tax_efficiency(&asset_class);
                    
                    if tax_efficiency <= 0.5 { // Very tax-inefficient
                        // Find a suitable tax-advantaged account
                        if let Some(target_account) = tax_deferred_accounts.first() {
                            recommendations.push(AssetLocationRecommendation {
                                security_id: holding.security_id.clone(),
                                source_account_id: account.id.clone(),
                                target_account_id: target_account.id.clone(),
                                market_value: holding.market_value,
                                reason: format!("{} is tax-inefficient and should be held in a tax-advantaged account", asset_class),
                                priority: 1, // Low priority
                                amount: holding.market_value,
                                tax_efficiency_score: tax_efficiency,
                                estimated_tax_savings: holding.market_value * 0.15, // Estimated tax savings
                            });
                        }
                    }
                }
            }
        }
        
        // Find tax-efficient assets in tax-advantaged accounts
        let tax_advantaged_accounts: Vec<&UnifiedManagedAccount> = [&tax_deferred_accounts[..], &tax_exempt_accounts[..]].concat();
        for account in tax_advantaged_accounts {
            for sleeve in &account.sleeves {
                for holding in &sleeve.holdings {
                    let asset_class = self.get_security_asset_class(&holding.security_id);
                    let tax_efficiency = self.get_asset_tax_efficiency(&asset_class);
                    
                    if tax_efficiency >= 0.8 { // Very tax-efficient
                        // Find a suitable taxable account
                        if let Some(target_account) = taxable_accounts.first() {
                            recommendations.push(AssetLocationRecommendation {
                                security_id: holding.security_id.clone(),
                                source_account_id: account.id.clone(),
                                target_account_id: target_account.id.clone(),
                                market_value: holding.market_value,
                                reason: format!("{} is tax-efficient and could be held in a taxable account", asset_class),
                                priority: 2, // Medium priority
                                amount: holding.market_value,
                                tax_efficiency_score: tax_efficiency,
                                estimated_tax_savings: holding.market_value * 0.05, // Estimated tax savings
                            });
                        }
                    }
                }
            }
        }
        
        // Sort recommendations by priority
        recommendations.sort_by_key(|r| r.priority);
        
        recommendations
    }
    
    /// Analyze risk across the household
    pub fn analyze_household_risk(
        &self,
        household: &UnifiedManagedHousehold,
    ) -> HouseholdRiskAnalysis {
        let mut security_weights = HashMap::new();
        let mut asset_class_weights = HashMap::new();
        let mut sector_weights = HashMap::new();
        let total_value = household.total_market_value();
        
        // Aggregate holdings across all accounts
        for account in household.accounts.values() {
            for sleeve in &account.sleeves {
                for holding in &sleeve.holdings {
                    // Aggregate by security
                    *security_weights.entry(holding.security_id.clone())
                        .or_insert(0.0) += holding.market_value / total_value;
                    
                    // Get security metadata
                    let asset_class = self.get_security_asset_class(&holding.security_id);
                    let sector = self.get_security_sector(&holding.security_id);
                    
                    // Aggregate by asset class
                    *asset_class_weights.entry(asset_class)
                        .or_insert(0.0) += holding.market_value / total_value;
                    
                    // Aggregate by sector
                    *sector_weights.entry(sector)
                        .or_insert(0.0) += holding.market_value / total_value;
                }
            }
        }
        
        // Calculate concentration metrics
        let _security_concentration = self.calculate_concentration_score(&security_weights);
        let _asset_class_concentration = self.calculate_concentration_score(&asset_class_weights);
        let _sector_concentration = self.calculate_concentration_score(&sector_weights);
        
        // Calculate overall risk metrics
        let volatility = self.calculate_portfolio_volatility(&security_weights);
        let var_95 = self.calculate_value_at_risk(&security_weights, 0.95);
        let cvar_95 = self.calculate_conditional_var(&security_weights, 0.95);
        
        // Identify concentration risks
        let concentration_risks = self.identify_concentration_risks(
            &security_weights,
            &asset_class_weights,
            &sector_weights
        );
        
        // Generate risk reduction recommendations
        let _risk_reduction_recommendations = self.generate_risk_reduction_recommendations(
            household,
            &security_weights,
            &asset_class_weights,
            &sector_weights,
            volatility
        );
        
        HouseholdRiskAnalysis {
            portfolio_volatility: volatility,
            value_at_risk: var_95,
            conditional_var: cvar_95,
            volatility,
            value_at_risk_95: var_95,
            conditional_var_95: cvar_95,
            security_concentration: concentration_risks.iter()
                .filter(|r| matches!(r.risk_type, ConcentrationRiskType::SingleSecurity))
                .cloned()
                .collect(),
            asset_class_concentration: concentration_risks.iter()
                .filter(|r| matches!(r.risk_type, ConcentrationRiskType::AssetClass))
                .cloned()
                .collect(),
            sector_concentration: concentration_risks.iter()
                .filter(|r| matches!(r.risk_type, ConcentrationRiskType::Sector))
                .cloned()
                .collect(),
        }
    }
    
    /// Calculate concentration score (Herfindahl-Hirschman Index)
    fn calculate_concentration_score(&self, weights: &HashMap<String, f64>) -> f64 {
        // Herfindahl-Hirschman Index (HHI) is the sum of squared weights
        // Higher values indicate more concentration
        weights.values().map(|w| w * w).sum()
    }
    
    /// Calculate portfolio volatility (mock implementation)
    fn calculate_portfolio_volatility(&self, security_weights: &HashMap<String, f64>) -> f64 {
        // In a real implementation, this would use a factor model or historical data
        // For now, use a simple heuristic based on the securities
        let mut volatility = 0.0;
        
        for (security_id, weight) in security_weights {
            let security_vol = match security_id.as_str() {
                "AAPL" => 0.25, // 25% annualized volatility
                "MSFT" => 0.22,
                "AMZN" => 0.30,
                "GOOGL" => 0.24,
                "JPM" => 0.28,
                "BAC" => 0.32,
                "WFC" => 0.30,
                "JNJ" => 0.15,
                "PFE" => 0.18,
                "MRK" => 0.17,
                "XOM" => 0.22,
                "CVX" => 0.24,
                "COP" => 0.28,
                "PG" => 0.14,
                "KO" => 0.15,
                "PEP" => 0.16,
                "TSLA" => 0.45,
                "HD" => 0.20,
                _ => 0.25, // Default
            };
            
            // Add weighted contribution to portfolio volatility
            // This is a simplification; real portfolio volatility would account for correlations
            volatility += weight * security_vol;
        }
        
        volatility
    }
    
    /// Calculate Value at Risk (VaR) at a given confidence level (mock implementation)
    fn calculate_value_at_risk(&self, security_weights: &HashMap<String, f64>, confidence_level: f64) -> f64 {
        // In a real implementation, this would use a more sophisticated approach
        // For now, use a simple parametric VaR calculation
        let volatility = self.calculate_portfolio_volatility(security_weights);
        
        // For a normal distribution, 95% confidence level corresponds to 1.645 standard deviations
        let z_score = match confidence_level {
            0.90 => 1.282,
            0.95 => 1.645,
            0.99 => 2.326,
            _ => 1.645, // Default to 95%
        };
        
        // VaR as a percentage of portfolio value
        z_score * volatility / (252.0_f64).sqrt() // Assuming daily VaR, annualized volatility
    }
    
    /// Calculate Conditional VaR (Expected Shortfall) at a given confidence level (mock implementation)
    fn calculate_conditional_var(&self, security_weights: &HashMap<String, f64>, confidence_level: f64) -> f64 {
        // In a real implementation, this would use a more sophisticated approach
        // For now, use a simple approximation based on VaR
        let var = self.calculate_value_at_risk(security_weights, confidence_level);
        
        // For a normal distribution, CVaR is approximately 1.25 times VaR at 95% confidence
        let cvar_multiplier = match confidence_level {
            0.90 => 1.20,
            0.95 => 1.25,
            0.99 => 1.33,
            _ => 1.25, // Default to 95%
        };
        
        var * cvar_multiplier
    }
    
    /// Identify concentration risks in the household portfolio
    fn identify_concentration_risks(
        &self,
        security_weights: &HashMap<String, f64>,
        asset_class_weights: &HashMap<String, f64>,
        sector_weights: &HashMap<String, f64>,
    ) -> Vec<ConcentrationRisk> {
        let mut risks = Vec::new();
        
        // Check for concentrated positions in individual securities
        for (security_id, weight) in security_weights {
            if *weight > 0.05 { // More than 5% in a single security
                risks.push(ConcentrationRisk {
                    risk_type: ConcentrationRiskType::SingleSecurity,
                    severity: self.calculate_risk_severity(*weight, 0.05, 0.10),
                    percentage: *weight,
                    description: format!("Concentrated position in {} ({:.1}% of portfolio)", security_id, weight * 100.0),
                });
            }
        }
        
        // Check for concentrated positions in asset classes
        for (asset_class, weight) in asset_class_weights {
            let threshold = match asset_class.as_str() {
                "Equity" => 0.70, // 70% threshold for equities
                "Fixed Income" => 0.80, // 80% threshold for fixed income
                "Real Estate" => 0.30, // 30% threshold for real estate
                _ => 0.50, // Default threshold
            };
            
            if *weight > threshold {
                risks.push(ConcentrationRisk {
                    risk_type: ConcentrationRiskType::AssetClass,
                    severity: self.calculate_risk_severity(*weight, threshold, threshold * 1.2),
                    percentage: *weight,
                    description: format!("High concentration in {} asset class ({:.1}% of portfolio)", asset_class, weight * 100.0),
                });
            }
        }
        
        // Check for concentrated positions in sectors
        for (sector, weight) in sector_weights {
            if *weight > 0.25 { // More than 25% in a single sector
                risks.push(ConcentrationRisk {
                    risk_type: ConcentrationRiskType::Sector,
                    severity: self.calculate_risk_severity(*weight, 0.25, 0.40),
                    percentage: *weight,
                    description: format!("Concentrated exposure to {} sector ({:.1}% of portfolio)", sector, weight * 100.0),
                });
            }
        }
        
        // Sort risks by severity (highest first)
        risks.sort_by(|a, b| b.severity.cmp(&a.severity));
        
        risks
    }
    
    /// Calculate risk severity based on weight and thresholds
    fn calculate_risk_severity(&self, weight: f64, warning_threshold: f64, critical_threshold: f64) -> RiskSeverity {
        if weight >= critical_threshold {
            RiskSeverity::High
        } else if weight >= warning_threshold {
            RiskSeverity::Medium
        } else {
            RiskSeverity::Low
        }
    }
    
    /// Generate risk reduction recommendations
    fn generate_risk_reduction_recommendations(
        &self,
        household: &UnifiedManagedHousehold,
        security_weights: &HashMap<String, f64>,
        asset_class_weights: &HashMap<String, f64>,
        sector_weights: &HashMap<String, f64>,
        portfolio_volatility: f64,
    ) -> Vec<RiskReductionRecommendation> {
        let mut recommendations = Vec::new();
        let total_value = household.total_market_value();
        
        // Recommend diversification for concentrated securities
        for (security_id, weight) in security_weights {
            if *weight > 0.05 { // More than 5% in a single security
                let excess_weight = weight - 0.05;
                let excess_value = excess_weight * total_value;
                
                recommendations.push(RiskReductionRecommendation {
                    description: format!(
                        "Reduce position in {} by ${:.2} to limit single-security risk",
                        security_id, excess_value
                    ),
                    estimated_return_impact: -0.002,
                    priority: RecommendationPriority::High, // High priority
                });
            }
        }
        
        // Recommend sector diversification
        for (sector, weight) in sector_weights {
            if *weight > 0.25 { // More than 25% in a single sector
                let excess_weight = weight - 0.25;
                
                recommendations.push(RiskReductionRecommendation {
                    description: format!(
                        "Reduce exposure to {} sector from {:.1}% to 25% of portfolio",
                        sector, weight * 100.0
                    ),
                    estimated_return_impact: -0.005 * excess_weight, // Estimated impact on returns
                    priority: RecommendationPriority::Medium, // Medium priority
                });
            }
        }
        
        // Recommend asset class diversification if needed
        if let Some((asset_class, weight)) = asset_class_weights.iter()
            .find(|(class, weight)| 
                (*class == "Equity" && **weight > 0.70) || 
                (*class == "Fixed Income" && **weight > 0.80) ||
                (*class == "Real Estate" && **weight > 0.30)
            ) 
        {
            let threshold = match asset_class.as_str() {
                "Equity" => 0.70,
                "Fixed Income" => 0.80,
                "Real Estate" => 0.30,
                _ => 0.50,
            };
            
            let excess_weight = weight - threshold;
            
            recommendations.push(RiskReductionRecommendation {
                description: format!(
                    "Reduce allocation to {} asset class from {:.1}% to {:.1}% of portfolio",
                    asset_class, weight * 100.0, threshold * 100.0
                ),
                estimated_return_impact: -0.008 * excess_weight, // Estimated impact on returns
                priority: RecommendationPriority::Medium, // Medium priority
            });
        }
        
        // Recommend overall volatility reduction if portfolio is too volatile
        if portfolio_volatility > 0.20 { // If annualized volatility is above 20%
            recommendations.push(RiskReductionRecommendation {
                description: format!(
                    "Reduce overall portfolio volatility from {:.1}% to 20% by adding more defensive assets",
                    portfolio_volatility * 100.0
                ),
                estimated_return_impact: -0.02 * (portfolio_volatility - 0.20), // Estimated impact on returns
                priority: RecommendationPriority::Low, // Low priority
            });
        }
        
        // Sort recommendations by priority
        recommendations.sort_by(|a, b| a.priority.cmp(&b.priority));
        
        recommendations
    }
    
    /// Get volatility for a specific security (mock implementation)
    /// 
    /// Note: This method is currently not used in the codebase but is kept for future implementation
    /// of more sophisticated risk analysis features. It provides a foundation for calculating
    /// portfolio volatility based on individual security volatilities.
    #[allow(dead_code)]
    fn get_security_volatility(&self, security_id: &str) -> f64 {
        match security_id {
            "AAPL" => 0.25, // 25% annualized volatility
            "MSFT" => 0.22,
            "AMZN" => 0.30,
            "GOOGL" => 0.24,
            "JPM" => 0.28,
            _ => 0.20, // Default volatility for unknown securities
        }
    }
    
    /// Calculate recommendation priority based on weight and thresholds
    /// 
    /// Note: This method is currently not used in the codebase but is kept for future implementation
    /// of the risk recommendation system. It will be used to prioritize recommendations based on
    /// the severity of the risk.
    #[allow(dead_code)]
    fn calculate_recommendation_priority(
        &self, 
        weight: f64, 
        warning_threshold: f64, 
        critical_threshold: f64
    ) -> RecommendationPriority {
        if weight >= critical_threshold {
            RecommendationPriority::High
        } else if weight >= warning_threshold {
            RecommendationPriority::Medium
        } else {
            RecommendationPriority::Low
        }
    }
    
    /// Calculate ESG score for the household
    /// 
    /// Note: This method is currently not used in the codebase but is kept for future implementation
    /// of ESG analysis features. It will be used to calculate an overall ESG score for the household
    /// based on the ESG scores of individual securities and accounts.
    #[allow(dead_code)]
    fn calculate_household_esg_score(&self, _household: &UnifiedManagedHousehold) -> Option<f64> {
        // This would calculate an overall ESG score for the household
        // For now, return a placeholder value
        Some(65.0)
    }

    /// Generate household financial report
    pub fn generate_household_report(
        &self,
        household: &UnifiedManagedHousehold,
    ) -> HouseholdReport {
        // This would generate a comprehensive household financial report
        HouseholdReport {
            cash_balance: household.total_cash_balance(),
            total_cash_balance: household.total_cash_balance(),
            account_summary: household.accounts.iter()
                .map(|(id, _)| id.clone())
                .collect(),
            tlh_efficiency_score: self.calculate_tlh_efficiency(household),
            risk_analysis: self.analyze_household_risk(household),
        }
    }
    
    /// Calculate tax efficiency score for the household
    fn calculate_tax_efficiency_score(&self, household: &UnifiedManagedHousehold) -> f64 {
        // This would calculate a tax efficiency score based on asset location
        // and tax-loss harvesting opportunities
        
        // Get asset location efficiency
        let asset_location_efficiency = self.calculate_asset_location_efficiency(household);
        
        // Get tax-loss harvesting efficiency
        let tlh_efficiency = self.calculate_tlh_efficiency(household);
        
        // Combine the scores (weighted average)
        0.7 * asset_location_efficiency + 0.3 * tlh_efficiency
    }
    
    /// Calculate tax-loss harvesting efficiency
    fn calculate_tlh_efficiency(&self, _household: &UnifiedManagedHousehold) -> f64 {
        // This would calculate how efficiently the household is utilizing
        // tax-loss harvesting opportunities
        
        // For now, return a placeholder value
        0.8
    }
    
    /// Generate tax-efficient withdrawal plan for the household
    pub fn generate_tax_efficient_withdrawal_plan(
        &self,
        household: &UnifiedManagedHousehold,
        withdrawal_amount: f64,
        withdrawal_timeframe: WithdrawalTimeframe,
    ) -> WithdrawalPlan {
        // Validate input
        if withdrawal_amount <= 0.0 {
            return WithdrawalPlan {
                total_amount: 0.0,
                withdrawals: Vec::new(),
                total_tax_impact: 0.0,
                after_tax_amount: 0.0,
                estimated_tax_impact: 0.0,
                tax_efficiency_score: 1.0,
            };
        }
        
        // Get tax rates from household settings or use defaults
        let (short_term_rate, long_term_rate) = self.get_household_tax_rates(household);
        
        // Categorize accounts by tax type
        let taxable_accounts = household.get_accounts_by_tax_type(&AccountTaxType::Taxable);
        let tax_deferred_accounts = household.get_accounts_by_tax_type(&AccountTaxType::TaxDeferred);
        let tax_exempt_accounts = household.get_accounts_by_tax_type(&AccountTaxType::TaxExempt);
        
        // Calculate required minimum distributions (RMDs) if applicable
        let rmd_withdrawals = self.calculate_required_minimum_distributions(
            household, 
            withdrawal_timeframe.clone()
        );
        let mut remaining_withdrawal = withdrawal_amount;
        let mut account_withdrawals = Vec::new();
        
        // First, satisfy RMDs (these are mandatory)
        for rmd in &rmd_withdrawals {
            account_withdrawals.push(rmd.clone());
            remaining_withdrawal -= rmd.amount;
        }
        
        // If RMDs exceed the requested withdrawal amount, we're done
        if remaining_withdrawal <= 0.0 {
            return self.finalize_withdrawal_plan(
                household,
                withdrawal_amount,
                withdrawal_timeframe,
                account_withdrawals,
                short_term_rate,
                long_term_rate,
            );
        }
        
        // Next, withdraw from tax-exempt accounts (Roth IRAs)
        // This is generally the most tax-efficient for withdrawals
        if !tax_exempt_accounts.is_empty() && remaining_withdrawal > 0.0 {
            let tax_exempt_withdrawals = self.generate_account_type_withdrawals(
                &tax_exempt_accounts,
                remaining_withdrawal,
                WithdrawalReason::TaxEfficient,
                0.0, // No tax impact
            );
            
            for withdrawal in tax_exempt_withdrawals {
                account_withdrawals.push(withdrawal.clone());
                remaining_withdrawal -= withdrawal.amount;
            }
        }
        
        // Next, withdraw from taxable accounts with losses or minimal gains
        if !taxable_accounts.is_empty() && remaining_withdrawal > 0.0 {
            let taxable_withdrawals = self.generate_taxable_account_withdrawals(
                &taxable_accounts,
                remaining_withdrawal,
                short_term_rate,
                long_term_rate,
            );
            
            for withdrawal in taxable_withdrawals {
                account_withdrawals.push(withdrawal.clone());
                remaining_withdrawal -= withdrawal.amount;
            }
        }
        
        // Finally, withdraw from tax-deferred accounts (Traditional IRAs, 401(k)s)
        if !tax_deferred_accounts.is_empty() && remaining_withdrawal > 0.0 {
            let tax_deferred_withdrawals = self.generate_account_type_withdrawals(
                &tax_deferred_accounts,
                remaining_withdrawal,
                WithdrawalReason::LastResort,
                short_term_rate, // Taxed as ordinary income
            );
            
            for withdrawal in tax_deferred_withdrawals {
                account_withdrawals.push(withdrawal.clone());
                remaining_withdrawal -= withdrawal.amount;
            }
        }
        
        // Finalize the withdrawal plan
        self.finalize_withdrawal_plan(
            household,
            withdrawal_amount,
            withdrawal_timeframe,
            account_withdrawals,
            short_term_rate,
            long_term_rate,
        )
    }
    
    /// Get household tax rates from settings or use defaults
    fn get_household_tax_rates(&self, household: &UnifiedManagedHousehold) -> (f64, f64) {
        if let Some(tax_settings) = &household.household_tax_settings {
            (tax_settings.short_term_tax_rate, tax_settings.long_term_tax_rate)
        } else {
            // Default tax rates
            (0.35, 0.15)
        }
    }
    
    /// Calculate required minimum distributions for applicable accounts
    fn calculate_required_minimum_distributions(
        &self,
        household: &UnifiedManagedHousehold,
        timeframe: WithdrawalTimeframe,
    ) -> Vec<AccountWithdrawal> {
        let mut rmd_withdrawals = Vec::new();
        
        // Only calculate RMDs for annual withdrawals
        if timeframe != WithdrawalTimeframe::Annual {
            return rmd_withdrawals;
        }
        
        // Get tax-deferred accounts
        let tax_deferred_accounts = household.get_accounts_by_tax_type(&AccountTaxType::TaxDeferred);
        
        // Check each account for RMD requirements
        for account in tax_deferred_accounts {
            // Get account owner's age
            let owner_id = if let Some(owners) = household.account_ownership.get(&account.id) {
                if owners.is_empty() {
                    continue;
                }
                owners[0].clone()
            } else {
                continue;
            };
            
            let owner_age = self.get_member_age(household, &owner_id);
            
            // RMDs start at age 72 (or 73 for those born after 1950)
            if let Some(age) = owner_age {
                if age >= 72 {
                    // Calculate RMD based on account value and age
                    let rmd_amount = self.calculate_rmd(account, age);
                    
                    if rmd_amount > 0.0 {
                        rmd_withdrawals.push(AccountWithdrawal {
                            account_id: account.id.clone(),
                            amount: rmd_amount,
                            tax_impact: rmd_amount * 0.24, // Estimated tax impact
                            holdings_to_sell: Vec::new(), // Would be populated in a real implementation
                        });
                    }
                }
            }
        }
        
        rmd_withdrawals
    }
    
    /// Calculate RMD for an account based on age (mock implementation)
    fn calculate_rmd(&self, account: &UnifiedManagedAccount, age: u32) -> f64 {
        // In a real implementation, this would use the IRS Uniform Lifetime Table
        // For now, use a simplified calculation
        let divisor = match age {
            72..=74 => 25.6,
            75..=79 => 22.9,
            80..=84 => 18.7,
            85..=89 => 14.8,
            90..=94 => 11.4,
            _ => 8.6,
        };
        
        account.total_market_value / divisor
    }
    
    /// Get member age from birth date (mock implementation)
    fn get_member_age(&self, _household: &UnifiedManagedHousehold, _member_id: &str) -> Option<u32> {
        // In a real implementation, this would calculate age from birth date
        // For now, return None (unknown age)
        None
    }
    
    /// Generate withdrawals from a specific account type
    fn generate_account_type_withdrawals(
        &self,
        accounts: &[&UnifiedManagedAccount],
        amount_needed: f64,
        _reason: WithdrawalReason,
        tax_rate: f64,
    ) -> Vec<AccountWithdrawal> {
        let mut withdrawals = Vec::new();
        let mut remaining = amount_needed;
        
        // Sort accounts by cash balance (highest first) to minimize selling securities
        let mut sorted_accounts = accounts.to_vec();
        sorted_accounts.sort_by(|a, b| b.cash_balance.partial_cmp(&a.cash_balance).unwrap());
        
        for account in sorted_accounts {
            if remaining <= 0.0 {
                break;
            }
            
            let _account_type = if account.name.contains("Roth") {
                AccountTaxType::TaxExempt
            } else if account.name.contains("IRA") || account.name.contains("401") {
                AccountTaxType::TaxDeferred
            } else {
                AccountTaxType::Taxable
            };
            
            // Determine how much to withdraw from this account
            let available = account.total_market_value;
            let withdrawal_amount = remaining.min(available);
            
            if withdrawal_amount > 0.0 {
                // Calculate tax impact
                let tax_impact = withdrawal_amount * tax_rate;
                
                // Create withdrawal
                withdrawals.push(AccountWithdrawal {
                    account_id: account.id.clone(),
                    amount: withdrawal_amount,
                    tax_impact,
                    holdings_to_sell: Vec::new(), // Would be populated in a real implementation
                });
                
                remaining -= withdrawal_amount;
            }
        }
        
        withdrawals
    }
    
    /// Generate withdrawals from taxable accounts, prioritizing tax efficiency
    fn generate_taxable_account_withdrawals(
        &self,
        accounts: &[&UnifiedManagedAccount],
        amount_needed: f64,
        short_term_rate: f64,
        long_term_rate: f64,
    ) -> Vec<AccountWithdrawal> {
        let mut withdrawals = Vec::new();
        let mut remaining = amount_needed;
        
        // First, use available cash
        for account in accounts {
            if remaining <= 0.0 {
                break;
            }
            
            let cash_withdrawal = remaining.min(account.cash_balance);
            if cash_withdrawal > 0.0 {
                withdrawals.push(AccountWithdrawal {
                    account_id: account.id.clone(),
                    amount: cash_withdrawal,
                    tax_impact: 0.0, // No tax impact for withdrawing cash
                    holdings_to_sell: Vec::new(),
                });
                
                remaining -= cash_withdrawal;
            }
        }
        
        // If we still need more, sell securities with losses first
        if remaining > 0.0 {
            // Collect all holdings across accounts
            let mut all_holdings = Vec::new();
            for account in accounts {
                for sleeve in &account.sleeves {
                    for holding in &sleeve.holdings {
                        let unrealized_gain_loss = holding.market_value - holding.cost_basis;
                        let tax_impact = if unrealized_gain_loss > 0.0 {
                            // Determine if long-term or short-term gain
                            let is_long_term = self.is_long_term_holding(holding);
                            let tax_rate = if is_long_term { long_term_rate } else { short_term_rate };
                            unrealized_gain_loss * tax_rate
                        } else {
                            // Tax loss (benefit)
                            unrealized_gain_loss * short_term_rate
                        };
                        
                        all_holdings.push((account, holding, tax_impact));
                    }
                }
            }
            
            // Sort by tax impact (lowest/most negative first)
            all_holdings.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
            
            // Sell securities in order of tax efficiency
            for (account, holding, tax_impact) in all_holdings {
                if remaining <= 0.0 {
                    break;
                }
                
                let sell_amount = remaining.min(holding.market_value);
                if sell_amount > 0.0 {
                    // Calculate proportional tax impact
                    let proportional_tax_impact = tax_impact * (sell_amount / holding.market_value);
                    
                    // Check if we already have a withdrawal for this account
                    if let Some(pos) = withdrawals.iter().position(|w| w.account_id == account.id) {
                        // Update existing withdrawal
                        withdrawals[pos].amount += sell_amount;
                        withdrawals[pos].tax_impact += proportional_tax_impact;
                        withdrawals[pos].holdings_to_sell.push(holding.security_id.clone());
                    } else {
                        // Create new withdrawal
                        let mut holdings_to_sell = Vec::new();
                        holdings_to_sell.push(holding.security_id.clone());
                        
                        withdrawals.push(AccountWithdrawal {
                            account_id: account.id.clone(),
                            amount: sell_amount,
                            tax_impact: proportional_tax_impact,
                            holdings_to_sell,
                        });
                    }
                    
                    remaining -= sell_amount;
                }
            }
        }
        
        withdrawals
    }
    
    /// Determine if a holding is long-term (mock implementation)
    fn is_long_term_holding(&self, holding: &PortfolioHolding) -> bool {
        // In a real implementation, this would compare purchase date to current date
        // For now, use a simple heuristic based on the purchase date string
        holding.purchase_date.starts_with("2022") || holding.purchase_date.starts_with("2021")
    }
    
    /// Finalize the withdrawal plan with recommendations
    fn finalize_withdrawal_plan(
        &self,
        household: &UnifiedManagedHousehold,
        total_amount: f64,
        timeframe: WithdrawalTimeframe,
        account_withdrawals: Vec<AccountWithdrawal>,
        short_term_rate: f64,
        long_term_rate: f64,
    ) -> WithdrawalPlan {
        // Calculate total tax impact
        let total_tax_impact: f64 = account_withdrawals.iter()
            .map(|w| w.tax_impact)
            .sum();
        
        // Calculate tax efficiency score (1.0 is best, 0.0 is worst)
        let max_possible_tax = total_amount * short_term_rate;
        let tax_efficiency_score = if max_possible_tax > 0.0 {
            1.0 - (total_tax_impact / max_possible_tax)
        } else {
            1.0
        };
        
        // Generate recommendations for improving tax efficiency
        let _recommendations = self.generate_withdrawal_recommendations(
            household,
            &account_withdrawals,
            timeframe,
            short_term_rate,
            long_term_rate,
        );
        
        WithdrawalPlan {
            total_amount,
            withdrawals: account_withdrawals,
            total_tax_impact,
            after_tax_amount: total_amount - total_tax_impact,
            estimated_tax_impact: total_tax_impact,
            tax_efficiency_score,
        }
    }
    
    /// Generate recommendations for improving withdrawal tax efficiency
    fn generate_withdrawal_recommendations(
        &self,
        household: &UnifiedManagedHousehold,
        withdrawals: &[AccountWithdrawal],
        timeframe: WithdrawalTimeframe,
        short_term_rate: f64,
        long_term_rate: f64,
    ) -> Vec<WithdrawalRecommendation> {
        let mut recommendations = Vec::new();
        
        // Check if we're withdrawing from tax-deferred accounts before exhausting tax-exempt accounts
        let using_tax_deferred = withdrawals.iter().any(|w| w.account_id.contains("tax-deferred"));
        let tax_exempt_accounts = household.get_accounts_by_tax_type(&AccountTaxType::TaxExempt);
        let tax_exempt_capacity: f64 = tax_exempt_accounts.iter().map(|a| a.total_market_value).sum();
        
        if using_tax_deferred && tax_exempt_capacity > 0.0 {
            recommendations.push(WithdrawalRecommendation {
                id: Uuid::new_v4().to_string(),
                description: "Consider withdrawing more from tax-exempt accounts before using tax-deferred accounts".to_string(),
                estimated_tax_savings: 0.05 * withdrawals.iter()
                    .filter(|w| w.account_id.contains("tax-deferred"))
                    .map(|w| w.amount)
                    .sum::<f64>(),
                priority: 3, // High priority
            });
        }
        
        // Check if we're selling securities with short-term gains
        let selling_short_term_gains = withdrawals.iter()
            .flat_map(|w| &w.holdings_to_sell)
            .any(|s| s.contains("short-term"));
        
        if selling_short_term_gains {
            recommendations.push(WithdrawalRecommendation {
                id: Uuid::new_v4().to_string(),
                description: "Consider waiting until short-term gains become long-term to reduce tax impact".to_string(),
                estimated_tax_savings: withdrawals.iter()
                    .flat_map(|w| &w.holdings_to_sell)
                    .filter(|s| s.contains("short-term"))
                    .map(|_| 0.1 * (short_term_rate - long_term_rate))
                    .sum(),
                priority: 2, // Medium priority
            });
        }
        
        // Check if we're not fully utilizing tax-loss harvesting opportunities
        let has_taxable_withdrawals = withdrawals.iter().any(|w| w.account_id.contains("taxable"));
        
        if has_taxable_withdrawals {
            recommendations.push(WithdrawalRecommendation {
                id: Uuid::new_v4().to_string(),
                description: "Consider tax-loss harvesting in taxable accounts to offset capital gains".to_string(),
                estimated_tax_savings: 0.02 * withdrawals.iter()
                    .filter(|w| w.account_id.contains("taxable"))
                    .map(|w| w.amount)
                    .sum::<f64>(),
                priority: 2, // Medium priority
            });
        }
        
        // Check if Roth conversion might be beneficial
        if timeframe == WithdrawalTimeframe::Annual {
            recommendations.push(WithdrawalRecommendation {
                id: Uuid::new_v4().to_string(),
                description: "Consider Roth conversion for some tax-deferred assets in years with lower income".to_string(),
                estimated_tax_savings: 0.10 * withdrawals.iter()
                    .filter(|w| w.account_id.contains("tax-deferred"))
                    .map(|w| w.amount)
                    .sum::<f64>(),
                priority: 1, // Low priority
            });
        }
        
        // Sort recommendations by priority
        recommendations.sort_by(|a, b| a.priority.cmp(&b.priority));
        
        recommendations
    }

    // Financial Goals Management
    pub fn add_financial_goal(&self, household: &mut UnifiedManagedHousehold, goal: FinancialGoal) -> Result<()> {
        household.financial_goals.push(goal);
        Ok(())
    }

    pub fn update_financial_goal(&self, household: &mut UnifiedManagedHousehold, goal_id: &str, updated_goal: FinancialGoal) -> Result<()> {
        if let Some(index) = household.financial_goals.iter().position(|g| g.id == goal_id) {
            household.financial_goals[index] = updated_goal;
            Ok(())
        } else {
            Err("Goal not found".into())
        }
    }

    pub fn delete_financial_goal(&self, household: &mut UnifiedManagedHousehold, goal_id: &str) -> Result<()> {
        if let Some(index) = household.financial_goals.iter().position(|g| g.id == goal_id) {
            household.financial_goals.remove(index);
            Ok(())
        } else {
            Err("Goal not found".into())
        }
    }

    pub fn add_goal_contribution(&self, household: &mut UnifiedManagedHousehold, goal_id: &str, contribution: GoalContribution) -> Result<()> {
        if let Some(goal) = household.financial_goals.iter_mut().find(|g| g.id == goal_id) {
            goal.current_amount += contribution.amount;
            goal.updated_at = Utc::now().date_naive();
            
            // Store contribution in history
            if household.goal_contributions.get(goal_id).is_none() {
                household.goal_contributions.insert(goal_id.to_string(), Vec::new());
            }
            
            if let Some(contributions) = household.goal_contributions.get_mut(goal_id) {
                contributions.push(contribution);
            }
            
            Ok(())
        } else {
            Err("Goal not found".into())
        }
    }

    pub fn track_goal_progress(&self, household: &UnifiedManagedHousehold, goal_id: &str) -> Result<GoalProgress> {
        let goal = household.financial_goals.iter()
            .find(|g| g.id == goal_id)
            .ok_or("Goal not found")?
            .clone();
        
        let percent_complete = (goal.current_amount / goal.target_amount).min(1.0);
        
        // Calculate monthly contribution needed
        let monthly_contribution_needed = if true {
            let today = Utc::now().date_naive();
            if goal.target_date <= today {
                0.0 // Goal date has passed
            } else {
                let months_remaining = (goal.target_date.year() - today.year()) as f64 * 12.0 + 
                                      (goal.target_date.month() - today.month()) as f64;
                let amount_remaining = goal.target_amount - goal.current_amount;
                if months_remaining > 0.0 {
                    amount_remaining / months_remaining
                } else {
                    0.0
                }
            }
        } else {
            0.0 // No target date
        };
        
        // Determine goal status
        let status = if percent_complete >= 1.0 {
            GoalStatus::Achieved
        } else if true {
            let today = Utc::now().date_naive();
            if goal.target_date <= today {
                GoalStatus::OffTrack
            } else {
                let months_remaining = (goal.target_date.year() - today.year()) as f64 * 12.0 + 
                                      (goal.target_date.month() - today.month()) as f64;
                let expected_progress = 1.0 - (months_remaining / 
                    ((goal.target_date.year() - goal.created_at.year()) as f64 * 12.0 + 
                     (goal.target_date.month() - goal.created_at.month()) as f64));
                
                if percent_complete >= expected_progress * 0.9 {
                    GoalStatus::OnTrack
                } else if percent_complete >= expected_progress * 0.7 {
                    GoalStatus::AtRisk
                } else {
                    GoalStatus::OffTrack
                }
            }
        } else {
            // No target date, just look at percentage
            if percent_complete >= 0.5 {
                GoalStatus::OnTrack
            } else {
                GoalStatus::AtRisk
            }
        };
        
        // Calculate projected completion date
        let projected_completion_date = if percent_complete >= 1.0 {
            // Already complete
            Some(Utc::now().date_naive())
        } else {
            // Get average monthly contribution from history
            let contributions = household.goal_contributions.get(goal_id).cloned().unwrap_or_default();
            
            if contributions.is_empty() {
                None
            } else {
                let total_contribution = contributions.iter().map(|c| c.amount).sum::<f64>();
                let _oldest_contribution = contributions.iter()
                    .map(|c| c.contribution_date)
                    .min()
                    .unwrap_or_else(|| goal.created_at);
                
                let months_contributing = (Utc::now().timestamp() - Utc::now().timestamp()) as f64 / (30.0 * 24.0 * 60.0 * 60.0);
                
                if months_contributing <= 0.0 || total_contribution <= 0.0 {
                    None
                } else {
                    let avg_monthly_contribution = total_contribution / months_contributing;
                    let amount_remaining = goal.target_amount - goal.current_amount;
                    
                    if avg_monthly_contribution <= 0.0 {
                        None
                    } else {
                        let months_to_completion = amount_remaining / avg_monthly_contribution;
                        let today = Utc::now().date_naive();
                        Some(today + Duration::days((months_to_completion * 30.0) as i64))
                    }
                }
            }
        };
        
        // Get recent contributions
        let recent_contributions = household.goal_contributions.get(goal_id)
            .map(|contributions| {
                let mut recent = contributions.clone();
                recent.sort_by(|a, b| b.contribution_date.cmp(&a.contribution_date));
                recent.truncate(5); // Last 5 contributions
                recent
            })
            .unwrap_or_default();
        
        // Generate recommendations
        let mut recommendations = Vec::new();
        
        match status {
            GoalStatus::OffTrack | GoalStatus::AtRisk => {
                let amount_remaining = goal.target_amount - goal.current_amount;
                
                if true {
                    let today = Utc::now().date_naive();
                    if goal.target_date > today {
                        let months_remaining = (goal.target_date.year() as f64 - today.year() as f64) * 12.0 + 
                                              (goal.target_date.month() - today.month()) as f64;
                        
                        if months_remaining > 0.0 {
                            let required_monthly = amount_remaining / months_remaining;
                            
                            recommendations.push(GoalRecommendation {
                                id: Uuid::new_v4().to_string(),
                                description: format!("Increase monthly contribution to ${:.2} to meet goal by target date", required_monthly),
                                estimated_impact: 1.0,
                                priority: RecommendationPriority::High, // High priority
                            });
                        }
                    } else {
                        recommendations.push(GoalRecommendation {
                            id: Uuid::new_v4().to_string(),
                            description: format!("Revise target date as the original date has passed"),
                            estimated_impact: 1.0,
                            priority: RecommendationPriority::High, // High priority
                        });
                    }
                }
                
                // Recommend reallocation from lower priority goals if applicable
                if goal.priority == 3 {
                    recommendations.push(GoalRecommendation {
                        id: Uuid::new_v4().to_string(),
                        description: "Consider reallocating funds from lower priority goals to this high priority goal".to_string(),
                        estimated_impact: 0.8,
                        priority: RecommendationPriority::Medium, // Medium priority
                    });
                }
            },
            GoalStatus::OnTrack => {
                if percent_complete < 0.5 {
                    recommendations.push(GoalRecommendation {
                        id: Uuid::new_v4().to_string(),
                        description: "Continue current contribution rate to stay on track".to_string(),
                        estimated_impact: 0.5,
                        priority: RecommendationPriority::Low, // Low priority
                    });
                }
            },
            GoalStatus::Achieved => {
                recommendations.push(GoalRecommendation {
                    id: Uuid::new_v4().to_string(),
                    description: "Goal achieved! Consider setting a new goal or increasing the target amount".to_string(),
                    estimated_impact: 0.3,
                    priority: RecommendationPriority::Low, // Low priority
                });
            },
            GoalStatus::Active | GoalStatus::Abandoned => {
                // No recommendations for these statuses
            }
        }
        
        Ok(GoalProgress {
            goal,
            percent_complete,
            monthly_contribution_needed: Some(monthly_contribution_needed),
            on_track: matches!(status, GoalStatus::OnTrack | GoalStatus::Achieved),
            months_remaining: 0, // This should be calculated properly
            status,
            projected_completion_date,
            recent_contributions,
            recommendations,
        })
    }

    pub fn generate_household_goals_report(&self, household: &UnifiedManagedHousehold) -> Result<HouseholdGoalsReport> {
        let mut goals_progress = Vec::new();
        let mut total_goal_amount = 0.0;
        let mut total_current_amount = 0.0;
        let mut priority_goals_at_risk = Vec::new();
        
        for goal in &household.financial_goals {
            let progress = self.track_goal_progress(household, &goal.id)?;
            
            total_goal_amount += goal.target_amount;
            total_current_amount += goal.current_amount;
            
            if matches!(progress.status, GoalStatus::AtRisk | GoalStatus::OffTrack) && 
               goal.priority == 3 {
                priority_goals_at_risk.push(progress.clone());
            }
            
            goals_progress.push(progress);
        }
        
        let overall_progress = if total_goal_amount > 0.0 {
            total_current_amount / total_goal_amount
        } else {
            0.0
        };
        
        Ok(HouseholdGoalsReport {
            goals: goals_progress,
            total_goal_amount,
            total_current_amount,
            overall_progress,
            priority_goals_at_risk,
            recommendations: Vec::new(), // Add empty recommendations
        })
    }
    
    // Estate Planning Methods
    
    pub fn create_estate_plan(
        &self,
        household: &mut UnifiedManagedHousehold,
        plan_type: EstatePlanType,
        name: String,
        assets: Vec<String>,
        beneficiaries: Vec<Beneficiary>,
        executor: Option<String>,
        trustee: Option<String>,
        notes: Option<String>,
    ) -> Result<EstatePlan> {
        let plan = EstatePlan {
            id: format!("estate-plan-{}", Uuid::new_v4()),
            plan_type,
            name,
            creation_date: Utc::now().date_naive(),
            last_updated: Utc::now().date_naive(),
            assets,
            beneficiaries,
            executor,
            trustee,
            notes,
            status: DocumentStatus::Draft,
            created_at: Utc::now().date_naive(),
            last_reviewed: Utc::now().date_naive(),
            attorney: None,
            location: "Home Office".to_string(),
        };
        
        household.add_estate_plan(plan.clone());
        Ok(plan)
    }
    
    pub fn create_beneficiary_designation(
        &self,
        household: &mut UnifiedManagedHousehold,
        account_id: String,
        beneficiaries: Vec<Beneficiary>,
    ) -> Result<BeneficiaryDesignation> {
        // Verify account exists
        if !household.accounts.contains_key(&account_id) {
            return Err("Account not found".into());
        }
        
        // Verify total allocation is 100%
        let total_allocation: f64 = beneficiaries.iter()
            .filter(|b| matches!(b.beneficiary_type, BeneficiaryType::Primary))
            .map(|b| b.allocation_percentage)
            .sum();
            
        if (total_allocation - 100.0).abs() > 0.01 {
            return Err(format!("Primary beneficiary allocation must total 100%, got {}", total_allocation).into());
        }
        
        let designation = BeneficiaryDesignation {
            account_id,
            beneficiaries,
            last_reviewed: Utc::now().date_naive(),
        };
        
        household.add_beneficiary_designation(designation.clone());
        Ok(designation)
    }
    
    pub fn analyze_estate_taxes(
        &self,
        household: &UnifiedManagedHousehold,
        state: Option<String>,
        lifetime_gifts: f64,
    ) -> EstateTaxAnalysis {
        // Calculate gross estate value
        let gross_estate_value = household.total_market_value();
        
        // Apply standard deductions
        let mut deductions = HashMap::new();
        deductions.insert("Marital Deduction".to_string(), 0.0);
        deductions.insert("Charitable Deduction".to_string(), 0.0);
        deductions.insert("Administrative Expenses".to_string(), gross_estate_value * 0.02);
        
        // Check for spouse
        if household.members.iter().any(|m| matches!(m.relationship, MemberRelationship::Spouse)) {
            // Unlimited marital deduction if assets pass to spouse
            deductions.insert("Marital Deduction".to_string(), gross_estate_value * 0.5);
        }
        
        // Calculate charitable giving from estate plans
        let charitable_giving = household.estate_plans.iter()
            .filter(|p| matches!(p.plan_type, EstatePlanType::CharitableRemainder))
            .flat_map(|p| &p.assets)
            .filter_map(|asset_id| {
                household.accounts.get(asset_id)
                    .map(|a| a.total_market_value)
            })
            .sum::<f64>();
            
        deductions.insert("Charitable Deduction".to_string(), charitable_giving);
        
        // Calculate total deductions
        let total_deductions: f64 = deductions.values().sum();
        
        // Calculate taxable estate
        let taxable_estate = (gross_estate_value - total_deductions).max(0.0);
        
        // Apply lifetime exemption (2023 values)
        let federal_exemption = 12_920_000.0;
        let exemption_used = lifetime_gifts;
        let exemption_remaining = (federal_exemption - exemption_used).max(0.0);
        let taxable_after_exemption = (taxable_estate - exemption_remaining).max(0.0);
        
        // Calculate federal estate tax (simplified 2023 rates)
        let mut estimated_taxes = HashMap::new();
        let federal_tax = if taxable_after_exemption > 0.0 {
            taxable_after_exemption * 0.40 // 40% federal estate tax rate
        } else {
            0.0
        };
        
        estimated_taxes.insert(TaxJurisdiction::Federal, federal_tax);
        
        // Add state estate tax if applicable
        if let Some(state_name) = state {
            let state_tax = match state_name.as_str() {
                "Washington" => taxable_estate * 0.20,
                "Oregon" => taxable_estate * 0.16,
                "Massachusetts" => taxable_estate * 0.16,
                "New York" => taxable_estate * 0.16,
                "Hawaii" => taxable_estate * 0.20,
                "Maine" => taxable_estate * 0.12,
                "Connecticut" => taxable_estate * 0.12,
                "Vermont" => taxable_estate * 0.16,
                "Maryland" => taxable_estate * 0.16,
                "Minnesota" => taxable_estate * 0.16,
                "Illinois" => taxable_estate * 0.16,
                "Rhode Island" => taxable_estate * 0.16,
                "District of Columbia" => taxable_estate * 0.16,
                _ => 0.0, // No state estate tax
            };
            
            if state_tax > 0.0 {
                estimated_taxes.insert(TaxJurisdiction::State(state_name), state_tax);
            }
        }
        
        // Calculate total tax and effective rate
        let total_tax: f64 = estimated_taxes.values().sum();
        let effective_tax_rate = if gross_estate_value > 0.0 {
            total_tax / gross_estate_value
        } else {
            0.0
        };
        
        // Generate tax reduction strategies
        let mut strategies = Vec::new();
        
        // Strategy 1: Annual gifting
        if taxable_after_exemption > 0.0 {
            strategies.push(EstateTaxStrategy {
                id: Uuid::new_v4().to_string(),
                description: "Utilize annual gift tax exclusion ($17,000 per recipient in 2023) to reduce taxable estate".to_string(),
                estimated_tax_savings: (taxable_after_exemption.min(100_000.0) * 0.40),
                complexity: 1,
                priority: 3, // High priority
            });
        }
        
        // Strategy 2: Irrevocable Life Insurance Trust
        if total_tax > 250_000.0 {
            strategies.push(EstateTaxStrategy {
                id: Uuid::new_v4().to_string(),
                description: "Create an ILIT to provide liquidity for estate taxes without increasing taxable estate".to_string(),
                estimated_tax_savings: total_tax * 0.40,
                complexity: 2,
                priority: 2, // Medium priority
            });
        }
        
        // Strategy 3: Charitable Remainder Trust
        if taxable_after_exemption > 1_000_000.0 {
            strategies.push(EstateTaxStrategy {
                id: Uuid::new_v4().to_string(),
                description: "Establish a charitable remainder trust to provide income during life and reduce taxable estate".to_string(),
                estimated_tax_savings: (taxable_after_exemption * 0.30 * 0.40),
                complexity: 3,
                priority: 2, // Medium priority
            });
        }
        
        // Strategy 4: Family Limited Partnership
        if taxable_after_exemption > 2_000_000.0 {
            strategies.push(EstateTaxStrategy {
                id: Uuid::new_v4().to_string(),
                description: "Create a family limited partnership to transfer assets with valuation discounts".to_string(),
                estimated_tax_savings: (taxable_after_exemption * 0.25 * 0.40),
                complexity: 3,
                priority: 2, // Medium priority
            });
        }
        
        // Strategy 5: Qualified Personal Residence Trust
        let has_valuable_residence = household.estate_plans.iter()
            .any(|p| matches!(p.plan_type, EstatePlanType::QualifiedPersonalResidence));
            
        if !has_valuable_residence && taxable_after_exemption > 1_000_000.0 {
            strategies.push(EstateTaxStrategy {
                id: Uuid::new_v4().to_string(),
                description: "Transfer primary or vacation home to a QPRT to remove future appreciation from taxable estate".to_string(),
                estimated_tax_savings: (taxable_after_exemption * 0.15 * 0.40),
                complexity: 2,
                priority: 2, // Medium priority
            });
        }
        
        EstateTaxAnalysis {
            gross_estate_value,
            deductions: deductions.values().sum(),
            taxable_estate,
            federal_exemption_used: exemption_used,
            taxable_after_exemption: taxable_after_exemption,
            estimated_taxes: HashMap::new(), // This should be properly initialized
            lifetime_exemption_used: exemption_used,
            lifetime_exemption_remaining: exemption_remaining,
            estimated_estate_tax: estimated_taxes.values().sum(),
            total_estimated_tax: total_tax,
            effective_tax_rate,
            tax_reduction_strategies: strategies,
        }
    }
    
    pub fn analyze_estate_distribution(
        &self,
        household: &UnifiedManagedHousehold,
    ) -> EstateDistributionAnalysis {
        let total_estate_value = household.total_market_value();
        
        // Get estate tax analysis
        let tax_analysis = self.analyze_estate_taxes(household, None, 0.0);
        let after_tax_estate_value = total_estate_value - tax_analysis.total_estimated_tax;
        
        // Calculate distributions to beneficiaries
        let mut beneficiary_distributions = HashMap::new();
        let mut trust_distributions = HashMap::new();
        let mut charitable_distributions = 0.0;
        
        // Process account-level beneficiary designations
        for designation in &household.beneficiary_designations {
            if let Some(account) = household.accounts.get(&designation.account_id) {
                let account_value = account.total_market_value;
                
                for beneficiary in &designation.beneficiaries {
                    if matches!(beneficiary.beneficiary_type, BeneficiaryType::Primary) {
                        let amount = account_value * (beneficiary.allocation_percentage / 100.0);
                        
                        if beneficiary.relationship.to_lowercase().contains("charity") {
                            charitable_distributions += amount;
                        } else {
                            *beneficiary_distributions.entry(beneficiary.name.clone()).or_insert(0.0) += amount;
                        }
                    }
                }
            }
        }
        
        // Process estate plans
        for plan in &household.estate_plans {
            let plan_assets_value: f64 = plan.assets.iter()
                .filter_map(|asset_id| {
                    household.accounts.get(asset_id)
                        .map(|a| a.total_market_value)
                })
                .sum();
                
            match plan.plan_type {
                EstatePlanType::CharitableRemainder => {
                    charitable_distributions += plan_assets_value;
                },
                EstatePlanType::RevocableTrust | EstatePlanType::IrrevocableTrust => {
                    *trust_distributions.entry(plan.name.clone()).or_insert(0.0) += plan_assets_value;
                    
                    // Distribute trust assets to beneficiaries
                    for beneficiary in &plan.beneficiaries {
                        let amount = plan_assets_value * (beneficiary.allocation_percentage / 100.0);
                        *beneficiary_distributions.entry(beneficiary.name.clone()).or_insert(0.0) += amount;
                    }
                },
                _ => {
                    // For wills and other plans, distribute directly to beneficiaries
                    for beneficiary in &plan.beneficiaries {
                        let amount = plan_assets_value * (beneficiary.allocation_percentage / 100.0);
                        
                        if beneficiary.relationship.to_lowercase().contains("charity") {
                            charitable_distributions += amount;
                        } else {
                            *beneficiary_distributions.entry(beneficiary.name.clone()).or_insert(0.0) += amount;
                        }
                    }
                }
            }
        }
        
        // Create a simple distribution timeline (immediate distribution)
        let mut distribution_timeline = HashMap::new();
        let distribution_date = Utc::now().date_naive() + Duration::days(90); // Assume 90 days for estate settlement
        
        for (beneficiary, amount) in &beneficiary_distributions {
            distribution_timeline.insert(
                beneficiary.clone(),
                vec![(distribution_date, *amount)]
            );
        }
        
        EstateDistributionAnalysis {
            total_estate_value,
            after_tax_value: after_tax_estate_value,
            distributions_by_beneficiary: HashMap::new(), // This should be properly initialized
            after_tax_estate_value,
            beneficiary_distributions,
            charitable_distributions,
            trust_distributions,
            distribution_timeline: distribution_timeline.iter().map(|(k, v)| (k.clone(), v.iter().map(|(_, amount)| *amount).sum())).collect(),
        }
    }
    
    pub fn check_estate_document_status(&self, household: &UnifiedManagedHousehold) -> Vec<EstateDocumentStatus> {
        let mut document_status = Vec::new();
        
        // Check for will
        let has_will = household.estate_plans.iter()
            .any(|p| matches!(p.plan_type, EstatePlanType::Will));
            
        document_status.push(EstateDocumentStatus {
            document_type: "Will".to_string(),
            exists: has_will,
            last_updated: household.estate_plans.iter()
                .filter(|p| matches!(p.plan_type, EstatePlanType::Will))
                .map(|p| p.last_updated)
                .next(),
            location: None,
            notes: if has_will { None } else { Some("No will found. Creating a will is essential for all household members.".to_string()) },
        });
        
        // Check for revocable trust
        let has_revocable_trust = household.estate_plans.iter()
            .any(|p| matches!(p.plan_type, EstatePlanType::RevocableTrust));
            
        document_status.push(EstateDocumentStatus {
            document_type: "Revocable Living Trust".to_string(),
            exists: has_revocable_trust,
            last_updated: household.estate_plans.iter()
                .filter(|p| matches!(p.plan_type, EstatePlanType::RevocableTrust))
                .map(|p| p.last_updated)
                .next(),
            location: None,
            notes: if has_revocable_trust { None } else { Some("Consider creating a revocable living trust to avoid probate.".to_string()) },
        });
        
        // Check for power of attorney
        document_status.push(EstateDocumentStatus {
            document_type: "Durable Power of Attorney".to_string(),
            exists: false, // We don't track this yet
            last_updated: None,
            location: None,
            notes: Some("Ensure all adult household members have a durable power of attorney for financial matters.".to_string()),
        });
        
        // Check for healthcare directive
        document_status.push(EstateDocumentStatus {
            document_type: "Healthcare Directive".to_string(),
            exists: false, // We don't track this yet
            last_updated: None,
            location: None,
            notes: Some("Ensure all adult household members have a healthcare directive and medical power of attorney.".to_string()),
        });
        
        // Check for beneficiary designations
        let accounts_with_designations: Vec<String> = household.beneficiary_designations.iter()
            .map(|d| d.account_id.clone())
            .collect();
            
        let accounts_missing_designations: Vec<String> = household.accounts.keys()
            .filter(|id| !accounts_with_designations.contains(*id))
            .cloned()
            .collect();
            
        document_status.push(EstateDocumentStatus {
            document_type: "Beneficiary Designations".to_string(),
            exists: accounts_missing_designations.is_empty(),
            last_updated: None,
            location: None,
            notes: if accounts_missing_designations.is_empty() {
                None
            } else {
                Some(format!("Missing beneficiary designations for {} accounts.", accounts_missing_designations.len()))
            },
        });
        
        document_status
    }
    
    pub fn generate_estate_planning_recommendations(&self, household: &UnifiedManagedHousehold) -> Vec<EstatePlanningRecommendation> {
        let mut recommendations = Vec::new();
        
        // Get estate tax analysis
        let tax_analysis = self.analyze_estate_taxes(household, None, 0.0);
        
        // Check document status
        let document_status = self.check_estate_document_status(household);
        
        // Recommendation 1: Create missing essential documents
        for doc in &document_status {
            if !doc.exists {
                recommendations.push(EstatePlanningRecommendation {
                    id: Uuid::new_v4().to_string(),
                    recommendation_type: EstatePlanningRecommendationType::DocumentCreation,
                    description: format!("Create {}", doc.document_type),
                    rationale: doc.notes.clone().unwrap_or_else(|| format!("{} is an essential estate planning document.", doc.document_type)),
                    priority: 3, // High priority
                    estimated_benefit: None,
                });
            }
        }
        
        // Recommendation 2: Update outdated documents
        for doc in &document_status {
            if let Some(last_updated) = doc.last_updated {
                let years_since_update = (Utc::now().date_naive().year() - last_updated.year()) as f64;
                
                if years_since_update >= 5.0 {
                    recommendations.push(EstatePlanningRecommendation {
                        id: Uuid::new_v4().to_string(),
                        recommendation_type: EstatePlanningRecommendationType::DocumentUpdate,
                        description: format!("Update {} (last updated {} years ago)", doc.document_type, years_since_update as i32),
                        rationale: "Estate planning documents should be reviewed every 3-5 years or after major life events.".to_string(),
                        priority: 2, // Medium priority
                        estimated_benefit: None,
                    });
                }
            }
        }
        
        // Recommendation 3: Tax reduction strategies
        for strategy in &tax_analysis.tax_reduction_strategies {
            recommendations.push(EstatePlanningRecommendation {
                id: Uuid::new_v4().to_string(),
                recommendation_type: EstatePlanningRecommendationType::TaxReduction,
                description: strategy.description.clone(),
                rationale: strategy.description.clone(),
                priority: strategy.priority.clone(),
                estimated_benefit: Some(strategy.estimated_tax_savings),
            });
        }
        
        // Recommendation 4: Create revocable trust if estate is large
        if tax_analysis.gross_estate_value > 1_000_000.0 && 
           !household.estate_plans.iter().any(|p| matches!(p.plan_type, EstatePlanType::RevocableTrust)) {
            recommendations.push(EstatePlanningRecommendation {
                id: Uuid::new_v4().to_string(),
                recommendation_type: EstatePlanningRecommendationType::ProbateAvoidance,
                description: "Create a Revocable Living Trust".to_string(),
                rationale: "A revocable living trust can help avoid probate, maintain privacy, and provide for incapacity planning.".to_string(),
                priority: 2,
                estimated_benefit: Some(tax_analysis.gross_estate_value * 0.04), // Estimated probate savings
            });
        }
        
        // Recommendation 5: Review beneficiary designations
        if !household.beneficiary_designations.is_empty() {
            recommendations.push(EstatePlanningRecommendation {
                id: Uuid::new_v4().to_string(),
                recommendation_type: EstatePlanningRecommendationType::BeneficiaryReview,
                description: "Review all beneficiary designations".to_string(),
                rationale: "Ensure beneficiary designations are up-to-date and aligned with overall estate plan.".to_string(),
                priority: 1, // Low priority
                estimated_benefit: None,
            });
        }
        
        recommendations
    }
    
    pub fn generate_estate_planning_report(&self, household: &UnifiedManagedHousehold) -> EstatePlanningReport {
        let tax_analysis = self.analyze_estate_taxes(household, None, 0.0);
        let distribution_analysis = self.analyze_estate_distribution(household);
        let document_status = self.check_estate_document_status(household);
        let recommendations = self.generate_estate_planning_recommendations(household);
        
        EstatePlanningReport {
            estate_tax_analysis: tax_analysis.clone(),
            household_id: household.id.clone(),
            household_name: household.name.clone(),
            total_estate_value: household.total_market_value(),
            estate_plans: household.estate_plans.clone(),
            beneficiary_designations: household.beneficiary_designations.iter()
                .map(|bd| (bd.account_id.clone(), bd.beneficiaries.clone()))
                .collect(),
            tax_analysis,
            distribution_analysis,
            document_status,
            recommendations,
        }
    }

    // Charitable Giving Methods
    
    pub fn create_charity(
        &self,
        household: &mut UnifiedManagedHousehold,
        name: String,
        ein: Option<String>,
        category: String,
        is_qualified_501c3: bool,
        notes: Option<String>,
    ) -> Charity {
        let charity = Charity {
            id: format!("charity-{}", Uuid::new_v4()),
            name,
            ein,
            mission: "Supporting charitable causes".to_string(), // Default mission
            category,
            created_at: Utc::now().date_naive(),
            updated_at: Utc::now().date_naive(),
            is_qualified_501c3,
            notes,
        };
        
        household.add_charity(charity.clone());
        charity
    }
    
    pub fn create_charitable_vehicle(
        &self,
        household: &mut UnifiedManagedHousehold,
        name: String,
        vehicle_type: CharitableVehicleType,
        account_id: Option<String>,
        market_value: f64,
        annual_contribution: f64,
        annual_distribution: f64,
        beneficiary_charities: Vec<(String, f64)>,
        notes: Option<String>,
    ) -> Result<CharitableVehicle> {
        // Validate account exists if provided
        if let Some(account_id) = &account_id {
            if !household.accounts.contains_key(account_id) {
                return Err("Account not found".into());
            }
        }
        
        // Validate charities exist
        for (charity_id, _) in &beneficiary_charities {
            if !household.charities.iter().any(|c| c.id == *charity_id) {
                return Err(format!("Charity not found: {}", charity_id).into());
            }
        }
        
        // Validate allocation percentages sum to 100%
        let total_allocation: f64 = beneficiary_charities.iter().map(|(_, percentage)| percentage).sum();
        if (total_allocation - 100.0).abs() > 0.01 {
            return Err(format!("Beneficiary charity allocations must total 100%, got {}", total_allocation).into());
        }
        
        let vehicle = CharitableVehicle {
            id: format!("vehicle-{}", Uuid::new_v4()),
            name,
            vehicle_type,
            balance: 0.0,
            created_at: Utc::now().date_naive(),
            updated_at: Utc::now().date_naive(),
            account_id,
            market_value,
            annual_contribution,
            annual_distribution,
            beneficiary_charities,
            creation_date: Utc::now().date_naive(),
            notes,
        };
        
        household.add_charitable_vehicle(vehicle.clone());
        Ok(vehicle)
    }
    
    pub fn record_donation(
        &self,
        household: &mut UnifiedManagedHousehold,
        charity_id: String,
        vehicle_id: Option<String>,
        amount: f64,
        donation_date: NaiveDate,
        asset_type: String,
        security_id: Option<String>,
        cost_basis: Option<f64>,
        fair_market_value: f64,
        tax_year: i32,
        receipt_received: bool,
        notes: Option<String>,
    ) -> Result<CharitableDonation> {
        // Validate charity exists
        if !household.charities.iter().any(|c| c.id == charity_id) {
            return Err(format!("Charity not found: {}", charity_id).into());
        }
        
        // Validate vehicle exists if provided
        if let Some(vehicle_id) = &vehicle_id {
            if !household.charitable_vehicles.iter().any(|v| v.id == *vehicle_id) {
                return Err(format!("Charitable vehicle not found: {}", vehicle_id).into());
            }
        }
        
        let donation = CharitableDonation {
            id: format!("donation-{}", Uuid::new_v4()),
            charity_id,
            amount,
            donation_type: if security_id.is_some() {
                DonationType::Securities
            } else {
                DonationType::Cash
            },
            donation_date,
            vehicle_id,
            security_id,
            tax_year,
            receipt_received,
            notes,
            asset_type,
            cost_basis,
            fair_market_value,
        };
        
        household.add_donation(donation.clone());
        Ok(donation)
    }
    
    pub fn analyze_charitable_tax_impact(
        &self,
        household: &UnifiedManagedHousehold,
        tax_year: i32,
        carryover_from_previous_years: f64,
    ) -> CharitableTaxImpact {
        // Get household AGI (mock implementation)
        let household_agi = self.estimate_household_agi(household);
        
        // Calculate donations by type
        let year_donations: Vec<&CharitableDonation> = household.donations.iter()
            .filter(|d| d.tax_year == tax_year)
            .collect();
            
        let total_donations: f64 = year_donations.iter().map(|d| d.amount).sum();
        
        let cash_donations: f64 = year_donations.iter()
            .filter(|d| d.asset_type.to_lowercase() == "cash")
            .map(|d| d.amount)
            .sum();
            
        let non_cash_donations: f64 = year_donations.iter()
            .filter(|d| d.asset_type.to_lowercase() != "cash")
            .map(|d| d.fair_market_value)
            .sum();
            
        let qualified_charitable_distributions: f64 = year_donations.iter()
            .filter(|d| {
                if let Some(vehicle_id) = &d.vehicle_id {
                    household.charitable_vehicles.iter()
                        .any(|v| v.id == *vehicle_id && v.vehicle_type == CharitableVehicleType::QualifiedCharitableDistribution)
                } else {
                    false
                }
            })
            .map(|d| d.amount)
            .sum();
        
        // Apply AGI limitations (simplified)
        // Cash donations limited to 60% of AGI
        // Non-cash donations limited to 30% of AGI
        let cash_limitation = household_agi * 0.6;
        let non_cash_limitation = household_agi * 0.3;
        
        let deductible_cash = cash_donations.min(cash_limitation);
        let deductible_non_cash = non_cash_donations.min(non_cash_limitation);
        
        let total_deductible = deductible_cash + deductible_non_cash + carryover_from_previous_years;
        let total_deductible_limited = total_deductible.min(household_agi * 0.6);
        
        let agi_limitation_impact = total_donations - total_deductible_limited;
        let carryover_to_next_year = if agi_limitation_impact > 0.0 { agi_limitation_impact } else { 0.0 };
        
        // Calculate tax savings (simplified)
        let (marginal_rate, _) = self.get_household_tax_rates(household);
        let tax_savings = total_deductible_limited * marginal_rate;
        
        let effective_tax_rate_reduction = if household_agi > 0.0 {
            tax_savings / household_agi
        } else {
            0.0
        };
        
        CharitableTaxImpact {
            total_donations,
            cash_donations,
            non_cash_donations,
            qualified_charitable_distributions,
            agi_limitation_impact,
            carryover_from_previous_years,
            carryover_to_next_year,
            tax_savings_current_year: tax_savings,
            effective_tax_rate_reduction,
        }
    }
    
    fn estimate_household_agi(&self, household: &UnifiedManagedHousehold) -> f64 {
        // Mock implementation - in a real system, this would be calculated based on income sources
        // For this example, we'll estimate AGI as 5% of the household's total market value
        household.total_market_value() * 0.05
    }
    
    pub fn generate_donation_strategies(&self, household: &UnifiedManagedHousehold) -> Vec<DonationStrategy> {
        let mut strategies = Vec::new();
        let household_agi = self.estimate_household_agi(household);
        let (marginal_rate, _) = self.get_household_tax_rates(household);
        let total_market_value = household.total_market_value();
        
        // Strategy 1: Donate appreciated securities instead of cash
        let has_appreciated_securities = household.accounts.values()
            .flat_map(|a| a.sleeves.iter().flat_map(|s| &s.holdings))
            .any(|h| h.market_value > 0.0);
            
        if has_appreciated_securities {
            strategies.push(DonationStrategy {
                strategy_type: "Appreciated Securities".to_string(),
                description: "Donate appreciated securities instead of cash to avoid capital gains tax".to_string(),
                potential_tax_savings: household_agi * 0.02 * marginal_rate,
                potential_additional_donation: household_agi * 0.02 * 0.15, // Assuming 15% capital gains tax
                complexity: "Low".to_string(),
                priority: RecommendationPriority::High, // High priority
            });
        }
        
        // Strategy 2: Donor-Advised Fund
        let has_daf = household.charitable_vehicles.iter()
            .any(|v| matches!(v.vehicle_type, CharitableVehicleType::DonorAdvisedFund));
            
        if !has_daf && total_market_value > 500_000.0 {
            strategies.push(DonationStrategy {
                strategy_type: "Donor-Advised Fund".to_string(),
                description: "Establish a donor-advised fund to bunch multiple years of donations for tax efficiency".to_string(),
                potential_tax_savings: household_agi * 0.05 * marginal_rate,
                potential_additional_donation: 0.0,
                complexity: "Medium".to_string(),
                priority: RecommendationPriority::Medium, // Medium priority
            });
        }
        
        // Strategy 3: Qualified Charitable Distribution from IRA
        let has_ira = household.accounts.iter()
            .any(|(_, account)| account.name.contains("IRA"));
            
        let has_qcd = household.charitable_vehicles.iter()
            .any(|v| matches!(v.vehicle_type, CharitableVehicleType::QualifiedCharitableDistribution));
            
        let has_member_over_70 = household.members.iter()
            .any(|m| {
                if let Some(age) = self.get_member_age(household, &m.id) {
                    age >= 70
                } else {
                    false
                }
            });
            
        if has_ira && !has_qcd && has_member_over_70 {
            strategies.push(DonationStrategy {
                strategy_type: "Qualified Charitable Distribution".to_string(),
                description: "Make charitable donations directly from IRA to satisfy RMD requirements without increasing AGI".to_string(),
                potential_tax_savings: household_agi * 0.03 * marginal_rate,
                potential_additional_donation: 0.0,
                complexity: "Low".to_string(),
                priority: RecommendationPriority::High, // High priority
            });
        }
        
        // Strategy 4: Charitable Remainder Trust
        let has_crt = household.charitable_vehicles.iter()
            .any(|v| matches!(v.vehicle_type, CharitableVehicleType::CharitableRemainder));
            
        if !has_crt && total_market_value > 1_000_000.0 {
            strategies.push(DonationStrategy {
                strategy_type: "Charitable Remainder Trust".to_string(),
                description: "Establish a charitable remainder trust to generate income while making a significant charitable gift".to_string(),
                potential_tax_savings: household_agi * 0.08 * marginal_rate,
                potential_additional_donation: household_agi * 0.05,
                complexity: "High".to_string(),
                priority: RecommendationPriority::Medium, // Medium priority
            });
        }
        
        // Strategy 5: Bunching Donations
        strategies.push(DonationStrategy {
            strategy_type: "Donation Bunching".to_string(),
            description: "Concentrate multiple years of charitable giving into a single tax year to exceed standard deduction threshold".to_string(),
            potential_tax_savings: household_agi * 0.02 * marginal_rate,
            potential_additional_donation: 0.0,
            complexity: "Low".to_string(),
            priority: RecommendationPriority::Low, // Low priority
        });
        
        // Strategy 6: Private Foundation
        if total_market_value > 5_000_000.0 {
            strategies.push(DonationStrategy {
                strategy_type: "Private Foundation".to_string(),
                description: "Establish a private foundation for greater control over charitable giving and family legacy".to_string(),
                potential_tax_savings: household_agi * 0.10 * marginal_rate,
                potential_additional_donation: 0.0,
                complexity: "Very High".to_string(),
                priority: RecommendationPriority::Low, // Low priority
            });
        }
        
        strategies
    }
    
    pub fn create_charitable_giving_plan(
        &self,
        household: &UnifiedManagedHousehold,
        target_annual_giving: f64,
        tax_year: i32,
        carryover_from_previous_years: f64,
    ) -> CharitableGivingPlan {
        let tax_impact = self.analyze_charitable_tax_impact(household, tax_year, carryover_from_previous_years);
        let strategies = self.generate_donation_strategies(household);
        
        // Create multi-year projection
        let mut multi_year_projection = HashMap::new();
        let mut carryover = tax_impact.carryover_to_next_year;
        
        for year_offset in 1..6 {
            let projected_year = tax_year + year_offset;
            let projected_impact = self.analyze_charitable_tax_impact(household, projected_year, carryover);
            carryover = projected_impact.carryover_to_next_year;
            multi_year_projection.insert(projected_year, projected_impact);
        }
        
        // Recommend charitable vehicles based on strategies
        let mut recommended_vehicles = Vec::new();
        
        // If DAF strategy is recommended and no DAF exists
        if strategies.iter().any(|s| s.strategy_type == "Donor-Advised Fund") &&
           !household.charitable_vehicles.iter().any(|v| matches!(v.vehicle_type, CharitableVehicleType::DonorAdvisedFund)) {
            // Create a recommended DAF
            let daf = CharitableVehicle {
                id: "recommended-daf".to_string(),
                name: "Recommended Donor-Advised Fund".to_string(),
                vehicle_type: CharitableVehicleType::DonorAdvisedFund,
                balance: 0.0,
                created_at: Utc::now().date_naive(),
                updated_at: Utc::now().date_naive(),
                account_id: None,
                market_value: 0.0,
                annual_contribution: target_annual_giving * 0.5,
                annual_distribution: target_annual_giving * 0.4,
                beneficiary_charities: Vec::new(),
                creation_date: Utc::now().date_naive(),
                notes: Some("Recommended as part of charitable giving plan".to_string()),
            };
            recommended_vehicles.push(daf);
        }
        
        // If CRT strategy is recommended and no CRT exists
        if strategies.iter().any(|s| s.strategy_type == "Charitable Remainder Trust") &&
           !household.charitable_vehicles.iter().any(|v| matches!(v.vehicle_type, CharitableVehicleType::CharitableRemainder)) {
            // Create a recommended CRT
            let crt = CharitableVehicle {
                id: "recommended-crt".to_string(),
                name: "Recommended Charitable Remainder Trust".to_string(),
                vehicle_type: CharitableVehicleType::CharitableRemainder,
                account_id: None,
                market_value: 100000.0,
                annual_contribution: 10000.0,
                annual_distribution: 5000.0,
                beneficiary_charities: Vec::new(),
                creation_date: Utc::now().date_naive(),
                notes: Some("Recommended for tax-efficient charitable giving".to_string()),
                balance: 100000.0,
                created_at: Utc::now().date_naive(),
                updated_at: Utc::now().date_naive(),
            };
            recommended_vehicles.push(crt);
        }
        
        CharitableGivingPlan {
            total_annual_giving: target_annual_giving,
            recommended_vehicles,
            recommended_donation_strategies: strategies,
            tax_impact,
            multi_year_projection,
        }
    }
    
    pub fn generate_charitable_giving_report(
        &self,
        household: &UnifiedManagedHousehold,
        tax_year: i32,
        carryover_from_previous_years: f64,
    ) -> CharitableGivingReport {
        // Calculate total lifetime donations
        let total_lifetime_donations: f64 = household.donations.iter()
            .map(|d| d.amount)
            .sum();
            
        // Calculate current year donations
        let current_year_donations: f64 = household.donations.iter()
            .filter(|d| d.tax_year == tax_year)
            .map(|d| d.amount)
            .sum();
            
        // Get favorite charities (based on donation frequency and amount)
        let mut charity_donations: HashMap<String, f64> = HashMap::new();
        for donation in &household.donations {
            *charity_donations.entry(donation.charity_id.clone()).or_insert(0.0) += donation.amount;
        }
        
        let mut favorite_charities: Vec<(String, f64)> = charity_donations.into_iter().collect();
        favorite_charities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        let favorite_charities: Vec<Charity> = favorite_charities.iter()
            .take(5) // Top 5 charities
            .filter_map(|(charity_id, _)| household.get_charity(charity_id).cloned())
            .collect();
            
        // Analyze tax impact
        let tax_impact = self.analyze_charitable_tax_impact(household, tax_year, carryover_from_previous_years);
        
        // Generate donation strategies
        let recommended_strategies = self.generate_donation_strategies(household);
        
        // Create giving plan if donations exceed threshold
        let giving_plan = if current_year_donations > 10_000.0 {
            Some(self.create_charitable_giving_plan(
                household,
                current_year_donations, // Use current giving as target
                tax_year,
                carryover_from_previous_years,
            ))
        } else {
            None
        };
        
        CharitableGivingReport {
            household_id: household.id.clone(),
            household_name: household.name.clone(),
            total_lifetime_donations,
            current_year_donations,
            charitable_vehicles: household.charitable_vehicles.clone(),
            donation_history: household.donations.clone(),
            favorite_charities,
            tax_impact,
            recommended_strategies,
            giving_plan,
        }
    }
}

/// Household asset allocation analysis
#[derive(Debug, Clone)]
pub struct HouseholdAssetAllocation {
    pub household_id: String,
    pub household_name: String,
    pub total_market_value: f64,
    pub asset_class_allocation: HashMap<String, f64>,
    pub sector_allocation: HashMap<String, f64>,
    pub security_allocation: HashMap<String, f64>,
    pub asset_location_score: f64,
    pub tax_efficiency_score: f64,
    pub asset_location_recommendations: Vec<AssetLocationRecommendation>,
}

#[derive(Debug, Clone)]
pub struct DonationStrategy {
    pub strategy_type: String,
    pub description: String,
    pub potential_tax_savings: f64,
    pub potential_additional_donation: f64,
    pub complexity: String,
    pub priority: RecommendationPriority,
}

#[derive(Debug, Clone)]
pub struct CharitableTaxImpact {
    pub total_donations: f64,
    pub cash_donations: f64,
    pub non_cash_donations: f64,
    pub qualified_charitable_distributions: f64,
    pub agi_limitation_impact: f64,
    pub carryover_from_previous_years: f64,
    pub carryover_to_next_year: f64,
    pub tax_savings_current_year: f64,
    pub effective_tax_rate_reduction: f64,
}

#[derive(Debug, Clone)]
pub struct CharitableGivingPlan {
    pub total_annual_giving: f64,
    pub recommended_vehicles: Vec<CharitableVehicle>,
    pub recommended_donation_strategies: Vec<DonationStrategy>,
    pub tax_impact: CharitableTaxImpact,
    pub multi_year_projection: HashMap<i32, CharitableTaxImpact>,
}

#[derive(Debug, Clone)]
pub struct CharitableGivingReport {
    pub household_id: String,
    pub household_name: String,
    pub total_lifetime_donations: f64,
    pub current_year_donations: f64,
    pub charitable_vehicles: Vec<CharitableVehicle>,
    pub donation_history: Vec<CharitableDonation>,
    pub favorite_charities: Vec<Charity>,
    pub tax_impact: CharitableTaxImpact,
    pub recommended_strategies: Vec<DonationStrategy>,
    pub giving_plan: Option<CharitableGivingPlan>,
} 