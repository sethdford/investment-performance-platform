# Enhanced Context Management

This document provides a comprehensive overview of the enhanced context management capabilities implemented in the conversational financial advisor platform.

## Overview

Effective context management is crucial for maintaining coherent and relevant conversations, especially in complex domains like financial advising. Our enhanced context management system addresses several key challenges:

1. **Context Window Management**: Managing the limited context window of LLMs by intelligently selecting the most relevant information.
2. **Relevance Scoring**: Prioritizing information based on its relevance to the current conversation.
3. **Context Persistence**: Saving and loading conversation context to maintain continuity across sessions.

## Key Components

### ContextManagerConfig

The `ContextManagerConfig` struct provides configuration options for the context manager:

```rust
pub struct ContextManagerConfig {
    /// Maximum number of messages to keep in active context
    pub max_context_messages: usize,
    /// Maximum number of tokens to keep in active context
    pub max_context_tokens: usize,
    /// Maximum number of segments to keep
    pub max_segments: usize,
    /// Minimum importance score for a segment to be included in relevant context
    pub min_segment_importance: f32,
    /// Path for persisting context (optional)
    pub persistence_path: Option<String>,
    /// Whether to automatically persist context on changes
    pub auto_persist: bool,
}
```

Default values are provided through the `Default` implementation:

```rust
impl Default for ContextManagerConfig {
    fn default() -> Self {
        Self {
            max_context_messages: 20,
            max_context_tokens: 4000,
            max_segments: 10,
            min_segment_importance: 0.3,
            persistence_path: None,
            auto_persist: false,
        }
    }
}
```

### ImportanceLevel

The `ImportanceLevel` enum represents the importance of topics and context segments:

```rust
pub enum ImportanceLevel {
    Low,
    Medium,
    High,
    Critical,
}
```

Methods are provided to convert between numeric scores and importance levels:

```rust
impl ImportanceLevel {
    /// Convert importance level to a numeric score
    pub fn to_score(&self) -> f32 {
        match self {
            ImportanceLevel::Low => 0.25,
            ImportanceLevel::Medium => 0.5,
            ImportanceLevel::High => 0.75,
            ImportanceLevel::Critical => 1.0,
        }
    }
    
    /// Create from a numeric score
    pub fn from_score(score: f32) -> Self {
        if score < 0.3 {
            ImportanceLevel::Low
        } else if score < 0.6 {
            ImportanceLevel::Medium
        } else if score < 0.9 {
            ImportanceLevel::High
        } else {
            ImportanceLevel::Critical
        }
    }
}
```

### ConversationTopic

The `ConversationTopic` struct represents a topic discussed in the conversation:

```rust
pub struct ConversationTopic {
    /// Unique identifier for the topic
    pub id: String,
    /// Name of the topic
    pub name: String,
    /// Description of the topic
    pub description: String,
    /// Importance level of the topic
    pub importance: ImportanceLevel,
    /// When the topic was first mentioned (message index)
    pub first_mentioned: usize,
    /// When the topic was last mentioned (message index)
    pub last_mentioned: usize,
    /// Related financial entities
    pub related_entities: Vec<FinancialEntity>,
    /// Recency score (0.0 to 1.0) - higher means more recent
    pub recency_score: f32,
    /// Relevance score (0.0 to 1.0) - higher means more relevant to current context
    pub relevance_score: f32,
}
```

Key methods include:

- `new`: Creates a new conversation topic
- `update_mention`: Updates the topic with a new mention
- `add_related_entity`: Adds a related entity to the topic
- `calculate_overall_score`: Calculates the overall importance score

### ContextSegment

The `ContextSegment` struct represents a segment of the conversation:

```rust
pub struct ContextSegment {
    /// Unique identifier for the segment
    pub id: String,
    /// Messages in this segment
    pub messages: Vec<Message>,
    /// Topics discussed in this segment
    pub topics: Vec<String>,
    /// Importance score (0.0 to 1.0)
    pub importance_score: f32,
    /// Financial entities mentioned in this segment
    pub entities: Vec<FinancialEntity>,
    /// Segment summary
    pub summary: Option<String>,
    /// Recency score (0.0 to 1.0) - higher means more recent
    pub recency_score: f32,
    /// Relevance score (0.0 to 1.0) - higher means more relevant to current context
    pub relevance_score: f32,
    /// Approximate token count for this segment
    pub token_count: usize,
}
```

