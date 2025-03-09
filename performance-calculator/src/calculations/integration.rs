//! Enterprise Integration Module
//!
//! This module provides integration capabilities for the Performance Calculator,
//! including API integration, data import/export, and notification services.

use anyhow::{Result, Context, anyhow};
use chrono::{DateTime, Utc, NaiveDate};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use tracing::{info, warn, error};

use crate::calculations::audit::AuditTrail;
use crate::calculations::distributed_cache::StringCache;

/// Configuration for the Integration module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    /// Whether integration features are enabled
    pub enabled: bool,
    /// API endpoint configurations
    pub api_endpoints: HashMap<String, ApiEndpointConfig>,
    /// Email notification configuration
    pub email: EmailConfig,
    /// Webhook notification configuration
    pub webhooks: WebhookConfig,
    /// Data import configuration
    pub data_import: DataImportConfig,
    /// Whether to cache integration results
    pub enable_caching: bool,
    /// TTL for cached integration results in seconds
    pub cache_ttl_seconds: u64,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_endpoints: HashMap::new(),
            email: EmailConfig::default(),
            webhooks: WebhookConfig::default(),
            data_import: DataImportConfig::default(),
            enable_caching: true,
            cache_ttl_seconds: 3600,
        }
    }
}

/// API endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpointConfig {
    /// Endpoint URL
    pub url: String,
    /// Authentication type
    pub auth_type: AuthType,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Whether to retry failed requests
    pub retry_enabled: bool,
    /// Maximum number of retries
    pub max_retries: u32,
    /// Retry delay in seconds
    pub retry_delay_seconds: u64,
}

/// Authentication type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    /// No authentication
    None,
    /// Basic authentication
    Basic {
        username: String,
        password: String,
    },
    /// API key authentication
    ApiKey {
        key_name: String,
        key_value: String,
        in_header: bool,
    },
    /// OAuth2 authentication
    OAuth2 {
        client_id: String,
        client_secret: String,
        token_url: String,
        scope: Option<String>,
    },
}

/// Email configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// Whether email notifications are enabled
    pub enabled: bool,
    /// SMTP server
    pub smtp_server: String,
    /// SMTP port
    pub smtp_port: u16,
    /// SMTP username
    pub smtp_username: String,
    /// SMTP password
    pub smtp_password: String,
    /// From email address
    pub from_address: String,
    /// Default recipients
    pub default_recipients: Vec<String>,
    /// Whether to use TLS
    pub use_tls: bool,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            smtp_server: "smtp.example.com".to_string(),
            smtp_port: 587,
            smtp_username: "".to_string(),
            smtp_password: "".to_string(),
            from_address: "performance-calculator@example.com".to_string(),
            default_recipients: Vec::new(),
            use_tls: true,
        }
    }
}

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    /// Whether webhook notifications are enabled
    pub enabled: bool,
    /// Registered webhooks
    pub webhooks: HashMap<String, WebhookEndpoint>,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            webhooks: HashMap::new(),
        }
    }
}

/// Webhook endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebhookEndpoint {
    /// Endpoint URL
    pub url: String,
    /// HTTP method
    pub method: String,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Event types to send to this webhook
    pub event_types: Vec<String>,
    /// Whether to retry failed requests
    pub retry_enabled: bool,
    /// Maximum number of retries
    pub max_retries: u32,
}

/// Data import configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataImportConfig {
    /// Whether data import is enabled
    pub enabled: bool,
    /// Supported file formats
    pub supported_formats: Vec<String>,
    /// Maximum file size in bytes
    pub max_file_size: u64,
    /// Whether to validate data before import
    pub validate_data: bool,
    /// Whether to backup data before import
    pub backup_before_import: bool,
}

impl Default for DataImportConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            supported_formats: vec!["CSV".to_string(), "JSON".to_string(), "Excel".to_string()],
            max_file_size: 10 * 1024 * 1024, // 10 MB
            validate_data: true,
            backup_before_import: true,
        }
    }
}

/// Email notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailNotification {
    /// Email subject
    pub subject: String,
    /// Email body
    pub body: String,
    /// Recipients
    pub recipients: Vec<String>,
    /// CC recipients
    pub cc: Option<Vec<String>>,
    /// BCC recipients
    pub bcc: Option<Vec<String>>,
    /// Attachments
    pub attachments: Option<Vec<EmailAttachment>>,
    /// Whether to send as HTML
    pub is_html: bool,
}

