#[cfg(test)]
mod property_tests {
    use crate::{
        calculate_twr, calculate_mwr, calculate_volatility, calculate_sharpe_ratio,
        calculate_max_drawdown, calculate_tracking_error, calculate_information_ratio,
        calculate_sortino_ratio, calculate_treynor_ratio, calculate_portfolio_beta,
        dynamodb_repository::{Transaction, Valuation},
        convert_valuations_to_returns
    };
    use chrono::{DateTime, Utc, TimeZone, Duration};
    use proptest::prelude::*;
    use proptest::collection::vec;
    use proptest::num::f64;
    
    // Helper function to create a test transaction
    fn create_test_transaction(
        id: &str,
        account_id: &str,
        portfolio_id: &str,
        transaction_type: &str,
        date: DateTime<Utc>,
        amount: f64,
    ) -> Transaction {
        Transaction {
            id: id.to_string(),
            account_id: account_id.to_string(),
            portfolio_id: portfolio_id.to_string(),
            transaction_type: transaction_type.to_string(),
            transaction_date: date,
            settlement_date: Some(date),
            amount,
            currency: "USD".to_string(),
            security_id: None,
            quantity: None,
            price: None,
            fees: None,
            taxes: None,
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    // Helper function to create a test valuation
    fn create_test_valuation(
        id: &str,
        portfolio_id: &str,
        date: DateTime<Utc>,
        value: f64,
    ) -> Valuation {
        Valuation {
            id: id.to_string(),
            portfolio_id: portfolio_id.to_string(),
            date,
            value,
            cash_balance: value * 0.1, // 10% cash for testing
            currency: "USD".to_string(),
            created_at: Utc::now(),
        }
    }
    
    // Strategy to generate a valid date within a reasonable range
    fn date_strategy() -> impl Strategy<Value = DateTime<Utc>> {
        // Generate dates between 2020-01-01 and 2025-12-31
        (2020..=2025, 1..=12, 1..=28).prop_map(|(year, month, day)| {
            Utc.ymd(year, month, day).and_hms(0, 0, 0)
        })
    }
    
    // Strategy to generate a sorted list of dates
    fn sorted_dates_strategy(min_size: usize, max_size: usize) -> impl Strategy<Value = Vec<DateTime<Utc>>> {
        vec(date_strategy(), min_size..=max_size)
            .prop_map(|mut dates| {
                dates.sort();
                dates
            })
    }
    
    // Strategy to generate a positive f64 value
    fn positive_f64() -> impl Strategy<Value = f64> {
        f64::POSITIVE
    }
    
    // Strategy to generate a non-zero f64 value
    fn non_zero_f64() -> impl Strategy<Value = f64> {
        f64::NONZERO
    }
    
    // Strategy to generate a list of valuations with increasing dates
    fn valuations_strategy(portfolio_id: &'static str, min_size: usize, max_size: usize) -> impl Strategy<Value = Vec<Valuation>> {
        sorted_dates_strategy(min_size, max_size)
            .prop_flat_map(|dates| {
                // Generate a series of values that are somewhat realistic
                // Start with a base value and apply random changes
                let base_value = 10000.0;
                vec(f64::range(-0.05..0.05).boxed(), dates.len())
                    .prop_map(move |changes| {
                        let mut value = base_value;
                        let mut valuations = Vec::new();
                        
                        for (i, (date, change)) in dates.iter().zip(changes.iter()).enumerate() {
                            value *= 1.0 + change;
                            valuations.push(create_test_valuation(
                                &format!("val{}", i + 1),
                                portfolio_id,
                                *date,
                                value,
                            ));
                        }
                        
                        valuations
                    })
            })
    }
    
    // Strategy to generate a list of transactions with dates between the valuation dates
    fn transactions_strategy(
        portfolio_id: &'static str, 
        account_id: &'static str, 
        valuations: Vec<Valuation>
    ) -> impl Strategy<Value = Vec<Transaction>> {
        if valuations.len() < 2 {
            return vec(Just(Vec::new())).boxed();
        }
        
        let start_date = valuations.first().unwrap().date;
        let end_date = valuations.last().unwrap().date;
        
        // Generate between 0 and 5 transactions
        vec(
            (
                // Generate a date between start_date and end_date
                Just(start_date)
                    .prop_perturb(move |_, _| {
                        let days_between = (end_date - start_date).num_days();
                        let random_days = rand::random::<u64>() % (days_between as u64 + 1);
                        start_date + Duration::days(random_days as i64)
                    }),
                // Generate an amount between 100 and 10000
                (100.0..10000.0),
                // Generate a transaction type (DEPOSIT or WITHDRAWAL)
                prop_oneof![Just("DEPOSIT"), Just("WITHDRAWAL")]
            ),
            0..5
        ).prop_map(move |tx_data| {
            tx_data.into_iter()
                .enumerate()
                .map(|(i, (date, amount, tx_type))| {
                    create_test_transaction(
                        &format!("tx{}", i + 1),
                        account_id,
                        portfolio_id,
                        tx_type,
                        date,
                        amount,
                    )
                })
                .collect()
        }).boxed()
    }
    
    proptest! {
        // Test that TWR calculation doesn't panic and returns a reasonable value
        #[test]
        fn twr_doesnt_panic(
            valuations in valuations_strategy("portfolio1", 2, 10)
        ) {
            let transactions = vec![];
            let result = calculate_twr(&transactions, &valuations);
            
            // The calculation should succeed
            prop_assert!(result.is_ok());
            
            // The TWR should be a finite number
            let twr = result.unwrap();
            prop_assert!(twr.is_finite());
            
            // For reasonable inputs, TWR should be within a reasonable range
            // For our test data, we expect TWR to be between -50% and +100%
            prop_assert!(twr > -0.5 && twr < 1.0, "TWR outside reasonable range: {}", twr);
        }
        
        // Test that MWR calculation doesn't panic and returns a reasonable value
        #[test]
        fn mwr_doesnt_panic(
            valuations in valuations_strategy("portfolio1", 2, 10)
        ) {
            let transactions = vec![];
            let result = calculate_mwr(&transactions, &valuations);
            
            // The calculation should succeed or return InsufficientData
            match result {
                Ok(mwr) => {
                    // The MWR should be a finite number
                    prop_assert!(mwr.is_finite());
                    
                    // For reasonable inputs, MWR should be within a reasonable range
                    prop_assert!(mwr > -0.5 && mwr < 1.0, "MWR outside reasonable range: {}", mwr);
                },
                Err(e) => {
                    // If there's an error, it should be InsufficientData
                    match e {
                        crate::CalculationError::InsufficientData(_) => {},
                        crate::CalculationError::ConvergenceFailure(_) => {},
                        _ => prop_assert!(false, "Unexpected error: {:?}", e),
                    }
                }
            }
        }
        
        // Test that volatility calculation doesn't panic
        #[test]
        fn volatility_doesnt_panic(
            valuations in valuations_strategy("portfolio1", 5, 20)
        ) {
            let transactions = vec![];
            let result = calculate_volatility(&transactions, &valuations);
            
            // The calculation should succeed
            prop_assert!(result.is_ok());
            
            // The volatility should be None or a positive number
            match result.unwrap() {
                Some(vol) => {
                    prop_assert!(vol > 0.0, "Volatility should be positive: {}", vol);
                    prop_assert!(vol < 1.0, "Volatility should be less than 100%: {}", vol);
                },
                None => {}
            }
        }
        
        // Test that Sharpe ratio calculation doesn't panic
        #[test]
        fn sharpe_ratio_doesnt_panic(
            return_value in -0.5..1.0,
            volatility in 0.01..0.5,
            risk_free_rate in 0.0..0.1
        ) {
            let result = calculate_sharpe_ratio(return_value, volatility, risk_free_rate);
            
            // The calculation should succeed
            prop_assert!(result.is_ok());
            
            // The Sharpe ratio should be a finite number
            let sharpe = result.unwrap();
            prop_assert!(sharpe.is_finite());
        }
        
        // Test that max drawdown calculation doesn't panic
        #[test]
        fn max_drawdown_doesnt_panic(
            valuations in valuations_strategy("portfolio1", 2, 20)
        ) {
            let result = calculate_max_drawdown(&valuations);
            
            // The calculation should succeed
            prop_assert!(result.is_ok());
            
            // The max drawdown should be between 0 and 1
            let drawdown = result.unwrap();
            prop_assert!(drawdown >= 0.0 && drawdown <= 1.0, "Max drawdown outside [0,1]: {}", drawdown);
        }
        
        // Test that tracking error calculation doesn't panic
        #[test]
        fn tracking_error_doesnt_panic(
            portfolio_returns in vec(f64::range(-0.1..0.1), 5..20),
            benchmark_returns in vec(f64::range(-0.1..0.1), 5..20)
        ) {
            // Create dates for the returns
            let dates: Vec<DateTime<Utc>> = (0..portfolio_returns.len().max(benchmark_returns.len()))
                .map(|i| Utc.ymd(2023, 1, 1).and_hms(0, 0, 0) + Duration::days(i as i64 * 30))
                .collect();
            
            // Create return series with dates
            let portfolio_returns_with_dates: Vec<(DateTime<Utc>, f64)> = dates.iter()
                .zip(portfolio_returns.iter())
                .take(portfolio_returns.len().min(dates.len()))
                .map(|(date, ret)| (*date, *ret))
                .collect();
            
            let benchmark_returns_with_dates: Vec<(DateTime<Utc>, f64)> = dates.iter()
                .zip(benchmark_returns.iter())
                .take(benchmark_returns.len().min(dates.len()))
                .map(|(date, ret)| (*date, *ret))
                .collect();
            
            // Only test if we have enough data points
            if portfolio_returns_with_dates.len() >= 2 && benchmark_returns_with_dates.len() >= 2 {
                let result = calculate_tracking_error(&portfolio_returns_with_dates, &benchmark_returns_with_dates, false);
                
                // The calculation should succeed or return InsufficientData
                match result {
                    Ok(te) => {
                        // The tracking error should be a positive number
                        prop_assert!(te >= 0.0, "Tracking error should be non-negative: {}", te);
                    },
                    Err(e) => {
                        // If there's an error, it should be InsufficientData
                        match e {
                            crate::CalculationError::InsufficientData(_) => {},
                            _ => prop_assert!(false, "Unexpected error: {:?}", e),
                        }
                    }
                }
            }
        }
        
        // Test that information ratio calculation doesn't panic
        #[test]
        fn information_ratio_doesnt_panic(
            portfolio_return in -0.5..1.0,
            benchmark_return in -0.5..1.0,
            tracking_error in 0.01..0.5
        ) {
            let result = calculate_information_ratio(portfolio_return, benchmark_return, tracking_error);
            
            // The calculation should succeed
            prop_assert!(result.is_ok());
            
            // The information ratio should be a finite number
            let ir = result.unwrap();
            prop_assert!(ir.is_finite());
        }
        
        // Test that Sortino ratio calculation doesn't panic
        #[test]
        fn sortino_ratio_doesnt_panic(
            return_value in -0.5..1.0,
            returns in vec(f64::range(-0.1..0.1), 5..20),
            risk_free_rate in 0.0..0.1
        ) {
            let result = calculate_sortino_ratio(return_value, &returns, risk_free_rate, None);
            
            // The calculation should succeed or return InsufficientData
            match result {
                Ok(sortino) => {
                    // The Sortino ratio should be a finite number
                    prop_assert!(sortino.is_finite());
                },
                Err(e) => {
                    // If there's an error, it should be InsufficientData
                    match e {
                        crate::CalculationError::InsufficientData(_) => {},
                        _ => prop_assert!(false, "Unexpected error: {:?}", e),
                    }
                }
            }
        }
        
        // Test that Treynor ratio calculation doesn't panic
        #[test]
        fn treynor_ratio_doesnt_panic(
            return_value in -0.5..1.0,
            portfolio_beta in -2.0..2.0,
            risk_free_rate in 0.0..0.1
        ) {
            // Skip the test if beta is too close to zero to avoid division by zero
            if portfolio_beta.abs() < 0.01 {
                return Ok(());
            }
            
            let result = calculate_treynor_ratio(return_value, portfolio_beta, risk_free_rate);
            
            // The calculation should succeed
            prop_assert!(result.is_ok());
            
            // The Treynor ratio should be a finite number
            let treynor = result.unwrap();
            prop_assert!(treynor.is_finite());
        }
        
        // Test that portfolio beta calculation doesn't panic
        #[test]
        fn portfolio_beta_doesnt_panic(
            portfolio_returns in vec(f64::range(-0.1..0.1), 5..20),
            benchmark_returns in vec(f64::range(-0.1..0.1), 5..20)
        ) {
            // Only test if we have enough data points and they have the same length
            let min_len = portfolio_returns.len().min(benchmark_returns.len());
            if min_len >= 2 {
                let portfolio_returns = &portfolio_returns[0..min_len];
                let benchmark_returns = &benchmark_returns[0..min_len];
                
                let result = calculate_portfolio_beta(portfolio_returns, benchmark_returns);
                
                // The calculation should succeed or return InsufficientData
                match result {
                    Ok(beta) => {
                        // The beta should be a finite number
                        prop_assert!(beta.is_finite());
                    },
                    Err(e) => {
                        // If there's an error, it should be InsufficientData
                        match e {
                            crate::CalculationError::InsufficientData(_) => {},
                            _ => prop_assert!(false, "Unexpected error: {:?}", e),
                        }
                    }
                }
            }
        }
        
        // Test that convert_valuations_to_returns works correctly
        #[test]
        fn convert_valuations_to_returns_works_correctly(
            valuations in valuations_strategy("portfolio1", 2, 10)
        ) {
            let returns = convert_valuations_to_returns(&valuations);
            
            // The number of returns should be one less than the number of valuations
            prop_assert_eq!(returns.len(), valuations.len() - 1);
            
            // Each return should correspond to the correct date
            for i in 0..returns.len() {
                prop_assert_eq!(returns[i].0, valuations[i + 1].date);
                
                // Calculate the expected return
                let expected_return = (valuations[i + 1].value - valuations[i].value) / valuations[i].value;
                
                // The calculated return should be close to the expected return
                prop_assert!((returns[i].1 - expected_return).abs() < 1e-10, 
                            "Return calculation mismatch: got {}, expected {}", 
                            returns[i].1, expected_return);
            }
        }
    }
} 