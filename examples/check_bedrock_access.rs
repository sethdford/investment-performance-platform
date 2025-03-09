// Check Bedrock Access
//
// This example checks if we can access the AWS Bedrock service.
// It helps verify that your AWS credentials are properly configured
// and that you have access to the Bedrock service.

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
    
    println!("Checking access to AWS Bedrock...");
    
    // Try to access Bedrock by making a simple API call
    match bedrock_client.list_custom_models().send().await {
        Ok(_) => {
            println!("\n✅ Successfully connected to AWS Bedrock!");
            println!("Your AWS credentials are valid and you have access to the Bedrock service.");
        },
        Err(e) => {
            println!("\n❌ Failed to connect to AWS Bedrock: {}", e);
            println!("\nTROUBLESHOOTING TIPS:");
            println!("1. Verify your AWS credentials are valid by running:");
            println!("   aws sts get-caller-identity");
            println!("2. Check if Bedrock service is enabled in your AWS account");
            println!("3. Try a different AWS region (currently using us-east-1)");
            println!("4. Ensure your AWS account has access to Amazon Bedrock");
            println!("5. Verify your IAM user/role has the necessary permissions to access Bedrock");
        }
    }
    
    Ok(())
} 