/// Email attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAttachment {
    /// Attachment name
    pub name: String,
    /// Attachment content type
    pub content_type: String,
    /// Attachment data
    pub data: Vec<u8>,
}

/// Webhook notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookNotification {
    /// Event type
    pub event_type: String,
    /// Event data
    pub data: serde_json::Value,
    /// Target webhook IDs (if empty, send to all matching webhooks)
    pub target_webhooks: Option<Vec<String>>,
}

/// API request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    /// API endpoint ID
    pub endpoint_id: String,
    /// HTTP method
    pub method: String,
    /// Path (appended to endpoint URL)
    pub path: String,
    /// Query parameters
    pub query_params: Option<HashMap<String, String>>,
    /// Headers
    pub headers: Option<HashMap<String, String>>,
    /// Request body
    pub body: Option<serde_json::Value>,
}

/// API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    /// Status code
    pub status_code: u16,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Option<serde_json::Value>,
    /// Error message (if any)
    pub error: Option<String>,
}

/// Data import request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataImportRequest {
    /// Import type
    pub import_type: String,
    /// File format
    pub format: String,
    /// File data
    pub data: Vec<u8>,
    /// Import options
    pub options: Option<HashMap<String, String>>,
}

/// Data import result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataImportResult {
    /// Import ID
    pub import_id: String,
    /// Import type
    pub import_type: String,
    /// Number of records processed
    pub records_processed: usize,
    /// Number of records imported
    pub records_imported: usize,
    /// Number of records with errors
    pub records_with_errors: usize,
    /// Error details
    pub errors: Vec<String>,
    /// Import timestamp
    pub timestamp: DateTime<Utc>,
}

/// Notification service trait
#[async_trait]
pub trait NotificationService: Send + Sync {
    /// Send an email notification
    async fn send_email(&self, notification: EmailNotification) -> Result<()>;
    
    /// Send a webhook notification
    async fn send_webhook(&self, notification: WebhookNotification) -> Result<()>;
}

/// API client trait
#[async_trait]
pub trait ApiClient: Send + Sync {
    /// Send an API request
    async fn send_request(&self, request: ApiRequest) -> Result<ApiResponse>;
    
    /// Get an OAuth2 token
    async fn get_oauth2_token(&self, endpoint_id: &str) -> Result<String>;
}

/// Data import service trait
#[async_trait]
pub trait DataImportService: Send + Sync {
    /// Import data
    async fn import_data(&self, request: DataImportRequest) -> Result<DataImportResult>;
    
    /// Get import history
    async fn get_import_history(&self) -> Result<Vec<DataImportResult>>;
    
    /// Get import details
    async fn get_import_details(&self, import_id: &str) -> Result<DataImportResult>;
}

/// Integration engine for enterprise integration
pub struct IntegrationEngine {
    /// Configuration
    config: IntegrationConfig,
    /// Cache for integration results
    cache: Arc<dyn StringCache + Send + Sync>,
    /// Audit trail
    audit_trail: Arc<dyn AuditTrail + Send + Sync>,
    /// Notification service
    notification_service: Box<dyn NotificationService>,
    /// API client
    api_client: Box<dyn ApiClient>,
    /// Data import service
    data_import_service: Box<dyn DataImportService>,
    /// Import history
    import_history: RwLock<Vec<DataImportResult>>,
}

/// Default notification service implementation
pub struct DefaultNotificationService {
    /// Configuration
    config: IntegrationConfig,
    /// Audit trail
    audit_trail: Arc<dyn AuditTrail + Send + Sync>,
}

#[async_trait]
impl NotificationService for DefaultNotificationService {
    async fn send_email(&self, notification: EmailNotification) -> Result<()> {
        // Check if email is enabled
        if !self.config.email.enabled {
            return Err(anyhow!("Email notifications are not enabled"));
        }
        
        // In a real implementation, this would use an SMTP client
        // For demonstration, we'll log the notification
        
        // Record in audit trail
        self.audit_trail.record(crate::calculations::audit::AuditRecord {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            entity_id: "email".to_string(),
            entity_type: "notification".to_string(),
            action: "send_email".to_string(),
            user_id: "system".to_string(),
            parameters: format!(
                "subject={},recipients={}",
                notification.subject,
                notification.recipients.join(",")
            ),
            result: format!("email_sent"),
            tenant_id: "default".to_string(),
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: "email_notification".to_string(),
            resource_id: "email".to_string(),
            resource_type: "notification".to_string(),
            operation: "send".to_string(),
            details: format!("Email sent to {} recipients", notification.recipients.len()),
            status: "success".to_string(),
        }).await?;
        
        Ok(())
    }
    
