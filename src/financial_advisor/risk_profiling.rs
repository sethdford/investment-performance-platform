use std::collections::{HashMap, HashSet};
use anyhow::{Result, anyhow};
use tracing::info;
use chrono::Utc;

use crate::financial_advisor::{
    RiskToleranceLevel, 
    BehavioralBiasType,
    RiskProfileResponse,
    RiskProfileQuestion,
    RiskProfileAnswerOption,
    RiskProfileQuestionnaire
};

/// Risk profiling service for assessing client risk tolerance and behavioral biases
#[derive(Clone)]
pub struct RiskProfilingService {
    /// Available questionnaires
    questionnaires: HashMap<String, RiskProfileQuestionnaire>,
    
    /// Mapping of question IDs to behavioral bias detection rules
    bias_detection_rules: HashMap<String, Vec<(i32, BehavioralBiasType)>>,
}

impl RiskProfilingService {
    /// Create a new risk profiling service with default questionnaires
    pub fn new() -> Self {
        let mut service = Self {
            questionnaires: HashMap::new(),
            bias_detection_rules: HashMap::new(),
        };
        
        service.initialize_default_questionnaire();
        service.initialize_bias_detection_rules();
        
        service
    }
    
    /// Initialize the default comprehensive risk profiling questionnaire
    fn initialize_default_questionnaire(&mut self) {
        let questions = vec![
            // Risk capacity questions
            RiskProfileQuestion {
                id: "time_horizon".to_string(),
                text: "How long do you plan to invest your money before you'll need to access a significant portion of it?".to_string(),
                options: vec![
                    RiskProfileAnswerOption {
                        value: 1,
                        text: "Less than 1 year".to_string(),
                        risk_score: 1.0,
                    },
                    RiskProfileAnswerOption {
                        value: 2,
                        text: "1-3 years".to_string(),
                        risk_score: 2.0,
                    },
                    RiskProfileAnswerOption {
                        value: 3,
                        text: "3-5 years".to_string(),
                        risk_score: 3.0,
                    },
                    RiskProfileAnswerOption {
                        value: 4,
                        text: "5-10 years".to_string(),
                        risk_score: 4.0,
                    },
                    RiskProfileAnswerOption {
                        value: 5,
                        text: "More than 10 years".to_string(),
                        risk_score: 5.0,
                    },
                ],
                category: "risk_capacity".to_string(),
                weight: 1.5, // Higher weight due to importance of time horizon
            },
            
            RiskProfileQuestion {
                id: "income_stability".to_string(),
                text: "How stable is your current and future income?".to_string(),
                options: vec![
                    RiskProfileAnswerOption {
                        value: 1,
                        text: "Very unstable - I work in a volatile industry or have irregular income".to_string(),
                        risk_score: 1.0,
                    },
                    RiskProfileAnswerOption {
                        value: 2,
                        text: "Somewhat unstable - My income fluctuates year to year".to_string(),
                        risk_score: 2.0,
                    },
                    RiskProfileAnswerOption {
                        value: 3,
                        text: "Moderately stable - My income is generally reliable but not guaranteed".to_string(),
                        risk_score: 3.0,
                    },
                    RiskProfileAnswerOption {
                        value: 4,
                        text: "Stable - I have a secure job with predictable income".to_string(),
                        risk_score: 4.0,
                    },
                    RiskProfileAnswerOption {
                        value: 5,
                        text: "Very stable - I have guaranteed income or multiple secure income sources".to_string(),
                        risk_score: 5.0,
                    },
                ],
                category: "risk_capacity".to_string(),
                weight: 1.2,
            },
            
            RiskProfileQuestion {
                id: "emergency_fund".to_string(),
                text: "How many months of expenses do you have saved in an emergency fund?".to_string(),
                options: vec![
                    RiskProfileAnswerOption {
                        value: 1,
                        text: "Less than 1 month".to_string(),
                        risk_score: 1.0,
                    },
                    RiskProfileAnswerOption {
                        value: 2,
                        text: "1-3 months".to_string(),
                        risk_score: 2.0,
                    },
                    RiskProfileAnswerOption {
                        value: 3,
                        text: "3-6 months".to_string(),
                        risk_score: 3.0,
                    },
                    RiskProfileAnswerOption {
                        value: 4,
                        text: "6-12 months".to_string(),
                        risk_score: 4.0,
                    },
                    RiskProfileAnswerOption {
                        value: 5,
                        text: "More than 12 months".to_string(),
                        risk_score: 5.0,
                    },
                ],
                category: "risk_capacity".to_string(),
                weight: 1.0,
            },
            
            // Risk willingness questions
            RiskProfileQuestion {
                id: "market_decline_reaction".to_string(),
                text: "If your portfolio lost 20% of its value in a month, what would you do?".to_string(),
                options: vec![
                    RiskProfileAnswerOption {
                        value: 1,
                        text: "Sell all investments and move to cash".to_string(),
                        risk_score: 1.0,
                    },
                    RiskProfileAnswerOption {
                        value: 2,
                        text: "Sell some investments to reduce risk".to_string(),
                        risk_score: 2.0,
                    },
                    RiskProfileAnswerOption {
                        value: 3,
                        text: "Do nothing and wait for recovery".to_string(),
                        risk_score: 3.0,
                    },
                    RiskProfileAnswerOption {
                        value: 4,
                        text: "Invest a small amount more to take advantage of lower prices".to_string(),
                        risk_score: 4.0,
                    },
                    RiskProfileAnswerOption {
                        value: 5,
                        text: "Invest significantly more to take advantage of lower prices".to_string(),
                        risk_score: 5.0,
                    },
                ],
                category: "risk_willingness".to_string(),
                weight: 1.3,
            },
            
            RiskProfileQuestion {
                id: "risk_reward_tradeoff".to_string(),
                text: "Which investment approach appeals to you most?".to_string(),
                options: vec![
                    RiskProfileAnswerOption {
                        value: 1,
                        text: "Preserving capital with minimal risk, accepting lower returns".to_string(),
                        risk_score: 1.0,
                    },
                    RiskProfileAnswerOption {
                        value: 2,
                        text: "Taking minimal risk with focus on income generation".to_string(),
                        risk_score: 2.0,
                    },
                    RiskProfileAnswerOption {
                        value: 3,
                        text: "Balanced approach with moderate risk and returns".to_string(),
                        risk_score: 3.0,
                    },
                    RiskProfileAnswerOption {
                        value: 4,
                        text: "Growth-focused with higher risk for higher potential returns".to_string(),
                        risk_score: 4.0,
                    },
                    RiskProfileAnswerOption {
                        value: 5,
                        text: "Aggressive growth with maximum risk for maximum potential returns".to_string(),
                        risk_score: 5.0,
                    },
                ],
                category: "risk_willingness".to_string(),
                weight: 1.0,
            },
            
            // Behavioral bias detection questions
            RiskProfileQuestion {
                id: "loss_aversion".to_string(),
                text: "Which would bother you more?".to_string(),
                options: vec![
                    RiskProfileAnswerOption {
                        value: 1,
                        text: "Losing $1,000 on an investment".to_string(),
                        risk_score: 2.0,
                    },
                    RiskProfileAnswerOption {
                        value: 2,
                        text: "Missing out on a $1,000 gain by not investing".to_string(),
                        risk_score: 3.0,
                    },
                    RiskProfileAnswerOption {
                        value: 3,
                        text: "Both would bother me equally".to_string(),
                        risk_score: 4.0,
                    },
                ],
                category: "behavioral_bias".to_string(),
                weight: 0.8,
            },
            
            RiskProfileQuestion {
                id: "recency_bias".to_string(),
                text: "How much do recent market events influence your investment decisions?".to_string(),
                options: vec![
                    RiskProfileAnswerOption {
                        value: 1,
                        text: "Significantly - I tend to react to recent market movements".to_string(),
                        risk_score: 2.0,
                    },
                    RiskProfileAnswerOption {
                        value: 2,
                        text: "Somewhat - I consider recent events but try to maintain perspective".to_string(),
                        risk_score: 3.0,
                    },
                    RiskProfileAnswerOption {
                        value: 3,
                        text: "Minimally - I focus on long-term trends rather than recent events".to_string(),
                        risk_score: 4.0,
                    },
                ],
                category: "behavioral_bias".to_string(),
                weight: 0.8,
            },
            
            RiskProfileQuestion {
                id: "overconfidence".to_string(),
                text: "How would you rate your investment knowledge compared to others?".to_string(),
                options: vec![
                    RiskProfileAnswerOption {
                        value: 1,
                        text: "Below average".to_string(),
                        risk_score: 2.0,
                    },
                    RiskProfileAnswerOption {
                        value: 2,
                        text: "Average".to_string(),
                        risk_score: 3.0,
                    },
                    RiskProfileAnswerOption {
                        value: 3,
                        text: "Above average".to_string(),
                        risk_score: 3.5,
                    },
                    RiskProfileAnswerOption {
                        value: 4,
                        text: "Expert".to_string(),
                        risk_score: 4.0,
                    },
                ],
                category: "behavioral_bias".to_string(),
                weight: 0.7,
            },
            
            RiskProfileQuestion {
                id: "mental_accounting".to_string(),
                text: "Do you view different investments (retirement, education, etc.) as separate buckets with different risk levels?".to_string(),
                options: vec![
                    RiskProfileAnswerOption {
                        value: 1,
                        text: "Yes, I have different risk approaches for different goals".to_string(),
                        risk_score: 3.0,
                    },
                    RiskProfileAnswerOption {
                        value: 2,
                        text: "Somewhat, but I try to maintain a consistent overall approach".to_string(),
                        risk_score: 3.5,
                    },
                    RiskProfileAnswerOption {
                        value: 3,
                        text: "No, I view my portfolio holistically with one risk approach".to_string(),
                        risk_score: 4.0,
                    },
                ],
                category: "behavioral_bias".to_string(),
                weight: 0.6,
            },
            
            RiskProfileQuestion {
                id: "herding".to_string(),
                text: "How influenced are you by what other investors are doing?".to_string(),
                options: vec![
                    RiskProfileAnswerOption {
                        value: 1,
                        text: "Very influenced - I often follow investment trends".to_string(),
                        risk_score: 2.0,
                    },
                    RiskProfileAnswerOption {
                        value: 2,
                        text: "Somewhat influenced - I consider trends but make my own decisions".to_string(),
                        risk_score: 3.0,
                    },
                    RiskProfileAnswerOption {
                        value: 3,
                        text: "Rarely influenced - I make decisions independently of others".to_string(),
                        risk_score: 4.0,
                    },
                ],
                category: "behavioral_bias".to_string(),
                weight: 0.7,
            },
            
            // Financial knowledge questions
            RiskProfileQuestion {
                id: "investment_knowledge".to_string(),
                text: "How would you rate your understanding of different investment types and their risks?".to_string(),
                options: vec![
                    RiskProfileAnswerOption {
                        value: 1,
                        text: "Very limited - I have minimal understanding of investments".to_string(),
                        risk_score: 1.0,
                    },
                    RiskProfileAnswerOption {
                        value: 2,
                        text: "Basic - I understand simple investments like savings accounts and CDs".to_string(),
                        risk_score: 2.0,
                    },
                    RiskProfileAnswerOption {
                        value: 3,
                        text: "Moderate - I understand stocks, bonds, and mutual funds".to_string(),
                        risk_score: 3.0,
                    },
                    RiskProfileAnswerOption {
                        value: 4,
                        text: "Advanced - I understand complex investments like options and alternatives".to_string(),
                        risk_score: 4.0,
                    },
                    RiskProfileAnswerOption {
                        value: 5,
                        text: "Expert - I have professional-level investment knowledge".to_string(),
                        risk_score: 5.0,
                    },
                ],
                category: "financial_knowledge".to_string(),
                weight: 0.9,
            },
        ];
        
        let questionnaire = RiskProfileQuestionnaire {
            id: "comprehensive_risk_profile".to_string(),
            name: "Comprehensive Risk Profile Assessment".to_string(),
            description: "A holistic assessment of risk tolerance incorporating capacity, willingness, and behavioral factors".to_string(),
            questions,
        };
        
        self.questionnaires.insert(questionnaire.id.clone(), questionnaire);
    }
    
