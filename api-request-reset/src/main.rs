use lambda_runtime::{service_fn, Error, LambdaEvent};
use aws_lambda_events::event::cloudwatch_events::CloudWatchEvent;
use aws_sdk_dynamodb::Client as DynamoDbClient;
use tracing::{info, error};
use serde_json::Value;
use performance_calculator::calculations::tenant::{get_tenant_manager, get_tenant_metrics_manager, TenantConfig};

/// Main function to run the Lambda
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();
    
    info!("API Request Reset Lambda starting up");
    
    // Start Lambda runtime
    lambda_runtime::run(service_fn(handle_event)).await?;
    
    Ok(())
}

/// Handle CloudWatch Events
async fn handle_event(event: LambdaEvent<CloudWatchEvent>) -> Result<(), Error> {
    let (event, _context) = event.into_parts();
    
    info!("Received CloudWatch event: {}", event.id);
    
    // Reset API request counters for all tenants
    match reset_all_api_request_counters().await {
        Ok(count) => {
            info!("Successfully reset API request counters for {} tenants", count);
            Ok(())
        },
        Err(e) => {
            error!("Failed to reset API request counters: {}", e);
            Err(Box::new(e))
        }
    }
}

/// Reset API request counters for all tenants
async fn reset_all_api_request_counters() -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    // Get tenant manager
    let tenant_manager = get_tenant_manager(&TenantConfig::default()).await?;
    
    // Get metrics manager
    let metrics_manager = get_tenant_metrics_manager().await?;
    
    // Get all tenants
    let tenants = tenant_manager.list_tenants(None, None).await?;
    
    // Reset API request counters for each tenant
    let mut reset_count = 0;
    for tenant in tenants {
        if tenant.is_active() {
            match metrics_manager.reset_api_requests(&tenant.id).await {
                Ok(_) => {
                    info!("Reset API request counter for tenant: {}", tenant.name);
                    reset_count += 1;
                },
                Err(e) => {
                    error!("Failed to reset API request counter for tenant {}: {}", tenant.name, e);
                }
            }
        }
    }
    
    Ok(reset_count)
} 