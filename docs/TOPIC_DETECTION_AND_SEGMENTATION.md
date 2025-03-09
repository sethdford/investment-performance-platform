# Topic Detection and Segmentation

This document provides a comprehensive overview of the topic detection and segmentation capabilities implemented in the conversational financial advisor platform.

## Overview

Effective topic detection and segmentation are crucial for maintaining coherent and contextually relevant conversations, especially in complex domains like financial advising. Our enhanced system addresses several key challenges:

1. **Topic Detection**: Identifying financial topics discussed in conversations
2. **Topic Shift Detection**: Recognizing when the conversation shifts from one topic to another
3. **Conversation Segmentation**: Dividing conversations into meaningful segments based on topics and other signals
4. **Segment Importance Scoring**: Prioritizing segments based on their relevance and importance
5. **Segment Summarization**: Creating concise summaries of conversation segments

## Key Components

### FinancialTopicCategory

The `FinancialTopicCategory` enum represents different financial topics that can be detected in conversations:

```rust
pub enum FinancialTopicCategory {
    RetirementPlanning,
    Investment,
    TaxPlanning,
    EstatePlanning,
    Insurance,
    Budgeting,
    DebtManagement,
    EducationPlanning,
    HomeOwnership,
    IncomeGeneration,
    CharitableGiving,
    HealthcarePlanning,
    RiskManagement,
    AssetAllocation,
    Other(String),
}
```

Each category has associated methods:
- `display_name()`: Returns a user-friendly name for the category
- `description()`: Provides a detailed description of the category
- `default_importance()`: Assigns a default importance level to the category

### TopicDetectorConfig

The `TopicDetectorConfig` struct provides configuration options for the topic detector:

```rust
pub struct TopicDetectorConfig {
    /// Minimum confidence score to consider a topic detected (0.0 to 1.0)
    pub min_confidence: f32,
    /// Maximum number of topics to track
    pub max_topics: usize,
    /// Whether to use LLM for topic detection
    pub use_llm: bool,
    /// Whether to use rule-based detection as fallback
    pub use_rule_based_fallback: bool,
    /// Threshold for detecting a topic shift (0.0 to 1.0)
    pub topic_shift_threshold: f32,
    /// Number of messages to consider for topic shift detection
    pub topic_shift_window: usize,
}
```

### TopicShift

The `TopicShift` struct represents a detected shift in conversation topics:

```rust
pub struct TopicShift {
    /// Unique identifier for the topic shift
    pub id: String,
    /// Message index where the shift occurred
    pub message_index: usize,
    /// Previous dominant topic
    pub previous_topic: Option<String>,
    /// New dominant topic
    pub new_topic: String,
    /// Confidence score for the shift detection (0.0 to 1.0)
    pub confidence: f32,
    /// Timestamp when the shift was detected
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
```

### TopicDetector

The `TopicDetector` struct is the main component responsible for detecting topics and topic shifts:

```rust
pub struct TopicDetector {
    /// Configuration for the topic detector
    config: TopicDetectorConfig,
    /// Known financial topics with their keywords
    topic_keywords: HashMap<FinancialTopicCategory, Vec<String>>,
    /// Currently active topics
    active_topics: HashMap<String, ConversationTopic>,
    /// Message counter for tracking when topics are mentioned
    message_counter: usize,
    /// Recent topic distributions for detecting shifts
    recent_topic_distributions: VecDeque<HashMap<String, f32>>,
    /// Detected topic shifts
    topic_shifts: Vec<TopicShift>,
}
```

## Topic Detection

The topic detection system uses a hybrid approach combining rule-based detection and (optionally) LLM-based detection:

### Rule-Based Detection

The rule-based detection uses keyword matching to identify topics in messages:

1. Each `FinancialTopicCategory` has an associated list of keywords
2. When a message is processed, the system checks if any keywords are present
3. If a keyword is found, the corresponding topic is detected
4. Topics are assigned importance levels based on their category

Example keywords for Retirement Planning:
```
"retirement", "retire", "401k", "401(k)", "ira", "roth", "pension", "social security", "annuity", "retirement age", "retirement income", "retirement plan", "retirement account", "retirement savings", "early retirement", "required minimum distribution", "rmd"
```

### LLM-Based Detection

The system also supports LLM-based topic detection, which can be more sophisticated:

1. The message is sent to an LLM with a prompt to identify financial topics
2. The LLM returns detected topics with confidence scores
3. Topics with confidence scores above the threshold are considered detected

## Topic Shift Detection

