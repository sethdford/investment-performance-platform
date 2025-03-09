use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Utc};

use super::bedrock::BedrockNlpClient;
use super::types::{ClientData, NlpResponse};
use super::rule_based::FinancialQueryIntent;
use super::knowledge_retriever::KnowledgeRetriever;
use crate::financial_advisor::client_profiling::ClientProfile;

/// Financial reasoning step type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReasoningStepType {
    /// Information gathering
    InformationGathering,
    
    /// Goal identification
    GoalIdentification,
    
    /// Risk assessment
    RiskAssessment,
    
    /// Scenario analysis
    ScenarioAnalysis,
    
    /// Recommendation generation
    RecommendationGeneration,
    
    /// Explanation
    Explanation,
}

/// Financial reasoning step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    /// Step type
    pub step_type: ReasoningStepType,
    
    /// Step description
    pub description: String,
    
    /// Input data
    pub input: HashMap<String, String>,
    
    /// Output data
    pub output: HashMap<String, String>,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl ReasoningStep {
    /// Create a new reasoning step
    pub fn new(step_type: ReasoningStepType, description: &str) -> Self {
        Self {
            step_type,
            description: description.to_string(),
            input: HashMap::new(),
            output: HashMap::new(),
            timestamp: Utc::now(),
        }
    }
    
    /// Add input data
    pub fn add_input(&mut self, key: &str, value: &str) {
        self.input.insert(key.to_string(), value.to_string());
    }
    
    /// Add output data
    pub fn add_output(&mut self, key: &str, value: &str) {
        self.output.insert(key.to_string(), value.to_string());
    }
}

/// Financial scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialScenario {
    /// Scenario ID
    pub id: String,
    
    /// Scenario name
    pub name: String,
    
    /// Scenario description
    pub description: String,
    
    /// Scenario parameters
    pub parameters: HashMap<String, String>,
    
    /// Scenario outcomes
    pub outcomes: HashMap<String, String>,
    
    /// Probability of success
    pub success_probability: Option<f64>,
    
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

impl FinancialScenario {
    /// Create a new financial scenario
    pub fn new(id: &str, name: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            parameters: HashMap::new(),
            outcomes: HashMap::new(),
            success_probability: None,
            created_at: Utc::now(),
        }
    }
    
    /// Add a parameter
    pub fn add_parameter(&mut self, key: &str, value: &str) {
        self.parameters.insert(key.to_string(), value.to_string());
    }
    
    /// Add an outcome
    pub fn add_outcome(&mut self, key: &str, value: &str) {
        self.outcomes.insert(key.to_string(), value.to_string());
    }
    
    /// Set success probability
    pub fn set_success_probability(&mut self, probability: f64) {
        self.success_probability = Some(probability);
    }
}

/// Financial recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialRecommendation {
    /// Recommendation ID
    pub id: String,
    
    /// Recommendation title
    pub title: String,
    
    /// Recommendation description
    pub description: String,
    
    /// Recommendation type
    pub recommendation_type: String,
    
    /// Priority (1-5, with 1 being highest)
    pub priority: u8,
    
    /// Impact (1-5, with 5 being highest)
    pub impact: u8,
    
    /// Effort (1-5, with 5 being highest)
    pub effort: u8,
    
    /// Timeframe
    pub timeframe: String,
    
    /// Action steps
    pub action_steps: Vec<String>,
    
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