    async fn send_webhook(&self, notification: WebhookNotification) -> Result<()> {
        // Check if webhooks are enabled
        if !self.config.webhooks.enabled {
            return Err(anyhow!("Webhook notifications are not enabled"));
        }
        
        // Get matching webhooks
        let matching_webhooks: Vec<&WebhookEndpoint> = self.config.webhooks.webhooks.values()
            .filter(|w| w.event_types.contains(&notification.event_type))
            .filter(|w| {
                if let Some(targets) = &notification.target_webhooks {
                    // Only include webhooks that are in the target list
                    self.config.webhooks.webhooks.iter()
                        .any(|(id, endpoint)| targets.contains(id) && endpoint == *w)
                } else {
                    // Include all webhooks that match the event type
                    true
                }
            })
            .collect();
        
        if matching_webhooks.is_empty() {
            return Err(anyhow!("No matching webhooks found for event type: {}", notification.event_type));
        }
        
        // In a real implementation, this would send HTTP requests to the webhooks
        // For demonstration, we'll log the notification
        
        // Record in audit trail
        self.audit_trail.record(crate::calculations::audit::AuditRecord {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            entity_id: "webhook".to_string(),
            entity_type: "notification".to_string(),
            action: "send_webhook".to_string(),
            user_id: "system".to_string(),
            parameters: format!(
                "event_type={},webhook_count={}",
                notification.event_type,
                matching_webhooks.len()
            ),
            result: format!("webhook_sent"),
            tenant_id: "default".to_string(),
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: "webhook_notification".to_string(),
            resource_id: "webhook".to_string(),
            resource_type: "notification".to_string(),
            operation: "send".to_string(),
            details: format!("Webhook sent to {} endpoints", matching_webhooks.len()),
            status: "success".to_string(),
        }).await?;
        
        Ok(())
    }
}

/// Default API client implementation
#[derive(Clone)]
pub struct DefaultApiClient {
    /// Configuration
    config: IntegrationConfig,
    /// Audit trail
    audit_trail: Arc<dyn AuditTrail + Send + Sync>,
    /// OAuth2 tokens
    oauth2_tokens: Arc<RwLock<HashMap<String, (String, DateTime<Utc>)>>>,
    /// Cache for integration results
    cache: Arc<dyn StringCache + Send + Sync>,
}

