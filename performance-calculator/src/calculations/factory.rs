//! Component factory for creating and initializing application components.
//!
//! This module provides a factory pattern implementation for creating various
//! components of the performance calculator application, such as caches, APIs,
//! and processors. It simplifies the initialization process and ensures proper
//! dependency injection.
//!
//! # Examples
//!
//! ```
//! use performance_calculator::calculations::{
//!     config::AppConfig,
//!     factory::ComponentFactory,
//! };
//!
//! async fn init_components() {
//!     // Load configuration
//!     let config = AppConfig::from_env();
//!     
//!     // Create factory
//!     let factory = ComponentFactory::new(config);
//!     
//!     // Create Redis cache
//!     let cache = factory.create_redis_cache().await.expect("Failed to create cache");
//! }
//! ```

use anyhow::{Result, anyhow};
use std::sync::Arc;
use tracing::info;

use crate::calculations::{
    config::AppConfig,
    audit::AuditTrailManager,
    distributed_cache::{Cache, CacheFactory},
    currency::CurrencyConverter,
    query_api::{QueryApi, DataAccessService},
    streaming::{StreamingProcessor, EventSource, EventHandler, KafkaEventSource},
    scheduler::{CalculationScheduler, NotificationService, DefaultNotificationService, EmailClient, AwsClient},
    analytics::{AnalyticsConfig, AnalyticsEngine},
    visualization::{VisualizationConfig, VisualizationEngine},
    integration::{IntegrationConfig, IntegrationEngine},
};

/// Factory for creating application components.
///
/// This factory simplifies the creation of various components by handling
/// dependency injection and configuration. It ensures that components are
/// properly initialized with their required dependencies.
pub struct ComponentFactory {
    /// Application configuration
    config: AppConfig,
}

impl ComponentFactory {
    /// Creates a new component factory.
    ///
    /// # Arguments
    ///
    /// * `config` - Application configuration
    ///
    /// # Returns
    ///
    /// * `ComponentFactory` - New component factory
    ///
    /// # Examples
    ///
    /// ```
    /// use performance_calculator::calculations::{
    ///     config::AppConfig,
    ///     factory::ComponentFactory,
    /// };
    ///
    /// let config = AppConfig::default();
    /// let factory = ComponentFactory::new(config);
    /// ```
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
    
    /// Creates a Redis cache.
    ///
    /// # Returns
    ///
    /// * `Result<Arc<dyn Cache<String, serde_json::Value> + Send + Sync>>` - Redis cache or error
    ///
    /// # Examples
    ///
    /// ```
    /// use performance_calculator::calculations::{
    ///     config::AppConfig,
    ///     factory::ComponentFactory,
    /// };
    ///
    /// async fn create_cache() {
    ///     let config = AppConfig::default();
    ///     let factory = ComponentFactory::new(config);
    ///     
    ///     let cache = factory.create_redis_cache().await.expect("Failed to create cache");
    /// }
    /// ```
    pub async fn create_redis_cache(&self) -> Result<Arc<dyn Cache<String, serde_json::Value> + Send + Sync>> {
        let redis_config = &self.config.redis_cache;
        
        // Create Redis cache with configuration
        CacheFactory::create_redis_cache(
            &redis_config.url,
            redis_config.max_connections,
        ).await
    }
    
