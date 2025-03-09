use anyhow::Result;
use std::io::{self, Write};
use std::sync::Arc;

// Import the nlp module
use investment_management::financial_advisor::nlp::{
    EmbeddingService,
    TitanEmbeddingConfig,
    KnowledgeRetriever,
    KnowledgeRetrieverConfig,
    FinancialQueryIntent,
};

// Import the financial knowledge base module directly
use investment_management::financial_advisor::nlp::financial_knowledge_base::{
    FinancialKnowledgeBase,
    FinancialKnowledgeBaseConfig,
    FinancialKnowledgeCategory,
    FinancialKnowledgeSource,
    create_default_knowledge_base,
};

use aws_sdk_bedrockruntime::Client as BedrockRuntimeClient;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Financial Knowledge Base Demo");
    println!("============================");
    
    // Initialize AWS clients
    let bedrock_client = initialize_aws_clients().await?;
    
    // Create Titan embedding config
    let embedding_config = TitanEmbeddingConfig {
        model_id: "amazon.titan-embed-text-v1".to_string(),
        max_cache_size: 1000,
        cache_ttl_seconds: 3600,
    };
    
    // Create embedding service
    let embedding_service = EmbeddingService::new(bedrock_client.clone(), embedding_config);
    let embedding_service_arc = Arc::new(embedding_service);
    
    // Create knowledge retriever
    let mut knowledge_retriever = KnowledgeRetriever::new(KnowledgeRetrieverConfig {
        max_items: 5,
        similarity_threshold: 0.7,
        use_embeddings: true,
    }).with_embedding_service(embedding_service_arc.clone());
    
    // Create financial knowledge base
    let kb = create_default_knowledge_base()?;
    
    // Populate knowledge retriever with entries from the knowledge base
    kb.populate_knowledge_retriever(&mut knowledge_retriever)?;
    
    // Generate embeddings for knowledge items
    println!("Generating embeddings for knowledge items...");
    knowledge_retriever.generate_embeddings().await?;
    
    // Demo menu
    loop {
        println!("\nFinancial Knowledge Base Options:");
        println!("1. Search by Query");
        println!("2. Search by Category");
        println!("3. Search by Intent");
        println!("4. List All Categories");
        println!("5. Retrieve Knowledge for Financial Query");
        println!("6. Exit");
        print!("\nSelect an option (1-6): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "1" => demo_search_by_query(&kb)?,
            "2" => demo_search_by_category(&kb)?,
            "3" => demo_search_by_intent(&kb)?,
            "4" => demo_list_categories(&kb)?,
            "5" => demo_retrieve_knowledge(&knowledge_retriever).await?,
            "6" => break,
            _ => println!("Invalid option, please try again."),
        }
    }
    
    Ok(())
}

/// Initialize AWS clients
async fn initialize_aws_clients() -> Result<BedrockRuntimeClient> {
    println!("Initializing AWS clients...");
    
    // Load AWS configuration
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    
    // Create Bedrock client
    let bedrock_client = BedrockRuntimeClient::new(&config);
    
    Ok(bedrock_client)
}

/// Demo search by query
fn demo_search_by_query(kb: &FinancialKnowledgeBase) -> Result<()> {
    print!("Enter search query: ");
    io::stdout().flush()?;
    
    let mut query = String::new();
    io::stdin().read_line(&mut query)?;
    
    let results = kb.search(query.trim());
    
    println!("\nSearch Results:");
    if results.is_empty() {
        println!("No results found.");
    } else {
        for (i, entry) in results.iter().enumerate() {
            println!("{}. {} (Relevance: {:.2})", i+1, entry.title, entry.relevance_score);
            println!("   Category: {:?}", entry.category);
            println!("   Tags: {}", entry.tags.join(", "));
            println!("   Source: {} ({})", entry.source.name, entry.source.last_updated);
            println!("   Content: {}", entry.content);
            println!();
        }
    }
    
    Ok(())
}

