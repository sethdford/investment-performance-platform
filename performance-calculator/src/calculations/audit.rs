use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::Mutex;

/// Represents a single calculation event in the audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationEvent {
    /// Unique identifier for this calculation event
    pub event_id: String,
    
    /// Request ID that triggered this calculation
    pub request_id: String,
    
    /// Type of calculation performed
    pub calculation_type: String,
    
    /// Timestamp when the calculation started
    pub start_time: DateTime<Utc>,
    
    /// Timestamp when the calculation completed
    pub end_time: DateTime<Utc>,
    
    /// Duration of the calculation in milliseconds
    pub duration_ms: u64,
    
    /// User or service that initiated the calculation
    pub initiated_by: String,
    
    /// Version of the calculation method used
    pub calculation_version: String,
    
    /// Input parameters used for the calculation
    pub input_parameters: HashMap<String, serde_json::Value>,
    
    /// References to input data used (e.g., file paths, database keys)
    pub input_data_references: Vec<String>,
    
    /// Output data references (e.g., where results were stored)
    pub output_references: Vec<String>,
    
    /// Status of the calculation (success, failure, etc.)
    pub status: CalculationStatus,
    
    /// Error message if the calculation failed
    pub error_message: Option<String>,
    
    /// Parent calculation event ID if this was part of a larger calculation
    pub parent_event_id: Option<String>,
    
    /// Child calculation event IDs if this calculation spawned sub-calculations
    pub child_event_ids: Vec<String>,
    
    /// Additional metadata about the calculation
    pub metadata: HashMap<String, String>,
}

/// Status of a calculation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CalculationStatus {
    /// Calculation is in progress
    InProgress,
    
    /// Calculation completed successfully
    Success,
    
    /// Calculation failed
    Failure,
    
    /// Calculation was cancelled
    Cancelled,
}

/// Builder for creating calculation events
pub struct CalculationEventBuilder {
    event: CalculationEvent,
}

impl CalculationEventBuilder {
    /// Create a new calculation event builder
    pub fn new(calculation_type: &str, request_id: &str, initiated_by: &str) -> Self {
        let now = Utc::now();
        Self {
            event: CalculationEvent {
                event_id: Uuid::new_v4().to_string(),
                request_id: request_id.to_string(),
                calculation_type: calculation_type.to_string(),
                start_time: now,
                end_time: now,
                duration_ms: 0,
                initiated_by: initiated_by.to_string(),
                calculation_version: "1.0".to_string(),
                input_parameters: HashMap::new(),
                input_data_references: Vec::new(),
                output_references: Vec::new(),
                status: CalculationStatus::InProgress,
                error_message: None,
                parent_event_id: None,
                child_event_ids: Vec::new(),
                metadata: HashMap::new(),
            },
        }
    }
    
    /// Set the calculation version
    pub fn with_version(mut self, version: &str) -> Self {
        self.event.calculation_version = version.to_string();
        self
    }
    
    /// Add an input parameter
    pub fn with_input_parameter(mut self, key: &str, value: serde_json::Value) -> Self {
        self.event.input_parameters.insert(key.to_string(), value);
        self
    }
    
    /// Add multiple input parameters
    pub fn with_input_parameters(mut self, params: HashMap<String, serde_json::Value>) -> Self {
        self.event.input_parameters.extend(params);
        self
    }
    
    /// Add an input data reference
    pub fn with_input_data_reference(mut self, reference: &str) -> Self {
        self.event.input_data_references.push(reference.to_string());
        self
    }
    
    /// Add multiple input data references
    pub fn with_input_data_references(mut self, references: Vec<String>) -> Self {
        self.event.input_data_references.extend(references);
        self
    }
    
    /// Set the parent event ID
    pub fn with_parent_event_id(mut self, parent_id: &str) -> Self {
        self.event.parent_event_id = Some(parent_id.to_string());
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.event.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Build the calculation event
    pub fn build(self) -> CalculationEvent {
        self.event
    }
}

/// Interface for audit trail storage
#[async_trait]
pub trait AuditTrailStorage: Send + Sync {
    /// Store a calculation event
    async fn store_event(&self, event: &CalculationEvent) -> Result<()>;
    
