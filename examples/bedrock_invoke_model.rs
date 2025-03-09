// Bedrock Invoke Model Example
//
// This example demonstrates how to invoke a model using the AWS Bedrock Runtime client.
// It sends a simple prompt to a Claude model and displays the response.

use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_bedrockruntime::Client as BedrockRuntimeClient;
use serde_json::{json, Value};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Initializing AWS Bedrock Runtime client...");
    
    // Load AWS configuration
    let aws_config = aws_config::defaults(BehaviorVersion::latest())
        .region("us-east-1")
        .load()
        .await;
    
    // Create Bedrock Runtime client
    let bedrock_client = BedrockRuntimeClient::new(&aws_config);
    
    // Claude model ID
    let model_id = "anthropic.claude-3-7-sonnet-20250219-v1:0";
    
    println!("Testing connection to AWS Bedrock Runtime with model: {}", model_id);
    
    // Create a simple prompt for Claude
    let prompt = "Human: Tell me a short joke about programming.\n\nAssistant:";
    
    // Create the request body for Claude
    let request_body = json!({
        "prompt": prompt,
        "max_tokens_to_sample": 300,
        "temperature": 0.7,
        "top_p": 0.9,
    });
    
    // Convert the request body to bytes
    let request_bytes = serde_json::to_vec(&request_body)?;
    
    // Invoke the model
    match bedrock_client
        .invoke_model()
        .model_id(model_id)
        .content_type("application/json")
        .accept("application/json")
        .body(request_bytes.into())
        .send()
        .await
    {
        Ok(response) => {
            // Parse the response body
            let response_body = response.body.as_ref();
            let response_json: Value = serde_json::from_slice(response_body)?;
            
            println!("\n✅ Successfully invoked AWS Bedrock model!");
            
            // Extract the completion from the response
            if let Some(completion) = response_json.get("completion") {
                println!("\nResponse from Claude:");
                println!("{}", completion.as_str().unwrap_or(""));
            } else {
                println!("\nUnexpected response format:");
                println!("{}", serde_json::to_string_pretty(&response_json)?);
            }
            
            // Interactive mode
            println!("\nEntering interactive mode. Type 'exit' to quit.");
            let mut input = String::new();
            
            loop {
                print!("\nEnter a prompt for Claude: ");
                io::stdout().flush()?;
                
                input.clear();
                io::stdin().read_line(&mut input)?;
                
                let user_input = input.trim();
                if user_input.to_lowercase() == "exit" {
                    break;
                }
                
                // Format the prompt for Claude
                let prompt = format!("Human: {}\n\nAssistant:", user_input);
                
                // Create the request body
                let request_body = json!({
                    "prompt": prompt,
                    "max_tokens_to_sample": 300,
                    "temperature": 0.7,
                    "top_p": 0.9,
                });
                
                // Convert the request body to bytes
                let request_bytes = serde_json::to_vec(&request_body)?;
                
                // Invoke the model
                match bedrock_client
                    .invoke_model()
                    .model_id(model_id)
                    .content_type("application/json")
                    .accept("application/json")
                    .body(request_bytes.into())
                    .send()
                    .await
                {
                    Ok(response) => {
                        // Parse the response body
                        let response_body = response.body.as_ref();
                        let response_json: Value = serde_json::from_slice(response_body)?;
                        
                        // Extract the completion from the response
                        if let Some(completion) = response_json.get("completion") {
                            println!("\nClaude: {}", completion.as_str().unwrap_or(""));
                        } else {
                            println!("\nUnexpected response format:");
                            println!("{}", serde_json::to_string_pretty(&response_json)?);
                        }
                    },
                    Err(e) => {
                        println!("\n❌ Failed to invoke AWS Bedrock model: {}", e);
                    }
                }
            }
        },
        Err(e) => {
            println!("\n❌ Failed to invoke AWS Bedrock model: {}", e);
            println!("\nTROUBLESHOOTING TIPS:");
            println!("1. Verify your AWS credentials have the 'bedrock:InvokeModel' permission");
            println!("2. Check if the model ID is correct: {}", model_id);
            println!("3. Ensure you have access to the specified model in your AWS account");
            println!("4. Try a different AWS region (currently using us-east-1)");
        }
    }
    
    Ok(())
} 