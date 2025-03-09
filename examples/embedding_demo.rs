use anyhow::{Result, anyhow};
use aws_sdk_bedrockruntime::Client as BedrockRuntimeClient;
use std::sync::Arc;
use std::io::{self, Write};
use serde::{Serialize, Deserialize};
use aws_sdk_bedrock::Client as BedrockManagementClient;

use investment_management::financial_advisor::nlp::{
    EmbeddingService, TitanEmbeddingConfig,
    KnowledgeRetriever, KnowledgeItem, KnowledgeSourceType, KnowledgeRetrieverConfig,
    FinancialQueryIntent,
    ClientData,
    PortfolioData,
    AssetAllocation,
    PerformanceData
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Embedding Service Demo");
    println!("=====================");

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
    
    // Add sample knowledge items
    populate_knowledge_base(&mut knowledge_retriever)?;
    
    // Generate embeddings for knowledge items
    println!("Generating embeddings for knowledge items...");
    knowledge_retriever.generate_embeddings().await?;
    
    // Create sample client data
    let client_data = create_sample_client_data();
    let similar_clients = create_similar_client_data();
    
    // Demo menu
    loop {
        println!("\nEmbedding Demo Options:");
        println!("1. Semantic Search for Financial Questions");
        println!("2. Find Similar Clients");
        println!("3. Knowledge Retrieval for Financial Topics");
        println!("4. Intent Classification");
        println!("5. Embedding Cache Performance");
        println!("6. Exit");
        print!("\nSelect an option (1-6): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "1" => demo_semantic_search(&embedding_service_arc).await?,
            "2" => demo_similar_clients(&embedding_service_arc, &client_data, &similar_clients.as_slice()).await?,
            "3" => demo_knowledge_retrieval(&knowledge_retriever).await?,
            "4" => demo_intent_classification(&embedding_service_arc).await?,
            "5" => demo_cache_performance(&embedding_service_arc).await?,
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
    let config = aws_config::load_from_env().await;
    
    // Create Bedrock client
    let bedrock_client = BedrockRuntimeClient::new(&config);
    
    Ok(bedrock_client)
}

/// Populate knowledge base with sample items
fn populate_knowledge_base(knowledge_retriever: &mut KnowledgeRetriever) -> Result<()> {
    // Tax-Loss Harvesting
    knowledge_retriever.add_item(KnowledgeItem {
        id: "tax-loss-harvesting-1".to_string(),
        title: "Tax-Loss Harvesting".to_string(),
        content: "Tax-loss harvesting is the practice of selling a security that has experienced a loss. By realizing a loss, investors can offset taxes on both gains and income. The sold security is replaced by a similar one, maintaining the optimal asset allocation and expected returns.".to_string(),
        source_type: KnowledgeSourceType::TaxRules,
        tags: vec!["tax".to_string(), "investment".to_string(), "strategy".to_string()],
        related_intents: vec![FinancialQueryIntent::TaxOptimization],
        embedding: None,
    })?;
    
    // Roth Conversion
    knowledge_retriever.add_item(KnowledgeItem {
        id: "roth-conversion-1".to_string(),
        title: "Roth IRA Conversion".to_string(),
        content: "A Roth IRA conversion involves transferring retirement funds from a traditional IRA, SEP IRA, SIMPLE IRA, or retirement plan like a 401(k) to a Roth IRA. The conversion amount is generally subject to income tax in the year of conversion, but qualified withdrawals from the Roth IRA in the future are tax-free.".to_string(),
        source_type: KnowledgeSourceType::TaxRules,
        tags: vec!["tax".to_string(), "retirement".to_string(), "IRA".to_string()],
        related_intents: vec![FinancialQueryIntent::RetirementPlanning, FinancialQueryIntent::TaxOptimization],
        embedding: None,
    })?;
    
    // Asset Allocation
    knowledge_retriever.add_item(KnowledgeItem {
        id: "asset-allocation-1".to_string(),
        title: "Asset Allocation".to_string(),
        content: "Asset allocation is an investment strategy that aims to balance risk and reward by apportioning a portfolio's assets according to an individual's goals, risk tolerance, and investment horizon. The three main asset classes - equities, fixed-income, and cash and equivalents - have different levels of risk and return, so each will behave differently over time.".to_string(),
        source_type: KnowledgeSourceType::InvestmentStrategy,
        tags: vec!["investment".to_string(), "portfolio".to_string(), "strategy".to_string()],
        related_intents: vec![FinancialQueryIntent::AssetAllocation, FinancialQueryIntent::PortfolioPerformance],
        embedding: None,
    })?;
    
    // Retirement Planning
    knowledge_retriever.add_item(KnowledgeItem {
        id: "retirement-planning-1".to_string(),
        title: "Retirement Planning".to_string(),
        content: "Retirement planning refers to the process of determining retirement income goals, and the actions and decisions necessary to achieve those goals. Retirement planning includes identifying sources of income, estimating expenses, implementing a savings program, and managing assets and risk. Future cash flows are estimated to determine if the retirement income goal will be achieved.".to_string(),
        source_type: KnowledgeSourceType::RetirementPlanning,
        tags: vec!["retirement".to_string(), "planning".to_string(), "goals".to_string()],
        related_intents: vec![FinancialQueryIntent::RetirementPlanning, FinancialQueryIntent::GoalProgress],
        embedding: None,
    })?;
    
    // 4% Rule
    knowledge_retriever.add_item(KnowledgeItem {
        id: "four-percent-rule-1".to_string(),
        title: "The 4% Rule".to_string(),
        content: "The 4% rule is a guideline used to determine how much a retiree should withdraw from a retirement account each year. This rule seeks to provide a steady income stream to the retiree while also maintaining an account balance that keeps income flowing through retirement. Experts recommend withdrawing 4% of your retirement portfolio in the first year of retirement, then adjusting that amount for inflation each subsequent year.".to_string(),
        source_type: KnowledgeSourceType::RetirementPlanning,
        tags: vec!["retirement".to_string(), "withdrawal".to_string(), "strategy".to_string()],
        related_intents: vec![FinancialQueryIntent::RetirementPlanning],
        embedding: None,
    })?;
    
    Ok(())
}

/// Create sample client data
fn create_sample_client_data() -> ClientData {
    ClientData {
        client_id: "client-123".to_string(),
        client_name: Some("John Doe".to_string()),
        portfolio: Some(PortfolioData {
            portfolio_id: "portfolio-123".to_string(),
            portfolio_name: Some("Main Portfolio".to_string()),
            total_value: 500000.0,
            asset_allocation: vec![
                AssetAllocation {
                    asset_class: "Stocks".to_string(),
                    allocation_percentage: 0.6,
                    current_value: 300000.0,
                    target_allocation_percentage: Some(0.6),
                },
                AssetAllocation {
                    asset_class: "Bonds".to_string(),
                    allocation_percentage: 0.3,
                    current_value: 150000.0,
                    target_allocation_percentage: Some(0.3),
                },
                AssetAllocation {
                    asset_class: "Cash".to_string(),
                    allocation_percentage: 0.1,
                    current_value: 50000.0,
                    target_allocation_percentage: Some(0.1),
                },
            ],
            performance: PerformanceData {
                ytd_return: 0.08,
                one_year_return: Some(0.12),
                three_year_return: Some(0.25),
                five_year_return: Some(0.40),
                since_inception_return: Some(0.45),
                risk_metrics: None,
            },
        }),
        goals: None,
        cash_flow: None,
        tax_data: None,
    }
}

/// Create similar client data
fn create_similar_client_data() -> Vec<ClientData> {
    vec![
        ClientData {
            client_id: "client-456".to_string(),
            client_name: Some("Jane Smith".to_string()),
            portfolio: Some(PortfolioData {
                portfolio_id: "portfolio-456".to_string(),
                portfolio_name: Some("Main Portfolio".to_string()),
                total_value: 450000.0,
                asset_allocation: vec![
                    AssetAllocation {
                        asset_class: "Stocks".to_string(),
                        allocation_percentage: 0.55,
                        current_value: 247500.0,
                        target_allocation_percentage: Some(0.6),
                    },
                    AssetAllocation {
                        asset_class: "Bonds".to_string(),
                        allocation_percentage: 0.35,
                        current_value: 157500.0,
                        target_allocation_percentage: Some(0.3),
                    },
                    AssetAllocation {
                        asset_class: "Cash".to_string(),
                        allocation_percentage: 0.1,
                        current_value: 45000.0,
                        target_allocation_percentage: Some(0.1),
                    },
                ],
                performance: PerformanceData {
                    ytd_return: 0.07,
                    one_year_return: Some(0.11),
                    three_year_return: Some(0.22),
                    five_year_return: Some(0.38),
                    since_inception_return: Some(0.42),
                    risk_metrics: None,
                },
            }),
            goals: None,
            cash_flow: None,
            tax_data: None,
        },
        ClientData {
            client_id: "client-789".to_string(),
            client_name: Some("Robert Johnson".to_string()),
            portfolio: Some(PortfolioData {
                portfolio_id: "portfolio-789".to_string(),
                portfolio_name: Some("Main Portfolio".to_string()),
                total_value: 750000.0,
                asset_allocation: vec![
                    AssetAllocation {
                        asset_class: "Stocks".to_string(),
                        allocation_percentage: 0.8,
                        current_value: 600000.0,
                        target_allocation_percentage: Some(0.8),
                    },
                    AssetAllocation {
                        asset_class: "Bonds".to_string(),
                        allocation_percentage: 0.15,
                        current_value: 112500.0,
                        target_allocation_percentage: Some(0.15),
                    },
                    AssetAllocation {
                        asset_class: "Cash".to_string(),
                        allocation_percentage: 0.05,
                        current_value: 37500.0,
                        target_allocation_percentage: Some(0.05),
                    },
                ],
                performance: PerformanceData {
                    ytd_return: 0.10,
                    one_year_return: Some(0.15),
                    three_year_return: Some(0.30),
                    five_year_return: Some(0.50),
                    since_inception_return: Some(0.55),
                    risk_metrics: None,
                },
            }),
            goals: None,
            cash_flow: None,
            tax_data: None,
        },
    ]
}

/// Demo semantic search for financial questions
async fn demo_semantic_search(embedding_service: &Arc<EmbeddingService>) -> Result<()> {
    println!("\n=== Semantic Search for Financial Questions ===");
    println!("This demo shows how embeddings enable semantic understanding of financial queries.");
    
    // Initialize intent embeddings
    let mut embedding_service_mut = embedding_service.as_ref().clone();
    embedding_service_mut.initialize_intent_embeddings().await?;
    
    // Sample queries
    let queries = [
        "How much should I save for retirement?",
        "What's the best way to reduce my taxes?",
        "Is my portfolio diversified enough?",
        "Am I on track to meet my financial goals?",
    ];
    
    for query in &queries {
        println!("\nQuery: {}", query);
        
        // Find similar intent
        let (intent, confidence) = embedding_service.find_similar_intent(query).await?;
        
        println!("Matched Intent: {:?} (Confidence: {:.2})", intent, confidence);
    }
    
    // Custom query
    println!("\nEnter your own financial question (or press Enter to continue):");
    print!("> ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let input = input.trim();
    if !input.is_empty() {
        let (intent, confidence) = embedding_service.find_similar_intent(input).await?;
        println!("Matched Intent: {:?} (Confidence: {:.2})", intent, confidence);
    }
    
    Ok(())
}

/// Demo finding similar clients
async fn demo_similar_clients(
    embedding_service: &Arc<EmbeddingService>,
    client_data: &ClientData,
    similar_clients: &[ClientData],
) -> Result<()> {
    println!("\n=== Finding Similar Clients ===");
    println!("This demo shows how embeddings can find clients with similar financial profiles.");
    
    // Find similar clients
    let similar_results = embedding_service.find_similar_clients(client_data, similar_clients).await?;
    
    println!("\nClient: {}", client_data.client_name.as_ref().unwrap_or(&"Unknown".to_string()));
    println!("Portfolio: ${:.2} ({}% stocks, {}% bonds, {}% cash)",
        client_data.portfolio.as_ref().unwrap().total_value,
        client_data.portfolio.as_ref().unwrap().asset_allocation[0].allocation_percentage * 100.0,
        client_data.portfolio.as_ref().unwrap().asset_allocation[1].allocation_percentage * 100.0,
        client_data.portfolio.as_ref().unwrap().asset_allocation[2].allocation_percentage * 100.0,
    );
    
    println!("\nSimilar Clients:");
    for (client_id, similarity) in similar_results {
        let similar_client = similar_clients.iter().find(|c| c.client_id == client_id).unwrap();
        
        println!("- {} (Similarity: {:.2}%)",
            similar_client.client_name.as_ref().unwrap_or(&"Unknown".to_string()),
            similarity * 100.0,
        );
        
        println!("  Portfolio: ${:.2} ({}% stocks, {}% bonds, {}% cash)",
            similar_client.portfolio.as_ref().unwrap().total_value,
            similar_client.portfolio.as_ref().unwrap().asset_allocation[0].allocation_percentage * 100.0,
            similar_client.portfolio.as_ref().unwrap().asset_allocation[1].allocation_percentage * 100.0,
            similar_client.portfolio.as_ref().unwrap().asset_allocation[2].allocation_percentage * 100.0,
        );
    }
    
    Ok(())
}

/// Demo knowledge retrieval for financial topics
async fn demo_knowledge_retrieval(knowledge_retriever: &KnowledgeRetriever) -> Result<()> {
    println!("\n=== Knowledge Retrieval for Financial Topics ===");
    println!("This demo shows how embeddings enable retrieving relevant financial knowledge.");
    
    // Sample queries
    let queries = [
        "How does tax-loss harvesting work?",
        "What is a Roth conversion?",
        "How should I allocate my assets?",
        "What is the 4% rule for retirement?",
    ];
    
    for query in &queries {
        println!("\nQuery: {}", query);
        
        // Retrieve knowledge
        let knowledge_items = knowledge_retriever.retrieve_by_query(query).await?;
        
        println!("Retrieved {} knowledge items:", knowledge_items.len());
        for (i, item) in knowledge_items.iter().enumerate() {
            println!("{}. {} (Source: {:?})", i + 1, item.title, item.source_type);
            println!("   Tags: {}", item.tags.join(", "));
        }
    }
    
    // Custom query
    println!("\nEnter your own financial question (or press Enter to continue):");
    print!("> ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let input = input.trim();
    if !input.is_empty() {
        let knowledge_items = knowledge_retriever.retrieve_by_query(input).await?;
        
        println!("Retrieved {} knowledge items:", knowledge_items.len());
        for (i, item) in knowledge_items.iter().enumerate() {
            println!("{}. {} (Source: {:?})", i + 1, item.title, item.source_type);
            println!("   Tags: {}", item.tags.join(", "));
        }
    }
    
    Ok(())
}

/// Demo intent classification
async fn demo_intent_classification(embedding_service: &Arc<EmbeddingService>) -> Result<()> {
    println!("\n=== Intent Classification ===");
    println!("This demo shows how embeddings enable classifying financial intents with nuance.");
    
    // Initialize intent embeddings
    let mut embedding_service_mut = embedding_service.as_ref().clone();
    embedding_service_mut.initialize_intent_embeddings().await?;
    
    // Sample queries with nuanced differences
    let queries = [
        "How is my portfolio performing?",
        "What's the performance of my investments?",
        "Are my investments doing well?",
        "What's my current asset allocation?",
        "How are my assets distributed?",
        "What's the breakdown of my portfolio?",
    ];
    
    for query in &queries {
        println!("\nQuery: {}", query);
        
        // Find similar intent
        let (intent, confidence) = embedding_service.find_similar_intent(query).await?;
        
        println!("Classified Intent: {:?} (Confidence: {:.2})", intent, confidence);
    }
    
    Ok(())
}

/// Demo cache performance
async fn demo_cache_performance(embedding_service: &Arc<EmbeddingService>) -> Result<()> {
    println!("\n=== Embedding Cache Performance ===");
    println!("This demo shows the performance benefits of embedding caching.");
    
    // Sample texts
    let texts = [
        "How much should I save for retirement?".to_string(),
        "What's the best way to reduce my taxes?".to_string(),
        "Is my portfolio diversified enough?".to_string(),
        "Am I on track to meet my financial goals?".to_string(),
    ];
    
    // First run - should be slower as it populates the cache
    println!("\nFirst run (populating cache):");
    let start = std::time::Instant::now();
    let embeddings = embedding_service.generate_embeddings_batch(&texts).await?;
    let duration = start.elapsed();
    
    println!("Generated {} embeddings in {:?}", embeddings.len(), duration);
    
    // Get cache stats
    let (cache_size, cache_capacity) = embedding_service.get_cache_stats();
    println!("Cache stats: {}/{} items", cache_size, cache_capacity);
    
    // Second run - should be faster as it uses the cache
    println!("\nSecond run (using cache):");
    let start = std::time::Instant::now();
    let embeddings = embedding_service.generate_embeddings_batch(&texts).await?;
    let duration = start.elapsed();
    
    println!("Generated {} embeddings in {:?}", embeddings.len(), duration);
    
    // Get cache stats
    let (cache_size, cache_capacity) = embedding_service.get_cache_stats();
    println!("Cache stats: {}/{} items", cache_size, cache_capacity);
    
    // Invalidate cache
    println!("\nInvalidating cache for first item...");
    embedding_service.invalidate_embeddings(&[texts[0].clone()])?;
    
    // Get cache stats
    let (cache_size, cache_capacity) = embedding_service.get_cache_stats();
    println!("Cache stats after invalidation: {}/{} items", cache_size, cache_capacity);
    
    // Third run - should be partially cached
    println!("\nThird run (partially cached):");
    let start = std::time::Instant::now();
    let embeddings = embedding_service.generate_embeddings_batch(&texts).await?;
    let duration = start.elapsed();
    
    println!("Generated {} embeddings in {:?}", embeddings.len(), duration);
    
    // Get cache stats
    let (cache_size, cache_capacity) = embedding_service.get_cache_stats();
    println!("Cache stats: {}/{} items", cache_size, cache_capacity);
    
    Ok(())
} 