    /// Retrieve a calculation event by ID
    async fn get_event(&self, event_id: &str) -> Result<Option<CalculationEvent>>;
    
    /// Retrieve calculation events by request ID
    async fn get_events_by_request_id(&self, request_id: &str) -> Result<Vec<CalculationEvent>>;
    
    /// Retrieve calculation events by calculation type
    async fn get_events_by_calculation_type(&self, calculation_type: &str) -> Result<Vec<CalculationEvent>>;
    
    /// Retrieve child events of a parent event
    async fn get_child_events(&self, parent_event_id: &str) -> Result<Vec<CalculationEvent>>;
}

/// DynamoDB implementation of audit trail storage
pub struct DynamoDbAuditTrailStorage {
    client: aws_sdk_dynamodb::Client,
    table_name: String,
}

impl DynamoDbAuditTrailStorage {
    /// Create a new DynamoDB audit trail storage
    pub fn new(client: aws_sdk_dynamodb::Client, table_name: String) -> Self {
        Self {
            client,
            table_name,
        }
    }
}

#[async_trait]
impl AuditTrailStorage for DynamoDbAuditTrailStorage {
    async fn store_event(&self, event: &CalculationEvent) -> Result<()> {
        // Convert the event to a DynamoDB item directly instead of using serde
        let mut attributes = HashMap::new();
        
        // Add the main event data as individual attributes
        attributes.insert("event_id".to_string(), aws_sdk_dynamodb::types::AttributeValue::S(event.event_id.clone()));
        attributes.insert("request_id".to_string(), aws_sdk_dynamodb::types::AttributeValue::S(event.request_id.clone()));
        attributes.insert("calculation_type".to_string(), aws_sdk_dynamodb::types::AttributeValue::S(event.calculation_type.clone()));
        attributes.insert("start_time".to_string(), aws_sdk_dynamodb::types::AttributeValue::S(event.start_time.to_rfc3339()));
        attributes.insert("end_time".to_string(), aws_sdk_dynamodb::types::AttributeValue::S(event.end_time.to_rfc3339()));
        attributes.insert("duration_ms".to_string(), aws_sdk_dynamodb::types::AttributeValue::N(event.duration_ms.to_string()));
        attributes.insert("initiated_by".to_string(), aws_sdk_dynamodb::types::AttributeValue::S(event.initiated_by.clone()));
        attributes.insert("calculation_version".to_string(), aws_sdk_dynamodb::types::AttributeValue::S(event.calculation_version.clone()));
        
        // Convert input parameters to JSON string
        let input_params_json = serde_json::to_string(&event.input_parameters)
            .map_err(|e| anyhow!("Failed to serialize input parameters: {}", e))?;
        attributes.insert("input_parameters".to_string(), aws_sdk_dynamodb::types::AttributeValue::S(input_params_json));
        
        // Convert input data references to string set
        let input_refs: Vec<String> = event.input_data_references.clone();
        if !input_refs.is_empty() {
            attributes.insert("input_data_references".to_string(), 
                aws_sdk_dynamodb::types::AttributeValue::Ss(input_refs));
        } else {
            attributes.insert("input_data_references".to_string(), 
                aws_sdk_dynamodb::types::AttributeValue::Ss(vec![]));
        }
        
        // Convert output references to string set
        let output_refs: Vec<String> = event.output_references.clone();
        if !output_refs.is_empty() {
            attributes.insert("output_references".to_string(), 
                aws_sdk_dynamodb::types::AttributeValue::Ss(output_refs));
        } else {
            attributes.insert("output_references".to_string(), 
                aws_sdk_dynamodb::types::AttributeValue::Ss(vec![]));
        }
        
        // Convert status to string
        let status_str = match event.status {
            CalculationStatus::InProgress => "InProgress",
            CalculationStatus::Success => "Success",
            CalculationStatus::Failure => "Failure",
            CalculationStatus::Cancelled => "Cancelled",
        };
        attributes.insert("status".to_string(), aws_sdk_dynamodb::types::AttributeValue::S(status_str.to_string()));
        
        // Add error message if present
        if let Some(error_msg) = &event.error_message {
            attributes.insert("error_message".to_string(), aws_sdk_dynamodb::types::AttributeValue::S(error_msg.clone()));
        }
        
        // Add parent event ID if present
        if let Some(parent_id) = &event.parent_event_id {
            attributes.insert("parent_event_id".to_string(), aws_sdk_dynamodb::types::AttributeValue::S(parent_id.clone()));
        }
        
        // Convert child event IDs to string set
        let child_ids: Vec<String> = event.child_event_ids.clone();
        if !child_ids.is_empty() {
            attributes.insert("child_event_ids".to_string(), 
                aws_sdk_dynamodb::types::AttributeValue::Ss(child_ids));
        } else {
            attributes.insert("child_event_ids".to_string(), 
                aws_sdk_dynamodb::types::AttributeValue::Ss(vec![]));
        }
        
        // Convert metadata to map of strings
        let mut metadata_map = HashMap::new();
        for (key, value) in &event.metadata {
            metadata_map.insert(key.clone(), aws_sdk_dynamodb::types::AttributeValue::S(value.clone()));
        }
        if !metadata_map.is_empty() {
            attributes.insert("metadata".to_string(), aws_sdk_dynamodb::types::AttributeValue::M(metadata_map));
        } else {
            attributes.insert("metadata".to_string(), aws_sdk_dynamodb::types::AttributeValue::M(HashMap::new()));
        }
        
        // Store the item in DynamoDB
        self.client.put_item()
            .table_name(&self.table_name)
            .set_item(Some(attributes))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to store calculation event in DynamoDB: {}", e))?;
        
        Ok(())
    }
    