Topic shift detection identifies when the conversation moves from one topic to another:

1. The system maintains a sliding window of recent topic distributions
2. Each distribution maps topic names to their normalized importance scores
3. When a new message is processed, the system compares the current distribution to previous ones
4. If the dominant topic changes and the confidence exceeds the threshold, a topic shift is detected

The confidence score for a topic shift is calculated based on the difference in topic scores between distributions.

## Conversation Segmentation

Conversation segmentation divides the conversation into meaningful segments based on several factors:

### Segmentation Triggers

A new segment is created when:

1. **Topic Shift**: A significant change in the dominant topic is detected
2. **Time-Based**: A certain number of messages have accumulated since the last segment
3. **State Change**: The conversation state changes (e.g., from information gathering to advising)
4. **Explicit Indicators**: The user explicitly changes the subject or introduces a new topic

### Segment Creation

When a new segment is created:

1. The system determines how many messages to include based on the segmentation trigger
2. For topic shifts, fewer messages are included to keep segments focused
3. For regular segments, more messages are included for context
4. The segment includes the messages, topics, and entities
5. An importance score is calculated based on the topics and entities
6. A summary is generated for the segment

### Segment Importance Scoring

Segment importance is calculated based on:

1. The importance of the topics discussed in the segment
2. The number and significance of financial entities mentioned
3. The recency of the segment (newer segments are more important)
4. The relevance of the segment to the current conversation

## Segment Summarization

Each segment can have a summary that concisely describes its content:

1. In a production system, this would use an LLM to generate a summary
2. In the current implementation, a simple summary is created based on topics and entities
3. The summary includes the main topics discussed and key financial entities mentioned

Example summary:
```
"Discussion about Retirement Planning, 401(k) involving 401(k) ($250,000.00), Annual Income ($120,000.00)"
```

## Integration with Context Management

The topic detection and segmentation system is tightly integrated with the context management system:

1. Detected topics influence which segments are included in the context window
2. Topic shifts trigger the creation of new segments
3. Segment importance scores help prioritize which segments to include in the context window
4. Segment summaries provide concise representations of conversation history

## Usage Examples

### Basic Topic Detection

```rust
let mut detector = TopicDetector::new();
let message = Message::from_user("I'm planning for retirement and want to save for my 401k");
let topics = detector.detect_topics(&message);

// Topics will contain "Retirement Planning"
```

### Topic Shift Detection

```rust
let mut detector = TopicDetector::new();

// First message about retirement
detector.detect_topics(&Message::from_user("I'm planning for retirement"));

// Second message about investments
detector.detect_topics(&Message::from_user("I want to invest in stocks and bonds"));

// Get detected shifts
let shifts = detector.get_topic_shifts();
// shifts will contain a shift from "Retirement Planning" to "Investment"
```

### Conversation Segmentation

```rust
let mut context_manager = ContextManager::new();

// Add messages about retirement
context_manager.add_message(Message::from_user("I'm planning for retirement"));
context_manager.add_message(Message::from_assistant("That's great! When do you plan to retire?"));

// Add messages about investments (topic shift)
context_manager.add_message(Message::from_user("I want to invest in stocks and bonds"));
context_manager.add_message(Message::from_assistant("What's your risk tolerance for investments?"));

// The context manager will have created two segments
```

## Best Practices

1. **Configure Appropriately**: Adjust the configuration based on the specific needs of your application.
2. **Balance Sensitivity**: Set the topic shift threshold to balance between too many segments (fragmentation) and too few segments (loss of context).
3. **Customize Keywords**: Add domain-specific keywords to improve topic detection accuracy.
4. **Combine with LLM**: For best results, use both rule-based and LLM-based detection.
5. **Monitor Performance**: Track topic detection accuracy and adjust as needed.

## Future Improvements

1. **Enhanced LLM Integration**: Implement more sophisticated LLM-based topic detection.
2. **Hierarchical Topics**: Support for topic hierarchies and subtopics.
3. **User Feedback**: Incorporate user feedback to improve topic detection accuracy.
4. **Personalized Topics**: Learn user-specific topics and interests over time.
5. **Multi-modal Detection**: Incorporate other signals like sentiment and intent for better segmentation.

## Conclusion

The enhanced topic detection and segmentation system provides a robust foundation for maintaining coherent and contextually relevant conversations in the financial advising domain. By intelligently identifying topics, detecting shifts, and segmenting conversations, the system ensures that the most relevant information is available for generating responses, leading to more accurate and helpful financial advice. 