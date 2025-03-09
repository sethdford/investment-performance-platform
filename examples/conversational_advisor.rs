use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_bedrockruntime::Client as BedrockRuntimeClient;
use chrono::Utc;
use std::sync::Arc;
use std::io::{self, Write};
use uuid::Uuid;
use std::collections::HashSet;

use investment_management::financial_advisor::{
    client_profiling::{ClientProfile, FinancialGoal, GoalType, GoalStatus, RiskToleranceLevel, TimeHorizon, GoalPriority},
    nlp::{
        bedrock::{BedrockNlpClient, BedrockNlpConfig, BedrockModelProvider},
        embeddings::TitanEmbeddingConfig,
        enhanced_hybrid::{EnhancedHybridService, EnhancedHybridConfig},
        knowledge_retriever::{KnowledgeRetriever, KnowledgeRetrieverConfig},
        reasoning::{FinancialReasoningService, ReasoningServiceConfig},
        conversation_manager::ConversationManager,
        FinancialQueryIntent,
    },
};

// Define a simple ClientData struct for the example
#[derive(Debug, Clone)]
struct ClientData {
    name: String,
    age: u32,
    income: f64,
    assets: f64,
    liabilities: f64,
}

/// Create a sample client profile
fn create_sample_client_profile() -> ClientProfile {
    let mut profile = ClientProfile {
        id: Uuid::new_v4(),
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        phone: "555-123-4567".to_string(),
        date_of_birth: Utc::now() - chrono::Duration::days(365 * 40), // 40 years old
        tax_bracket: 24.0,
        state_of_residence: "California".to_string(),
        retirement_age: Some(65),
        risk_tolerance: RiskToleranceLevel::Moderate,
        investment_experience: 10, // 10 years
        financial_goals: Vec::new(),
        income_sources: Vec::new(),
        expenses: Vec::new(),
        assets: Vec::new(),
        liabilities: Vec::new(),
        insurance_policies: Vec::new(),
        behavioral_biases: HashSet::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    // Add retirement goal
    let retirement_goal = FinancialGoal {
        id: Uuid::new_v4(),
        name: "Retirement".to_string(),
        description: "Comfortable retirement at age 65".to_string(),
        goal_type: GoalType::Retirement,
        target_amount: 2_000_000.0,
        current_amount: 500_000.0,
        target_date: Utc::now() + chrono::Duration::days(365 * 25), // 25 years from now
        priority: GoalPriority::Essential,
        status: GoalStatus::OnTrack,
        time_horizon: TimeHorizon::VeryLong,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    profile.financial_goals.push(retirement_goal);
    
    // Add education goal
    let education_goal = FinancialGoal {
        id: Uuid::new_v4(),
        name: "College Education".to_string(),
        description: "Fund children's college education".to_string(),
        goal_type: GoalType::Education,
        target_amount: 300_000.0,
        current_amount: 50_000.0,
        target_date: Utc::now() + chrono::Duration::days(365 * 10), // 10 years from now
        priority: GoalPriority::Important,
        status: GoalStatus::OnTrack,
        time_horizon: TimeHorizon::Long,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    profile.financial_goals.push(education_goal);
    
    // Add home purchase goal
    let home_goal = FinancialGoal {
        id: Uuid::new_v4(),
        name: "Vacation Home".to_string(),
        description: "Purchase a vacation home".to_string(),
        goal_type: GoalType::HomePurchase,
        target_amount: 500_000.0,
        current_amount: 100_000.0,
        target_date: Utc::now() + chrono::Duration::days(365 * 5), // 5 years from now
        priority: GoalPriority::WantToHave,
        status: GoalStatus::BehindSchedule,
        time_horizon: TimeHorizon::Medium,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    profile.financial_goals.push(home_goal);
    
    profile
}

/// Initialize AWS clients
async fn initialize_aws_clients() -> Result<BedrockRuntimeClient> {
    let aws_config = aws_config::defaults(BehaviorVersion::latest())
        .region("us-east-1")
        .load()
        .await;
    
    let bedrock_client = BedrockRuntimeClient::new(&aws_config);
    
    Ok(bedrock_client)
}

/// Create the financial advisor services
async fn create_services() -> Result<(EnhancedHybridService, Arc<KnowledgeRetriever>, FinancialReasoningService)> {
    // Initialize AWS clients
    let bedrock_client = initialize_aws_clients().await?;

    // Create Bedrock NLP config
    let bedrock_config = BedrockNlpConfig {
        provider: BedrockModelProvider::Claude,
        version: "v2".to_string(),
        temperature: 0.7,
        max_tokens: 1000,
        top_p: 0.9,
        top_k: 250,
        stop_sequences: vec!["\n\nHuman:".to_string()],
    };

    // Create Titan embedding config
    let embedding_config = TitanEmbeddingConfig {
        model_id: "amazon.titan-embed-text-v1".to_string(),
        max_cache_size: 1000,
        cache_ttl_seconds: 3600,
    };

    // Create knowledge retriever
    let knowledge_retriever = KnowledgeRetriever::new(KnowledgeRetrieverConfig::default());
    let knowledge_retriever_arc = Arc::new(knowledge_retriever);

    // Create enhanced hybrid service
    let enhanced_service = EnhancedHybridService::new_with_embeddings(
        bedrock_client.clone(),
        bedrock_config.clone(),
        embedding_config,
    )
    .with_knowledge_retriever(knowledge_retriever_arc.clone())
    .with_config(EnhancedHybridConfig::default());

    // Create reasoning service
    let reasoning_service = FinancialReasoningService::new(ReasoningServiceConfig::default())
        .with_bedrock(Arc::new(BedrockNlpClient::new(bedrock_client.clone(), bedrock_config.clone())));

    Ok((enhanced_service, knowledge_retriever_arc, reasoning_service))
}

/// Run the conversational advisor
async fn run_advisor(
    enhanced_service: &EnhancedHybridService,
    reasoning_service: &FinancialReasoningService,
    conversation_manager: &mut ConversationManager,
    client_profile: &ClientProfile,
) -> Result<()> {
    println!("Financial Advisor AI (type 'exit' to quit)");
    println!("------------------------------------------");
    
    loop {
        // Get user input
        print!("\nYou: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let input = input.trim();
        if input.to_lowercase() == "exit" {
            break;
        }
        
        // Add user query to conversation
        conversation_manager.add_user_query(input);
        
        // Process the query
        let response = enhanced_service.process_query(input, Some(conversation_manager), None).await?;
        
        // Update conversation with response
        if let Some(processed_query) = &response.processed_query {
            conversation_manager.update_current_turn_with_processed_query(processed_query.clone())?;
        }
        conversation_manager.update_current_turn_with_response(response.clone())?;
        
        // Generate reasoning chain for complex queries
        if response.confidence > 0.8 && matches!(
            response.intent,
            FinancialQueryIntent::RetirementPlanning |
            FinancialQueryIntent::AssetAllocation |
            FinancialQueryIntent::TaxOptimization |
            FinancialQueryIntent::GoalProgress
        ) {
            let reasoning_chain = reasoning_service.generate_reasoning_chain(
                input,
                response.intent.clone(),
                None,
                Some(client_profile),
            ).await?;
            
            println!("\nReasoning Chain: {}", reasoning_chain.generate_summary());
        }
        
        // Print the response
        println!("\nAdvisor: {}", response.response_text);
    }
    
    Ok(())
}

/// Main function
#[tokio::main]
async fn main() -> Result<()> {
    // Create client profile and data
    let client_profile = create_sample_client_profile();
    let _client_data = ClientData {
        name: format!("{} {}", client_profile.first_name, client_profile.last_name),
        age: 45,
        income: 150000.0,
        assets: 750000.0,
        liabilities: 250000.0,
    };

    // Create services
    let (enhanced_service, _knowledge_retriever, reasoning_service) = create_services().await?;

    // Run advisor
    let mut conversation_manager = ConversationManager::new("client123");
    run_advisor(&enhanced_service, &reasoning_service, &mut conversation_manager, &client_profile).await?;

    Ok(())
} 