    async fn get_event(&self, event_id: &str) -> Result<Option<CalculationEvent>> {
        let response = self.client.get_item()
            .table_name(&self.table_name)
            .key("event_id", aws_sdk_dynamodb::types::AttributeValue::S(event_id.to_string()))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to retrieve calculation event from DynamoDB: {}", e))?;
        
        if let Some(item) = response.item() {
            return Ok(Some(self.item_to_calculation_event(item)?));
        }
        
        Ok(None)
    }
    
    async fn get_events_by_request_id(&self, request_id: &str) -> Result<Vec<CalculationEvent>> {
        let response = self.client.query()
            .table_name(&self.table_name)
            .index_name("request_id-index")
            .key_condition_expression("request_id = :request_id")
            .expression_attribute_values(":request_id", aws_sdk_dynamodb::types::AttributeValue::S(request_id.to_string()))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to query calculation events by request ID: {}", e))?;
        
        let mut events = Vec::new();
        
        let items = response.items();
        for item in items {
            events.push(self.item_to_calculation_event(item)?);
        }
        
        Ok(events)
    }
    
    async fn get_events_by_calculation_type(&self, calculation_type: &str) -> Result<Vec<CalculationEvent>> {
        let response = self.client.query()
            .table_name(&self.table_name)
            .index_name("calculation_type-index")
            .key_condition_expression("calculation_type = :calculation_type")
            .expression_attribute_values(":calculation_type", aws_sdk_dynamodb::types::AttributeValue::S(calculation_type.to_string()))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to query calculation events by calculation type: {}", e))?;
        
        let mut events = Vec::new();
        
        let items = response.items();
        for item in items {
            events.push(self.item_to_calculation_event(item)?);
        }
        
        Ok(events)
    }
    