#[async_trait]
impl ApiClient for DefaultApiClient {
    async fn send_request(&self, request: ApiRequest) -> Result<ApiResponse> {
        // Check if integration is enabled
        if !self.config.enabled {
            return Err(anyhow!("Integration is not enabled"));
        }
        
        // Get endpoint configuration
        let endpoint_config = self.config.api_endpoints.get(&request.endpoint_id)
            .ok_or_else(|| anyhow!("API endpoint not found: {}", request.endpoint_id))?;
        
        // In a real implementation, this would use an HTTP client
        // For demonstration, we'll return a mock response
        
        // Record in audit trail
        self.audit_trail.record(crate::calculations::audit::AuditRecord {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            entity_id: request.endpoint_id.clone(),
            entity_type: "api_request".to_string(),
            action: "send_request".to_string(),
            user_id: "system".to_string(),
            parameters: format!(
                "method={},path={},url={}",
                request.method,
                request.path,
                endpoint_config.url
            ),
            result: format!("mock_response"),
            tenant_id: "default".to_string(),
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: "api_request".to_string(),
            resource_id: request.endpoint_id.clone(),
            resource_type: "api_endpoint".to_string(),
            operation: "send".to_string(),
            details: format!("API request sent to {}", endpoint_config.url),
            status: "success".to_string(),
        }).await?;
        
        // Return mock response
        Ok(ApiResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: Some(serde_json::json!({
                "success": true,
                "message": "Mock response",
                "data": {
                    "request_id": uuid::Uuid::new_v4().to_string(),
                    "timestamp": Utc::now().to_rfc3339(),
                }
            })),
            error: None,
        })
    }
    
    async fn get_oauth2_token(&self, endpoint_id: &str) -> Result<String> {
        // Check if integration is enabled
        if !self.config.enabled {
            return Err(anyhow!("Integration is not enabled"));
        }
        
        // Get endpoint configuration
        let endpoint_config = self.config.api_endpoints.get(endpoint_id)
            .ok_or_else(|| anyhow!("API endpoint not found: {}", endpoint_id))?;
        
        // Check if endpoint uses OAuth2
        match &endpoint_config.auth_type {
            AuthType::OAuth2 { client_id, client_secret, token_url, scope } => {
                // Check if we have a valid token
                let mut tokens = self.oauth2_tokens.write().await;
                if let Some((token, expiry)) = tokens.get(endpoint_id) {
                    if expiry > &Utc::now() {
                        return Ok(token.clone());
                    }
                }
                
                // In a real implementation, this would request a new token
                // For demonstration, we'll return a mock token
                
                // Generate mock token
                let token = format!("mock_token_{}", uuid::Uuid::new_v4());
                let expiry = Utc::now() + chrono::Duration::hours(1);
                
                // Store token
                tokens.insert(endpoint_id.to_string(), (token.clone(), expiry));
                
                // Record in audit trail
                self.audit_trail.record(crate::calculations::audit::AuditRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    entity_id: endpoint_id.to_string(),
                    entity_type: "oauth2_token".to_string(),
                    action: "get_oauth2_token".to_string(),
                    user_id: "system".to_string(),
                    parameters: format!(
                        "client_id={},token_url={}",
                        client_id,
                        token_url
                    ),
                    result: format!("token_generated"),
                    tenant_id: "default".to_string(),
                    event_id: uuid::Uuid::new_v4().to_string(),
                    event_type: "oauth2_token".to_string(),
                    resource_id: endpoint_id.to_string(),
                    resource_type: "oauth2_token".to_string(),
                    operation: "get".to_string(),
                    details: format!("OAuth2 token generated"),
                    status: "success".to_string(),
                }).await?;
                
                Ok(token)
            },
            _ => Err(anyhow!("Endpoint does not use OAuth2: {}", endpoint_id)),
        }
    }
}

impl IntegrationEngine {
    /// Create a new integration engine
    pub fn new(
        config: IntegrationConfig,
        cache: Arc<dyn StringCache + Send + Sync>,
        audit_trail: Arc<dyn AuditTrail + Send + Sync>,
    ) -> Self {
        // Create default notification service
        let notification_service = Box::new(DefaultNotificationService {
            config: config.clone(),
            audit_trail: audit_trail.clone(),
        });
        
        // Create default API client
        let api_client = Box::new(DefaultApiClient {
            config: config.clone(),
            audit_trail: audit_trail.clone(),
            oauth2_tokens: Arc::new(RwLock::new(HashMap::new())),
            cache: cache.clone(),
        });
        
        // Create data import service
        let data_import_service = Box::new(DefaultDataImportService {
            config: config.clone(),
            audit_trail: audit_trail.clone(),
            import_history: RwLock::new(Vec::new()),
        });
        
        Self {
            config,
            cache,
            audit_trail,
            notification_service,
            api_client,
            data_import_service,
            import_history: RwLock::new(Vec::new()),
        }
    }
    
    /// Send an email notification
    pub async fn send_email(&self, notification: EmailNotification, request_id: &str) -> Result<()> {
        // Check if integration is enabled
        if !self.config.enabled {
            return Err(anyhow!("Integration module is not enabled"));
        }
        
        // Try to get from cache (to avoid duplicate emails)
        if self.config.enable_caching {
            let cache_key = format!("email:{}:{}:{}", 
                notification.subject,
                notification.recipients.join(","),
                request_id
            );
            
            if let Some(cached_result) = self.cache.get_string(&cache_key).await? {
                if cached_result == "sent" {
                    // Record cache hit in audit trail
                    self.audit_trail.record(crate::calculations::audit::AuditRecord {
                        id: uuid::Uuid::new_v4().to_string(),
                        timestamp: Utc::now(),
                        entity_id: "email".to_string(),
                        entity_type: "notification".to_string(),
                        action: "send_email_cache_hit".to_string(),
                        user_id: "system".to_string(),
                        parameters: format!(
                            "subject={},recipients={}",
                            notification.subject,
                            notification.recipients.join(",")
                        ),
                        result: format!("email_already_sent"),
                        event_id: uuid::Uuid::new_v4().to_string(),
                        event_type: "cache_hit".to_string(),
                        resource_id: cache_key.clone(),
                        resource_type: "email".to_string(),
                        operation: "get".to_string(),
                        details: format!("Email already sent: {}", notification.subject),
                        status: "success".to_string(),
                        tenant_id: "default".to_string(),
                    }).await?;
                    
                    return Ok(());
                }
            }
        }
        
        // Send email
        self.notification_service.send_email(notification.clone()).await?;
        
        // Cache the result
        if self.config.enable_caching {
            let cache_key = format!("email:{}:{}:{}", 
                notification.subject,
                notification.recipients.join(","),
                request_id
            );
            
            self.cache.set_string(cache_key.clone(), "sent".to_string(), Some(self.config.cache_ttl_seconds)).await?;
        }
        
        Ok(())
    }
    
