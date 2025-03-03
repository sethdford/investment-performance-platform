use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use aws_sdk_dynamodb::model::AttributeValue;

/// Represents pagination parameters for repository queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    /// Maximum number of items to return
    pub limit: Option<i32>,
    
    /// Exclusive start key for pagination
    pub start_key: Option<HashMap<String, AttributeValue>>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: None,
            start_key: None,
        }
    }
}

impl PaginationParams {
    /// Create new pagination parameters with a limit
    pub fn with_limit(limit: i32) -> Self {
        Self {
            limit: Some(limit),
            start_key: None,
        }
    }
    
    /// Create new pagination parameters with a start key
    pub fn with_start_key(start_key: HashMap<String, AttributeValue>) -> Self {
        Self {
            limit: None,
            start_key: Some(start_key),
        }
    }
    
    /// Create new pagination parameters with both limit and start key
    pub fn with_limit_and_start_key(
        limit: i32,
        start_key: HashMap<String, AttributeValue>,
    ) -> Self {
        Self {
            limit: Some(limit),
            start_key: Some(start_key),
        }
    }
}

/// Represents a paginated result from a repository query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    /// The items returned by the query
    pub items: Vec<T>,
    
    /// The last evaluated key for pagination
    pub last_evaluated_key: Option<HashMap<String, AttributeValue>>,
    
    /// Whether there are more items to fetch
    pub has_more: bool,
}

impl<T> PaginatedResult<T> {
    /// Create a new paginated result
    pub fn new(
        items: Vec<T>,
        last_evaluated_key: Option<HashMap<String, AttributeValue>>,
    ) -> Self {
        let has_more = last_evaluated_key.is_some();
        
        Self {
            items,
            last_evaluated_key,
            has_more,
        }
    }
    
    /// Create an empty paginated result
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            last_evaluated_key: None,
            has_more: false,
        }
    }
    
    /// Get the number of items in the result
    pub fn len(&self) -> usize {
        self.items.len()
    }
    
    /// Check if the result is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    
    /// Map the items to a different type
    pub fn map<U, F>(self, f: F) -> PaginatedResult<U>
    where
        F: FnMut(T) -> U,
    {
        PaginatedResult {
            items: self.items.into_iter().map(f).collect(),
            last_evaluated_key: self.last_evaluated_key,
            has_more: self.has_more,
        }
    }
    
    /// Get pagination parameters for the next page
    pub fn next_page_params(&self, limit: Option<i32>) -> Option<PaginationParams> {
        self.last_evaluated_key.clone().map(|key| PaginationParams {
            limit,
            start_key: Some(key),
        })
    }
}

/// Helper trait for applying pagination parameters to DynamoDB queries
pub trait ApplyPagination {
    /// Apply pagination parameters to self
    fn apply_pagination(self, params: &PaginationParams) -> Self;
}

impl ApplyPagination for aws_sdk_dynamodb::operation::query::builders::QueryFluentBuilder {
    fn apply_pagination(self, params: &PaginationParams) -> Self {
        let mut builder = self;
        
        if let Some(limit) = params.limit {
            builder = builder.limit(limit);
        }
        
        if let Some(start_key) = &params.start_key {
            builder = builder.set_exclusive_start_key(Some(start_key.clone()));
        }
        
        builder
    }
}

impl ApplyPagination for aws_sdk_dynamodb::operation::scan::builders::ScanFluentBuilder {
    fn apply_pagination(self, params: &PaginationParams) -> Self {
        let mut builder = self;
        
        if let Some(limit) = params.limit {
            builder = builder.limit(limit);
        }
        
        if let Some(start_key) = &params.start_key {
            builder = builder.set_exclusive_start_key(Some(start_key.clone()));
        }
        
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pagination_params_default() {
        let params = PaginationParams::default();
        assert_eq!(params.limit, None);
        assert_eq!(params.start_key, None);
    }
    
    #[test]
    fn test_pagination_params_with_limit() {
        let params = PaginationParams::with_limit(10);
        assert_eq!(params.limit, Some(10));
        assert_eq!(params.start_key, None);
    }
    
    #[test]
    fn test_pagination_params_with_start_key() {
        let mut start_key = HashMap::new();
        start_key.insert(
            "id".to_string(),
            AttributeValue::S("123".to_string()),
        );
        
        let params = PaginationParams::with_start_key(start_key.clone());
        assert_eq!(params.limit, None);
        assert_eq!(params.start_key, Some(start_key));
    }
    
    #[test]
    fn test_pagination_params_with_limit_and_start_key() {
        let mut start_key = HashMap::new();
        start_key.insert(
            "id".to_string(),
            AttributeValue::S("123".to_string()),
        );
        
        let params = PaginationParams::with_limit_and_start_key(10, start_key.clone());
        assert_eq!(params.limit, Some(10));
        assert_eq!(params.start_key, Some(start_key));
    }
    
    #[test]
    fn test_paginated_result_new() {
        let items = vec![1, 2, 3];
        let mut last_evaluated_key = HashMap::new();
        last_evaluated_key.insert(
            "id".to_string(),
            AttributeValue::S("123".to_string()),
        );
        
        let result = PaginatedResult::new(items.clone(), Some(last_evaluated_key.clone()));
        assert_eq!(result.items, items);
        assert_eq!(result.last_evaluated_key, Some(last_evaluated_key));
        assert!(result.has_more);
    }
    
    #[test]
    fn test_paginated_result_empty() {
        let result = PaginatedResult::<i32>::empty();
        assert!(result.items.is_empty());
        assert_eq!(result.last_evaluated_key, None);
        assert!(!result.has_more);
    }
    
    #[test]
    fn test_paginated_result_len() {
        let items = vec![1, 2, 3];
        let result = PaginatedResult::new(items, None);
        assert_eq!(result.len(), 3);
    }
    
    #[test]
    fn test_paginated_result_is_empty() {
        let empty_result = PaginatedResult::<i32>::empty();
        assert!(empty_result.is_empty());
        
        let non_empty_result = PaginatedResult::new(vec![1, 2, 3], None);
        assert!(!non_empty_result.is_empty());
    }
    
    #[test]
    fn test_paginated_result_map() {
        let items = vec![1, 2, 3];
        let mut last_evaluated_key = HashMap::new();
        last_evaluated_key.insert(
            "id".to_string(),
            AttributeValue::S("123".to_string()),
        );
        
        let result = PaginatedResult::new(items, Some(last_evaluated_key.clone()));
        let mapped_result = result.map(|i| i.to_string());
        
        assert_eq!(mapped_result.items, vec!["1", "2", "3"]);
        assert_eq!(mapped_result.last_evaluated_key, Some(last_evaluated_key));
        assert!(mapped_result.has_more);
    }
    
    #[test]
    fn test_paginated_result_next_page_params() {
        let items = vec![1, 2, 3];
        let mut last_evaluated_key = HashMap::new();
        last_evaluated_key.insert(
            "id".to_string(),
            AttributeValue::S("123".to_string()),
        );
        
        let result = PaginatedResult::new(items, Some(last_evaluated_key.clone()));
        let next_params = result.next_page_params(Some(10)).unwrap();
        
        assert_eq!(next_params.limit, Some(10));
        assert_eq!(next_params.start_key, Some(last_evaluated_key));
        
        let result_without_key = PaginatedResult::new(items, None);
        assert!(result_without_key.next_page_params(Some(10)).is_none());
    }
} 