    async fn get_child_events(&self, parent_event_id: &str) -> Result<Vec<CalculationEvent>> {
        // This would typically use a query with a filter expression
        // For simplicity, we'll scan the table and filter client-side
        let response = self.client.scan()
            .table_name(&self.table_name)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to scan calculation events: {}", e))?;
        
        let mut events = Vec::new();
        
        let items = response.items();
        for item in items {
            let event = self.item_to_calculation_event(item)?;
            
            if let Some(parent_id) = &event.parent_event_id {
                if parent_id == parent_event_id {
                    events.push(event);
                }
            }
        }
        
        Ok(events)
    }
}

impl DynamoDbAuditTrailStorage {
    // Helper method to convert a DynamoDB item to a CalculationEvent
    fn item_to_calculation_event(&self, item: &std::collections::HashMap<String, aws_sdk_dynamodb::types::AttributeValue>) -> Result<CalculationEvent> {
        // Extract required string fields
        let event_id = self.get_string_attribute(item, "event_id")?;
        let request_id = self.get_string_attribute(item, "request_id")?;
        let calculation_type = self.get_string_attribute(item, "calculation_type")?;
        let initiated_by = self.get_string_attribute(item, "initiated_by")?;
        let calculation_version = self.get_string_attribute(item, "calculation_version")?;
        
        // Parse timestamps
        let start_time_str = self.get_string_attribute(item, "start_time")?;
        let start_time = DateTime::parse_from_rfc3339(&start_time_str)
            .map_err(|e| anyhow!("Failed to parse start_time: {}", e))?
            .with_timezone(&Utc);
        
        let end_time_str = self.get_string_attribute(item, "end_time")?;
        let end_time = DateTime::parse_from_rfc3339(&end_time_str)
            .map_err(|e| anyhow!("Failed to parse end_time: {}", e))?
            .with_timezone(&Utc);
        
        // Parse duration
        let duration_str = self.get_string_attribute(item, "duration_ms")?;
        let duration_ms = duration_str.parse::<u64>()
            .map_err(|e| anyhow!("Failed to parse duration_ms: {}", e))?;
        
        // Parse input parameters from JSON string
        let input_params_json = self.get_string_attribute(item, "input_parameters")?;
        let input_parameters: HashMap<String, serde_json::Value> = serde_json::from_str(&input_params_json)
            .map_err(|e| anyhow!("Failed to parse input parameters: {}", e))?;
        
        // Get string sets
        let input_data_references = self.get_string_set_attribute(item, "input_data_references")?;
        let output_references = self.get_string_set_attribute(item, "output_references")?;
        let child_event_ids = self.get_string_set_attribute(item, "child_event_ids")?;
        
        // Parse status
        let status_str = self.get_string_attribute(item, "status")?;
        let status = match status_str.as_str() {
            "InProgress" => CalculationStatus::InProgress,
            "Success" => CalculationStatus::Success,
            "Failure" => CalculationStatus::Failure,
            "Cancelled" => CalculationStatus::Cancelled,
            _ => return Err(anyhow!("Invalid calculation status: {}", status_str)),
        };
        
        // Get optional fields
        let error_message = self.get_optional_string_attribute(item, "error_message");
        let parent_event_id = self.get_optional_string_attribute(item, "parent_event_id");
        
        // Parse metadata
        let metadata = self.get_string_map_attribute(item, "metadata")?;
        
        Ok(CalculationEvent {
            event_id,
            request_id,
            calculation_type,
            start_time,
            end_time,
            duration_ms,
            initiated_by,
            calculation_version,
            input_parameters,
            input_data_references,
            output_references,
            status,
            error_message,
            parent_event_id,
            child_event_ids,
            metadata,
        })
    }
    
    // Helper to get a string attribute
    fn get_string_attribute(&self, item: &std::collections::HashMap<String, aws_sdk_dynamodb::types::AttributeValue>, key: &str) -> Result<String> {
        match item.get(key) {
            Some(aws_sdk_dynamodb::types::AttributeValue::S(value)) => Ok(value.clone()),
            Some(aws_sdk_dynamodb::types::AttributeValue::N(value)) => Ok(value.clone()),
            _ => Err(anyhow!("Missing or invalid string attribute: {}", key)),
        }
    }
    
    // Helper to get an optional string attribute
    fn get_optional_string_attribute(&self, item: &std::collections::HashMap<String, aws_sdk_dynamodb::types::AttributeValue>, key: &str) -> Option<String> {
        match item.get(key) {
            Some(aws_sdk_dynamodb::types::AttributeValue::S(value)) => Some(value.clone()),
            _ => None,
        }
    }
    