Key methods include:

- `new`: Creates a new context segment
- `estimate_token_count`: Estimates the token count for a set of messages
- `calculate_overall_score`: Calculates the overall importance score
- `set_summary`: Updates the segment with a summary
- `update_relevance`: Updates the segment's relevance score

### ContextManager

The `ContextManager` struct is the main component that manages conversation context:

```rust
pub struct ContextManager {
    /// Configuration for the context manager
    config: ContextManagerConfig,
    /// Active messages in the conversation (sliding window)
    active_messages: VecDeque<Message>,
    /// Segments of the conversation
    segments: Vec<ContextSegment>,
    /// Topics detected in the conversation
    topics: HashMap<String, ConversationTopic>,
    /// Financial entities mentioned in the conversation
    entities: HashMap<String, FinancialEntity>,
    /// Global conversation summary
    summary: Option<String>,
    /// Current conversation state
    state: ConversationState,
    /// Total message count (including pruned messages)
    total_message_count: usize,
    /// Conversation ID for persistence
    conversation_id: String,
}
```

## Context Window Management

The context window management system ensures that the most relevant information is included in the context window, while staying within the token limits of the LLM.

### Key Features

1. **Token-Based Pruning**: Messages are pruned based on token count, not just message count.
2. **Segment-Based Organization**: Conversations are organized into segments based on topic shifts.
3. **Prioritization**: Messages are prioritized based on recency, relevance, and importance.

### Implementation

The `prune_context` method removes messages when the context exceeds the maximum limits:

```rust
fn prune_context(&mut self) {
    // Remove messages if we exceed the maximum
    while self.active_messages.len() > self.config.max_context_messages {
        self.active_messages.pop_front();
    }
    
    // Calculate total tokens in active context
    let total_tokens: usize = self.active_messages.iter()
        .map(|m| m.content.len() / 4 + 1) // Simple token estimation
        .sum();
        
    // Remove oldest messages if we exceed token limit
    while total_tokens > self.config.max_context_tokens && !self.active_messages.is_empty() {
        if let Some(oldest) = self.active_messages.pop_front() {
            let tokens = oldest.content.len() / 4 + 1;
            // This is not entirely accurate as we're recalculating inside the loop,
            // but it's a simple approach for now
        }
    }
}
```

## Relevance Scoring

The relevance scoring system ensures that the most relevant information is prioritized in the context window.

### Key Features

1. **Topic-Based Scoring**: Topics are scored based on importance, recency, and relevance.
2. **Entity-Based Scoring**: Entities are used to determine relevance between segments.
3. **Segment Scoring**: Segments are scored based on their topics, entities, and recency.

### Implementation

The `get_relevant_context` method selects the most relevant messages for the current context:

```rust
pub fn get_relevant_context(&self) -> Vec<Message> {
    // If we have few messages, just return all of them
    if self.active_messages.len() <= self.config.max_context_messages / 2 {
        return self.get_active_context();
    }
    
    // Get current topics and entities
    let current_topics: Vec<String> = self.get_current_topics()
        .iter()
        .map(|t| t.name.clone())
        .collect();
        
    let current_entities: Vec<FinancialEntity> = self.get_entities();
    
    // Score segments by relevance to current context
    let mut scored_segments: Vec<(usize, f32)> = self.segments.iter()
        .enumerate()
        .map(|(i, segment)| {
            let mut segment_clone = segment.clone();
            let relevance = segment_clone.update_relevance(&current_topics, &current_entities);
            (i, segment.calculate_overall_score() * relevance)
        })
        .filter(|(_, score)| *score >= self.config.min_segment_importance)
        .collect();
    
    // Sort by score (descending)
    scored_segments.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    // Collect messages from top segments up to token limit
    let mut relevant_messages = Vec::new();
    let mut token_count = 0;
    
    // Always include the most recent messages
    let recent_messages: Vec<Message> = self.active_messages.iter()
        .rev()
        .take(5)
        .cloned()
        .collect();
        
    for msg in recent_messages.iter().rev() {
        relevant_messages.push(msg.clone());
        token_count += msg.content.len() / 4 + 1; // Simple token estimation
    }
    
    // Add messages from relevant segments
    for (segment_idx, _) in scored_segments {
        let segment = &self.segments[segment_idx];
        
        // Skip if adding this segment would exceed token limit
        if token_count + segment.token_count > self.config.max_context_tokens {
            continue;
        }
        
        // Add messages from this segment
        for msg in &segment.messages {
            // Skip if message is already included
            if !relevant_messages.iter().any(|m| m.id == msg.id) {
                relevant_messages.push(msg.clone());
                token_count += msg.content.len() / 4 + 1;
            }
        }
        
        // Stop if we've reached the token limit
        if token_count >= self.config.max_context_tokens {
            break;
        }
    }
    
    // Sort messages by timestamp
    relevant_messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    
    relevant_messages
}
```