impl FinancialRecommendation {
    /// Create a new financial recommendation
    pub fn new(id: &str, title: &str, description: &str, recommendation_type: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            recommendation_type: recommendation_type.to_string(),
            priority: 3,
            impact: 3,
            effort: 3,
            timeframe: "Medium-term".to_string(),
            action_steps: Vec::new(),
            created_at: Utc::now(),
        }
    }
    
    /// Add an action step
    pub fn add_action_step(&mut self, step: &str) {
        self.action_steps.push(step.to_string());
    }
    
    /// Set priority
    pub fn set_priority(&mut self, priority: u8) {
        self.priority = priority.min(5).max(1);
    }
    
    /// Set impact
    pub fn set_impact(&mut self, impact: u8) {
        self.impact = impact.min(5).max(1);
    }
    
    /// Set effort
    pub fn set_effort(&mut self, effort: u8) {
        self.effort = effort.min(5).max(1);
    }
    
    /// Set timeframe
    pub fn set_timeframe(&mut self, timeframe: &str) {
        self.timeframe = timeframe.to_string();
    }
}

/// Financial reasoning chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningChain {
    /// Chain ID
    pub id: String,
    
    /// Query that initiated this reasoning chain
    pub query: String,
    
    /// Intent
    pub intent: FinancialQueryIntent,
    
    /// Steps in the reasoning chain
    pub steps: Vec<ReasoningStep>,
    
    /// Scenarios analyzed
    pub scenarios: Vec<FinancialScenario>,
    
    /// Recommendations
    pub recommendations: Vec<FinancialRecommendation>,
    
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl ReasoningChain {
    /// Create a new reasoning chain
    pub fn new(id: &str, query: &str, intent: FinancialQueryIntent) -> Self {
        let now = Utc::now();
        Self {
            id: id.to_string(),
            query: query.to_string(),
            intent,
            steps: Vec::new(),
            scenarios: Vec::new(),
            recommendations: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Add a reasoning step
    pub fn add_step(&mut self, step: ReasoningStep) {
        self.steps.push(step);
        self.updated_at = Utc::now();
    }
    
    /// Add a scenario
    pub fn add_scenario(&mut self, scenario: FinancialScenario) {
        self.scenarios.push(scenario);
        self.updated_at = Utc::now();
    }
    
    /// Add a recommendation
    pub fn add_recommendation(&mut self, recommendation: FinancialRecommendation) {
        self.recommendations.push(recommendation);
        self.updated_at = Utc::now();
    }
    
    /// Get the last step
    pub fn get_last_step(&self) -> Option<&ReasoningStep> {
        self.steps.last()
    }
    
    /// Generate a summary of the reasoning chain
    pub fn generate_summary(&self) -> String {
        let mut summary = String::new();
        
        summary.push_str(&format!("Reasoning for query: \"{}\"\n", self.query));
        summary.push_str(&format!("Intent: {:?}\n\n", self.intent));
        
        if !self.steps.is_empty() {
            summary.push_str("Reasoning steps:\n");
            for (i, step) in self.steps.iter().enumerate() {
                summary.push_str(&format!("{}. {:?}: {}\n", i + 1, step.step_type, step.description));
            }
            summary.push_str("\n");
        }
        
        if !self.scenarios.is_empty() {
            summary.push_str("Scenarios analyzed:\n");
            for (i, scenario) in self.scenarios.iter().enumerate() {
                let probability = scenario.success_probability.map_or("N/A".to_string(), |p| format!("{:.1}%", p * 100.0));
                summary.push_str(&format!("{}. {} (Success probability: {})\n", i + 1, scenario.name, probability));
            }
            summary.push_str("\n");
        }
        
        if !self.recommendations.is_empty() {
            summary.push_str("Recommendations:\n");
            for (i, recommendation) in self.recommendations.iter().enumerate() {
                summary.push_str(&format!("{}. {} (Priority: {})\n", i + 1, recommendation.title, recommendation.priority));
            }
        }
        
        summary
    }
}

/// Financial reasoning service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningServiceConfig {
    /// Whether to use the LLM for reasoning
    pub use_llm: bool,
    
    /// Whether to use knowledge retrieval
    pub use_knowledge_retrieval: bool,
    
    /// Maximum number of reasoning steps
    pub max_reasoning_steps: usize,
    
    /// Maximum number of scenarios to analyze
    pub max_scenarios: usize,
    
    /// Maximum number of recommendations to generate
    pub max_recommendations: usize,
}

impl Default for ReasoningServiceConfig {
    fn default() -> Self {
        Self {
            use_llm: true,
            use_knowledge_retrieval: true,
            max_reasoning_steps: 5,
            max_scenarios: 3,
            max_recommendations: 3,
        }
    }
}

/// Financial reasoning service
#[derive(Debug)]
pub struct FinancialReasoningService {
    /// Bedrock NLP client (optional)
    bedrock: Option<Arc<BedrockNlpClient>>,
    
    /// Knowledge retriever (optional)
    knowledge_retriever: Option<Arc<KnowledgeRetriever>>,
    
    /// Configuration
    config: ReasoningServiceConfig,
}

impl FinancialReasoningService {
    /// Create a new financial reasoning service
    pub fn new(config: ReasoningServiceConfig) -> Self {
        Self {
            bedrock: None,
            knowledge_retriever: None,
            config,
        }
    }
    
    /// Set the Bedrock NLP client
    pub fn with_bedrock(mut self, client: Arc<BedrockNlpClient>) -> Self {
        self.bedrock = Some(client);
        self
    }
    
    /// Set the knowledge retriever
    pub fn with_knowledge_retriever(mut self, retriever: Arc<KnowledgeRetriever>) -> Self {
        self.knowledge_retriever = Some(retriever);
        self
    }
    
    /// Generate a reasoning chain for a query
    pub async fn generate_reasoning_chain(&self, query: &str, intent: FinancialQueryIntent, client_data: Option<&ClientData>, client_profile: Option<&ClientProfile>) -> Result<ReasoningChain> {
        let chain_id = format!("chain_{}", Utc::now().timestamp());
        let mut chain = ReasoningChain::new(&chain_id, query, intent.clone());
        
        // Step 1: Information gathering
        let mut info_step = ReasoningStep::new(
            ReasoningStepType::InformationGathering,
            "Gathering relevant information",
        );
        
        // Add client data if available
        if let Some(data) = client_data {
            info_step.add_input("client_data", &format!("{:?}", data));
        }
        
        // Add client profile if available
        if let Some(_profile) = client_profile {
            info_step.add_input("client_profile", &format!("{:?}", _profile));
        }
        
        // Retrieve relevant knowledge if available
        if self.config.use_knowledge_retrieval && self.knowledge_retriever.is_some() {
            let knowledge = self.retrieve_relevant_knowledge(query, &intent).await?;
            if !knowledge.is_empty() {
                info_step.add_input("knowledge", &knowledge);
            }
        }
        
        // Add query and intent
        info_step.add_input("query", query);
        info_step.add_input("intent", &format!("{:?}", intent));
        
        // Add information gathering output
        info_step.add_output("status", "completed");
        chain.add_step(info_step);
        
        // Step 2: Goal identification (for goal-related intents)
        match intent {
            FinancialQueryIntent::RetirementPlanning |
            FinancialQueryIntent::EducationPlanning |
            FinancialQueryIntent::GoalProgress |
            FinancialQueryIntent::HomePurchase => {
                let mut goal_step = ReasoningStep::new(
                    ReasoningStepType::GoalIdentification,
                    "Identifying financial goals",
                );
                
                // Extract goal information from client profile if available
                if let Some(_profile) = client_profile {
                    // In a real implementation, we would extract goals from the profile
                    goal_step.add_input("client_goals", "Goals extracted from client profile");
                }
                
                // Add goal identification output
                goal_step.add_output("identified_goals", "Retirement at age 65 with $80,000 annual income");
                chain.add_step(goal_step);
            },
            _ => {}
        }
        
        // Step 3: Risk assessment (for risk or investment related intents)
        match intent {
            FinancialQueryIntent::RiskAssessment |
            FinancialQueryIntent::InvestmentRecommendation |
            FinancialQueryIntent::AssetAllocation => {
                let mut risk_step = ReasoningStep::new(
                    ReasoningStepType::RiskAssessment,
                    "Assessing risk tolerance and capacity",
                );
                
                // Extract risk information from client profile if available
                if let Some(_profile) = client_profile {
                    // In a real implementation, we would extract risk information from the profile
                    risk_step.add_input("risk_tolerance", "Moderate");
                    risk_step.add_input("risk_capacity", "Moderate");
                }
                
                // Add risk assessment output
                risk_step.add_output("risk_assessment", "Client has moderate risk tolerance and capacity");
                chain.add_step(risk_step);
            },
            _ => {}
        }
        
        // Step 4: Scenario analysis (for planning or projection intents)
        match intent {
            FinancialQueryIntent::RetirementPlanning |
            FinancialQueryIntent::EducationPlanning |
            FinancialQueryIntent::HomePurchase => {
                let mut scenario_step = ReasoningStep::new(
                    ReasoningStepType::ScenarioAnalysis,
                    "Analyzing different scenarios",
                );
                
                // Create scenarios
                let base_scenario = FinancialScenario::new(
                    "scenario_1",
                    "Base scenario",
                    "Current savings rate and investment allocation",
                );
                
                let aggressive_scenario = FinancialScenario::new(
                    "scenario_2",
                    "Aggressive scenario",
                    "Increased savings rate and more aggressive allocation",
                );
                
                let conservative_scenario = FinancialScenario::new(
                    "scenario_3",
                    "Conservative scenario",
                    "Current savings rate and more conservative allocation",
                );
                
                // Add scenarios to the chain
                chain.add_scenario(base_scenario);
                chain.add_scenario(aggressive_scenario);
                chain.add_scenario(conservative_scenario);
                
                // Add scenario analysis output
                scenario_step.add_output("scenarios_analyzed", "3");
                scenario_step.add_output("recommended_scenario", "Base scenario");
                chain.add_step(scenario_step);
            },
            _ => {}
        }
        
        // Step 5: Recommendation generation
        let mut recommendation_step = ReasoningStep::new(
            ReasoningStepType::RecommendationGeneration,
            "Generating recommendations",
        );
        
        // Create recommendations based on intent
        match intent {
            FinancialQueryIntent::RetirementPlanning => {
                let mut rec1 = FinancialRecommendation::new(
                    "rec_1",
                    "Increase retirement contributions",
                    "Increase your 401(k) contributions to the maximum allowed",
                    "Retirement",
                );
                rec1.set_priority(1);
                rec1.add_action_step("Increase 401(k) contribution to 15% of salary");
                rec1.add_action_step("Set up automatic increases of 1% per year");
                
                let mut rec2 = FinancialRecommendation::new(
                    "rec_2",
                    "Open a Roth IRA",
                    "Supplement your 401(k) with a Roth IRA for tax diversification",
                    "Retirement",
                );
                rec2.set_priority(2);
                rec2.add_action_step("Open a Roth IRA account");
                rec2.add_action_step("Set up automatic monthly contributions");
                
                chain.add_recommendation(rec1);
                chain.add_recommendation(rec2);
            },
            FinancialQueryIntent::AssetAllocation => {
                let mut rec = FinancialRecommendation::new(
                    "rec_1",
                    "Rebalance portfolio",
                    "Rebalance your portfolio to align with your target asset allocation",
                    "Investment",
                );
                rec.set_priority(1);
                rec.add_action_step("Sell overweight asset classes");
                rec.add_action_step("Buy underweight asset classes");
                rec.add_action_step("Set up quarterly rebalancing reminders");
                
                chain.add_recommendation(rec);
            },
            FinancialQueryIntent::TaxOptimization => {
                let mut rec = FinancialRecommendation::new(
                    "rec_1",
                    "Tax-loss harvesting",
                    "Harvest losses in your taxable accounts to offset gains",
                    "Tax",
                );
                rec.set_priority(2);
                rec.add_action_step("Identify investments with unrealized losses");
                rec.add_action_step("Sell and replace with similar but not substantially identical investments");
                rec.add_action_step("Wait 31 days before repurchasing original investments");
                
                chain.add_recommendation(rec);
            },
            _ => {
                // Generic recommendation for other intents
                let mut rec = FinancialRecommendation::new(
                    "rec_1",
                    "Review financial plan",
                    "Schedule a comprehensive financial plan review",
                    "General",
                );
                rec.set_priority(3);
                rec.add_action_step("Schedule a meeting with your financial advisor");
                rec.add_action_step("Prepare a list of questions and concerns");
                rec.add_action_step("Update your financial goals and priorities");
                
                chain.add_recommendation(rec);
            }
        }
        
        // Add recommendation generation output
        recommendation_step.add_output("recommendations_generated", &format!("{}", chain.recommendations.len()));
        chain.add_step(recommendation_step);
        
        // Step 6: Explanation
        let mut explanation_step = ReasoningStep::new(
            ReasoningStepType::Explanation,
            "Generating explanation",
        );
        
        // Generate explanation
        let explanation = self.generate_explanation(&chain).await?;
        explanation_step.add_output("explanation", &explanation);
        chain.add_step(explanation_step);
        
        Ok(chain)
    }
    
    /// Retrieve relevant knowledge
    async fn retrieve_relevant_knowledge(&self, query: &str, intent: &FinancialQueryIntent) -> Result<String> {
        let mut knowledge = String::new();
        
        if let Some(retriever) = &self.knowledge_retriever {
            // Try to retrieve by query first
            let query_items = retriever.retrieve_by_query(query).await?;
            
            // If we don't have enough items, also retrieve by intent
            let mut all_items = query_items;
            if all_items.len() < 3 {
                let intent_items = retriever.retrieve_by_intent(intent);
                
                // Combine items, avoiding duplicates
                for item in intent_items {
                    if !all_items.iter().any(|i| i.id == item.id) {
                        all_items.push(item);
                        if all_items.len() >= 3 {
                            break;
                        }
                    }
                }
            }
            
            // Format the knowledge items
            if !all_items.is_empty() {
                for item in all_items.iter().take(3) {
                    knowledge.push_str(&format!("{}: {}\n", item.title, item.content));
                }
            }
        }
        
        Ok(knowledge)
    }
    
    /// Generate an explanation for a reasoning chain
    async fn generate_explanation(&self, chain: &ReasoningChain) -> Result<String> {
        // If we have a Bedrock client and LLM is enabled, use it to generate an explanation
        if self.config.use_llm && self.bedrock.is_some() {
            let _bedrock_client = self.bedrock.as_ref().unwrap();
            
            // In a real implementation, we would use the Bedrock client to generate an explanation
            // For now, return a placeholder explanation
            return Ok("Based on your financial situation and goals, I've analyzed several scenarios and generated recommendations to help you achieve your objectives. The most important action you can take is to increase your retirement contributions, which will significantly improve your retirement readiness.".to_string());
        }
        
        // Otherwise, generate a simple explanation based on the chain
        let mut explanation = String::new();
        
        explanation.push_str(&format!("Based on your query about {:?}, ", chain.intent));
        
        if !chain.recommendations.is_empty() {
            explanation.push_str("I recommend the following actions: ");
            for (i, rec) in chain.recommendations.iter().enumerate() {
                if i > 0 {
                    explanation.push_str(", ");
                }
                explanation.push_str(&rec.title);
            }
            explanation.push_str(".");
        }
        
        if !chain.scenarios.is_empty() {
            explanation.push_str(" I've analyzed different scenarios, and the most promising approach is ");
            explanation.push_str(&chain.scenarios[0].name);
            explanation.push_str(".");
        }
        
        Ok(explanation)
    }
    
    /// Generate a response from a reasoning chain
    pub fn generate_response(&self, chain: &ReasoningChain) -> NlpResponse {
        // Extract the explanation from the last step
        let explanation = chain.steps.iter()
            .filter(|step| step.step_type == ReasoningStepType::Explanation)
            .last()
            .and_then(|step| step.output.get("explanation"))
            .map(|s| s.to_string())
            .unwrap_or_else(|| "I've analyzed your query and generated recommendations.".to_string());
        
        // Create a response
        NlpResponse {
            query: chain.query.clone(),
            intent: chain.intent.clone(),
            confidence: 0.9, // Placeholder confidence
            processed_query: None, // No processed query for reasoning responses
            response_text: explanation,
            source: super::types::NlpResponseSource::Reasoning,
            confidence_level: super::types::NlpConfidenceLevel::from_score(0.9),
            is_uncertain: false,
            explanation: Some(chain.generate_summary()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_reasoning_step() {
        let mut step = ReasoningStep::new(
            ReasoningStepType::InformationGathering,
            "Gathering information",
        );
        
        step.add_input("query", "How much should I save for retirement?");
        step.add_output("status", "completed");
        
        assert_eq!(step.step_type, ReasoningStepType::InformationGathering);
        assert_eq!(step.description, "Gathering information");
        assert_eq!(step.input.get("query").unwrap(), "How much should I save for retirement?");
        assert_eq!(step.output.get("status").unwrap(), "completed");
    }
    
    #[test]
    fn test_financial_scenario() {
        let mut scenario = FinancialScenario::new(
            "scenario_1",
            "Base scenario",
            "Current savings rate and investment allocation",
        );
        
        scenario.add_parameter("savings_rate", "15%");
        scenario.add_outcome("retirement_age", "67");
        scenario.set_success_probability(0.75);
        
        assert_eq!(scenario.id, "scenario_1");
        assert_eq!(scenario.name, "Base scenario");
        assert_eq!(scenario.parameters.get("savings_rate").unwrap(), "15%");
        assert_eq!(scenario.outcomes.get("retirement_age").unwrap(), "67");
        assert_eq!(scenario.success_probability.unwrap(), 0.75);
    }
    
    #[test]
    fn test_financial_recommendation() {
        let mut recommendation = FinancialRecommendation::new(
            "rec_1",
            "Increase retirement contributions",
            "Increase your 401(k) contributions to the maximum allowed",
            "Retirement",
        );
        
        recommendation.set_priority(1);
        recommendation.add_action_step("Increase 401(k) contribution to 15% of salary");
        
        assert_eq!(recommendation.id, "rec_1");
        assert_eq!(recommendation.title, "Increase retirement contributions");
        assert_eq!(recommendation.priority, 1);
        assert_eq!(recommendation.action_steps[0], "Increase 401(k) contribution to 15% of salary");
    }
    
    #[test]
    fn test_reasoning_chain() {
        let mut chain = ReasoningChain::new(
            "chain_1",
            "How much should I save for retirement?",
            FinancialQueryIntent::RetirementPlanning,
        );
        
        let step = ReasoningStep::new(
            ReasoningStepType::InformationGathering,
            "Gathering information",
        );
        
        chain.add_step(step);
        
        assert_eq!(chain.id, "chain_1");
        assert_eq!(chain.query, "How much should I save for retirement?");
        assert_eq!(chain.intent, FinancialQueryIntent::RetirementPlanning);
        assert_eq!(chain.steps.len(), 1);
        assert_eq!(chain.steps[0].step_type, ReasoningStepType::InformationGathering);
    }
    
    #[test]
    fn test_reasoning_service_config() {
        let config = ReasoningServiceConfig::default();
        
        assert!(config.use_llm);
        assert!(config.use_knowledge_retrieval);
        assert_eq!(config.max_reasoning_steps, 5);
        assert_eq!(config.max_scenarios, 3);
        assert_eq!(config.max_recommendations, 3);
    }
} 