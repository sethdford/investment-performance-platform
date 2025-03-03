//! Streaming Processing Module
//!
//! This module provides real-time streaming capabilities for processing
//! performance data as it arrives, enabling immediate calculation updates
//! and real-time analytics.

use crate::calculations::{
    audit::AuditTrail,
    distributed_cache::Cache,
    currency::CurrencyConverter,
};
use anyhow::{Result, Context};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{info, warn, error, debug};

/// Represents a streaming event that can be processed by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingEvent {
    /// Unique identifier for the event
    pub id: String,
    
    /// Timestamp when the event occurred
    pub timestamp: DateTime<Utc>,
    
    /// Type of event (e.g., "transaction", "price_update", "valuation")
    pub event_type: String,
    
    /// Source of the event (e.g., "market_data", "accounting_system")
    pub source: String,
    
    /// Entity identifier (e.g., portfolio ID, security ID)
    pub entity_id: String,
    
    /// Event payload containing the actual data
    pub payload: serde_json::Value,
}

/// Configuration for the streaming processor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Maximum number of events to process concurrently
    pub max_concurrent_events: usize,
    
    /// Size of the event buffer
    pub buffer_size: usize,
    
    /// Whether to enable batch processing
    pub enable_batch_processing: bool,
    
    /// Maximum batch size when batch processing is enabled
    pub max_batch_size: usize,
    
    /// Maximum wait time in milliseconds for batch processing
    pub batch_wait_ms: u64,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_concurrent_events: 100,
            buffer_size: 1000,
            enable_batch_processing: true,
            max_batch_size: 50,
            batch_wait_ms: 100,
        }
    }
}

/// Trait defining the interface for event handlers
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Process a single event
    async fn process_event(&self, event: StreamingEvent) -> Result<()>;
    
    /// Process a batch of events (default implementation processes one by one)
    async fn process_batch(&self, events: Vec<StreamingEvent>) -> Result<()> {
        for event in events {
            self.process_event(event).await?;
        }
        Ok(())
    }
}

/// Streaming processor that handles incoming events
pub struct StreamingProcessor {
    /// Configuration for the streaming processor
    config: StreamingConfig,
    
    /// Channel sender for submitting events
    event_sender: mpsc::Sender<StreamingEvent>,
    
    /// Registered event handlers
    handlers: Arc<Mutex<Vec<Arc<dyn EventHandler>>>>,
    
    /// Audit trail for logging processing activities
    audit_trail: Option<Arc<dyn AuditTrail>>,
    
    /// Cache for storing intermediate results
    cache: Option<Arc<dyn Cache>>,
}

impl StreamingProcessor {
    /// Create a new streaming processor with the given configuration
    pub async fn new(
        config: StreamingConfig,
        audit_trail: Option<Arc<dyn AuditTrail>>,
        cache: Option<Arc<dyn Cache>>,
    ) -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::channel(config.buffer_size);
        
        let handlers = Arc::new(Mutex::new(Vec::new()));
        
        let processor = Self {
            config,
            event_sender,
            handlers: handlers.clone(),
            audit_trail,
            cache,
        };
        
        // Start the event processing loop
        let config_clone = processor.config.clone();
        let handlers_clone = handlers.clone();
        let audit_trail_clone = audit_trail.clone();
        
        tokio::spawn(async move {
            if config_clone.enable_batch_processing {
                Self::batch_processing_loop(
                    event_receiver,
                    handlers_clone,
                    config_clone.max_batch_size,
                    config_clone.batch_wait_ms,
                    audit_trail_clone,
                ).await;
            } else {
                Self::individual_processing_loop(
                    event_receiver,
                    handlers_clone,
                    config_clone.max_concurrent_events,
                    audit_trail_clone,
                ).await;
            }
        });
        