## Context Persistence

The context persistence system ensures that conversation context can be saved and loaded across sessions.

### Key Features

1. **File-Based Storage**: Context is saved to and loaded from JSON files.
2. **Auto-Persistence**: Context can be automatically persisted on changes.
3. **Conversation ID**: Each conversation has a unique ID for persistence.

### Implementation

The `persist_to_file` method saves the context to a file:

```rust
pub fn persist_to_file(&self, path: &str) -> Result<(), std::io::Error> {
    // Create directory if it doesn't exist
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Open file for writing
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    
    // Serialize and write
    serde_json::to_writer(writer, self)?;
    
    Ok(())
}
```

The `load_from_file` method loads the context from a file:

```rust
pub fn load_from_file(path: &str) -> Result<Self, std::io::Error> {
    // Open file for reading
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    // Deserialize
    let context_manager: Self = serde_json::from_reader(reader)?;
    
    Ok(context_manager)
}
```

## Usage Examples

### Basic Usage

```rust
// Create a context manager with default settings
let mut context_manager = ContextManager::new();

// Add a message
let message = Message::from_user("Hello, I need help with retirement planning.");
context_manager.add_message(message);

// Get the relevant context
let relevant_context = context_manager.get_relevant_context();
```

### Custom Configuration

```rust
// Create a context manager with custom configuration
let config = ContextManagerConfig {
    max_context_messages: 30,
    max_context_tokens: 6000,
    max_segments: 15,
    min_segment_importance: 0.2,
    persistence_path: Some("conversations/context_123.json".to_string()),
    auto_persist: true,
};

let mut context_manager = ContextManager::with_config(config);
```

### Persistence

```rust
// Save context to a file
context_manager.persist_to_file("conversations/context_123.json")?;

// Load context from a file
let loaded_context = ContextManager::load_from_file("conversations/context_123.json")?;
```

## Best Practices

1. **Configure Appropriately**: Adjust the configuration based on the specific needs of your application.
2. **Monitor Token Usage**: Keep an eye on token usage to ensure you're staying within the limits of your LLM.
3. **Use Auto-Persistence**: Enable auto-persistence for critical applications to ensure context is not lost.
4. **Segment Appropriately**: Ensure that segments are created at appropriate points in the conversation.
5. **Balance Recency and Relevance**: Adjust the weights for recency and relevance based on your specific use case.

## Future Improvements

1. **More Sophisticated Token Counting**: Implement more accurate token counting based on the specific LLM being used.
2. **Dynamic Segmentation**: Implement more sophisticated segmentation based on topic shifts and sentiment changes.
3. **Improved Relevance Scoring**: Enhance relevance scoring with more sophisticated algorithms.
4. **Compression**: Implement compression techniques to reduce token usage.
5. **Summarization**: Integrate with the summarization system to create more compact representations of context.

## Conclusion

The enhanced context management system provides a robust foundation for maintaining coherent and relevant conversations in the financial advising domain. By intelligently managing the context window, scoring relevance, and persisting context, the system ensures that the most important information is always available to the LLM, leading to more accurate and helpful responses. 