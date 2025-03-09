use async_graphql::{Context, Object, Schema, SimpleObject, InputObject, ID, Enum};
use chrono::{DateTime, Utc, Duration};
use shared::models::{Client as DomainClient, Portfolio as DomainPortfolio, Account as DomainAccount, Security as DomainSecurity, Transaction as DomainTransaction, PerformanceMetric as DomainPerformanceMetric};
use shared::repository::{Repository, DynamoDbRepository};
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, error};
use anyhow::{Result, anyhow};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use rust_decimal::prelude::*;

pub type InvestmentSchema = Schema<Query, Mutation, async_graphql::EmptySubscription>;

// GraphQL wrapper types
#[derive(SimpleObject)]
pub struct Client {
    pub id: ID,
    pub name: String,
}

#[derive(SimpleObject)]
pub struct Portfolio {
    pub id: ID,
    pub name: String,
    pub client_id: String,
}

#[derive(SimpleObject)]
pub struct Account {
    pub id: ID,
    pub name: String,
    pub portfolio_id: String,
    pub account_type: String,
}

#[derive(SimpleObject)]
pub struct Transaction {
    pub id: ID,
    pub account_id: String,
    pub security_id: Option<String>,
    pub transaction_date: String,
    pub transaction_type: String,
    pub amount: f64,
    pub quantity: Option<f64>,
    pub price: Option<f64>,
}

#[derive(SimpleObject)]
pub struct Security {
    pub id: ID,
    pub symbol: String,
    pub name: String,
    pub security_type: String,
}

#[derive(SimpleObject)]
pub struct PerformanceMetric {
    pub id: ID,
    pub portfolio_id: Option<String>,
    pub account_id: Option<String>,
    pub metric_type: String,
    pub value: f64,
    pub start_date: String,
    pub end_date: String,
}

// Conversion functions
impl From<DomainClient> for Client {
    fn from(client: DomainClient) -> Self {
        Self {
            id: client.id.into(),
            name: client.name,
        }
    }
}

impl From<DomainPortfolio> for Portfolio {
    fn from(portfolio: DomainPortfolio) -> Self {
        Self {
            id: portfolio.id.into(),
            name: portfolio.name,
            client_id: portfolio.client_id,
        }
    }
}

impl From<DomainAccount> for Account {
    fn from(account: DomainAccount) -> Self {
        Self {
            id: account.id.into(),
            name: account.name,
            portfolio_id: account.portfolio_id,
            account_type: format!("{:?}", account.account_type),
        }
    }
}

impl From<DomainTransaction> for Transaction {
    fn from(transaction: DomainTransaction) -> Self {
        Self {
            id: transaction.id.into(),
            account_id: transaction.account_id,
            security_id: transaction.security_id,
            transaction_date: transaction.transaction_date.to_string(),
            transaction_type: format!("{:?}", transaction.transaction_type),
            amount: transaction.amount,
            quantity: transaction.quantity,
            price: transaction.price,
        }
    }
}

impl From<DomainSecurity> for Security {
    fn from(security: DomainSecurity) -> Self {
        Self {
            id: security.id.into(),
            symbol: security.symbol,
            name: security.name,
            security_type: format!("{:?}", security.security_type),
        }
    }
}

