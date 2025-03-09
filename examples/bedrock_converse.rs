// Bedrock Converse API Example
//
// This example demonstrates how to use the AWS Bedrock Runtime Converse API
// to interact with a Claude model. The Converse API is a newer, more structured
// way to interact with Bedrock models compared to the InvokeModel API.

use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_bedrockruntime::{
    types::{ContentBlock, ConversationRole, SystemContentBlock},
    Client,
};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the AWS SDK
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .load()
        .await;
    let client = Client::new(&config);

    // Model ID for Claude 3 Sonnet
    let model_id = "anthropic.claude-3-7-sonnet-20250219-v1:0";

    // Store the conversation history as a formatted string
    let mut conversation_history = String::new();

    println!("Starting conversation with Claude 3 Sonnet. Type 'exit' to quit.");
    println!("You: ");
    
    // Main conversation loop
    loop {
        // Get user input
        let mut user_input = String::new();
        io::stdout().flush()?;
        io::stdin().read_line(&mut user_input)?;
        
        // Check if user wants to exit
        if user_input.trim().to_lowercase() == "exit" {
            break;
        }
        
        // Add user message to conversation history
        conversation_history.push_str(&format!("\nHuman: {}", user_input.trim()));
        
        // Create system content with conversation history
        let system_content = if !conversation_history.is_empty() {
            Some(
                SystemContentBlock::Text(format!(
                    "This is the conversation history so far: {}",
                    conversation_history
                ))
            )
        } else {
            None
        };
        
        // Create user message
        let user_message = ContentBlock::Text(user_input.trim().to_string());
        
        // Build the request
        let mut request = client
            .converse()
            .model_id(model_id)
            .messages(
                aws_sdk_bedrockruntime::types::Message::builder()
                    .role(ConversationRole::User)
                    .content(user_message)
                    .build()?
            );
            
        // Add system content if it exists
        if let Some(system_content) = system_content {
            request = request.system(system_content);
        }
        
        // Send the request
        let response = request.send().await?;
        
        // Extract the assistant's response
        if let Some(output) = response.output() {
            if let Ok(message) = output.as_message() {
                if let Some(content) = message.content().first() {
                    if let Ok(text) = content.as_text() {
                        println!("Assistant: {}", text);
                        
                        // Add assistant response to conversation history
                        conversation_history.push_str(&format!("\nAssistant: {}", text));
                    }
                }
            }
        }
        
        println!("You: ");
    }
    
    Ok(())
} 