use crate::calculations::twr;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use chrono::{NaiveDate, Utc};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_twr_calculation() {
        // TODO: Implement time-weighted return tests
        assert!(true);
    }
} 