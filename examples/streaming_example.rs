use std::collections::HashMap;

use anyhow::Result;
use chrono::Utc;
use investment_management::performance_calculator::calculations::streaming::{
    StreamingProcessor, StreamingConfig, StreamingEvent, EventHandler
};
use investment_management::financial_advisor::streaming_handler::FinancialAdvisorEventHandler;
use serde_json::{json, Value};
use std::sync::Arc;

struct SimpleEventHandler;

#[async_trait::async_trait]
impl EventHandler for SimpleEventHandler {
    async fn handle_event(&mut self, event: StreamingEvent) -> Result<()> {
        println!("Handling event: {:?}", event.event_type);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting streaming example...");
    
    // Create a simple event handler
    let handler = SimpleEventHandler;
    
    // Create a streaming processor
    let streaming_config = StreamingConfig::default();
    let _streaming_processor = StreamingProcessor::new(
        streaming_config,
        Box::new(handler) as Box<dyn EventHandler>,
    );
    
    // Create example events
    let transaction_event = create_event(
        "transaction",
        "tx123",
        serde_json::from_value::<HashMap<String, Value>>(json!({
            "transaction_type": "buy",
            "security_id": "AAPL",
            "quantity": 10,
            "price": 150.0,
            "transaction_date": Utc::now().to_rfc3339(),
        })).unwrap(),
    );
    
    let price_event = create_event(
        "price_update",
        "price123",
        serde_json::from_value::<HashMap<String, Value>>(json!({
            "price": 155.0,
            "currency": "USD",
            "timestamp": Utc::now().to_rfc3339(),
        })).unwrap(),
    );
    
    let market_event = create_event(
        "market_event",
        "market123",
        serde_json::from_value::<HashMap<String, Value>>(json!({
            "event": "market_volatility",
            "vix": 25.0,
            "timestamp": Utc::now().to_rfc3339(),
        })).unwrap(),
    );
    
    // In a real implementation, we would submit these events to the processor
    println!("Created events:");
    println!("  Transaction: {:?}", transaction_event.id);
    println!("  Price update: {:?}", price_event.id);
    println!("  Market: {:?}", market_event.id);
    
    println!("Streaming example completed successfully");
    Ok(())
}

fn create_event(event_type: &str, id: &str, payload: HashMap<String, Value>) -> StreamingEvent {
    StreamingEvent {
        id: id.to_string(),
        timestamp: Utc::now(),
        event_type: event_type.to_string(),
        source: "example_source".to_string(),
        entity_id: "example_entity".to_string(),
        payload,
    }
} 