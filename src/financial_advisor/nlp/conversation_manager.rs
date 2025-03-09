use std::collections::VecDeque;
use chrono::{DateTime, Utc};
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::sync::Arc;

use super::rule_based::{FinancialQueryIntent, ProcessedQuery, ExtractedEntity};
use super::types::{NlpResponse, ClientData};
use crate::financial_advisor::client_profiling::ClientProfile;

/// Maximum number of turns to keep in conversation history
const MAX_HISTORY_TURNS: usize = 10;

/// Conversation turn representing a single exchange between user and system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    /// Unique ID for this turn
    pub id: String,
    
    /// User query
    pub query: String,
    
    /// Processed query with intent and entities
    pub processed_query: Option<ProcessedQuery>,
    
    /// System response
    pub response: Option<NlpResponse>,
    
    /// Timestamp when this turn was created
    pub timestamp: DateTime<Utc>,
}

impl ConversationTurn {
    /// Create a new conversation turn from a user query
    pub fn new(query: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            query,
            processed_query: None,
            response: None,
            timestamp: Utc::now(),
        }
    }
    
    /// Set the processed query for this turn
    pub fn with_processed_query(mut self, processed_query: ProcessedQuery) -> Self {
        self.processed_query = Some(processed_query);
        self
    }
    
    /// Set the response for this turn
    pub fn with_response(mut self, response: NlpResponse) -> Self {
        self.response = Some(response);
        self
    }
}

/// Conversation goal type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConversationGoalType {
    /// Information gathering
    InformationGathering,
    
    /// Goal planning
    GoalPlanning,
    
    /// Portfolio review
    PortfolioReview,
    
    /// Risk assessment
    RiskAssessment,
    
    /// Financial education
    FinancialEducation,
    
    /// Problem resolution
    ProblemResolution,
}

/// Conversation goal status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GoalStatus {
    /// Not started
    NotStarted,
    
    /// In progress
    InProgress,
    
    /// Completed
    Completed,
    
    /// Abandoned
    Abandoned,
}

/// Required information for a conversation goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredInformation {
    /// Information type
    pub info_type: String,
    
    /// Whether this information is required
    pub is_required: bool,
    
    /// Whether this information has been collected
    pub is_collected: bool,
    
    /// The collected value, if any
    pub value: Option<String>,
}

/// Conversation goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationGoal {
    /// Goal ID
    pub id: String,
    
    /// Goal type
    pub goal_type: ConversationGoalType,
    
    /// Goal status
    pub status: GoalStatus,
    
    /// Required information
    pub required_information: Vec<RequiredInformation>,
    
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl ConversationGoal {
    /// Create a new conversation goal
    pub fn new(goal_type: ConversationGoalType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            goal_type,
            status: GoalStatus::NotStarted,
            required_information: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Add required information to this goal
    pub fn add_required_information(&mut self, info_type: &str, is_required: bool) {
        self.required_information.push(RequiredInformation {
            info_type: info_type.to_string(),
            is_required,
            is_collected: false,
            value: None,
        });
        self.updated_at = Utc::now();
    }
    
    /// Set information as collected
    pub fn set_information_collected(&mut self, info_type: &str, value: String) -> Result<()> {
        let info = self.required_information.iter_mut()
            .find(|info| info.info_type == info_type)
            .ok_or_else(|| anyhow!("Information type not found: {}", info_type))?;
        
        info.is_collected = true;
        info.value = Some(value);
        self.updated_at = Utc::now();
        
        // Check if all required information is collected
        let all_required_collected = self.required_information.iter()
            .filter(|info| info.is_required)
            .all(|info| info.is_collected);
        
        if all_required_collected && self.status == GoalStatus::InProgress {
            self.status = GoalStatus::Completed;
        }
        
        Ok(())
    }
    
    /// Start this goal
    pub fn start(&mut self) {
        if self.status == GoalStatus::NotStarted {
            self.status = GoalStatus::InProgress;
            self.updated_at = Utc::now();
        }
    }
    
    /// Complete this goal
    pub fn complete(&mut self) {
        self.status = GoalStatus::Completed;
        self.updated_at = Utc::now();
    }
    
    /// Abandon this goal
    pub fn abandon(&mut self) {
        self.status = GoalStatus::Abandoned;
        self.updated_at = Utc::now();
    }
}

/// Conversation state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConversationState {
    /// Initial greeting
    Greeting,
    
    /// Initial state
    Initial,
    
    /// Information gathering
    InformationGathering,
    
    /// Goal planning
    GoalPlanning,
    
    /// Recommendation
    Recommendation,
    
    /// Explanation
    Explanation,
    
    /// Closing
    Closing,
    
    /// Discussing portfolio performance
    DiscussingPerformance,
    
    /// Discussing retirement planning
    DiscussingRetirement,
    
    /// Discussing financial goals
    DiscussingGoals,
    
    /// Discussing education planning
    DiscussingEducation,
    
    /// Discussing tax optimization
    DiscussingTaxes,
    
    /// Discussing life events
    DiscussingLifeEvents,
    
    /// Providing emotional support
    ProvidingEmotionalSupport,
    
    /// Discussing investments
    DiscussingInvestments,
    
    /// Discussing risk assessment
    DiscussingRisk,
    
    /// Discussing estate planning
    DiscussingEstate,
    
    /// Discussing insurance planning
    DiscussingInsurance,
    
    /// Discussing debt management
    DiscussingDebt,
    
    /// Discussing budget
    DiscussingBudget,
    
    /// Discussing income strategies
    DiscussingIncome,
    
    /// Discussing charitable planning
    DiscussingCharity,
    
    /// Discussing business planning
    DiscussingBusiness,
    
    /// Discussing healthcare planning
    DiscussingHealthcare,
    
    /// Discussing international planning
    DiscussingInternational,
    
    /// Discussing social security optimization
    DiscussingSocialSecurity,
    
    /// Providing help
    ProvidingHelp,
}

