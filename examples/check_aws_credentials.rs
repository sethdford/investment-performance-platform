// AWS Credentials Check
//
// This example checks if AWS credentials are properly configured and displays information
// about the authenticated user/role.
//
// Run this script to verify your AWS credentials before attempting to use AWS services.

use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_sts::Client as StsClient;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Checking AWS credentials...");
    
    // Print environment variables (without showing actual values)
    println!("\nEnvironment variables:");
    println!("AWS_ACCESS_KEY_ID: {}", if env::var("AWS_ACCESS_KEY_ID").is_ok() { "Set" } else { "Not set" });
    println!("AWS_SECRET_ACCESS_KEY: {}", if env::var("AWS_SECRET_ACCESS_KEY").is_ok() { "Set" } else { "Not set" });
    println!("AWS_SESSION_TOKEN: {}", if env::var("AWS_SESSION_TOKEN").is_ok() { "Set" } else { "Not set" });
    println!("AWS_PROFILE: {}", env::var("AWS_PROFILE").unwrap_or_else(|_| "Not set".to_string()));
    println!("AWS_REGION: {}", env::var("AWS_REGION").unwrap_or_else(|_| "Not set".to_string()));
    
    // Load AWS configuration
    let aws_config = aws_config::defaults(BehaviorVersion::latest())
        .load()
        .await;
    
    println!("\nAWS Configuration:");
    println!("Region: {}", aws_config.region().map_or("Not set", |r| r.as_ref()));
    
    // Create STS client
    let sts_client = StsClient::new(&aws_config);
    
    // Call get-caller-identity
    println!("\nAttempting to call STS get-caller-identity...");
    match sts_client.get_caller_identity().send().await {
        Ok(identity) => {
            println!("\n✅ AWS credentials are valid!");
            println!("Account ID: {}", identity.account().unwrap_or("Unknown"));
            println!("User ID: {}", identity.user_id().unwrap_or("Unknown"));
            println!("ARN: {}", identity.arn().unwrap_or("Unknown"));
            
            // Check if the user has Bedrock permissions
            println!("\nNote: This script only verifies that your AWS credentials are valid.");
            println!("It does not check if you have permissions to use Amazon Bedrock.");
            println!("To use Bedrock, your IAM user/role needs the 'bedrock:InvokeModel' permission.");
        },
        Err(e) => {
            println!("\n❌ Failed to validate AWS credentials!");
            println!("Error: {}", e);
            println!("\nTROUBLESHOOTING TIPS:");
            println!("1. Check if your AWS credentials are correctly set in:");
            println!("   - Environment variables");
            println!("   - ~/.aws/credentials file");
            println!("   - ~/.aws/config file");
            println!("2. Verify that the credentials have not expired");
            println!("3. If using a named profile, make sure it's correctly specified");
            println!("4. If using SSO, make sure you've run 'aws sso login'");
        }
    }
    
    Ok(())
} 