    /// Send a webhook notification
    pub async fn send_webhook(&self, notification: WebhookNotification, request_id: &str) -> Result<()> {
        // Check if integration is enabled
        if !self.config.enabled {
            return Err(anyhow!("Integration module is not enabled"));
        }
        
        // Try to get from cache (to avoid duplicate webhooks)
        if self.config.enable_caching {
            let cache_key = format!("webhook:{}:{}:{}", 
                notification.event_type,
                serde_json::to_string(&notification.data)?,
                request_id
            );
            
            if let Some(cached_result) = self.cache.get_string(&cache_key).await? {
                if cached_result == "sent" {
                    // Record cache hit in audit trail
                    self.audit_trail.record(crate::calculations::audit::AuditRecord {
                        id: uuid::Uuid::new_v4().to_string(),
                        timestamp: Utc::now(),
                        entity_id: "webhook".to_string(),
                        entity_type: "notification".to_string(),
                        action: "send_webhook_cache_hit".to_string(),
                        user_id: "system".to_string(),
                        parameters: format!(
                            "event_type={},target_webhooks={:?}",
                            notification.event_type,
                            notification.target_webhooks
                        ),
                        result: format!("webhook_already_sent"),
                        event_id: uuid::Uuid::new_v4().to_string(),
                        event_type: "cache_hit".to_string(),
                        resource_id: cache_key.clone(),
                        resource_type: "webhook".to_string(),
                        operation: "get".to_string(),
                        details: format!("Webhook already sent: {}", notification.event_type),
                        status: "success".to_string(),
                        tenant_id: "default".to_string(),
                    }).await?;
                    
                    return Ok(());
                }
            }
        }
        
        // Send webhook
        self.notification_service.send_webhook(notification.clone()).await?;
        
        // Cache the result
        if self.config.enable_caching {
            let cache_key = format!("webhook:{}:{}:{}", 
                notification.event_type,
                serde_json::to_string(&notification.data)?,
                request_id
            );
            
            self.cache.set_string(cache_key.clone(), "sent".to_string(), Some(self.config.cache_ttl_seconds)).await?;
        }
        
        Ok(())
    }
    
    /// Send an API request
    pub async fn send_api_request(&self, request: ApiRequest, request_id: &str) -> Result<ApiResponse> {
        // Check if integration is enabled
        if !self.config.enabled {
            return Err(anyhow!("Integration module is not enabled"));
        }
        
        // Check if endpoint exists
        if !self.config.api_endpoints.contains_key(&request.endpoint_id) {
            return Err(anyhow!("API endpoint not found: {}", request.endpoint_id));
        }
        
        // Try to get from cache
        if self.config.enable_caching {
            let cache_key = format!("api_request:{}:{}:{}:{}:{}", 
                request.endpoint_id,
                request.method,
                request.path,
                serde_json::to_string(&request.query_params)?,
                request_id
            );
            
            if let Some(cached_json) = self.cache.get_string(&cache_key).await? {
                // Record cache hit in audit trail
                self.audit_trail.record(crate::calculations::audit::AuditRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    timestamp: Utc::now(),
                    entity_id: request.endpoint_id.clone(),
                    entity_type: "api_request".to_string(),
                    action: "api_request_cache_hit".to_string(),
                    user_id: "system".to_string(),
                    parameters: format!(
                        "method={},path={},endpoint={}",
                        request.method,
                        request.path,
                        request.endpoint_id
                    ),
                    result: format!("cached_result_found"),
                    event_id: uuid::Uuid::new_v4().to_string(),
                    event_type: "cache_hit".to_string(),
                    resource_id: cache_key.clone(),
                    resource_type: "api_request".to_string(),
                    operation: "get".to_string(),
                    details: format!("API request cache hit: {}", request.endpoint_id),
                    status: "success".to_string(),
                    tenant_id: "default".to_string(),
                }).await?;
                
                // Deserialize the cached response
                let response: ApiResponse = serde_json::from_str(&cached_json)?;
                return Ok(response);
            }
        }
        