    /// Creates a query API.
    ///
    /// # Arguments
    ///
    /// * `audit_manager` - Audit trail manager for tracking query execution
    /// * `currency_converter` - Currency converter for multi-currency support
    /// * `data_service` - Data access service for retrieving portfolio data
    ///
    /// # Returns
    ///
    /// * `Result<Arc<QueryApi>>` - Query API or error
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use performance_calculator::calculations::{
    ///     config::AppConfig,
    ///     factory::ComponentFactory,
    ///     audit::AuditTrailManager,
    ///     currency::CurrencyConverter,
    ///     query_api::MockDataAccessService,
    /// };
    ///
    /// async fn create_query_api(
    ///     audit_manager: Arc<AuditTrailManager>,
    ///     currency_converter: Arc<CurrencyConverter>,
    /// ) {
    ///     let config = AppConfig::default();
    ///     let factory = ComponentFactory::new(config);
    ///     
    ///     let data_service = Arc::new(MockDataAccessService);
    ///     
    ///     let query_api = factory.create_query_api(
    ///         audit_manager,
    ///         currency_converter,
    ///         data_service,
    ///     ).await.expect("Failed to create query API");
    /// }
    /// ```
    pub async fn create_query_api(
        &self,
        audit_manager: Arc<AuditTrailManager>,
        currency_converter: Arc<CurrencyConverter>,
        data_service: Arc<dyn DataAccessService>,
    ) -> Result<Arc<QueryApi>> {
        // Create cache if caching is enabled
        let cache = if self.config.query_api.enable_caching {
            self.create_redis_cache().await?
        } else {
            // Use a mock cache if caching is disabled
            #[cfg(not(test))]
            {
                return Err(anyhow!("Caching must be enabled in production"));
            }
            
            #[cfg(test)]
            CacheFactory::create_mock_cache()
        };
        
        // Create query API with dependencies
        let query_api = QueryApi::new(
            audit_manager,
            cache,
            currency_converter,
            data_service,
        );
        
        Ok(Arc::new(query_api))
    }
    
    /// Creates a streaming processor.
    ///
    /// # Arguments
    ///
    /// * `audit_manager` - Audit trail manager for tracking event processing
    ///
    /// # Returns
    ///
    /// * `Result<Arc<StreamingProcessor>>` - Streaming processor or error
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use performance_calculator::calculations::{
    ///     config::AppConfig,
    ///     factory::ComponentFactory,
    ///     audit::AuditTrailManager,
    /// };
    ///
    /// async fn create_streaming_processor(audit_manager: Arc<AuditTrailManager>) {
    ///     let mut config = AppConfig::default();
    ///     config.streaming.enabled = true;
    ///     config.streaming.kafka_bootstrap_servers = Some("localhost:9092".to_string());
    ///     config.streaming.kafka_consumer_group_id = Some("performance-calculator".to_string());
    ///     config.streaming.kafka_topics = vec!["performance-events".to_string()];
    ///     
    ///     let factory = ComponentFactory::new(config);
    ///     
    ///     let processor = factory.create_streaming_processor(audit_manager)
    ///         .await
    ///         .expect("Failed to create streaming processor");
    /// }
    /// ```
    pub async fn create_streaming_processor(
        &self,
        audit_manager: Arc<AuditTrailManager>,
    ) -> Result<Arc<StreamingProcessor>> {
        let streaming_config = &self.config.streaming;
        
        // Check if streaming is enabled
        if !streaming_config.enabled {
            return Err(anyhow!("Streaming processing is disabled in configuration"));
        }
        
        let mut processor = StreamingProcessor::new();
        
        // Add Kafka sources if configured
        if let Some(bootstrap_servers) = &streaming_config.kafka_bootstrap_servers {
            if let Some(group_id) = &streaming_config.kafka_consumer_group_id {
                for topic in &streaming_config.kafka_topics {
                    // Create Kafka event source for each topic
                    let source = KafkaEventSource::new(
                        bootstrap_servers.clone(),
                        topic.clone(),
                        group_id.clone(),
                    );
                    
                    processor.add_source(Arc::new(source));
                }
            }
        }
        
        // Add performance calculation handler
        let handler = crate::calculations::streaming::PerformanceCalculationHandler::new(
            audit_manager.clone(),
            "streaming".to_string(),
        );
        
        processor.add_handler(Arc::new(handler));
        
        Ok(Arc::new(processor))
    }
    
    /// Creates a notification service.
    ///
    /// # Returns
    ///
    /// * `Result<Arc<dyn NotificationService>>` - Notification service or error
    ///
    /// # Examples
    ///
    /// ```
    /// use performance_calculator::calculations::{
    ///     config::AppConfig,
    ///     factory::ComponentFactory,
    /// };
    ///
    /// async fn create_notification_service() {
    ///     let mut config = AppConfig::default();
    ///     config.email.smtp_server = Some("smtp.example.com".to_string());
    ///     config.email.from_address = Some("notifications@example.com".to_string());
    ///     
    ///     let factory = ComponentFactory::new(config);
    ///     
    ///     let notification_service = factory.create_notification_service()
    ///         .await
    ///         .expect("Failed to create notification service");
    /// }
    /// ```
    pub async fn create_notification_service(&self) -> Result<Arc<dyn NotificationService>> {
        let email_config = &self.config.email;
        let aws_config = &self.config.aws;
        
        // Create email client if configured
        let email_client = if let Some(server) = &email_config.smtp_server {
            let port = email_config.smtp_port.unwrap_or(587);
            let username = email_config.smtp_username.clone();
            let password = email_config.smtp_password.clone();
            let from_address = email_config.from_address.clone()
                .ok_or_else(|| anyhow!("Email from address is required"))?;
            
            // Create SMTP email client
            let client = SmtpEmailClient::new(
                server.clone(),
                port,
                username,
                password,
                from_address,
            )?;
            
            Some(Arc::new(client) as Arc<dyn EmailClient>)
        } else {
            None
        };
        
        // Create AWS client if configured
        let aws_client = if aws_config.notification_topic_arn.is_some() || aws_config.notification_queue_url.is_some() {
            let client = AwsClientImpl::new(aws_config.region.clone())?;
            Some(Arc::new(client) as Arc<dyn AwsClient>)
        } else {
            None
        };
        
        // Create notification service
        let service = DefaultNotificationService::new(
            email_client,
            aws_client,
        );
        
        Ok(Arc::new(service))
    }
    
    /// Create a calculation scheduler
    pub async fn create_calculation_scheduler(
        &self,
        query_api: Arc<QueryApi>,
        audit_manager: Arc<AuditTrailManager>,
    ) -> Result<Arc<CalculationScheduler>> {
        let scheduler_config = &self.config.scheduler;
        
        if !scheduler_config.enabled {
            return Err(anyhow!("Scheduler is disabled in configuration"));
        }
        
        // Create notification service
        let notification_service = self.create_notification_service().await?;
        
        // Create scheduler
        let scheduler = CalculationScheduler::new(
            query_api,
            audit_manager,
            notification_service,
        );
        
        Ok(Arc::new(scheduler))
    }

    /// Create an audit trail
    pub async fn create_audit_trail(&self) -> Result<Option<Arc<dyn AuditTrail>>> {
        if !self.config.audit.enabled {
            return Ok(None);
        }
        
        if self.config.audit.use_dynamodb {
            let audit_trail = DynamoDbAuditTrail::new(
                &self.config.audit.dynamodb_table,
                &self.config.audit.dynamodb_region,
            ).await.context("Failed to create DynamoDB audit trail")?;
            
            Ok(Some(Arc::new(audit_trail)))
        } else {
            Ok(Some(Arc::new(InMemoryAuditTrail::new())))
        }
    }
    
    /// Create a streaming processor
    pub async fn create_streaming_processor(&self) -> Result<Option<Arc<StreamingProcessor>>> {
        if !self.config.streaming.enabled {
            return Ok(None);
        }
        
        // Create required dependencies
        let audit_trail = self.create_audit_trail().await?;
        let cache = self.create_redis_cache().await?;
        
        // Create streaming config
        let streaming_config = StreamingConfig {
            max_concurrent_events: self.config.streaming.max_concurrent_events,
            buffer_size: self.config.streaming.buffer_size,
            enable_batch_processing: self.config.streaming.enable_batch_processing,
            max_batch_size: self.config.streaming.max_batch_size,
            batch_wait_ms: self.config.streaming.batch_wait_ms,
        };
        
        // Create streaming processor
        let processor = StreamingProcessor::new(
            streaming_config,
            audit_trail.clone(),
            cache.clone(),
        ).await.context("Failed to create streaming processor")?;
        
        // Create and register event handlers if needed
        if self.config.streaming.register_default_handlers {
            if let (Some(cache), Some(audit_trail)) = (cache.clone(), audit_trail.clone()) {
                // Create currency converter if needed
                let currency_converter = self.create_currency_converter().await?;
                
                if let Some(currency_converter) = currency_converter {
                    // Create transaction event handler
                    let transaction_handler = Arc::new(TransactionEventHandler::new(
                        currency_converter.clone(),
                        cache.clone(),
                        audit_trail.clone(),
                    ));
                    
                    // Register transaction handler
                    processor.register_handler(transaction_handler).await;
                }
                
                // Create price update event handler
                let price_update_handler = Arc::new(PriceUpdateEventHandler::new(
                    cache.clone(),
                    audit_trail.clone(),
                ));
                
                // Register price update handler
                processor.register_handler(price_update_handler).await;
            }
        }
        
        Ok(Some(Arc::new(processor)))
    }
    
    /// Create a query API
    pub async fn create_query_api(&self) -> Result<Option<Arc<QueryApi>>> {
        if !self.config.query_api.enabled {
            return Ok(None);
        }
        
        // Create required dependencies
        let audit_trail = self.create_audit_trail().await?;
        let cache = self.create_redis_cache().await?;
        let currency_converter = self.create_currency_converter().await?;
        
        // Create data source
        // In a real implementation, this would create a proper data source
        // For now, we'll use a mock data source
        let data_source = Arc::new(MockDataSource::new());
        
        // Create query API config
        let query_api_config = QueryApiConfig {
            max_results: self.config.query_api.max_results,
            default_page_size: self.config.query_api.default_page_size,
            cache_ttl_seconds: self.config.query_api.cache_ttl_seconds,
            enable_caching: self.config.query_api.enable_caching,
        };
        
        // Create query API
        let query_api = QueryApi::new(
            query_api_config,
            data_source,
            cache,
            currency_converter,
            audit_trail,
        );
        
        Ok(Some(Arc::new(query_api)))
    }
    
    /// Create a scheduler
    pub async fn create_scheduler(&self) -> Result<Option<Arc<Scheduler>>> {
        if !self.config.scheduler.enabled {
            return Ok(None);
        }
        
        // Create required dependencies
        let audit_trail = self.create_audit_trail().await?;
        let cache = self.create_redis_cache().await?;
        
        // Create job repository
        // In a real implementation, this would create a proper repository
        // For now, we'll use an in-memory repository
        let repository = Arc::new(InMemoryJobRepository::new());
        
        // Create scheduler config
        let scheduler_config = SchedulerConfig {
            enabled: self.config.scheduler.enabled,
            max_concurrent_jobs: self.config.scheduler.max_concurrent_jobs,
            default_retry_count: self.config.scheduler.default_retry_count,
            default_retry_delay_seconds: self.config.scheduler.default_retry_delay_seconds,
            history_retention_days: self.config.scheduler.history_retention_days,
        };
        
        // Create scheduler
        let scheduler = Scheduler::new(
            scheduler_config,
            repository,
            audit_trail.clone(),
            cache.clone(),
        );
        
        // Register job handlers if needed
        if self.config.scheduler.register_default_handlers {
            if let (Some(cache), Some(audit_trail)) = (cache.clone(), audit_trail.clone()) {
                // Create performance calculation job handler
                let performance_handler = Arc::new(PerformanceCalculationJobHandler::new(
                    cache.clone(),
                    audit_trail.clone(),
                ));
                
                // Register performance calculation handler
                scheduler.register_handler(performance_handler).await;
            }
        }
        
        Ok(Some(Arc::new(scheduler)))
    }

    /// Create an analytics engine if enabled
    pub fn create_analytics_engine(&self) -> Option<Arc<AnalyticsEngine>> {
        if let Some(config) = &self.config.analytics {
            if config.enabled {
                let cache = self.create_cache();
                let audit_trail = self.create_audit_trail();
                
                let analytics_engine = Arc::new(AnalyticsEngine::new(
                    config.clone(),
                    cache,
                    audit_trail,
                ));
                
                return Some(analytics_engine);
            }
        }
        
        None
    }
    
    /// Create a visualization engine if enabled
    pub fn create_visualization_engine(&self) -> Option<VisualizationEngine> {
        if let Some(config) = &self.config.visualization {
            if config.enabled {
                let cache = self.create_cache();
                let audit_trail = self.create_audit_trail();
                
                let visualization_engine = VisualizationEngine::new(
                    config.clone(),
                    cache,
                    audit_trail,
                );
                
                return Some(visualization_engine);
            }
        }
        
        None
    }
    
    /// Create an integration engine if enabled
    pub fn create_integration_engine(&self) -> Option<Arc<IntegrationEngine>> {
        if let Some(config) = &self.config.integration {
            if config.enabled {
                let cache = self.create_cache();
                let audit_trail = self.create_audit_trail();
                
                let integration_engine = Arc::new(IntegrationEngine::new(
                    config.clone(),
                    cache,
                    audit_trail,
                ));
                
                return Some(integration_engine);
            }
        }
        
        None
    }
}

/// SMTP email client implementation
pub struct SmtpEmailClient {
    server: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    from_address: String,
}

impl SmtpEmailClient {
    /// Create a new SMTP email client
    pub fn new(
        server: String,
        port: u16,
        username: Option<String>,
        password: Option<String>,
        from_address: String,
    ) -> Result<Self> {
        Ok(Self {
            server,
            port,
            username,
            password,
            from_address,
        })
    }
}

#[async_trait::async_trait]
impl EmailClient for SmtpEmailClient {
    async fn send_email(
        &self,
        recipients: &[String],
        subject: &str,
        body: &str,
    ) -> Result<()> {
        // This is a simplified implementation
        // In a real application, you would use a library like lettre
        
        info!(
            server = %self.server,
            port = %self.port,
            from = %self.from_address,
            recipients = ?recipients,
            subject = %subject,
            "Sending email"
        );
        
        // Simulate sending email
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(())
    }
}

/// AWS client implementation
pub struct AwsClientImpl {
    region: String,
}

impl AwsClientImpl {
    /// Create a new AWS client
    pub fn new(region: String) -> Result<Self> {
        Ok(Self { region })
    }
}

#[async_trait::async_trait]
impl AwsClient for AwsClientImpl {
    async fn send_sns_message(
        &self,
        topic_arn: &str,
        subject: &str,
        message: &str,
    ) -> Result<()> {
        // This is a simplified implementation
        // In a real application, you would use the AWS SDK
        
        info!(
            region = %self.region,
            topic_arn = %topic_arn,
            subject = %subject,
            "Sending SNS message"
        );
        
        // Simulate sending SNS message
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(())
    }
    
    async fn send_sqs_message(
        &self,
        queue_url: &str,
        message: &str,
    ) -> Result<()> {
        // This is a simplified implementation
        // In a real application, you would use the AWS SDK
        
        info!(
            region = %self.region,
            queue_url = %queue_url,
            "Sending SQS message"
        );
        
        // Simulate sending SQS message
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_factory_create_components() -> Result<()> {
        // Create a factory with default configuration
        let factory = ComponentFactory::new_for_testing();
        
        // Test creating a mock cache
        let cache = factory.create_mock_cache().await?;
        assert!(cache.is_some());
        
        // Test creating a mock exchange rate provider
        let provider = factory.create_mock_exchange_rate_provider().await?;
        assert!(provider.is_some());
        
        // Test creating an audit trail
        let config = Config {
            audit: crate::calculations::config::AuditConfig {
                enabled: true,
                use_dynamodb: false,
                dynamodb_table: "".to_string(),
                dynamodb_region: "".to_string(),
            },
            ..Config::default()
        };
        
        let factory = ComponentFactory::new(config);
        let audit_trail = factory.create_audit_trail().await?;
        assert!(audit_trail.is_some());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_factory_create_phase2_components() -> Result<()> {
        // Create a factory with Phase 2 components enabled
        let config = Config {
            redis_cache: crate::calculations::config::RedisCacheConfig {
                enabled: true,
                url: "redis://localhost:6379".to_string(),
                ttl_seconds: 3600,
                prefix: "test:".to_string(),
            },
            audit: crate::calculations::config::AuditConfig {
                enabled: true,
                use_dynamodb: false,
                dynamodb_table: "".to_string(),
                dynamodb_region: "".to_string(),
            },
            streaming: crate::calculations::config::StreamingConfig {
                enabled: true,
                max_concurrent_events: 10,
                buffer_size: 100,
                enable_batch_processing: true,
                max_batch_size: 10,
                batch_wait_ms: 100,
                register_default_handlers: true,
            },
            query_api: crate::calculations::config::QueryApiConfig {
                enabled: true,
                max_results: 100,
                default_page_size: 10,
                cache_ttl_seconds: 300,
                enable_caching: true,
            },
            scheduler: crate::calculations::config::SchedulerConfig {
                enabled: true,
                max_concurrent_jobs: 5,
                default_retry_count: 3,
                default_retry_delay_seconds: 60,
                history_retention_days: 30,
                register_default_handlers: true,
            },
            ..Config::default()
        };
        
        // Use mock components for testing
        let factory = ComponentFactory::new_with_mocks(config);
        
        // Test creating a streaming processor
        let processor = factory.create_streaming_processor().await?;
        assert!(processor.is_some());
        
        // Test creating a query API
        let query_api = factory.create_query_api().await?;
        assert!(query_api.is_some());
        
        // Test creating a scheduler
        let scheduler = factory.create_scheduler().await?;
        assert!(scheduler.is_some());
        
        Ok(())
    }
} 