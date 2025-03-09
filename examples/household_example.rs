use investment_management::portfolio::model::household::{
    HouseholdService, AccountTaxType, MemberRelationship
};
use investment_management::portfolio::model::{
    TaxOptimizationSettings, ESGScreeningCriteria, ModelPortfolioService
};
use investment_management::portfolio::factor::FactorModelApi;
use investment_management::portfolio::rebalancing::TradeReason;

fn main() {
    println!("=== Investment Management Platform: Household Example ===\n");
    
    // Create services
    let factor_model_api = FactorModelApi::new();
    let model_service = ModelPortfolioService::new(factor_model_api);
    let household_service = HouseholdService::new();
    
    // Create a household
    let mut household = household_service.create_household(
        "Smith Family",
        "John Smith"
    );
    
    println!("Created household: {} ({})", household.name, household.id);
    println!("Primary member: {}", household.members[0].name);
    
    // Add additional household members
    let spouse = household.add_member("Jane Smith".to_string(), MemberRelationship::Spouse);
    println!("Added spouse: {} ({})", spouse.name, spouse.id);
    
    let child = household.add_member("Tommy Smith".to_string(), MemberRelationship::Child);
    println!("Added child: {} ({})", child.name, child.id);
    
    // Store member IDs for later use
    let primary_id = household.members[0].id.clone();
    let spouse_id = household.members[1].id.clone();
    // We don't use child_id in this simplified example
    
    // Create accounts for the household
    println!("\nCreating accounts for the household:");
    
    // Create a joint taxable account
    let joint_account = model_service.create_uma_from_model(
        "joint-account",
        "Joint Taxable Account",
        "John and Jane Smith",
        "model-1", // Assuming this model exists
        500000.0
    ).unwrap();
    
    println!("  Created joint account: {} ({})", joint_account.name, joint_account.id);
    
    // Add the joint account to the household
    household.add_account(
        joint_account,
        vec![primary_id.clone(), spouse_id.clone()],
        AccountTaxType::Taxable
    ).unwrap();
    
    // Create retirement accounts
    let primary_ira = model_service.create_uma_from_model(
        "primary-ira",
        "John's Traditional IRA",
        "John Smith",
        "model-2", // Assuming this model exists
        250000.0
    ).unwrap();
    
    println!("  Created primary IRA: {} ({})", primary_ira.name, primary_ira.id);
    
    // Add the primary IRA to the household
    household.add_account(
        primary_ira,
        vec![primary_id.clone()],
        AccountTaxType::TaxDeferred
    ).unwrap();
    
    let spouse_roth = model_service.create_uma_from_model(
        "spouse-roth",
        "Jane's Roth IRA",
        "Jane Smith",
        "model-3", // Assuming this model exists
        150000.0
    ).unwrap();
    
    println!("  Created spouse Roth IRA: {} ({})", spouse_roth.name, spouse_roth.id);
    
    // Add the spouse Roth IRA to the household
    household.add_account(
        spouse_roth,
        vec![spouse_id.clone()],
        AccountTaxType::TaxExempt
    ).unwrap();
    
    // Apply household-level tax optimization
    let tax_settings = TaxOptimizationSettings {
        annual_tax_budget: Some(10000.0),
        realized_gains_ytd: 2000.0,
        prioritize_loss_harvesting: true,
        defer_short_term_gains: true,
        min_tax_savings_threshold: Some(100.0),
        short_term_tax_rate: 0.35,
        long_term_tax_rate: 0.15,
    };
    
    household.apply_household_tax_optimization(tax_settings);
    println!("\nApplied household-level tax optimization");
    
    // Apply household-level ESG screening
    let esg_criteria = ESGScreeningCriteria {
        min_overall_score: Some(70.0),
        min_environmental_score: Some(65.0),
        min_social_score: None,
        min_governance_score: None,
        max_controversy_score: Some(30.0),
        excluded_sectors: vec!["Tobacco".to_string(), "Weapons".to_string()],
        excluded_activities: vec!["Animal Testing".to_string()],
    };
    
    household.apply_household_esg_screening(esg_criteria);
    println!("Applied household-level ESG screening");
    
    // Generate household report
    let report = household_service.generate_household_report(&household);
    
    println!("\n=== Household Financial Report ===");
    println!("Household: {} ({})", household.name, household.id);
    println!("Total Cash Balance: ${:.2}", report.total_cash_balance);
    
    println!("\nAccount Summary:");
    println!("  Number of accounts: {}", household.accounts.len());
    
    // Analyze asset allocation
    let asset_allocation = household_service.analyze_household_asset_allocation(&household);
    
    println!("\n=== Household Asset Allocation ===");
    println!("Asset Classes:");
    for (asset_class, percentage) in &asset_allocation.asset_class_allocation {
        println!("  {}: {:.2}%", asset_class, percentage * 100.0);
    }
    
    println!("\nSectors:");
    for (sector, percentage) in &asset_allocation.sector_allocation {
        println!("  {}: {:.2}%", sector, percentage * 100.0);
    }
    
    println!("\nAsset Location Efficiency Score: {:.2}", asset_allocation.asset_location_score);
    
    // Show asset location recommendations
    if !asset_allocation.asset_location_recommendations.is_empty() {
        println!("\nAsset Location Recommendations:");
        for recommendation in &asset_allocation.asset_location_recommendations {
            println!("  Move ${:.2} of {} from {} to {}", 
                recommendation.market_value,
                recommendation.security_id,
                recommendation.source_account_id,
                recommendation.target_account_id
            );
            println!("    Reason: {}", recommendation.reason);
        }
    }
    
    // Generate tax-optimized trades across the household
    let household_trades = household_service.generate_household_tax_optimized_trades(&household);
    
    println!("\n=== Tax-Optimized Trades Across Household ===");
    if household_trades.is_empty() {
        println!("  No trades generated");
    } else {
        for (account_id, trades) in household_trades {
            println!("  Account {}:", account_id);
            for trade in trades {
                println!("    {} ${:.2} of {} {}",
                    if trade.is_buy { "Buy" } else { "Sell" },
                    trade.amount,
                    trade.security_id,
                    match trade.reason {
                        TradeReason::Rebalance => "(Rebalance)",
                        TradeReason::TaxLossHarvesting => "(Tax-Loss Harvesting)",
                        _ => ""
                    }
                );
                
                if let Some(tax_impact) = trade.tax_impact {
                    if tax_impact < 0.0 {
                        println!("      Tax Savings: ${:.2}", -tax_impact);
                    } else if tax_impact > 0.0 {
                        println!("      Tax Cost: ${:.2}", tax_impact);
                    }
                }
            }
        }
    }
    
    // Get accounts by tax type
    let taxable_accounts = household.get_accounts_by_tax_type(&AccountTaxType::Taxable);
    let tax_deferred_accounts = household.get_accounts_by_tax_type(&AccountTaxType::TaxDeferred);
    let tax_exempt_accounts = household.get_accounts_by_tax_type(&AccountTaxType::TaxExempt);
    
    println!("\nAccounts by Tax Type:");
    println!("  Taxable Accounts: {}", taxable_accounts.len());
    println!("  Tax-Deferred Accounts: {}", tax_deferred_accounts.len());
    println!("  Tax-Exempt Accounts: {}", tax_exempt_accounts.len());
    
    // Get accounts by member
    let primary_accounts = household.get_member_accounts(&primary_id);
    let spouse_accounts = household.get_member_accounts(&spouse_id);
    
    println!("\nAccounts by Member:");
    println!("  {}'s Accounts: {}", household.members[0].name, primary_accounts.len());
    println!("  {}'s Accounts: {}", household.members[1].name, spouse_accounts.len());
    
    // Analyze household risk
    let risk_analysis = household_service.analyze_household_risk(&household);
    
    println!("\n=== Household Risk Analysis ===");
    println!("Portfolio Volatility: {:.2}%", risk_analysis.volatility * 100.0);
    println!("1-Day Value at Risk (95%): {:.2}%", risk_analysis.value_at_risk_95 * 100.0);
    println!("1-Day Conditional VaR (95%): {:.2}%", risk_analysis.conditional_var_95 * 100.0);
    
    println!("\nExample completed successfully!");
} 