    /// Initialize behavioral bias detection rules
    fn initialize_bias_detection_rules(&mut self) {
        // Loss aversion detection
        self.bias_detection_rules.insert(
            "loss_aversion".to_string(),
            vec![(1, BehavioralBiasType::LossAversion)],
        );
        
        // Recency bias detection
        self.bias_detection_rules.insert(
            "recency_bias".to_string(),
            vec![(1, BehavioralBiasType::RecencyBias)],
        );
        
        // Overconfidence detection (high self-rated knowledge)
        self.bias_detection_rules.insert(
            "overconfidence".to_string(),
            vec![(4, BehavioralBiasType::Overconfidence), (3, BehavioralBiasType::Overconfidence)],
        );
        
        // Mental accounting detection
        self.bias_detection_rules.insert(
            "mental_accounting".to_string(),
            vec![(1, BehavioralBiasType::MentalAccounting)],
        );
        
        // Herding behavior detection
        self.bias_detection_rules.insert(
            "herding".to_string(),
            vec![(1, BehavioralBiasType::HerdMentality)],
        );
    }
    
    /// Get a questionnaire by ID
    pub fn get_questionnaire(&self, id: &str) -> Option<&RiskProfileQuestionnaire> {
        self.questionnaires.get(id)
    }
    
    /// Calculate risk tolerance level based on questionnaire responses
    pub fn calculate_risk_tolerance(&self, responses: &[RiskProfileResponse], questionnaire_id: &str) -> Result<RiskToleranceLevel> {
        let questionnaire = self.questionnaires.get(questionnaire_id)
            .ok_or_else(|| anyhow!("Questionnaire not found: {}", questionnaire_id))?;
        
        let mut total_score = 0.0;
        let mut total_weight = 0.0;
        
        // Create a map of question IDs to questions for easy lookup
        let question_map: HashMap<String, &RiskProfileQuestion> = questionnaire.questions.iter()
            .map(|q| (q.id.clone(), q))
            .collect();
        
        // Calculate weighted average of risk scores
        for response in responses {
            if let Some(question) = question_map.get(&response.question_id) {
                // Find the selected option
                if let Some(option) = question.options.iter().find(|o| o.value == response.response_value) {
                    total_score += option.risk_score * question.weight;
                    total_weight += question.weight;
                }
            }
        }
        
        // Calculate final risk score (1-5 scale)
        let risk_score = if total_weight > 0.0 {
            total_score / total_weight
        } else {
            return Err(anyhow!("No valid responses found"));
        };
        
        // Map risk score to risk tolerance level
        let risk_tolerance = match risk_score {
            s if s < 1.5 => RiskToleranceLevel::VeryConservative,
            s if s < 2.5 => RiskToleranceLevel::Conservative,
            s if s < 3.5 => RiskToleranceLevel::Moderate,
            s if s < 4.5 => RiskToleranceLevel::Aggressive,
            _ => RiskToleranceLevel::VeryAggressive,
        };
        
        info!(
            risk_score = risk_score,
            risk_tolerance = ?risk_tolerance,
            "Calculated risk tolerance"
        );
        
        Ok(risk_tolerance)
    }
    
