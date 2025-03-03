#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_pagination() {
        // Create mock repository
        let mut mock_repo = MockRepository::new();
        
        // Set up expectations for first page
        let portfolio1 = Portfolio {
            id: "portfolio-1".to_string(),
            name: "Portfolio 1".to_string(),
            client_id: "client-1".to_string(),
            inception_date: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            benchmark_id: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            status: crate::models::Status::Active,
            metadata: HashMap::new(),
        };
        
        let portfolio2 = Portfolio {
            id: "portfolio-2".to_string(),
            name: "Portfolio 2".to_string(),
            client_id: "client-1".to_string(),
            inception_date: chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            benchmark_id: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            status: crate::models::Status::Active,
            metadata: HashMap::new(),
        };
        
        // First page
        mock_repo.expect_list_portfolios()
            .with(eq(Some("client-1")), eq(Some(PaginationOptions {
                limit: Some(1),
                next_token: None,
            })))
            .times(1)
            .returning(move |_, _| {
                Ok(PaginatedResult {
                    items: vec![portfolio1.clone()],
                    next_token: Some("next-page-token".to_string()),
                })
            });
        
        // Second page
        mock_repo.expect_list_portfolios()
            .with(eq(Some("client-1")), eq(Some(PaginationOptions {
                limit: Some(1),
                next_token: Some("next-page-token".to_string()),
            })))
            .times(1)
            .returning(move |_, _| {
                Ok(PaginatedResult {
                    items: vec![portfolio2.clone()],
                    next_token: None,
                })
            });
        
        // Create repository
        let repo = mock_repo;
        
        // Test first page
        let first_page = repo.list_portfolios(
            Some("client-1"),
            Some(PaginationOptions {
                limit: Some(1),
                next_token: None,
            }),
        ).await.unwrap();
        
        assert_eq!(first_page.items.len(), 1);
        assert_eq!(first_page.items[0].id, "portfolio-1");
        assert_eq!(first_page.next_token, Some("next-page-token".to_string()));
        
        // Test second page
        let second_page = repo.list_portfolios(
            Some("client-1"),
            Some(PaginationOptions {
                limit: Some(1),
                next_token: first_page.next_token,
            }),
        ).await.unwrap();
        
        assert_eq!(second_page.items.len(), 1);
        assert_eq!(second_page.items[0].id, "portfolio-2");
        assert_eq!(second_page.next_token, None);
    }
} 