/// Demo search by category
fn demo_search_by_category(kb: &FinancialKnowledgeBase) -> Result<()> {
    println!("Categories:");
    let categories = kb.get_categories();
    
    for (i, category) in categories.iter().enumerate() {
        println!("{}. {:?}", i+1, category);
    }
    
    print!("\nSelect category (1-{}): ", categories.len());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if let Ok(index) = input.trim().parse::<usize>() {
        if index > 0 && index <= categories.len() {
            let category = &categories[index - 1];
            let results = kb.get_entries_by_category(category);
            
            println!("\nEntries in {:?} category:", category);
            if results.is_empty() {
                println!("No entries found.");
            } else {
                for (i, entry) in results.iter().enumerate() {
                    println!("{}. {} (Relevance: {:.2})", i+1, entry.title, entry.relevance_score);
                    println!("   Tags: {}", entry.tags.join(", "));
                    println!("   Source: {} ({})", entry.source.name, entry.source.last_updated);
                    println!("   Content: {}", entry.content);
                    println!();
                }
            }
        } else {
            println!("Invalid category index.");
        }
    } else {
        println!("Invalid input.");
    }
    
    Ok(())
}

/// Demo search by intent
fn demo_search_by_intent(kb: &FinancialKnowledgeBase) -> Result<()> {
    println!("Intents:");
    let intents = [
        FinancialQueryIntent::RetirementPlanning,
        FinancialQueryIntent::TaxOptimization,
        FinancialQueryIntent::InvestmentRecommendation,
        FinancialQueryIntent::AssetAllocation,
        FinancialQueryIntent::PortfolioPerformance,
        FinancialQueryIntent::FinancialEducation,
        FinancialQueryIntent::RiskAssessment,
        FinancialQueryIntent::CashFlowAnalysis,
        FinancialQueryIntent::SocialSecurityOptimization,
    ];
    
    for (i, intent) in intents.iter().enumerate() {
        println!("{}. {:?}", i+1, intent);
    }
    
    print!("\nSelect intent (1-{}): ", intents.len());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if let Ok(index) = input.trim().parse::<usize>() {
        if index > 0 && index <= intents.len() {
            let intent = &intents[index - 1];
            let results = kb.search_by_intent(intent);
            
            println!("\nEntries related to {:?} intent:", intent);
            if results.is_empty() {
                println!("No entries found.");
            } else {
                for (i, entry) in results.iter().enumerate() {
                    println!("{}. {} (Relevance: {:.2})", i+1, entry.title, entry.relevance_score);
                    println!("   Category: {:?}", entry.category);
                    println!("   Tags: {}", entry.tags.join(", "));
                    println!("   Source: {} ({})", entry.source.name, entry.source.last_updated);
                    println!("   Content: {}", entry.content);
                    println!();
                }
            }
        } else {
            println!("Invalid intent index.");
        }
    } else {
        println!("Invalid input.");
    }
    
    Ok(())
}

/// Demo list categories
fn demo_list_categories(kb: &FinancialKnowledgeBase) -> Result<()> {
    println!("Categories in Knowledge Base:");
    
    let categories = kb.get_categories();
    
    for (i, category) in categories.iter().enumerate() {
        let entries = kb.get_entries_by_category(category);
        println!("{}. {:?} ({} entries)", i+1, category, entries.len());
    }
    
    Ok(())
}

/// Demo retrieve knowledge
async fn demo_retrieve_knowledge(retriever: &KnowledgeRetriever) -> Result<()> {
    print!("Enter financial query: ");
    io::stdout().flush()?;
    
    let mut query = String::new();
    io::stdin().read_line(&mut query)?;
    
    let results = retriever.retrieve_by_query(query.trim()).await?;
    
    println!("\nRelevant Knowledge:");
    if results.is_empty() {
        println!("No relevant knowledge found.");
    } else {
        for (i, item) in results.iter().enumerate() {
            println!("{}. {}", i+1, item.title);
            println!("   Source Type: {:?}", item.source_type);
            println!("   Tags: {}", item.tags.join(", "));
            println!("   Content: {}", item.content);
            println!();
        }
    }
    
    Ok(())
} 