    /// Detect behavioral biases based on questionnaire responses
    pub fn detect_behavioral_biases(&self, responses: &[RiskProfileResponse]) -> Vec<BehavioralBiasType> {
        let mut detected_biases = HashSet::new();
        
        // Check each response against bias detection rules
        for response in responses {
            if let Some(rules) = self.bias_detection_rules.get(&response.question_id) {
                for (trigger_value, bias_type) in rules {
                    if response.response_value == *trigger_value {
                        detected_biases.insert(bias_type.clone());
                    }
                }
            }
        }
        
        let biases: Vec<BehavioralBiasType> = detected_biases.into_iter().collect();
        
        info!(
            bias_count = biases.len(),
            "Detected behavioral biases"
        );
        
        biases
    }
    
    /// Generate personalized risk insights based on profile and detected biases
    pub fn generate_risk_insights(&self, risk_tolerance: &RiskToleranceLevel, biases: &[BehavioralBiasType]) -> Vec<String> {
        let mut insights = Vec::new();
        
        // Add risk tolerance insight
        match risk_tolerance {
            RiskToleranceLevel::VeryConservative => {
                insights.push("Your very conservative risk profile suggests a focus on capital preservation and income generation. Consider allocations heavily weighted toward high-quality bonds and cash equivalents.".to_string());
            },
            RiskToleranceLevel::Conservative => {
                insights.push("Your conservative risk profile suggests a focus on stability with some growth potential. Consider a portfolio with significant fixed income allocation and some high-quality equities.".to_string());
            },
            RiskToleranceLevel::Moderate => {
                insights.push("Your moderate risk profile suggests a balanced approach. Consider a diversified portfolio with roughly equal allocations to equities and fixed income.".to_string());
            },
            RiskToleranceLevel::Aggressive => {
                insights.push("Your aggressive risk profile suggests a focus on growth. Consider a portfolio primarily allocated to equities with some fixed income for diversification.".to_string());
            },
            RiskToleranceLevel::VeryAggressive => {
                insights.push("Your very aggressive risk profile suggests a focus on maximum growth potential. Consider a portfolio heavily weighted toward equities, including potentially higher-risk segments like small-cap and emerging markets.".to_string());
            },
        }
        
        // Add behavioral bias insights
        for bias in biases {
            match bias {
                BehavioralBiasType::LossAversion => {
                    insights.push("You show signs of loss aversion, where the pain of losses outweighs the pleasure of gains. This might lead you to sell investments during market downturns, potentially locking in losses. Consider setting up automatic rebalancing to reduce emotional decision-making.".to_string());
                },
                BehavioralBiasType::RecencyBias => {
                    insights.push("You show signs of recency bias, giving too much weight to recent events. This might lead you to chase performance or panic during market volatility. Consider focusing on long-term historical data and maintaining a consistent investment strategy.".to_string());
                },
                BehavioralBiasType::Overconfidence => {
                    insights.push("You show signs of overconfidence in your investment knowledge. This might lead to excessive trading or concentrated positions. Consider implementing a systematic investment approach and seeking diverse perspectives before making major decisions.".to_string());
                },
                BehavioralBiasType::MentalAccounting => {
                    insights.push("You show signs of mental accounting, treating different investments as separate buckets. While goal-based investing has benefits, consider the efficiency of your overall portfolio allocation and risk exposure.".to_string());
                },
                BehavioralBiasType::HerdMentality => {
                    insights.push("You show signs of herding behavior, being influenced by what other investors are doing. This might lead to buying high and selling low. Consider focusing on your personal financial goals rather than market trends.".to_string());
                },
                _ => {},
            }
        }
        
        insights
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_risk_profiling_service_initialization() {
        let service = RiskProfilingService::new();
        assert!(service.get_questionnaire("comprehensive_risk_profile").is_some());
    }
    
    #[test]
    fn test_calculate_risk_tolerance() {
        let service = RiskProfilingService::new();
        
        // Create test responses for a moderate risk profile
        let responses = vec![
            RiskProfileResponse {
                question_id: "time_horizon".to_string(),
                response_value: 3,
                comments: None,
                timestamp: Utc::now(),
            },
            RiskProfileResponse {
                question_id: "income_stability".to_string(),
                response_value: 4,
                comments: None,
                timestamp: Utc::now(),
            },
            RiskProfileResponse {
                question_id: "emergency_fund".to_string(),
                response_value: 3,
                comments: None,
                timestamp: Utc::now(),
            },
            RiskProfileResponse {
                question_id: "market_decline_reaction".to_string(),
                response_value: 3,
                comments: None,
                timestamp: Utc::now(),
            },
            RiskProfileResponse {
                question_id: "risk_reward_tradeoff".to_string(),
                response_value: 3,
                comments: None,
                timestamp: Utc::now(),
            },
        ];
        
        let risk_tolerance = service.calculate_risk_tolerance(&responses, "comprehensive_risk_profile").unwrap();
        assert_eq!(risk_tolerance, RiskToleranceLevel::Moderate);
    }
    
    #[test]
    fn test_detect_behavioral_biases() {
        let service = RiskProfilingService::new();
        
        // Create test responses with loss aversion and herding biases
        let responses = vec![
            RiskProfileResponse {
                question_id: "loss_aversion".to_string(),
                response_value: 1,
                comments: None,
                timestamp: Utc::now(),
            },
            RiskProfileResponse {
                question_id: "herding".to_string(),
                response_value: 1,
                comments: None,
                timestamp: Utc::now(),
            },
        ];
        
        let biases = service.detect_behavioral_biases(&responses);
        assert_eq!(biases.len(), 2);
        
        // Check that the expected biases are detected
        let has_loss_aversion = biases.iter().any(|b| matches!(b, BehavioralBiasType::LossAversion));
        let has_herding = biases.iter().any(|b| matches!(b, BehavioralBiasType::HerdMentality));
        
        assert!(has_loss_aversion);
        assert!(has_herding);
    }
    
    #[test]
    fn test_generate_risk_insights() {
        let service = RiskProfilingService::new();
        
        let risk_tolerance = RiskToleranceLevel::Moderate;
        let biases = vec![BehavioralBiasType::LossAversion, BehavioralBiasType::RecencyBias];
        
        let insights = service.generate_risk_insights(&risk_tolerance, &biases);
        
        // Should have 3 insights: 1 for risk tolerance and 2 for biases
        assert_eq!(insights.len(), 3);
    }
} 