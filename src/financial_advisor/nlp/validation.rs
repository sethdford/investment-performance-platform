use anyhow::{Result, anyhow, Context};
use serde_json::Value;
use super::rule_based::{FinancialQueryIntent, EntityType};
use super::types::{ValidatedLlmResponse, ValidatedEntity};

/// Validate an LLM response
pub fn validate_llm_response(response: &str) -> Result<ValidatedLlmResponse> {
    // Parse the JSON response
    let parsed: Value = serde_json::from_str(response)
        .context("Failed to parse LLM response as JSON")?;
    
    // Validate intent
    let intent = validate_intent(&parsed)?;
    
    // Validate intent confidence
    let intent_confidence = validate_intent_confidence(&parsed)?;
    
    // Validate entities
    let entities = validate_entities(&parsed)?;
    
    // Check if the LLM is uncertain
    let is_uncertain = parsed.get("uncertain")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    // Get response text if available
    let response_text = parsed.get("response_text")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    // Get explanation if available
    let explanation = parsed.get("explanation")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    Ok(ValidatedLlmResponse {
        intent,
        intent_confidence,
        entities,
        is_uncertain,
        response_text,
        explanation,
    })
}

/// Validate the intent from an LLM response
fn validate_intent(parsed: &Value) -> Result<FinancialQueryIntent> {
    let intent_str = parsed.get("intent")
        .ok_or_else(|| anyhow!("Missing 'intent' field in LLM response"))?
        .as_str()
        .ok_or_else(|| anyhow!("'intent' field is not a string"))?;
    
    // Map the intent string to a FinancialQueryIntent
    match intent_str {
        "PortfolioPerformance" => Ok(FinancialQueryIntent::PortfolioPerformance),
        "AssetAllocation" => Ok(FinancialQueryIntent::AssetAllocation),
        "GoalProgress" => Ok(FinancialQueryIntent::GoalProgress),
        "TaxOptimization" => Ok(FinancialQueryIntent::TaxOptimization),
        "RetirementPlanning" => Ok(FinancialQueryIntent::RetirementPlanning),
        "CashFlowAnalysis" => Ok(FinancialQueryIntent::CashFlowAnalysis),
        "InvestmentRecommendation" => Ok(FinancialQueryIntent::InvestmentRecommendation),
        "RiskAssessment" => Ok(FinancialQueryIntent::RiskAssessment),
        "MarketInformation" => Ok(FinancialQueryIntent::MarketInformation),
        "FinancialEducation" => Ok(FinancialQueryIntent::FinancialEducation),
        "AccountInformation" => Ok(FinancialQueryIntent::AccountInformation),
        "TransactionHistory" => Ok(FinancialQueryIntent::TransactionHistory),
        "BudgetAnalysis" => Ok(FinancialQueryIntent::BudgetAnalysis),
        "DebtManagement" => Ok(FinancialQueryIntent::DebtManagement),
        "InsuranceAnalysis" => Ok(FinancialQueryIntent::InsuranceAnalysis),
        "EstatePlanning" => Ok(FinancialQueryIntent::EstatePlanning),
        "CharitableGiving" => Ok(FinancialQueryIntent::CharitableGiving),
        "Unknown" => Ok(FinancialQueryIntent::Unknown),
        _ => {
            // If the intent is not recognized, return Unknown
            Ok(FinancialQueryIntent::Unknown)
        }
    }
}

/// Validate the intent confidence from an LLM response
fn validate_intent_confidence(parsed: &Value) -> Result<f64> {
    let confidence = parsed.get("confidence")
        .ok_or_else(|| anyhow!("Missing 'confidence' field in LLM response"))?
        .as_f64()
        .ok_or_else(|| anyhow!("'confidence' field is not a number"))?;
    
    // Ensure the confidence is between 0 and 1
    if confidence < 0.0 || confidence > 1.0 {
        return Err(anyhow!("Confidence score must be between 0 and 1"));
    }
    
    Ok(confidence)
}

