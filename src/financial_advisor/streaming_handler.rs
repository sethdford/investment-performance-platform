use crate::financial_advisor::{FinancialAdvisorService, FinancialAdvisorRecommendation};
use crate::portfolio::rebalancing::{Portfolio, CashFlow, CashFlowType};
// TODO: Uncomment when performance_calculator module is available
use crate::performance_calculator::calculations::streaming::{EventHandler, StreamingEvent};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, warn, error, debug};
use crate::financial_advisor::FinancialAdvisorConfig;
use serde_json::Value;
use tokio::sync::mpsc;
use std::collections::HashMap;

/// Financial advisor streaming event handler
pub struct FinancialAdvisorEventHandler {
    /// Financial advisor service
    advisor_service: Arc<FinancialAdvisorService>,
    
    /// Portfolio cache (in a real implementation, this would be a database or cache service)
    portfolios: Arc<tokio::sync::RwLock<std::collections::HashMap<String, Portfolio>>>,
}

impl FinancialAdvisorEventHandler {
    /// Create a new financial advisor event handler
    pub fn new(advisor_service: Arc<FinancialAdvisorService>) -> Self {
        Self {
            advisor_service,
            portfolios: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
    
    /// Add a portfolio to the cache
    pub async fn add_portfolio(&self, portfolio: Portfolio) -> Result<()> {
        let mut portfolios = self.portfolios.write().await;
        portfolios.insert(portfolio.id.clone(), portfolio);
        Ok(())
    }
    
    /// Get a portfolio from the cache
    pub async fn get_portfolio(&self, portfolio_id: &str) -> Result<Portfolio> {
        let portfolios = self.portfolios.read().await;
        portfolios.get(portfolio_id)
            .cloned()
            .ok_or_else(|| anyhow!("Portfolio not found: {}", portfolio_id))
    }
    
    /// Process an event and generate a recommendation if needed
    pub async fn process_event(&self, event: StreamingEvent) -> Result<Option<FinancialAdvisorRecommendation>> {
        debug!("Processing event: {:?}", event);
        
        match event.event_type.as_str() {
            "transaction" => self.process_transaction_event(&event).await,
            "price_update" => self.process_price_update_event(&event).await,
            "market_data" => self.process_market_data_event(&event).await,
            _ => {
                warn!("Unknown event type: {}", event.event_type);
                Ok(None)
            }
        }
    }
    
    /// Process a transaction event
    async fn process_transaction_event(&self, event: &StreamingEvent) -> Result<Option<FinancialAdvisorRecommendation>> {
        let portfolio_id = event.entity_id.clone();
        let user_id = event.payload.get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        // Get the portfolio
        let mut portfolio = self.get_portfolio(&portfolio_id).await?;
        
        // Extract transaction details from the payload
        let transaction_type = event.payload.get("transaction_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing transaction_type in payload"))?;
        
        let amount = event.payload.get("amount")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("Missing amount in payload"))?;
        
        // Handle different transaction types
        match transaction_type {
            "deposit" | "withdrawal" => {
                // Create a cash flow
                let cash_flow = CashFlow {
                    amount: if transaction_type == "deposit" { amount } else { -amount },
                    date: event.timestamp.to_rfc3339(),
                    flow_type: if transaction_type == "deposit" { 
                        CashFlowType::Deposit 
                    } else { 
                        CashFlowType::Withdrawal 
                    },
                };
                
                // Update portfolio cash balance
                portfolio.cash_balance += cash_flow.amount;
                
                // Update the portfolio in the cache
                {
                    let mut portfolios = self.portfolios.write().await;
                    portfolios.insert(portfolio_id.clone(), portfolio.clone());
                }
                
                // Generate recommendations for the cash flow
                self.advisor_service.handle_cash_flow(&portfolio, &cash_flow, &user_id).await
            },
            "buy" | "sell" => {
                // Extract security details
                let security_id = event.payload.get("security_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("Missing security_id in payload"))?
                    .to_string();
                
                let price = event.payload.get("price")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| anyhow!("Missing price in payload"))?;
                
                let quantity = event.payload.get("quantity")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| anyhow!("Missing quantity in payload"))?;
                
                let is_buy = transaction_type == "buy";
                let transaction_amount = price * quantity;
                
                // Update portfolio holdings
                let mut holding_updated = false;
                
                for holding in &mut portfolio.holdings {
                    if holding.security_id == security_id {
                        if is_buy {
                            // Update cost basis and market value for buy
                            let new_cost_basis = holding.cost_basis + transaction_amount;
                            let new_market_value = holding.market_value + transaction_amount;
                            holding.cost_basis = new_cost_basis;
                            holding.market_value = new_market_value;
                        } else {
                            // Update cost basis and market value for sell
                            let cost_basis_per_share = holding.cost_basis / (holding.market_value / price);
                            let cost_basis_reduction = cost_basis_per_share * quantity;
                            holding.cost_basis -= cost_basis_reduction;
                            holding.market_value -= transaction_amount;
                        }
                        
                        holding_updated = true;
                        break;
                    }
                }
                
                // If the security wasn't found and this is a buy, add a new holding
                if !holding_updated && is_buy {
                    use std::collections::HashMap;
                    
                    portfolio.holdings.push(crate::portfolio::rebalancing::PortfolioHolding {
                        security_id,
                        market_value: transaction_amount,
                        weight: transaction_amount / portfolio.total_market_value,
                        target_weight: 0.0, // This would need to be set based on the target allocation
                        cost_basis: transaction_amount,
                        purchase_date: event.timestamp.to_rfc3339(),
                        factor_exposures: HashMap::new(),
                    });
                }
                
                // Update portfolio total market value
                portfolio.total_market_value = portfolio.holdings.iter()
                    .map(|h| h.market_value)
                    .sum::<f64>() + portfolio.cash_balance;
                
                // Update portfolio cash balance
                if is_buy {
                    portfolio.cash_balance -= transaction_amount;
                } else {
                    portfolio.cash_balance += transaction_amount;
                }
                
                // Update the portfolio in the cache
                {
                    let mut portfolios = self.portfolios.write().await;
                    portfolios.insert(portfolio_id.clone(), portfolio.clone());
                }
                
                // Check for portfolio drift after the transaction
                self.advisor_service.check_portfolio_drift(&portfolio, &user_id).await
            },
            _ => Err(anyhow!("Unsupported transaction type: {}", transaction_type)),
        }
    }
    
    /// Process a price update event
    async fn process_price_update_event(&self, event: &StreamingEvent) -> Result<Option<FinancialAdvisorRecommendation>> {
        let portfolio_id = event.entity_id.clone();
        let user_id = event.payload.get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        // Get the portfolio
        let mut portfolio = self.get_portfolio(&portfolio_id).await?;
        
        // Extract price update details from the payload
        let security_id = event.payload.get("security_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing security_id in payload"))?
            .to_string();
        
        let new_price = event.payload.get("price")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("Missing price in payload"))?;
        
        // Update portfolio holdings
        let mut holding_updated = false;
        
        for holding in &mut portfolio.holdings {
            if holding.security_id == security_id {
                // Calculate quantity based on current market value and old price
                let old_price = event.payload.get("old_price")
                    .and_then(|v| v.as_f64())
                    .unwrap_or_else(|| holding.market_value / (holding.market_value / new_price));
                
                let quantity = holding.market_value / old_price;
                
                // Update market value based on new price
                let new_market_value = quantity * new_price;
                
                // Calculate price change percentage
                let price_change_pct = (new_price - old_price) / old_price;
                
                // Update holding
                holding.market_value = new_market_value;
                
                holding_updated = true;
                
                // Log significant price changes
                if price_change_pct.abs() > 0.05 {
                    info!(
                        security_id = %security_id,
                        price_change_pct = %format!("{:.2}%", price_change_pct * 100.0),
                        "Significant price change detected"
                    );
                }
                
                break;
            }
        }
        
        // If the holding was updated, update portfolio total market value and weights
        if holding_updated {
            // Update portfolio total market value
            portfolio.total_market_value = portfolio.holdings.iter()
                .map(|h| h.market_value)
                .sum::<f64>() + portfolio.cash_balance;
            
            // Update weights
            for holding in &mut portfolio.holdings {
                holding.weight = holding.market_value / portfolio.total_market_value;
            }
            
            // Update the portfolio in the cache
            {
                let mut portfolios = self.portfolios.write().await;
                portfolios.insert(portfolio_id.clone(), portfolio.clone());
            }
            
            // Check for portfolio drift after the price update
            let drift_recommendation = self.advisor_service.check_portfolio_drift(&portfolio, &user_id).await?;
            
            // If there's no drift recommendation, check for tax loss harvesting opportunities
            if drift_recommendation.is_none() {
                return self.advisor_service.check_tax_loss_harvesting(&portfolio, &user_id).await;
            }
            
            return Ok(drift_recommendation);
        }
        
        Ok(None)
    }
    
    /// Process a market data event
    async fn process_market_data_event(&self, event: &StreamingEvent) -> Result<Option<FinancialAdvisorRecommendation>> {
        // Extract market data details from the payload
        let data_type = event.payload.get("data_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing data_type in payload"))?;
        
        match data_type {
            "volatility_index" => {
                // Extract VIX value
                let vix_value = event.payload.get("value")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| anyhow!("Missing value in payload"))?;
                
                // If VIX is high (indicating high market volatility), create a recommendation
                if vix_value > 30.0 {
                    // In a real implementation, we would get all affected portfolios
                    // For simplicity, we'll just log the event
                    info!(
                        vix_value = %vix_value,
                        "High market volatility detected"
                    );
                    
                    // We would return a recommendation here, but for now we'll return None
                    // since we don't have a specific portfolio to associate with this event
                }
            },
            "interest_rate_change" => {
                // Extract interest rate change
                let rate_change = event.payload.get("change")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| anyhow!("Missing change in payload"))?;
                
                // If interest rate change is significant, create a recommendation
                if rate_change.abs() > 0.25 {
                    // In a real implementation, we would get all affected portfolios
                    // For simplicity, we'll just log the event
                    info!(
                        rate_change = %rate_change,
                        "Significant interest rate change detected"
                    );
                    
                    // We would return a recommendation here, but for now we'll return None
                    // since we don't have a specific portfolio to associate with this event
                }
            },
            _ => {
                // Unsupported market data type
                warn!(
                    data_type = %data_type,
                    "Unsupported market data type"
                );
            }
        }
        
        Ok(None)
    }
}

