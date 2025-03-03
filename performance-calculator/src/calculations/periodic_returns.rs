use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDate, Datelike};
use serde::{Serialize, Deserialize};
use crate::calculations::risk_metrics::ReturnSeries;

/// Period types for periodic returns
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Period {
    /// Monthly period
    Monthly,
    /// Quarterly period
    Quarterly,
    /// Annual period
    Annual,
    /// Year-to-date period
    YTD,
    /// Since inception period
    SinceInception,
}

/// Periodic return result
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PeriodicReturn {
    /// Period type
    pub period: Period,
    /// Period label (e.g., "Jan 2023", "Q1 2023", "2023")
    pub label: String,
    /// Start date of the period
    pub start_date: NaiveDate,
    /// End date of the period
    pub end_date: NaiveDate,
    /// Return value for the period
    pub return_value: Decimal,
}

/// Calculate monthly returns from a return series
pub fn calculate_monthly_returns(return_series: &ReturnSeries) -> Result<Vec<PeriodicReturn>> {
    if return_series.dates.len() != return_series.returns.len() {
        return Err(anyhow!("Return series has inconsistent lengths"));
    }
    
    if return_series.dates.is_empty() {
        return Ok(Vec::new());
    }
    
    // Group returns by month
    let mut monthly_returns = HashMap::new();
    
    for i in 0..return_series.dates.len() {
        let date = return_series.dates[i];
        let return_value = return_series.returns[i];
        
        let year = date.year();
        let month = date.month();
        let key = (year, month);
        
        monthly_returns.entry(key)
            .or_insert_with(Vec::new)
            .push((date, return_value));
    }
    
    // Calculate return for each month
    let mut results = Vec::new();
    
    for ((year, month), values) in monthly_returns {
        if values.is_empty() {
            continue;
        }
        
        // Sort by date
        let mut sorted_values = values;
        sorted_values.sort_by(|a, b| a.0.cmp(&b.0));
        
        // Get start and end dates
        let start_date = sorted_values.first().unwrap().0;
        let end_date = sorted_values.last().unwrap().0;
        
        // Calculate cumulative return for the month
        let cumulative_return = sorted_values.iter()
            .fold(Decimal::ONE, |acc, (_, r)| acc * (Decimal::ONE + *r)) - Decimal::ONE;
        
        // Create month label
        let month_name = match month {
            1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr", 5 => "May", 6 => "Jun",
            7 => "Jul", 8 => "Aug", 9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
            _ => "Unknown",
        };
        
        let label = format!("{} {}", month_name, year);
        
        results.push(PeriodicReturn {
            period: Period::Monthly,
            label,
            start_date,
            end_date,
            return_value: cumulative_return,
        });
    }
    
    // Sort results by date
    results.sort_by(|a, b| a.start_date.cmp(&b.start_date));
    
    Ok(results)
}

/// Calculate quarterly returns from a return series
pub fn calculate_quarterly_returns(return_series: &ReturnSeries) -> Result<Vec<PeriodicReturn>> {
    if return_series.dates.len() != return_series.returns.len() {
        return Err(anyhow!("Return series has inconsistent lengths"));
    }
    
    if return_series.dates.is_empty() {
        return Ok(Vec::new());
    }
    
    // Group returns by quarter
    let mut quarterly_returns = HashMap::new();
    
    for i in 0..return_series.dates.len() {
        let date = return_series.dates[i];
        let return_value = return_series.returns[i];
        
        let year = date.year();
        let quarter = (date.month() - 1) / 3 + 1;
        let key = (year, quarter);
        
        quarterly_returns.entry(key)
            .or_insert_with(Vec::new)
            .push((date, return_value));
    }
    
    // Calculate return for each quarter
    let mut results = Vec::new();
    
    for ((year, quarter), values) in quarterly_returns {
        if values.is_empty() {
            continue;
        }
        
        // Sort by date
        let mut sorted_values = values;
        sorted_values.sort_by(|a, b| a.0.cmp(&b.0));
        
        // Get start and end dates
        let start_date = sorted_values.first().unwrap().0;
        let end_date = sorted_values.last().unwrap().0;
        
        // Calculate cumulative return for the quarter
        let cumulative_return = sorted_values.iter()
            .fold(Decimal::ONE, |acc, (_, r)| acc * (Decimal::ONE + *r)) - Decimal::ONE;
        
        // Create quarter label
        let label = format!("Q{} {}", quarter, year);
        
        results.push(PeriodicReturn {
            period: Period::Quarterly,
            label,
            start_date,
            end_date,
            return_value: cumulative_return,
        });
    }
    
    // Sort results by date
    results.sort_by(|a, b| a.start_date.cmp(&b.start_date));
    
    Ok(results)
}