        Ok(processor)
    }
    
    /// Register an event handler
    pub async fn register_handler(&self, handler: Arc<dyn EventHandler>) {
        let mut handlers = self.handlers.lock().await;
        handlers.push(handler);
    }
    
    /// Submit an event for processing
    pub async fn submit_event(&self, event: StreamingEvent) -> Result<()> {
        self.event_sender.send(event.clone()).await
            .context("Failed to submit event to processing queue")?;
        
        if let Some(audit) = &self.audit_trail {
            audit.record_event_submission(&event).await?;
        }
        
        Ok(())
    }
    
    /// Submit multiple events for processing
    pub async fn submit_events(&self, events: Vec<StreamingEvent>) -> Result<()> {
        for event in events {
            self.submit_event(event).await?;
        }
        Ok(())
    }
    
    /// Process events individually
    async fn individual_processing_loop(
        mut event_receiver: mpsc::Receiver<StreamingEvent>,
        handlers: Arc<Mutex<Vec<Arc<dyn EventHandler>>>>,
        max_concurrent: usize,
        audit_trail: Option<Arc<dyn AuditTrail>>,
    ) {
        // Semaphore to limit concurrent processing
        let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));
        
        while let Some(event) = event_receiver.recv().await {
            let handlers_clone = handlers.clone();
            let semaphore_clone = semaphore.clone();
            let audit_trail_clone = audit_trail.clone();
            let event_id = event.id.clone();
            
            tokio::spawn(async move {
                // Acquire semaphore permit
                let _permit = match semaphore_clone.acquire().await {
                    Ok(permit) => permit,
                    Err(e) => {
                        error!("Failed to acquire semaphore: {}", e);
                        return;
                    }
                };
                
                let handlers = handlers_clone.lock().await;
                
                for handler in handlers.iter() {
                    match handler.process_event(event.clone()).await {
                        Ok(_) => {
                            if let Some(audit) = &audit_trail_clone {
                                let _ = audit.record_event_processing(&event, true).await;
                            }
                        }
                        Err(e) => {
                            error!("Error processing event {}: {}", event_id, e);
                            if let Some(audit) = &audit_trail_clone {
                                let _ = audit.record_event_processing(&event, false).await;
                            }
                        }
                    }
                }
            });
        }
    }
    
    /// Process events in batches
    async fn batch_processing_loop(
        mut event_receiver: mpsc::Receiver<StreamingEvent>,
        handlers: Arc<Mutex<Vec<Arc<dyn EventHandler>>>>,
        max_batch_size: usize,
        batch_wait_ms: u64,
        audit_trail: Option<Arc<dyn AuditTrail>>,
    ) {
        let mut batch = Vec::with_capacity(max_batch_size);
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(batch_wait_ms));
        
        loop {
            tokio::select! {
                // Try to receive an event
                event = event_receiver.recv() => {
                    match event {
                        Some(e) => {
                            batch.push(e);
                            
                            // If batch is full, process it
                            if batch.len() >= max_batch_size {
                                let handlers_clone = handlers.clone();
                                let batch_to_process = std::mem::replace(&mut batch, Vec::with_capacity(max_batch_size));
                                let audit_trail_clone = audit_trail.clone();
                                
                                tokio::spawn(async move {
                                    Self::process_batch(batch_to_process, handlers_clone, audit_trail_clone).await;
                                });
                            }
                        }
                        None => {
                            // Channel closed, exit loop
                            break;
                        }
                    }
                }
                
                // Process batch on interval tick if not empty
                _ = interval.tick() => {
                    if !batch.is_empty() {
                        let handlers_clone = handlers.clone();
                        let batch_to_process = std::mem::replace(&mut batch, Vec::with_capacity(max_batch_size));
                        let audit_trail_clone = audit_trail.clone();
                        
                        tokio::spawn(async move {
                            Self::process_batch(batch_to_process, handlers_clone, audit_trail_clone).await;
                        });
                    }
                }
            }
        }
    }
    
    /// Process a batch of events with all registered handlers
    async fn process_batch(
        batch: Vec<StreamingEvent>,
        handlers: Arc<Mutex<Vec<Arc<dyn EventHandler>>>>,
        audit_trail: Option<Arc<dyn AuditTrail>>,
    ) {
        let handlers = handlers.lock().await;
        
        for handler in handlers.iter() {
            match handler.process_batch(batch.clone()).await {
                Ok(_) => {
                    if let Some(audit) = &audit_trail {
                        for event in &batch {
                            let _ = audit.record_event_processing(event, true).await;
                        }
                    }
                }
                Err(e) => {
                    error!("Error processing batch: {}", e);
                    if let Some(audit) = &audit_trail {
                        for event in &batch {
                            let _ = audit.record_event_processing(event, false).await;
                        }
                    }
                }
            }
        }
    }
}

/// Transaction event handler that processes transaction events
pub struct TransactionEventHandler {
    currency_converter: Arc<CurrencyConverter>,
    cache: Arc<dyn Cache>,
    audit_trail: Arc<dyn AuditTrail>,
}

impl TransactionEventHandler {
    /// Create a new transaction event handler
    pub fn new(
        currency_converter: Arc<CurrencyConverter>,
        cache: Arc<dyn Cache>,
        audit_trail: Arc<dyn AuditTrail>,
    ) -> Self {
        Self {
            currency_converter,
            cache,
            audit_trail,
        }
    }
}

#[async_trait]
impl EventHandler for TransactionEventHandler {
    async fn process_event(&self, event: StreamingEvent) -> Result<()> {
        if event.event_type != "transaction" {
            return Ok(());
        }
        
        debug!("Processing transaction event: {}", event.id);
        
        // Extract transaction details from payload
        let portfolio_id = event.entity_id;
        let transaction = event.payload.clone();
        
        // Invalidate cache for the affected portfolio
        let cache_key_pattern = format!("portfolio:{}:*", portfolio_id);
        self.cache.invalidate_pattern(&cache_key_pattern).await?;
        
        // Record the transaction processing
        self.audit_trail.record_event_processing(&event, true).await?;
        
        Ok(())
    }
    
