# Test Case: Tax Optimization

## Test Case Information

**Test ID**: TC-2.4.1  
**Related TODO Item**: Implement tax optimization for asset location and withdrawal sequencing  
**Priority**: High  
**Type**: Integration  
**Created By**: Financial Advisor AI Team  
**Created Date**: 2023-09-16  

## Test Objective

Validate that the tax optimization module correctly recommends optimal asset location across different account types (taxable, tax-deferred, tax-free) and generates tax-efficient withdrawal sequences during retirement.

## Prerequisites

- Client profile data loaded (use `tests/data/client_profiles/sample_profiles.json`)
- Financial products catalog loaded (use `tests/data/financial_products/investment_vehicles.json`)
- Tax rate data for federal and state taxes
- Current tax law parameters (standard deduction, tax brackets, capital gains rates, etc.)

## Test Data

| Input | Expected Output | Notes |
|-------|----------------|-------|
| Client profile "client-001" with mixed account types | Tax-efficient asset location recommendations | Bonds in tax-deferred, international stocks in taxable, etc. |
| Client profile "client-002" nearing retirement | Tax-efficient withdrawal sequence | Taxable accounts first, then tax-deferred, then tax-free |
| Client profile "client-003" with high income | Tax-loss harvesting opportunities | Identify positions with unrealized losses |

## Test Steps

1. Load the client profile from the test data
2. Configure the tax optimization module with the following parameters:
   - Current federal and state tax rates
   - Expected future tax rates
   - Account types available (taxable, traditional IRA/401(k), Roth IRA/401(k))
   - Asset classes with expected returns, volatility, and tax characteristics
3. Run the asset location optimization algorithm
4. Generate tax-efficient withdrawal sequence recommendations
5. Identify tax-loss harvesting opportunities
6. Compare the results with expected outcomes

## Validation Criteria

- [ ] Asset location recommendations follow established tax-efficiency principles
- [ ] Withdrawal sequence recommendations minimize lifetime tax burden
- [ ] Tax-loss harvesting recommendations identify appropriate opportunities
- [ ] Roth conversion recommendations are made when beneficial
- [ ] Tax bracket management strategies are correctly identified
- [ ] Required Minimum Distribution (RMD) implications are correctly modeled
- [ ] State tax considerations are incorporated where relevant

## Test Code

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::financial_advisor::tax_optimization::{TaxOptimizer, AssetLocation, WithdrawalSequence};
    use crate::financial_advisor::client_profile::ClientProfile;
    use crate::financial_advisor::tax_data::{TaxRates, TaxBracket};
    
    #[test]
    fn test_asset_location_optimization() {
        // Arrange
        let client_profile = ClientProfile::load_from_file("tests/data/client_profiles/sample_profiles.json", "client-001").unwrap();
        let tax_rates = TaxRates::default_for_year(2023);
        
        let optimizer = TaxOptimizer::new()
            .with_client_profile(client_profile)
            .with_tax_rates(tax_rates)
            .with_future_tax_rate_assumption(TaxRateAssumption::Same);
        
        // Act
        let asset_location = optimizer.optimize_asset_location().unwrap();
        
        // Assert
        // Check that bonds are recommended for tax-deferred accounts
        let bond_location = asset_location.get_location_for_asset_class("us_bonds");
        assert_eq!(bond_location, AssetLocation::TaxDeferred, 
                  "Bonds should be located in tax-deferred accounts");
        
        // Check that international stocks with foreign tax credits are in taxable accounts
        let intl_stock_location = asset_location.get_location_for_asset_class("international_equity");
        assert_eq!(intl_stock_location, AssetLocation::Taxable,
                  "International stocks should be located in taxable accounts");
        
        // Check that high-growth assets are in tax-free accounts
        let growth_stock_location = asset_location.get_location_for_asset_class("us_small_cap_stocks");
        assert_eq!(growth_stock_location, AssetLocation::TaxFree,
                  "High-growth assets should be located in tax-free accounts");
    }
    
    #[test]
    fn test_withdrawal_sequence_optimization() {
        // Arrange
        let client_profile = ClientProfile::load_from_file("tests/data/client_profiles/sample_profiles.json", "client-002").unwrap();
        let tax_rates = TaxRates::default_for_year(2023);
        
        let optimizer = TaxOptimizer::new()
            .with_client_profile(client_profile)
            .with_tax_rates(tax_rates)
            .with_future_tax_rate_assumption(TaxRateAssumption::Lower);
        
        // Act
        let withdrawal_sequence = optimizer.optimize_withdrawal_sequence().unwrap();
        
        // Assert
        assert_eq!(withdrawal_sequence.first_account_type(), AccountType::Taxable,
                  "Taxable accounts should be withdrawn first");
        
        assert_eq!(withdrawal_sequence.last_account_type(), AccountType::TaxFree,
                  "Tax-free accounts should be withdrawn last");
        
        // Verify that RMDs are taken when required
        assert!(withdrawal_sequence.respects_rmds(),
               "Withdrawal sequence should respect Required Minimum Distributions");
    }
    
    #[test]
    fn test_tax_loss_harvesting() {
        // Arrange
        let client_profile = ClientProfile::load_from_file("tests/data/client_profiles/sample_profiles.json", "client-003").unwrap();
        let tax_rates = TaxRates::default_for_year(2023);
        
        let optimizer = TaxOptimizer::new()
            .with_client_profile(client_profile)
            .with_tax_rates(tax_rates);
        
        // Act
        let harvesting_opportunities = optimizer.identify_tax_loss_harvesting_opportunities().unwrap();
        
        // Assert
        assert!(!harvesting_opportunities.is_empty(),
               "Should identify at least one tax-loss harvesting opportunity");
        
        // Verify that wash sale rules are respected
        for opportunity in &harvesting_opportunities {
            assert!(opportunity.respects_wash_sale_rules(),
                   "Tax-loss harvesting opportunities should respect wash sale rules");
        }
    }
    
    #[test]
    fn test_roth_conversion_strategy() {
        // Test Roth conversion recommendations
        // ...
    }
}
```

## Reproduction Command

```bash
cargo test --package investment-management-platform --lib financial_advisor::tax_optimization::tests::test_asset_location_optimization
```

## Edge Cases to Consider

- Clients with only one account type
- Very high income clients in top tax brackets
- Clients with significant tax-loss carryforwards
- Clients with large Required Minimum Distributions
- Clients living in states with no income tax
- Clients with significant qualified charitable distributions
- Clients with net unrealized appreciation in employer stock

## Potential Failure Scenarios

- Tax law changes that invalidate optimization assumptions
- Complex tax situations not covered by the model (AMT, NIIT, etc.)
- Insufficient account diversification to implement optimal asset location
- Conflicting objectives (e.g., tax efficiency vs. desired asset allocation)
- Incorrect assumptions about future tax rates

## Dependencies

- Tax rate database
- Portfolio analysis module
- Client profile service
- Financial products database
- Retirement planning module

## Notes

This test validates the core functionality of the tax optimization module. The tax optimization strategies should be reviewed annually to account for tax law changes. The module should also be able to adapt to client-specific tax situations that may not be covered by general rules. Consider adding more sophisticated tests for specific tax strategies like Roth conversion ladders, charitable giving strategies, and tax-aware rebalancing. 