/// Calculate annual returns from a return series
pub fn calculate_annual_returns(return_series: &ReturnSeries) -> Result<Vec<PeriodicReturn>> {
    if return_series.dates.len() != return_series.returns.len() {
        return Err(anyhow!("Return series has inconsistent lengths"));
    }
    
    if return_series.dates.is_empty() {
        return Ok(Vec::new());
    }
    
    // Group returns by year
    let mut annual_returns = HashMap::new();
    
    for i in 0..return_series.dates.len() {
        let date = return_series.dates[i];
        let return_value = return_series.returns[i];
        
        let year = date.year();
        
        annual_returns.entry(year)
            .or_insert_with(Vec::new)
            .push((date, return_value));
    }
    
    // Calculate return for each year
    let mut results = Vec::new();
    
    for (year, values) in annual_returns {
        if values.is_empty() {
            continue;
        }
        
        // Sort by date
        let mut sorted_values = values;
        sorted_values.sort_by(|a, b| a.0.cmp(&b.0));
        
        // Get start and end dates
        let start_date = sorted_values.first().unwrap().0;
        let end_date = sorted_values.last().unwrap().0;
        
        // Calculate cumulative return for the year
        let cumulative_return = sorted_values.iter()
            .fold(Decimal::ONE, |acc, (_, r)| acc * (Decimal::ONE + *r)) - Decimal::ONE;
        
        // Create year label
        let label = year.to_string();
        
        results.push(PeriodicReturn {
            period: Period::Annual,
            label,
            start_date,
            end_date,
            return_value: cumulative_return,
        });
    }
    
    // Sort results by date
    results.sort_by(|a, b| a.start_date.cmp(&b.start_date));
    
    Ok(results)
}

/// Calculate year-to-date (YTD) return
pub fn calculate_ytd_return(return_series: &ReturnSeries, as_of_date: Option<NaiveDate>) -> Result<Option<PeriodicReturn>> {
    if return_series.dates.len() != return_series.returns.len() {
        return Err(anyhow!("Return series has inconsistent lengths"));
    }
    
    if return_series.dates.is_empty() {
        return Ok(None);
    }
    
    // Determine the as-of date (default to the last date in the series)
    let as_of = as_of_date.unwrap_or_else(|| *return_series.dates.last().unwrap());
    
    // Get the year of the as-of date
    let year = as_of.year();
    
    // Filter returns for the current year up to the as-of date
    let mut ytd_values = Vec::new();
    
    for i in 0..return_series.dates.len() {
        let date = return_series.dates[i];
        
        if date.year() == year && date <= as_of {
            ytd_values.push((date, return_series.returns[i]));
        }
    }
    
    if ytd_values.is_empty() {
        return Ok(None);
    }
    
    // Sort by date
    ytd_values.sort_by(|a, b| a.0.cmp(&b.0));
    
    // Get start and end dates
    let start_date = ytd_values.first().unwrap().0;
    let end_date = ytd_values.last().unwrap().0;
    
    // Calculate cumulative return for YTD
    let cumulative_return = ytd_values.iter()
        .fold(Decimal::ONE, |acc, (_, r)| acc * (Decimal::ONE + *r)) - Decimal::ONE;
    
    // Create YTD label
    let label = format!("YTD {}", year);
    
    Ok(Some(PeriodicReturn {
        period: Period::YTD,
        label,
        start_date,
        end_date,
        return_value: cumulative_return,
    }))
}

/// Calculate since inception return
pub fn calculate_since_inception_return(return_series: &ReturnSeries) -> Result<Option<PeriodicReturn>> {
    if return_series.dates.len() != return_series.returns.len() {
        return Err(anyhow!("Return series has inconsistent lengths"));
    }
    
    if return_series.dates.is_empty() {
        return Ok(None);
    }
    
    // Sort by date
    let mut sorted_dates = return_series.dates.clone();
    let mut sorted_returns = return_series.returns.clone();
    
    // Sort both arrays based on dates
    let mut date_return_pairs: Vec<_> = sorted_dates.iter().cloned().zip(sorted_returns.iter().cloned()).collect();
    date_return_pairs.sort_by(|a, b| a.0.cmp(&b.0));
    
    // Unzip the sorted pairs
    let (sorted_dates, sorted_returns): (Vec<_>, Vec<_>) = date_return_pairs.into_iter().unzip();
    
    // Get start and end dates
    let start_date = sorted_dates.first().unwrap();
    let end_date = sorted_dates.last().unwrap();
    
    // Calculate cumulative return since inception
    let cumulative_return = sorted_returns.iter()
        .fold(Decimal::ONE, |acc, r| acc * (Decimal::ONE + *r)) - Decimal::ONE;
    
    // Create since inception label
    let label = format!("Since Inception ({}-{})", 
        start_date.format("%b %Y"),
        end_date.format("%b %Y"));
    
    Ok(Some(PeriodicReturn {
        period: Period::SinceInception,
        label,
        start_date: *start_date,
        end_date: *end_date,
        return_value: cumulative_return,
    }))
}