    // Implement batch processing for better performance
    async fn process_batch(&self, events: Vec<StreamingEvent>) -> Result<()> {
        let transaction_events: Vec<_> = events.into_iter()
            .filter(|e| e.event_type == "transaction")
            .collect();
        
        if transaction_events.is_empty() {
            return Ok(());
        }
        
        debug!("Batch processing {} transaction events", transaction_events.len());
        
        // Group transactions by portfolio
        let mut portfolio_transactions: std::collections::HashMap<String, Vec<StreamingEvent>> = 
            std::collections::HashMap::new();
        
        for event in transaction_events {
            portfolio_transactions
                .entry(event.entity_id.clone())
                .or_insert_with(Vec::new)
                .push(event);
        }
        
        // Process each portfolio's transactions
        for (portfolio_id, events) in portfolio_transactions {
            // Invalidate cache for the affected portfolio
            let cache_key_pattern = format!("portfolio:{}:*", portfolio_id);
            self.cache.invalidate_pattern(&cache_key_pattern).await?;
            
            // Record the transaction processing
            for event in &events {
                self.audit_trail.record_event_processing(event, true).await?;
            }
        }
        
        Ok(())
    }
}

/// Price update event handler that processes price update events
pub struct PriceUpdateEventHandler {
    cache: Arc<dyn Cache>,
    audit_trail: Arc<dyn AuditTrail>,
}

impl PriceUpdateEventHandler {
    /// Create a new price update event handler
    pub fn new(
        cache: Arc<dyn Cache>,
        audit_trail: Arc<dyn AuditTrail>,
    ) -> Self {
        Self {
            cache,
            audit_trail,
        }
    }
}

#[async_trait]
impl EventHandler for PriceUpdateEventHandler {
    async fn process_event(&self, event: StreamingEvent) -> Result<()> {
        if event.event_type != "price_update" {
            return Ok(());
        }
        
        debug!("Processing price update event: {}", event.id);
        
        // Extract price update details from payload
        let security_id = event.entity_id;
        
        // Invalidate cache for the affected security
        let cache_key_pattern = format!("security:{}:price:*", security_id);
        self.cache.invalidate_pattern(&cache_key_pattern).await?;
        
        // Also invalidate any portfolio valuations that might use this security
        let portfolio_cache_pattern = "portfolio:*:performance:*";
        self.cache.invalidate_pattern(portfolio_cache_pattern).await?;
        
        // Record the price update processing
        self.audit_trail.record_event_processing(&event, true).await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculations::audit::InMemoryAuditTrail;
    use crate::calculations::distributed_cache::MockCache;
    use crate::calculations::currency::{MockExchangeRateProvider, ExchangeRate};
    use std::time::Duration;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_streaming_processor() -> Result<()> {
        // Create components
        let audit_trail = Arc::new(InMemoryAuditTrail::new());
        let cache = Arc::new(MockCache::new());
        
        // Create exchange rate provider and currency converter
        let exchange_rate_provider = Arc::new(MockExchangeRateProvider::new(vec![
            ExchangeRate {
                base_currency: "USD".to_string(),
                quote_currency: "EUR".to_string(),
                rate: rust_decimal_macros::dec!(0.85),
                date: Utc::now(),
                source: "test".to_string(),
            },
        ]));
        let currency_converter = Arc::new(CurrencyConverter::new(exchange_rate_provider));
        
        // Create streaming processor
        let config = StreamingConfig {
            max_concurrent_events: 10,
            buffer_size: 100,
            enable_batch_processing: true,
            max_batch_size: 5,
            batch_wait_ms: 50,
        };
        
        let processor = StreamingProcessor::new(
            config,
            Some(audit_trail.clone()),
            Some(cache.clone()),
        ).await?;
        
        // Create and register handlers
        let transaction_handler = Arc::new(TransactionEventHandler::new(
            currency_converter.clone(),
            cache.clone(),
            audit_trail.clone(),
        ));
        
        let price_update_handler = Arc::new(PriceUpdateEventHandler::new(
            cache.clone(),
            audit_trail.clone(),
        ));
        
        processor.register_handler(transaction_handler).await;
        processor.register_handler(price_update_handler).await;
        
        // Create test events
        let events = vec![
            StreamingEvent {
                id: "event-1".to_string(),
                timestamp: Utc::now(),
                event_type: "transaction".to_string(),
                source: "test".to_string(),
                entity_id: "portfolio-123".to_string(),
                payload: serde_json::json!({
                    "transaction_type": "buy",
                    "security_id": "AAPL",
                    "quantity": 100,
                    "price": 150.0,
                    "currency": "USD",
                    "date": "2023-01-15",
                }),
            },
            StreamingEvent {
                id: "event-2".to_string(),
                timestamp: Utc::now(),
                event_type: "price_update".to_string(),
                source: "test".to_string(),
                entity_id: "AAPL".to_string(),
                payload: serde_json::json!({
                    "price": 155.0,
                    "currency": "USD",
                    "date": "2023-01-16",
                }),
            },
        ];
        
        // Submit events
        for event in events {
            processor.submit_event(event).await?;
        }
        
        // Wait for processing to complete
        sleep(Duration::from_millis(200)).await;
        
        // Verify audit trail
        let records = audit_trail.get_records().await?;
        assert!(records.len() >= 2);
        
        // Verify cache invalidation
        let cache_stats = cache.get_stats().await;
        assert!(cache_stats.invalidations > 0);
        
        Ok(())
    }
} 