/// Conversation manager
#[derive(Debug)]
pub struct ConversationManager {
    /// Conversation ID
    pub id: String,
    
    /// Client ID
    pub client_id: String,
    
    /// Conversation history
    history: VecDeque<ConversationTurn>,
    
    /// Conversation state
    state: ConversationState,
    
    /// Active goals
    active_goals: Vec<ConversationGoal>,
    
    /// Client profile
    client_profile: Option<Arc<ClientProfile>>,
    
    /// Client data
    client_data: Option<Arc<ClientData>>,
    
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl ConversationManager {
    /// Create a new conversation manager
    pub fn new(client_id: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            client_id: client_id.to_string(),
            history: VecDeque::with_capacity(MAX_HISTORY_TURNS),
            state: ConversationState::Greeting,
            active_goals: Vec::new(),
            client_profile: None,
            client_data: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Set the client profile
    pub fn with_client_profile(mut self, profile: Arc<ClientProfile>) -> Self {
        self.client_profile = Some(profile);
        self
    }
    
    /// Set the client data
    pub fn with_client_data(mut self, data: Arc<ClientData>) -> Self {
        self.client_data = Some(data);
        self
    }
    
    /// Add a user query to the conversation
    pub fn add_user_query(&mut self, query: &str) -> &ConversationTurn {
        let turn = ConversationTurn::new(query.to_string());
        
        // Add to history and maintain max size
        self.history.push_back(turn);
        if self.history.len() > MAX_HISTORY_TURNS {
            self.history.pop_front();
        }
        
        self.updated_at = Utc::now();
        self.history.back().unwrap()
    }
    
    /// Update the current turn with a processed query
    pub fn update_current_turn_with_processed_query(&mut self, processed_query: ProcessedQuery) -> Result<()> {
        let turn = self.history.back_mut()
            .ok_or_else(|| anyhow!("No conversation turns available"))?;
        
        turn.processed_query = Some(processed_query);
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// Update the current turn with a response
    pub fn update_current_turn_with_response(&mut self, response: NlpResponse) -> Result<()> {
        let turn = self.history.back_mut()
            .ok_or_else(|| anyhow!("No conversation turns available"))?;
        
        turn.response = Some(response);
        self.updated_at = Utc::now();
        
        Ok(())
    }
    
    /// Get the conversation history
    pub fn get_history(&self) -> &VecDeque<ConversationTurn> {
        &self.history
    }
    
    /// Get the current conversation state
    pub fn get_state(&self) -> &ConversationState {
        &self.state
    }
    
    /// Set the conversation state
    pub fn set_state(&mut self, state: ConversationState) {
        self.state = state;
        self.updated_at = Utc::now();
    }
    
    /// Add a new goal to the conversation
    pub fn add_goal(&mut self, goal_type: ConversationGoalType) -> &ConversationGoal {
        let goal = ConversationGoal::new(goal_type);
        self.active_goals.push(goal);
        self.updated_at = Utc::now();
        self.active_goals.last().unwrap()
    }
    
    /// Get all active goals
    pub fn get_active_goals(&self) -> &[ConversationGoal] {
        &self.active_goals
    }
    
    /// Get a goal by ID
    pub fn get_goal_by_id(&self, goal_id: &str) -> Option<&ConversationGoal> {
        self.active_goals.iter().find(|goal| goal.id == goal_id)
    }
    
    /// Get a mutable goal by ID
    pub fn get_goal_by_id_mut(&mut self, goal_id: &str) -> Option<&mut ConversationGoal> {
        self.active_goals.iter_mut().find(|goal| goal.id == goal_id)
    }
    
    /// Get the client profile
    pub fn get_client_profile(&self) -> Option<&Arc<ClientProfile>> {
        self.client_profile.as_ref()
    }
    
    /// Get the client data
    pub fn get_client_data(&self) -> Option<&Arc<ClientData>> {
        self.client_data.as_ref()
    }
    
    /// Add a turn with query and response to the conversation
    pub fn add_turn(&mut self, query: String, response: NlpResponse) -> &ConversationTurn {
        // Create a new turn
        let turn = ConversationTurn::new(query.clone());
        
        // Create a processed query from the response
        let processed_query = ProcessedQuery {
            original_text: query.clone(),
            intent: response.intent.clone(),
            intent_confidence: response.confidence,
            entities: response.processed_query.as_ref()
                .map(|pq| pq.entities.clone())
                .unwrap_or_default(),
            normalized_text: query.to_lowercase(),
        };
        
        // Add the processed query and response to the turn
        let turn_with_data = turn
            .with_processed_query(processed_query)
            .with_response(response);
        
        // Add to history and maintain max size
        self.history.push_back(turn_with_data);
        if self.history.len() > MAX_HISTORY_TURNS {
            self.history.pop_front();
        }
        
        self.updated_at = Utc::now();
        self.history.back().unwrap()
    }
    
    /// Generate a context summary for the current conversation
    pub fn generate_context_summary(&self) -> String {
        let mut summary = String::new();
        
        // Add basic conversation info
        summary.push_str(&format!("Conversation ID: {}\n", self.id));
        summary.push_str(&format!("Client ID: {}\n", self.client_id));
        summary.push_str(&format!("Current state: {:?}\n", self.state));
        
        // Add recent history (last 3 turns)
        summary.push_str("\nRecent conversation history:\n");
        let recent_history: Vec<_> = self.history.iter().rev().take(3).collect();
        for (i, turn) in recent_history.iter().rev().enumerate() {
            summary.push_str(&format!("Turn {}: User: {}\n", i + 1, turn.query));
            if let Some(response) = &turn.response {
                summary.push_str(&format!("Turn {}: System: {}\n", i + 1, response.response_text));
            }
        }
        
        // Add active goals
        if !self.active_goals.is_empty() {
            summary.push_str("\nActive goals:\n");
            for goal in &self.active_goals {
                summary.push_str(&format!("- {:?} (Status: {:?})\n", goal.goal_type, goal.status));
            }
        }
        
        summary
    }
    
    /// Get the most recent user query
    pub fn get_most_recent_query(&self) -> Option<&str> {
        self.history.back().map(|turn| turn.query.as_str())
    }
    
    /// Get the most recent system response
    pub fn get_most_recent_response(&self) -> Option<&NlpResponse> {
        self.history.iter().rev()
            .find_map(|turn| turn.response.as_ref())
    }
    
    /// Get the most recent intent
    pub fn get_most_recent_intent(&self) -> Option<&FinancialQueryIntent> {
        self.history.iter().rev()
            .find_map(|turn| turn.processed_query.as_ref())
            .map(|pq| &pq.intent)
    }
    
    /// Get all entities from the current conversation
    pub fn get_all_entities(&self) -> Vec<&ExtractedEntity> {
        self.history.iter()
            .filter_map(|turn| turn.processed_query.as_ref())
            .flat_map(|pq| &pq.entities)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_conversation_turn() {
        let turn = ConversationTurn::new("How is my portfolio doing?".to_string());
        assert_eq!(turn.query, "How is my portfolio doing?");
        assert!(turn.processed_query.is_none());
        assert!(turn.response.is_none());
    }
    
    #[test]
    fn test_conversation_goal() {
        let mut goal = ConversationGoal::new(ConversationGoalType::PortfolioReview);
        assert_eq!(goal.status, GoalStatus::NotStarted);
        
        goal.add_required_information("risk_tolerance", true);
        goal.add_required_information("time_horizon", true);
        
        goal.start();
        assert_eq!(goal.status, GoalStatus::InProgress);
        
        goal.set_information_collected("risk_tolerance", "Moderate".to_string()).unwrap();
        assert_eq!(goal.status, GoalStatus::InProgress);
        
        goal.set_information_collected("time_horizon", "Long".to_string()).unwrap();
        assert_eq!(goal.status, GoalStatus::Completed);
    }
    
    #[test]
    fn test_conversation_manager() {
        let mut manager = ConversationManager::new("client123");
        assert_eq!(manager.get_state(), &ConversationState::Greeting);
        
        manager.add_user_query("Hello, I'd like to review my portfolio");
        assert_eq!(manager.get_history().len(), 1);
        
        manager.set_state(ConversationState::InformationGathering);
        assert_eq!(manager.get_state(), &ConversationState::InformationGathering);
        
        let goal = manager.add_goal(ConversationGoalType::PortfolioReview);
        assert_eq!(goal.goal_type, ConversationGoalType::PortfolioReview);
        
        let goal_id = goal.id.clone();
        let goal = manager.get_goal_by_id_mut(&goal_id).unwrap();
        goal.start();
        assert_eq!(goal.status, GoalStatus::InProgress);
    }
} 