#[async_trait]
impl EventHandler for FinancialAdvisorEventHandler {
    async fn handle_event(&mut self, event: StreamingEvent) -> Result<()> {
        match self.process_event(event).await {
            Ok(recommendation) => {
                if let Some(rec) = recommendation {
                    info!("Generated recommendation: {:?}", rec);
                }
                Ok(())
            },
            Err(e) => {
                error!("Error processing event: {}", e);
                Err(anyhow!("Error processing event: {}", e))
            }
        }
    }
}

impl FinancialAdvisorEventHandler {
    // Move process_batch here as a regular method
    async fn process_batch(&self, events: Vec<StreamingEvent>) -> Result<()> {
        for event in events {
            if let Err(e) = self.process_event(event).await {
                error!("Error processing event in batch: {}", e);
                // Continue processing other events even if one fails
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factor_model::FactorModelApi;
    use crate::portfolio::rebalancing::PortfolioRebalancingService;
    use crate::financial_advisor::FinancialAdvisorService;
    use chrono::Utc;
    use serde_json::json;
    
    // Create a test portfolio
    fn create_test_portfolio() -> Portfolio {
        Portfolio {
            id: "portfolio-123".to_string(),
            name: "Test Portfolio".to_string(),
            total_market_value: 100000.0,
            cash_balance: 5000.0,
            holdings: vec![
                crate::portfolio::rebalancing::PortfolioHolding {
                    security_id: "VTI".to_string(),
                    market_value: 30000.0,
                    weight: 0.3,
                    target_weight: 0.4,
                    cost_basis: 25000.0,
                    purchase_date: "2022-01-01".to_string(),
                    factor_exposures: std::collections::HashMap::new(),
                },
                crate::portfolio::rebalancing::PortfolioHolding {
                    security_id: "BND".to_string(),
                    market_value: 20000.0,
                    weight: 0.2,
                    target_weight: 0.3,
                    cost_basis: 22000.0,
                    purchase_date: "2022-01-01".to_string(),
                    factor_exposures: std::collections::HashMap::new(),
                },
            ],
        }
    }
    
    // Helper function to create a test event
    fn create_test_event(event_type: &str, entity_id: &str, payload: Value) -> StreamingEvent {
        let payload_map = payload.as_object()
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<HashMap<String, Value>>()
            })
            .unwrap_or_default();
            
        StreamingEvent {
            id: "test-event-id".to_string(),
            timestamp: chrono::Utc::now(),
            event_type: event_type.to_string(),
            source: "test".to_string(),
            entity_id: entity_id.to_string(),
            payload: payload_map,
        }
    }
    
    #[tokio::test]
    async fn test_process_transaction_event() {
        // Create dependencies
        let factor_model_api = FactorModelApi::new();
        let rebalancing_service = PortfolioRebalancingService::new(factor_model_api.clone());
        let config = FinancialAdvisorConfig::default();
        let advisor_service = Arc::new(FinancialAdvisorService::new(config, Some(rebalancing_service)).await.unwrap());
        
        // Create event handler
        let handler = FinancialAdvisorEventHandler::new(advisor_service);
        
        // Add a test portfolio
        let portfolio = create_test_portfolio();
        handler.add_portfolio(portfolio).await.unwrap();
        
        // Create a deposit event
        let deposit_event = create_test_event(
            "transaction",
            "portfolio-123",
            json!({
                "user_id": "user-123",
                "transaction_type": "deposit",
                "amount": 10000.0,
            }),
        );
        
        // Process the event
        handler.process_event(deposit_event).await.unwrap();
        
        // Verify the portfolio was updated
        let updated_portfolio = handler.get_portfolio("portfolio-123").await.unwrap();
        assert_eq!(updated_portfolio.cash_balance, 15000.0); // 5000 + 10000
        
        // Create a buy event
        let buy_event = create_test_event(
            "transaction",
            "portfolio-123",
            json!({
                "user_id": "user-123",
                "transaction_type": "buy",
                "security_id": "VTI",
                "price": 100.0,
                "quantity": 50.0,
            }),
        );
        
        // Process the event
        handler.process_event(buy_event).await.unwrap();
        
        // Verify the portfolio was updated
        let updated_portfolio = handler.get_portfolio("portfolio-123").await.unwrap();
        assert_eq!(updated_portfolio.cash_balance, 10000.0); // 15000 - 5000
        
        // Find the VTI holding
        let vti_holding = updated_portfolio.holdings.iter()
            .find(|h| h.security_id == "VTI")
            .unwrap();
        
        assert_eq!(vti_holding.market_value, 35000.0); // 30000 + 5000
        assert_eq!(vti_holding.cost_basis, 30000.0); // 25000 + 5000
    }
    
    #[tokio::test]
    async fn test_process_price_update_event() {
        // Create dependencies
        let factor_model_api = FactorModelApi::new();
        let rebalancing_service = PortfolioRebalancingService::new(factor_model_api.clone());
        let config = FinancialAdvisorConfig::default();
        let advisor_service = Arc::new(FinancialAdvisorService::new(config, Some(rebalancing_service)).await.unwrap());
        
        // Create event handler
        let handler = FinancialAdvisorEventHandler::new(advisor_service);
        
        // Add a test portfolio
        let portfolio = create_test_portfolio();
        handler.add_portfolio(portfolio).await.unwrap();
        
        // Create a price update event
        let price_update_event = create_test_event(
            "price_update",
            "portfolio-123",
            json!({
                "user_id": "user-123",
                "security_id": "VTI",
                "old_price": 100.0,
                "price": 110.0,
            }),
        );
        
        // Process the event
        handler.process_event(price_update_event).await.unwrap();
        
        // Verify the portfolio was updated
        let updated_portfolio = handler.get_portfolio("portfolio-123").await.unwrap();
        
        // Find the VTI holding
        let vti_holding = updated_portfolio.holdings.iter()
            .find(|h| h.security_id == "VTI")
            .unwrap();
        
        // Market value should be increased by 10%
        assert!(vti_holding.market_value > 30000.0);
        
        // Cost basis should remain the same
        assert_eq!(vti_holding.cost_basis, 25000.0);
    }
} 