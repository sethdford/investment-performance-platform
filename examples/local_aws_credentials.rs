// Local AWS Credentials Example
//
// This example demonstrates how to use local AWS credentials to make calls to AWS services,
// particularly Amazon Bedrock for NLP tasks.
//
// Before running this example, make sure you have set up your AWS credentials locally using one of these methods:
//
// 1. Environment Variables:
//    - Set AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY in your environment
//    - Optionally set AWS_SESSION_TOKEN if using temporary credentials
//    - Example:
//      export AWS_ACCESS_KEY_ID=your_access_key
//      export AWS_SECRET_ACCESS_KEY=your_secret_key
//      export AWS_SESSION_TOKEN=your_session_token  # if using temporary credentials
//
// 2. AWS Credentials File:
//    - Create or edit ~/.aws/credentials
//    - Add your credentials in the following format:
//      [default]
//      aws_access_key_id = your_access_key
//      aws_secret_access_key = your_secret_key
//      aws_session_token = your_session_token  # if using temporary credentials
//
//    - You can also create named profiles:
//      [profile_name]
//      aws_access_key_id = your_access_key
//      aws_secret_access_key = your_secret_key
//
// 3. AWS Config File:
//    - Create or edit ~/.aws/config
//    - Add your region and output format:
//      [default]
//      region = us-east-1
//      output = json
//
//    - For named profiles:
//      [profile profile_name]
//      region = us-east-1
//      output = json
//
// To use a specific profile, you can:
// - Set the AWS_PROFILE environment variable: export AWS_PROFILE=profile_name
// - Or modify the code to use a specific profile:
//   aws_config::from_env().profile_name("profile_name").load().await
//
// TROUBLESHOOTING:
// ---------------
// If you encounter errors like "Failed to invoke Bedrock model: service error", check the following:
//
// 1. Verify your AWS credentials are valid:
//    - Run `aws sts get-caller-identity` to check if your credentials are working
//    - If this fails, your credentials are not properly configured
//
// 2. Check if your AWS account has access to Amazon Bedrock:
//    - Bedrock is a relatively new service and might not be available in all regions
//    - Make sure you've enabled Bedrock in your AWS account
//    - Verify you have the necessary IAM permissions (bedrock:InvokeModel)
//
// 3. Verify the model ID is correct:
//    - Different models have different IDs and availability
//    - Some models require explicit access approval in the AWS console
//
// 4. Check the region:
//    - Bedrock might only be available in specific regions
//    - Try using 'us-east-1' or 'us-west-2' as these commonly support Bedrock
//
// Note: Make sure your AWS credentials have the necessary permissions to access Amazon Bedrock services.

use aws_config::BehaviorVersion;
use aws_sdk_bedrockruntime::Client as BedrockRuntimeClient;
use anyhow::{Result, anyhow};
use investment_management::financial_advisor::nlp::{
    bedrock::{BedrockModelProvider, BedrockNlpClient, BedrockNlpConfig},
    FinancialQueryIntent,
};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Initializing AWS clients with local credentials...");
    
    // Initialize AWS clients with local credentials
    let bedrock_client = initialize_aws_clients().await?;
    
    // Create Bedrock NLP config
    let bedrock_config = BedrockNlpConfig {
        provider: BedrockModelProvider::Claude,
        version: "claude-3-sonnet-20240229-v1:0".to_string(),
        temperature: 0.7,
        max_tokens: 500,
        top_p: 0.9,
        top_k: 250,
        stop_sequences: vec![],
    };
    
    // Create Bedrock NLP client
    let nlp_client = BedrockNlpClient::new(bedrock_client, bedrock_config);
    
    println!("AWS clients initialized successfully!");
    println!("Testing connection to Bedrock...");
    
    // Test the connection with a simple query
    let test_query = "Tell me about retirement planning";
    let intent = FinancialQueryIntent::RetirementPlanning;
    let entities = vec![];
    
    // Generate a response using Bedrock
    match nlp_client.generate_response(
        test_query,
        &intent,
        &entities,
        None,
        ""
    ).await {
        Ok(response) => {
            println!("Successfully connected to AWS Bedrock!");
            println!("Response from Bedrock: {}", response);
            
            // Interactive mode
            println!("\nEntering interactive mode. Type 'exit' to quit.");
            let mut input = String::new();
            
            loop {
                print!("\nEnter a financial question: ");
                io::stdout().flush().unwrap();
                
                input.clear();
                io::stdin().read_line(&mut input).unwrap();
                
                let query = input.trim();
                if query.to_lowercase() == "exit" {
                    break;
                }
                
                // For simplicity, we'll use a fixed intent for all queries
                match nlp_client.generate_response(
                    query,
                    &FinancialQueryIntent::FinancialEducation,
                    &entities,
                    None,
                    ""
                ).await {
                    Ok(response) => println!("Response: {}", response),
                    Err(e) => println!("Error getting response: {}", e),
                }
            }
        },
        Err(e) => {
            println!("Failed to connect to AWS Bedrock: {}", e);
            println!("\nTROUBLESHOOTING TIPS:");
            println!("1. Verify your AWS credentials are valid by running:");
            println!("   aws sts get-caller-identity");
            println!("2. Check if your AWS account has access to Amazon Bedrock");
            println!("3. Verify the model ID is correct (claude-3-sonnet-20240229-v1:0)");
            println!("4. Try a different AWS region (currently using us-east-1)");
            println!("5. Make sure you have the necessary IAM permissions (bedrock:InvokeModel)");
        }
    }
    
    Ok(())
}

/// Initialize AWS clients with local credentials
async fn initialize_aws_clients() -> Result<BedrockRuntimeClient> {
    // Load AWS configuration from the default credential chain
    // This will look for credentials in the following order:
    // 1. Environment variables (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY)
    // 2. Shared credentials file (~/.aws/credentials)
    // 3. IAM role for Amazon EC2 or ECS task role
    // 4. SSO token from AWS SSO
    
    // Method 1: Use default profile
    let aws_config = aws_config::defaults(BehaviorVersion::latest())
        // You can specify a region or use the one from your AWS config
        .region("us-east-1")
        // Load credentials from the default credential chain
        .load()
        .await;
    
    // Method 2: Use a specific profile
    // Uncomment the following code to use a specific profile
    /*
    let aws_config = aws_config::defaults(BehaviorVersion::latest())
        .region("us-east-1")
        .profile_name("your-profile-name") // Specify your profile name here
        .load()
        .await;
    */
    
    // Method 3: Use environment variables explicitly
    // This is useful when you want to ensure environment variables are used
    // regardless of other credential sources
    /*
    use aws_config::environment::EnvironmentVariableCredentialsProvider;
    
    let aws_config = aws_config::defaults(BehaviorVersion::latest())
        .region("us-east-1")
        .credentials_provider(EnvironmentVariableCredentialsProvider::new())
        .load()
        .await;
    */
    
    // Create a new Bedrock client with the loaded configuration
    let bedrock_client = BedrockRuntimeClient::new(&aws_config);
    
    // Check if we have valid credentials
    if aws_config.credentials_provider().is_none() {
        return Err(anyhow!("No AWS credentials found. Please configure your AWS credentials."));
    }
    
    println!("AWS client initialized with region: {}", aws_config.region().unwrap());
    
    Ok(bedrock_client)
} 