        // Send request
        let response = self.api_client.send_request(request.clone()).await?;
        
        // Cache the result if successful
        if self.config.enable_caching && response.status_code >= 200 && response.status_code < 300 {
            let cache_key = format!("api_request:{}:{}:{}:{}:{}", 
                request.endpoint_id,
                request.method,
                request.path,
                serde_json::to_string(&request.query_params)?,
                request_id
            );
            
            // Serialize the response
            let serialized = serde_json::to_string(&response)?;
            self.cache.set_string(cache_key.clone(), serialized, Some(self.config.cache_ttl_seconds)).await?;
        }
        
        // Record in audit trail
        self.audit_trail.record(crate::calculations::audit::AuditRecord {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            entity_id: request.endpoint_id.clone(),
            entity_type: "api_request".to_string(),
            action: "api_request".to_string(),
            user_id: "system".to_string(),
            parameters: format!(
                "method={},path={}",
                request.method,
                request.path
            ),
            result: format!("status_code={}", response.status_code),
            tenant_id: "default".to_string(),
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: "api_request".to_string(),
            resource_id: request.endpoint_id.clone(),
            resource_type: "api_endpoint".to_string(),
            operation: "send".to_string(),
            details: format!("API request completed with status {}", response.status_code),
            status: if response.status_code >= 200 && response.status_code < 300 { "success" } else { "failure" }.to_string(),
        }).await?;
        
        Ok(response)
    }
    
    /// Import data
    pub async fn import_data(&self, request: DataImportRequest, request_id: &str) -> Result<DataImportResult> {
        // Check if integration is enabled
        if !self.config.enabled {
            return Err(anyhow!("Integration module is not enabled"));
        }
        
        // Check if data import is enabled
        if !self.config.data_import.enabled {
            return Err(anyhow!("Data import is not enabled"));
        }
        
        // Check file format
        if !self.config.data_import.supported_formats.contains(&request.format) {
            return Err(anyhow!("Unsupported file format: {}", request.format));
        }
        
        // Check file size
        if request.data.len() as u64 > self.config.data_import.max_file_size {
            return Err(anyhow!(
                "File size exceeds maximum allowed size: {} bytes",
                self.config.data_import.max_file_size
            ));
        }
        
        // Import data
        let result = self.data_import_service.import_data(request.clone()).await?;
        
        // Add to import history
        let mut import_history = self.import_history.write().await;
        import_history.push(result.clone());
        
        // Record in audit trail
        self.audit_trail.record(crate::calculations::audit::AuditRecord {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            entity_id: result.import_id.clone(),
            entity_type: "data_import".to_string(),
            action: "import_data".to_string(),
            user_id: "system".to_string(),
            parameters: format!(
                "import_type={},format={}",
                request.import_type.clone(),
                request.format.clone()
            ),
            result: format!(
                "records_processed={},records_imported={},records_with_errors={}",
                result.records_processed,
                result.records_imported,
                result.records_with_errors
            ),
            tenant_id: "default".to_string(),
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: "data_import".to_string(),
            resource_id: result.import_id.clone(),
            resource_type: "import_job".to_string(),
            operation: "import".to_string(),
            details: format!("Data import completed with {} records processed", result.records_processed),
            status: if result.records_with_errors == 0 { "success" } else { "partial_success" }.to_string(),
        }).await?;
        
        Ok(result)
    }
    
    /// Get import history
    pub async fn get_import_history(&self) -> Result<Vec<DataImportResult>> {
        // Check if integration is enabled
        if !self.config.enabled {
            return Err(anyhow!("Integration module is not enabled"));
        }
        
        // Check if data import is enabled
        if !self.config.data_import.enabled {
            return Err(anyhow!("Data import is not enabled"));
        }
        
        // Get import history
        let import_history = self.import_history.read().await;
        Ok(import_history.clone())
    }
    
    /// Get import details
    pub async fn get_import_details(&self, import_id: &str) -> Result<DataImportResult> {
        // Check if integration is enabled
        if !self.config.enabled {
            return Err(anyhow!("Integration module is not enabled"));
        }
        
        // Check if data import is enabled
        if !self.config.data_import.enabled {
            return Err(anyhow!("Data import is not enabled"));
        }
        
        // Get import details
        let import_history = self.import_history.read().await;
        let result = import_history.iter()
            .find(|r| r.import_id == import_id)
            .cloned()
            .ok_or_else(|| anyhow!("Import not found: {}", import_id))?;
        
        Ok(result)
    }
}