#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    // Client queries
    async fn client(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<Client>> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        let client = repository.get_client(&id).await?;
        Ok(client.map(Client::from))
    }
    
    async fn clients(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Client>> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        let clients = repository.list_clients(None).await?;
        Ok(clients.items.into_iter().map(Client::from).collect())
    }
    
    // Portfolio queries
    async fn portfolio(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<Portfolio>> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        let portfolio = repository.get_portfolio(&id).await?;
        Ok(portfolio.map(Portfolio::from))
    }
    
    async fn portfolios(&self, ctx: &Context<'_>, client_id: Option<ID>) -> async_graphql::Result<Vec<Portfolio>> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        let portfolios = repository.list_portfolios(client_id.as_ref().map(|id| id.as_str()), None).await?;
        Ok(portfolios.items.into_iter().map(Portfolio::from).collect())
    }
    
    // Account queries
    async fn account(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<Account>> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        let account = repository.get_account(&id).await?;
        Ok(account.map(Account::from))
    }
    
    async fn accounts(&self, ctx: &Context<'_>, portfolio_id: Option<ID>) -> async_graphql::Result<Vec<Account>> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        let accounts = repository.list_accounts(portfolio_id.as_ref().map(|id| id.as_str()), None).await?;
        Ok(accounts.items.into_iter().map(Account::from).collect())
    }
    
    // Security queries
    async fn security(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<Security>> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        let security = repository.get_security(&id).await?;
        Ok(security.map(Security::from))
    }
    
    async fn securities(&self, ctx: &Context<'_>, account_id: Option<ID>) -> async_graphql::Result<Vec<Security>> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        let securities = repository.list_securities(None).await?;
        Ok(securities.items.into_iter().map(Security::from).collect())
    }
    
    // Transaction queries
    async fn transaction(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<Transaction>> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        let transaction = repository.get_transaction(&id).await?;
        Ok(transaction.map(Transaction::from))
    }
    
    async fn transactions(
        &self, 
        ctx: &Context<'_>, 
        account_id: Option<ID>,
        security_id: Option<ID>
    ) -> async_graphql::Result<Vec<Transaction>> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        
        let transactions = repository.list_transactions(
            account_id.as_ref().map(|id| id.as_str()),
            None
        ).await?;
        
        Ok(transactions.items.into_iter().map(Transaction::from).collect())
    }
    
    // Performance queries
    async fn performance_metric(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<PerformanceMetric>> {
        // Mock implementation since repository doesn't support this
        Ok(Some(PerformanceMetric {
            id,
            portfolio_id: Some("portfolio-123".to_string()),
            account_id: None,
            metric_type: "TimeWeightedReturn".to_string(),
            value: 0.0543, // 5.43%
            start_date: "2023-01-01".to_string(),
            end_date: "2023-12-31".to_string(),
        }))
    }
    
    async fn performance_metrics(
        &self, 
        ctx: &Context<'_>, 
        portfolio_id: Option<ID>,
        account_id: Option<ID>,
        start_date: Option<String>,
        end_date: Option<String>
    ) -> async_graphql::Result<Vec<PerformanceMetric>> {
        // Mock implementation since repository doesn't support this
        let portfolio_id_str = portfolio_id.as_ref().map(|id| id.to_string());
        let account_id_str = account_id.as_ref().map(|id| id.to_string());
        let start_date_str = start_date.clone().unwrap_or_else(|| "2023-01-01".to_string());
        let end_date_str = end_date.clone().unwrap_or_else(|| "2023-12-31".to_string());
        
        let metrics = vec![
            PerformanceMetric {
                id: "metric-1".into(),
                portfolio_id: portfolio_id_str.clone(),
                account_id: account_id_str.clone(),
                metric_type: "TimeWeightedReturn".to_string(),
                value: 0.0543, // 5.43%
                start_date: start_date_str.clone(),
                end_date: end_date_str.clone(),
            },
            PerformanceMetric {
                id: "metric-2".into(),
                portfolio_id: portfolio_id_str,
                account_id: account_id_str,
                metric_type: "MoneyWeightedReturn".to_string(),
                value: 0.0612, // 6.12%
                start_date: start_date_str,
                end_date: end_date_str,
            }
        ];
        
        Ok(metrics)
    }
    
    // Time series performance query
    async fn performance_time_series(
        &self,
        ctx: &Context<'_>,
        portfolio_id: Option<ID>,
        account_id: Option<ID>,
        start_date: String,
        end_date: String,
        metric_type: PerformanceMetricType,
        interval: TimeSeriesInterval,
        database: Option<String>
    ) -> async_graphql::Result<Vec<PerformanceDataPoint>> {
        // Mock implementation that returns sample data
        // In a real implementation, this would use the timestream-repository
        
        let start_date_parsed = DateTime::parse_from_rfc3339(&start_date)
            .map_err(|e| async_graphql::Error::new(format!("Invalid start date format: {}", e)))?
            .with_timezone(&Utc);
            
        let end_date_parsed = DateTime::parse_from_rfc3339(&end_date)
            .map_err(|e| async_graphql::Error::new(format!("Invalid end date format: {}", e)))?
            .with_timezone(&Utc);
        
        // Generate mock data points
        let mut data_points = Vec::new();
        let mut current_date = start_date_parsed;
        let day_increment = match interval {
            TimeSeriesInterval::Day => 1,
            TimeSeriesInterval::Week => 7,
            TimeSeriesInterval::Month => 30,
            TimeSeriesInterval::Quarter => 90,
            TimeSeriesInterval::Year => 365,
        };
        
        while current_date <= end_date_parsed {
            // Generate a random value between -5.0 and 15.0
            let random_value = (current_date.timestamp() % 20) as f64 - 5.0;
            
            data_points.push(PerformanceDataPoint {
                time: current_date.to_rfc3339(),
                value: random_value,
                metric_type: match metric_type {
                    PerformanceMetricType::TimeWeightedReturn => "time_weighted_return".to_string(),
                    PerformanceMetricType::MoneyWeightedReturn => "money_weighted_return".to_string(),
                },
            });
            
            current_date = current_date + Duration::days(day_increment);
        }
        
        Ok(data_points)
    }
}

