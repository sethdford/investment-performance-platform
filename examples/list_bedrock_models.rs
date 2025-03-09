// List Bedrock Models
//
// This example lists the available Bedrock models in your AWS account.
// It helps verify that you have access to the Bedrock service and shows
// which models are available for use.

use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_bedrock::Client as BedrockClient;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Initializing AWS Bedrock client...");
    
    // Load AWS configuration
    let aws_config = aws_config::defaults(BehaviorVersion::latest())
        .region("us-east-1")
        .load()
        .await;
    
    // Create Bedrock client
    let bedrock_client = BedrockClient::new(&aws_config);
    
    println!("Listing available foundation models...");
    
    // List foundation models
    match bedrock_client.list_foundation_models().send().await {
        Ok(response) => {
            println!("\n✅ Successfully connected to AWS Bedrock!");
            
            let models = response.model_summaries();
            if models.is_empty() {
                println!("No foundation models found. You may need to request access to models in the AWS console.");
            } else {
                println!("\nFound {} foundation models:", models.len());
                
                for (i, model) in models.iter().enumerate() {
                    println!("Model #{}", i + 1);
                    
                    // Print model details
                    println!("  Model ID: {}", model.model_id());
                    
                    // Print model name with better formatting
                    match model.model_name() {
                        Some(name) => println!("  Model Name: {}", name),
                        None => println!("  Model Name: N/A"),
                    }
                    
                    // Print provider with better formatting
                    match model.provider_name() {
                        Some(provider) => println!("  Provider: {}", provider),
                        None => println!("  Provider: N/A"),
                    }
                    
                    println!("");
                }
                
                println!("\nTo use a model, you need to specify the correct model ID in your code.");
                println!("For example: BedrockModelProvider::Claude, version: \"claude-3-sonnet-20240229-v1:0\"");
            }
        },
        Err(e) => {
            println!("\n❌ Failed to list Bedrock models: {}", e);
            println!("\nTROUBLESHOOTING TIPS:");
            println!("1. Verify your AWS credentials have the 'bedrock:ListFoundationModels' permission");
            println!("2. Check if Bedrock service is enabled in your AWS account");
            println!("3. Try a different AWS region (currently using us-east-1)");
            println!("4. Ensure your AWS account has access to Amazon Bedrock");
        }
    }
    
    Ok(())
} 