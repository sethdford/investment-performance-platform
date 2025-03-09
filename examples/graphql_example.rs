use async_graphql::{EmptySubscription, Schema, Object, SimpleObject, InputObject, ID, Context};
use investment_management::portfolio::model::{ModelPortfolio, ModelPortfolioService, ModelType};
use investment_management::portfolio::factor::FactorModelApi;
use std::collections::HashMap;
use std::sync::Arc;

// Define our GraphQL schema types
#[derive(SimpleObject)]
struct Model {
    id: ID,
    name: String,
    model_type: String,
    securities: Vec<SecurityWeight>,
}

#[derive(SimpleObject)]
struct SecurityWeight {
    security_id: String,
    weight: f64,
}

#[derive(InputObject)]
struct CreateModelInput {
    name: String,
    securities: Vec<SecurityWeightInput>,
}

#[derive(InputObject)]
struct SecurityWeightInput {
    security_id: String,
    weight: f64,
}

struct Query;

#[Object]
impl Query {
    async fn models<'a>(&self, ctx: &'a Context<'_>) -> Vec<Model> {
        let service = ctx.data::<Arc<ModelPortfolioService>>().unwrap();
        
        // For simplicity, we'll just return a mock model
        let model = service.get_model_portfolio("mock-model").unwrap();
        
        vec![convert_model(model)]
    }

    async fn model<'a>(&self, ctx: &'a Context<'_>, id: ID) -> Option<Model> {
        let service = ctx.data::<Arc<ModelPortfolioService>>().unwrap();
        
        service.get_model_portfolio(&id)
            .map(convert_model)
    }
}

struct Mutation;

#[Object]
impl Mutation {
    async fn create_model<'a>(&self, ctx: &'a Context<'_>, input: CreateModelInput) -> Result<Model, String> {
        let service = ctx.data::<Arc<ModelPortfolioService>>().unwrap();
        
        let mut securities = HashMap::new();
        for sec in input.securities {
            securities.insert(sec.security_id, sec.weight);
        }
        
        let model = ModelPortfolio {
            id: format!("model-{}", uuid::Uuid::new_v4()),
            name: input.name,
            description: None,
            model_type: ModelType::Direct,
            asset_allocation: HashMap::new(),
            sector_allocation: HashMap::new(),
            securities,
            child_models: HashMap::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        
        service.create_model_portfolio(model)
            .map(convert_model)
            .map_err(|err| err.to_string())
    }
}

fn convert_model(model: ModelPortfolio) -> Model {
    let securities = model.securities.iter()
        .map(|(id, weight)| SecurityWeight {
            security_id: id.clone(),
            weight: *weight,
        })
        .collect();
        
    Model {
        id: model.id.into(),
        name: model.name,
        model_type: format!("{:?}", model.model_type),
        securities,
    }
}

type InvestmentSchema = Schema<Query, Mutation, EmptySubscription>;

#[tokio::main]
async fn main() {
    // Create the model portfolio service
    let factor_model_api = FactorModelApi::new();
    let service = Arc::new(ModelPortfolioService::new(factor_model_api));
    
    // Create the GraphQL schema
    let schema = InvestmentSchema::build(Query, Mutation, EmptySubscription)
        .data(service.clone())
        .finish();
    
    // Define a GraphQL query
    let query = r#"
        query {
            models {
                id
                name
                modelType
                securities {
                    securityId
                    weight
                }
            }
        }
    "#;
    
    // Execute the query
    let res = schema.execute(query).await;
    
    // Print the result
    println!("GraphQL Query Result:");
    println!("{}", serde_json::to_string_pretty(&res).unwrap());
    
    // Define a GraphQL mutation
    let mutation = r#"
        mutation {
            createModel(input: {
                name: "Technology Growth",
                securities: [
                    { securityId: "AAPL", weight: 0.25 },
                    { securityId: "MSFT", weight: 0.25 },
                    { securityId: "AMZN", weight: 0.25 },
                    { securityId: "GOOGL", weight: 0.25 }
                ]
            }) {
                id
                name
                modelType
                securities {
                    securityId
                    weight
                }
            }
        }
    "#;
    
    // Execute the mutation
    let res = schema.execute(mutation).await;
    
    // Print the result
    println!("\nGraphQL Mutation Result:");
    println!("{}", serde_json::to_string_pretty(&res).unwrap());
} 