/// Calculate all periodic returns
pub fn calculate_all_periodic_returns(return_series: &ReturnSeries) -> Result<HashMap<Period, Vec<PeriodicReturn>>> {
    let mut results = HashMap::new();
    
    // Calculate monthly returns
    let monthly = calculate_monthly_returns(return_series)?;
    results.insert(Period::Monthly, monthly);
    
    // Calculate quarterly returns
    let quarterly = calculate_quarterly_returns(return_series)?;
    results.insert(Period::Quarterly, quarterly);
    
    // Calculate annual returns
    let annual = calculate_annual_returns(return_series)?;
    results.insert(Period::Annual, annual);
    
    // Calculate YTD return
    if let Some(ytd) = calculate_ytd_return(return_series, None)? {
        results.insert(Period::YTD, vec![ytd]);
    }
    
    // Calculate since inception return
    if let Some(since_inception) = calculate_since_inception_return(return_series)? {
        results.insert(Period::SinceInception, vec![since_inception]);
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    
    #[test]
    fn test_periodic_returns() {
        // Create test data spanning multiple periods
        let dates = vec![
            // 2022
            NaiveDate::from_ymd_opt(2022, 12, 31).unwrap(),
            // 2023 Q1
            NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
            NaiveDate::from_ymd_opt(2023, 2, 28).unwrap(),
            NaiveDate::from_ymd_opt(2023, 3, 31).unwrap(),
            // 2023 Q2
            NaiveDate::from_ymd_opt(2023, 4, 30).unwrap(),
            NaiveDate::from_ymd_opt(2023, 5, 31).unwrap(),
            NaiveDate::from_ymd_opt(2023, 6, 30).unwrap(),
        ];
        
        let returns = vec![
            dec!(0.01),   // Dec 2022: 1%
            dec!(0.02),   // Jan 2023: 2%
            dec!(-0.01),  // Feb 2023: -1%
            dec!(0.03),   // Mar 2023: 3%
            dec!(0.01),   // Apr 2023: 1%
            dec!(-0.02),  // May 2023: -2%
            dec!(0.04),   // Jun 2023: 4%
        ];
        
        let return_series = ReturnSeries {
            dates,
            returns,
        };
        
        // Test monthly returns
        let monthly = calculate_monthly_returns(&return_series).unwrap();
        assert_eq!(monthly.len(), 7);
        assert_eq!(monthly[1].period, Period::Monthly);
        assert_eq!(monthly[1].label, "Jan 2023");
        assert_eq!(monthly[1].return_value, dec!(0.02));
        
        // Test quarterly returns
        let quarterly = calculate_quarterly_returns(&return_series).unwrap();
        assert_eq!(quarterly.len(), 3); // Q4 2022, Q1 2023, Q2 2023
        
        // Q1 2023 should be approximately (1+0.02)*(1-0.01)*(1+0.03)-1 = 0.0401 or 4.01%
        let q1_2023 = quarterly.iter().find(|q| q.label == "Q1 2023").unwrap();
        assert!((q1_2023.return_value - dec!(0.0401)).abs() < dec!(0.0001));
        
        // Test annual returns
        let annual = calculate_annual_returns(&return_series).unwrap();
        assert_eq!(annual.len(), 2); // 2022, 2023
        
        // 2023 YTD (through June) should be approximately 
        // (1+0.02)*(1-0.01)*(1+0.03)*(1+0.01)*(1-0.02)*(1+0.04)-1 = 0.0697 or 6.97%
        let ytd = calculate_ytd_return(&return_series, None).unwrap().unwrap();
        assert!((ytd.return_value - dec!(0.0697)).abs() < dec!(0.001));
        
        // Test since inception
        let since_inception = calculate_since_inception_return(&return_series).unwrap().unwrap();
        assert_eq!(since_inception.period, Period::SinceInception);
        
        // Since inception should be approximately 
        // (1+0.01)*(1+0.02)*(1-0.01)*(1+0.03)*(1+0.01)*(1-0.02)*(1+0.04)-1 = 0.0804 or 8.04%
        assert!((since_inception.return_value - dec!(0.0804)).abs() < dec!(0.001));
        
        // Test all periodic returns
        let all_returns = calculate_all_periodic_returns(&return_series).unwrap();
        assert_eq!(all_returns.len(), 5); // Monthly, Quarterly, Annual, YTD, Since Inception
    }
} 