/// Validate the entities from an LLM response
fn validate_entities(parsed: &Value) -> Result<Vec<ValidatedEntity>> {
    let entities = parsed.get("entities")
        .ok_or_else(|| anyhow!("Missing 'entities' field in LLM response"))?
        .as_array()
        .ok_or_else(|| anyhow!("'entities' field is not an array"))?;
    
    let mut validated_entities = Vec::new();
    
    for entity in entities {
        // Get entity type
        let entity_type_str = entity.get("type")
            .ok_or_else(|| anyhow!("Missing 'type' field in entity"))?
            .as_str()
            .ok_or_else(|| anyhow!("'type' field is not a string"))?;
        
        // Map the entity type string to an EntityType
        let entity_type = match entity_type_str {
            "TimePeriod" => EntityType::TimePeriod,
            "AccountType" => EntityType::AccountType,
            "AssetClass" => EntityType::AssetClass,
            "Goal" => EntityType::Goal,
            "Amount" => EntityType::Amount,
            "Date" => EntityType::Date,
            "RiskLevel" => EntityType::RiskLevel,
            "Metric" => EntityType::Metric,
            "Security" => EntityType::Security,
            "TaxType" => EntityType::TaxType,
            "Expense" => EntityType::Expense,
            "Income" => EntityType::Income,
            "Insurance" => EntityType::Insurance,
            "Debt" => EntityType::Debt,
            _ => {
                // Skip unrecognized entity types
                continue;
            }
        };
        
        // Get entity value
        let value = entity.get("value")
            .ok_or_else(|| anyhow!("Missing 'value' field in entity"))?
            .as_str()
            .ok_or_else(|| anyhow!("'value' field is not a string"))?
            .to_string();
        
        // Get entity confidence
        let confidence = entity.get("confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.8);
        
        // Ensure the confidence is between 0 and 1
        let confidence = if confidence < 0.0 || confidence > 1.0 {
            0.8 // Default to 0.8 if out of range
        } else {
            confidence
        };
        
        validated_entities.push(ValidatedEntity {
            entity_type,
            value,
            confidence,
        });
    }
    
    Ok(validated_entities)
}

/// Validate a response against client data
pub fn validate_response_facts(response: &str, _client_data: &super::types::ClientData) -> Result<String> {
    // This is a placeholder for a more sophisticated validation system
    // In a real implementation, this would check for factual accuracy
    // by comparing the response to the client data
    
    // For now, we'll just return the response
    Ok(response.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_valid_response() {
        let response = r#"{
            "intent": "PortfolioPerformance",
            "confidence": 0.85,
            "entities": [
                {
                    "type": "TimePeriod",
                    "value": "last year",
                    "confidence": 0.9
                }
            ],
            "uncertain": false
        }"#;
        
        let validated = validate_llm_response(response).unwrap();
        
        assert_eq!(validated.intent, FinancialQueryIntent::PortfolioPerformance);
        assert_eq!(validated.intent_confidence, 0.85);
        assert_eq!(validated.entities.len(), 1);
        assert_eq!(validated.entities[0].entity_type, EntityType::TimePeriod);
        assert_eq!(validated.entities[0].value, "last year");
        assert_eq!(validated.entities[0].confidence, 0.9);
        assert!(!validated.is_uncertain);
    }
    
    #[test]
    fn test_validate_invalid_intent() {
        let response = r#"{
            "intent": "InvalidIntent",
            "confidence": 0.85,
            "entities": [],
            "uncertain": false
        }"#;
        
        let validated = validate_llm_response(response).unwrap();
        
        // Should default to Unknown for unrecognized intents
        assert_eq!(validated.intent, FinancialQueryIntent::Unknown);
    }
    
    #[test]
    fn test_validate_missing_fields() {
        let response = r#"{
            "intent": "PortfolioPerformance"
        }"#;
        
        // Should fail due to missing confidence field
        assert!(validate_llm_response(response).is_err());
    }
    
    #[test]
    fn test_validate_invalid_json() {
        let response = r#"This is not JSON"#;
        
        // Should fail due to invalid JSON
        assert!(validate_llm_response(response).is_err());
    }
} 