/// Default data import service implementation
pub struct DefaultDataImportService {
    /// Configuration
    config: IntegrationConfig,
    /// Audit trail
    audit_trail: Arc<dyn AuditTrail + Send + Sync>,
    /// Import history
    import_history: RwLock<Vec<DataImportResult>>,
}

#[async_trait]
impl DataImportService for DefaultDataImportService {
    async fn import_data(&self, request: DataImportRequest) -> Result<DataImportResult> {
        // Check if integration is enabled
        if !self.config.enabled {
            return Err(anyhow!("Integration is not enabled"));
        }
        
        // Check if data import is enabled
        if !self.config.data_import.enabled {
            return Err(anyhow!("Data import is not enabled"));
        }
        
        // In a real implementation, this would parse and import the data
        // For demonstration, we'll return a mock result
        
        // Generate import ID
        let import_id = uuid::Uuid::new_v4().to_string();
        
        // Store request fields before they're moved
        let import_type = request.import_type.clone();
        let format = request.format.clone();
        
        // Create mock result
        let result = DataImportResult {
            import_id,
            import_type: request.import_type,
            records_processed: 100,
            records_imported: 95,
            records_with_errors: 5,
            errors: vec![
                "Invalid data format in row 10".to_string(),
                "Missing required field in row 25".to_string(),
                "Duplicate record in row 42".to_string(),
                "Invalid date format in row 67".to_string(),
                "Value out of range in row 89".to_string(),
            ],
            timestamp: Utc::now(),
        };
        
        // Add to import history
        let mut import_history = self.import_history.write().await;
        import_history.push(result.clone());
        
        // Record in audit trail
        self.audit_trail.record(crate::calculations::audit::AuditRecord {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            entity_id: result.import_id.clone(),
            entity_type: "data_import".to_string(),
            action: "import_data".to_string(),
            user_id: "system".to_string(),
            parameters: format!(
                "import_type={},format={}",
                import_type,
                format
            ),
            result: format!(
                "records_processed={},records_imported={},records_with_errors={}",
                result.records_processed,
                result.records_imported,
                result.records_with_errors
            ),
            tenant_id: "default".to_string(),
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: "data_import".to_string(),
            resource_id: result.import_id.clone(),
            resource_type: "import_job".to_string(),
            operation: "import".to_string(),
            details: format!("Data import completed with {} records processed", result.records_processed),
            status: if result.records_with_errors == 0 { "success" } else { "partial_success" }.to_string(),
        }).await?;
        
        Ok(result)
    }
    
    async fn get_import_history(&self) -> Result<Vec<DataImportResult>> {
        // Check if integration is enabled
        if !self.config.enabled {
            return Err(anyhow!("Integration is not enabled"));
        }
        
        // Check if data import is enabled
        if !self.config.data_import.enabled {
            return Err(anyhow!("Data import is not enabled"));
        }
        
        // Get import history
        let import_history = self.import_history.read().await;
        Ok(import_history.clone())
    }
    
    async fn get_import_details(&self, import_id: &str) -> Result<DataImportResult> {
        // Check if integration is enabled
        if !self.config.enabled {
            return Err(anyhow!("Integration is not enabled"));
        }
        
        // Check if data import is enabled
        if !self.config.data_import.enabled {
            return Err(anyhow!("Data import is not enabled"));
        }
        
        // Get import details
        let import_history = self.import_history.read().await;
        let result = import_history.iter()
            .find(|r| r.import_id == import_id)
            .cloned()
            .ok_or_else(|| anyhow!("Import not found: {}", import_id))?;
        
        Ok(result)
    }
} 