    // Helper to get a string set attribute
    fn get_string_set_attribute(&self, item: &std::collections::HashMap<String, aws_sdk_dynamodb::types::AttributeValue>, key: &str) -> Result<Vec<String>> {
        match item.get(key) {
            Some(aws_sdk_dynamodb::types::AttributeValue::Ss(values)) => Ok(values.clone()),
            Some(aws_sdk_dynamodb::types::AttributeValue::L(values)) => {
                let mut result = Vec::new();
                for value in values {
                    if let aws_sdk_dynamodb::types::AttributeValue::S(s) = value {
                        result.push(s.clone());
                    }
                }
                Ok(result)
            },
            _ => Ok(Vec::new()), // Return empty vector if not found or invalid
        }
    }
    
    // Helper to get a string map attribute
    fn get_string_map_attribute(&self, item: &std::collections::HashMap<String, aws_sdk_dynamodb::types::AttributeValue>, key: &str) -> Result<HashMap<String, String>> {
        let mut result = HashMap::new();
        
        if let Some(aws_sdk_dynamodb::types::AttributeValue::M(map)) = item.get(key) {
            for (k, v) in map {
                if let aws_sdk_dynamodb::types::AttributeValue::S(s) = v {
                    result.insert(k.clone(), s.clone());
                }
            }
        }
        
        Ok(result)
    }
}

/// In-memory implementation of audit trail storage (for testing)
pub struct InMemoryAuditTrailStorage {
    events: std::sync::Mutex<Vec<CalculationEvent>>,
}

impl InMemoryAuditTrailStorage {
    /// Create a new in-memory audit trail storage
    pub fn new() -> Self {
        Self {
            events: std::sync::Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl AuditTrailStorage for InMemoryAuditTrailStorage {
    async fn store_event(&self, event: &CalculationEvent) -> Result<()> {
        let mut events = self.events.lock().unwrap();
        events.push(event.clone());
        Ok(())
    }
    
    async fn get_event(&self, event_id: &str) -> Result<Option<CalculationEvent>> {
        let events = self.events.lock().unwrap();
        let event = events.iter().find(|e| e.event_id == event_id).cloned();
        Ok(event)
    }
    
    async fn get_events_by_request_id(&self, request_id: &str) -> Result<Vec<CalculationEvent>> {
        let events = self.events.lock().unwrap();
        let filtered_events = events.iter()
            .filter(|e| e.request_id == request_id)
            .cloned()
            .collect();
        Ok(filtered_events)
    }
    
    async fn get_events_by_calculation_type(&self, calculation_type: &str) -> Result<Vec<CalculationEvent>> {
        let events = self.events.lock().unwrap();
        let filtered_events = events.iter()
            .filter(|e| e.calculation_type == calculation_type)
            .cloned()
            .collect();
        Ok(filtered_events)
    }
    
    async fn get_child_events(&self, parent_event_id: &str) -> Result<Vec<CalculationEvent>> {
        let events = self.events.lock().unwrap();
        let filtered_events = events.iter()
            .filter(|e| e.parent_event_id.as_deref() == Some(parent_event_id))
            .cloned()
            .collect();
        Ok(filtered_events)
    }
}

/// Audit trail manager
pub struct AuditTrailManager {
    storage: Arc<dyn AuditTrailStorage>,
}

impl AuditTrailManager {
    /// Create a new audit trail manager
    pub fn new(storage: Arc<dyn AuditTrailStorage>) -> Self {
        Self {
            storage,
        }
    }
    
    /// Start tracking a calculation
    pub async fn start_calculation(
        &self,
        calculation_type: &str,
        request_id: &str,
        initiated_by: &str,
        input_parameters: HashMap<String, serde_json::Value>,
        input_data_references: Vec<String>,
    ) -> Result<CalculationEvent> {
        let event = CalculationEventBuilder::new(calculation_type, request_id, initiated_by)
            .with_input_parameters(input_parameters)
            .with_input_data_references(input_data_references)
            .build();
        
        self.storage.store_event(&event).await?;
        
        Ok(event)
    }
    
    /// Complete a calculation successfully
    pub async fn complete_calculation(
        &self,
        event_id: &str,
        output_references: Vec<String>,
    ) -> Result<CalculationEvent> {
        let mut event = match self.storage.get_event(event_id).await? {
            Some(e) => e,
            None => return Err(anyhow!("Calculation event not found: {}", event_id)),
        };
        
        event.end_time = Utc::now();
        event.duration_ms = (event.end_time - event.start_time).num_milliseconds() as u64;
        event.status = CalculationStatus::Success;
        event.output_references = output_references;
        
        self.storage.store_event(&event).await?;
        
        Ok(event)
    }
    
    /// Mark a calculation as failed
    pub async fn fail_calculation(
        &self,
        event_id: &str,
        error_message: &str,
    ) -> Result<CalculationEvent> {
        let mut event = match self.storage.get_event(event_id).await? {
            Some(e) => e,
            None => return Err(anyhow!("Calculation event not found: {}", event_id)),
        };
        
        event.end_time = Utc::now();
        event.duration_ms = (event.end_time - event.start_time).num_milliseconds() as u64;
        event.status = CalculationStatus::Failure;
        event.error_message = Some(error_message.to_string());
        
        self.storage.store_event(&event).await?;
        
        Ok(event)
    }
    
    /// Add a child calculation to a parent calculation
    pub async fn add_child_calculation(
        &self,
        parent_event_id: &str,
        child_event_id: &str,
    ) -> Result<()> {
        let mut parent_event = match self.storage.get_event(parent_event_id).await? {
            Some(e) => e,
            None => return Err(anyhow!("Parent calculation event not found: {}", parent_event_id)),
        };
        
        let mut child_event = match self.storage.get_event(child_event_id).await? {
            Some(e) => e,
            None => return Err(anyhow!("Child calculation event not found: {}", child_event_id)),
        };
        
        // Update parent with child ID
        if !parent_event.child_event_ids.contains(&child_event_id.to_string()) {
            parent_event.child_event_ids.push(child_event_id.to_string());
            self.storage.store_event(&parent_event).await?;
        }
        
        // Update child with parent ID
        child_event.parent_event_id = Some(parent_event_id.to_string());
        self.storage.store_event(&child_event).await?;
        
        Ok(())
    }
    
    /// Get the full calculation lineage (parent and all descendants)
    pub async fn get_calculation_lineage(&self, event_id: &str) -> Result<Vec<CalculationEvent>> {
        let mut lineage = Vec::new();
        let mut to_process = vec![event_id.to_string()];
        
        while let Some(current_id) = to_process.pop() {
            if let Some(event) = self.storage.get_event(&current_id).await? {
                lineage.push(event.clone());
                
                // Add child events to processing queue
                to_process.extend(event.child_event_ids.clone());
            }
        }
        
        Ok(lineage)
    }
}

/// Represents an audit record for tracking calculation events
#[derive(Debug, Clone)]
pub struct AuditRecord {
    pub id: String,
    pub entity_id: String,
    pub entity_type: String,
    pub action: String,
    pub user_id: String,
    pub parameters: String,
    pub result: String,
    pub timestamp: DateTime<Utc>,
    pub tenant_id: String,
    pub event_id: String,
    pub event_type: String,
    pub resource_id: String,
    pub resource_type: String,
    pub operation: String,
    pub details: String,
    pub status: String,
}

/// Trait for audit trail functionality
#[async_trait]
pub trait AuditTrail: Send + Sync {
    /// Records an audit event
    async fn record(&self, record: AuditRecord) -> Result<()>;
    
    /// Retrieves audit events for a specific resource
    async fn get_events_for_resource(&self, resource_id: &str) -> Result<Vec<AuditRecord>>;
    
    /// Retrieves audit events for a specific tenant
    async fn get_events_for_tenant(&self, tenant_id: &str) -> Result<Vec<AuditRecord>>;
}

/// In-memory implementation of AuditTrail for testing
pub struct InMemoryAuditTrail {
    records: Arc<Mutex<Vec<AuditRecord>>>,
}

impl InMemoryAuditTrail {
    pub fn new() -> Self {
        Self {
            records: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Get all records stored in the audit trail
    pub async fn get_records(&self) -> Result<Vec<AuditRecord>> {
        let records = self.records.lock().await;
        Ok(records.clone())
    }
}

#[async_trait]
impl AuditTrail for InMemoryAuditTrail {
    async fn record(&self, record: AuditRecord) -> Result<()> {
        let mut records = self.records.lock().await;
        records.push(record);
        Ok(())
    }
    
    async fn get_events_for_resource(&self, resource_id: &str) -> Result<Vec<AuditRecord>> {
        let records = self.records.lock().await;
        let filtered_records = records
            .iter()
            .filter(|r| r.resource_id == resource_id)
            .cloned()
            .collect();
        Ok(filtered_records)
    }
    
    async fn get_events_for_tenant(&self, tenant_id: &str) -> Result<Vec<AuditRecord>> {
        let records = self.records.lock().await;
        let filtered_records = records
            .iter()
            .filter(|r| r.tenant_id == tenant_id)
            .cloned()
            .collect();
        Ok(filtered_records)
    }
}

/// DynamoDB implementation of AuditTrail
pub struct DynamoDbAuditTrail {
    // Implementation details would go here
}

impl DynamoDbAuditTrail {
    pub fn new(client: Arc<aws_sdk_dynamodb::Client>, table_name: String) -> Self {
        Self {}
    }
}

#[async_trait]
impl AuditTrail for DynamoDbAuditTrail {
    async fn record(&self, _record: AuditRecord) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }
    
    async fn get_events_for_resource(&self, _resource_id: &str) -> Result<Vec<AuditRecord>> {
        // Placeholder implementation
        Ok(vec![])
    }
    
    async fn get_events_for_tenant(&self, _tenant_id: &str) -> Result<Vec<AuditRecord>> {
        // Placeholder implementation
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_calculation_event_builder() {
        let event = CalculationEventBuilder::new("test_calculation", "req-123", "test_user")
            .with_version("2.0")
            .with_input_parameter("param1", serde_json::json!(42))
            .with_input_data_reference("data1")
            .with_metadata("source", "unit_test")
            .build();
        
        assert_eq!(event.calculation_type, "test_calculation");
        assert_eq!(event.request_id, "req-123");
        assert_eq!(event.initiated_by, "test_user");
        assert_eq!(event.calculation_version, "2.0");
        assert_eq!(event.input_parameters.get("param1"), Some(&serde_json::json!(42)));
        assert_eq!(event.input_data_references, vec!["data1"]);
        assert_eq!(event.metadata.get("source"), Some(&"unit_test".to_string()));
        assert_eq!(event.status, CalculationStatus::InProgress);
    }
    
    #[tokio::test]
    async fn test_in_memory_audit_trail() {
        let storage = Arc::new(InMemoryAuditTrailStorage::new());
        let manager = AuditTrailManager::new(storage.clone());
        
        // Start a calculation
        let mut input_params = HashMap::new();
        input_params.insert("param1".to_string(), serde_json::json!(42));
        
        let event = manager.start_calculation(
            "test_calculation",
            "req-123",
            "test_user",
            input_params,
            vec!["data1".to_string()],
        ).await.unwrap();
        
        // Verify the event was stored
        let stored_event = storage.get_event(&event.event_id).await.unwrap().unwrap();
        assert_eq!(stored_event.calculation_type, "test_calculation");
        assert_eq!(stored_event.status, CalculationStatus::InProgress);
        
        // Complete the calculation
        let completed_event = manager.complete_calculation(
            &event.event_id,
            vec!["output1".to_string()],
        ).await.unwrap();
        
        assert_eq!(completed_event.status, CalculationStatus::Success);
        assert_eq!(completed_event.output_references, vec!["output1"]);
        assert!(completed_event.duration_ms >= 0);
        
        // Test query by request ID
        let events = storage.get_events_by_request_id("req-123").await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_id, event.event_id);
    }
} 