#[derive(Default)]
pub struct Mutation;

#[Object]
impl Mutation {
    // Client mutations
    async fn create_client(&self, ctx: &Context<'_>, input: ClientInput) -> async_graphql::Result<Client> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        
        let client = DomainClient {
            id: Uuid::new_v4().to_string(),
            name: input.name,
            client_type: shared::models::ClientType::Individual,
            contact: shared::models::ContactInfo {
                email: None,
                phone: None,
                address: None,
            },
            classification: "Default".to_string(),
            status: shared::models::Status::Active,
            metadata: Default::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        repository.put_client(&client).await?;
        
        Ok(client.into())
    }
    
    async fn update_client(&self, ctx: &Context<'_>, id: ID, input: ClientInput) -> async_graphql::Result<Client> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        
        let mut client = repository.get_client(&id)
            .await?
            .ok_or_else(|| async_graphql::Error::new(format!("Client not found: {:?}", id)))?;
        
        client.name = input.name;
        
        repository.put_client(&client).await?;
        
        Ok(client.into())
    }
    
    async fn delete_client(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<ID> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        
        repository.delete_client(&id).await?;
        
        Ok(id)
    }
    
    // Portfolio mutations
    async fn create_portfolio(&self, ctx: &Context<'_>, input: PortfolioInput) -> async_graphql::Result<Portfolio> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        
        // Verify client exists
        repository.get_client(&input.client_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new(format!("Client not found: {}", input.client_id)))?;
        
        let portfolio = DomainPortfolio {
            id: Uuid::new_v4().to_string(),
            name: input.name,
            client_id: input.client_id,
            inception_date: Utc::now().date_naive(),
            benchmark_id: None,
            status: shared::models::Status::Active,
            metadata: Default::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            transactions: Vec::new(),
            holdings: Vec::new(),
        };
        
        repository.put_portfolio(&portfolio).await?;
        
        Ok(portfolio.into())
    }
    
    async fn update_portfolio(&self, ctx: &Context<'_>, id: ID, input: PortfolioInput) -> async_graphql::Result<Portfolio> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        
        let mut portfolio = repository.get_portfolio(&id)
            .await?
            .ok_or_else(|| async_graphql::Error::new(format!("Portfolio not found: {:?}", id)))?;
        
        // Verify client exists
        repository.get_client(&input.client_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new(format!("Client not found: {}", input.client_id)))?;
        
        portfolio.name = input.name;
        portfolio.client_id = input.client_id;
        
        repository.put_portfolio(&portfolio).await?;
        
        Ok(portfolio.into())
    }
    
    async fn delete_portfolio(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<ID> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        
        repository.delete_portfolio(&id).await?;
        
        Ok(id)
    }
    
    // Account mutations
    async fn create_account(&self, ctx: &Context<'_>, input: AccountInput) -> async_graphql::Result<Account> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        
        // Verify portfolio exists
        repository.get_portfolio(&input.portfolio_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new(format!("Portfolio not found: {}", input.portfolio_id)))?;
        
        let account = DomainAccount {
            id: Uuid::new_v4().to_string(),
            name: input.name,
            portfolio_id: input.portfolio_id,
            account_type: shared::models::AccountType::Other(input.account_type),
            account_number: "".to_string(),
            tax_status: shared::models::TaxStatus::Taxable,
            inception_date: Utc::now().date_naive(),
            status: shared::models::Status::Active,
            metadata: Default::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        repository.put_account(&account).await?;
        
        Ok(account.into())
    }
    
    async fn update_account(&self, ctx: &Context<'_>, id: ID, input: AccountInput) -> async_graphql::Result<Account> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        
        let mut account = repository.get_account(&id)
            .await?
            .ok_or_else(|| async_graphql::Error::new(format!("Account not found: {:?}", id)))?;
        
        // Verify portfolio exists
        repository.get_portfolio(&input.portfolio_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new(format!("Portfolio not found: {}", input.portfolio_id)))?;
        
        account.name = input.name;
        account.portfolio_id = input.portfolio_id;
        account.account_type = shared::models::AccountType::Other(input.account_type);
        
        repository.put_account(&account).await?;
        
        Ok(account.into())
    }
    
    async fn delete_account(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<ID> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        
        repository.delete_account(&id).await?;
        
        Ok(id)
    }
    
    // Transaction mutations
    async fn create_transaction(&self, ctx: &Context<'_>, input: TransactionInput) -> async_graphql::Result<Transaction> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        
        // Verify account exists
        repository.get_account(&input.account_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new(format!("Account not found: {}", input.account_id)))?;
        
        // Verify security exists
        repository.get_security(&input.security_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new(format!("Security not found: {}", input.security_id)))?;
        
        let transaction = DomainTransaction {
            id: Uuid::new_v4().to_string(),
            account_id: input.account_id,
            security_id: Some(input.security_id),
            transaction_date: Utc::now().date_naive(),
            settlement_date: None,
            transaction_type: shared::models::TransactionType::Other(input.transaction_type),
            amount: input.amount,
            quantity: Some(input.quantity),
            price: Some(input.price),
            fees: None,
            currency: "USD".to_string(),
            metadata: Default::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        repository.put_transaction(&transaction).await?;
        
        Ok(transaction.into())
    }
    
    async fn delete_transaction(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<ID> {
        let repository = ctx.data::<Arc<DynamoDbRepository>>()?;
        
        repository.delete_transaction(&id).await?;
        
        Ok(id)
    }
}

// Input types
#[derive(InputObject)]
pub struct ClientInput {
    pub name: String,
}

#[derive(InputObject)]
pub struct PortfolioInput {
    pub name: String,
    pub client_id: String,
}

#[derive(InputObject)]
pub struct AccountInput {
    pub name: String,
    pub portfolio_id: String,
    pub account_type: String,
}

#[derive(InputObject)]
pub struct TransactionInput {
    pub account_id: String,
    pub security_id: String,
    pub transaction_type: String,
    pub amount: f64,
    pub quantity: f64,
    pub price: f64,
}

// Time series types
#[derive(SimpleObject)]
pub struct TimeSeriesDataPoint {
    #[graphql(skip)]
    pub time: DateTime<Utc>,
    pub value: f64,
    pub dimension_id: String,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum PerformanceMetricType {
    TimeWeightedReturn,
    MoneyWeightedReturn,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum TimeSeriesInterval {
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

#[derive(SimpleObject)]
pub struct PerformanceDataPoint {
    pub time: String,
    pub value: f64,
    pub metric_type: String,
} 