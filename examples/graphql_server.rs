use async_graphql::{EmptySubscription, Schema, Object, SimpleObject, InputObject, ID, Context, http::GraphiQLSource};
use async_graphql_warp::GraphQLResponse;
use investment_management::portfolio::model::{ModelPortfolio, ModelPortfolioService, ModelType};
use investment_management::portfolio::factor::FactorModelApi;
use std::collections::HashMap;
use std::sync::Arc;
use warp::{Filter, http::Response as HttpResponse};

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
    async fn models(&self, ctx: &Context<'_>) -> Vec<Model> {
        let service = ctx.data::<Arc<ModelPortfolioService>>().unwrap();
        
        // For simplicity, we'll just return a mock model
        let model = service.get_model_portfolio("mock-model").unwrap();
        
        vec![convert_model(model)]
    }

    async fn model(&self, ctx: &Context<'_>, id: ID) -> Option<Model> {
        let service = ctx.data::<Arc<ModelPortfolioService>>().unwrap();
        
        service.get_model_portfolio(&id)
            .map(convert_model)
    }
}

struct Mutation;

#[Object]
impl Mutation {
    async fn create_model(&self, ctx: &Context<'_>, input: CreateModelInput) -> Result<Model, String> {
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
    
    println!("GraphQL server starting at http://localhost:8000/graphql");
    println!("GraphiQL playground available at http://localhost:8000/graphiql");
    
    // GraphQL endpoint
    let graphql_post = async_graphql_warp::graphql(schema.clone())
        .and_then(|(schema, request): (InvestmentSchema, async_graphql::Request)| async move {
            Ok::<_, std::convert::Infallible>(GraphQLResponse::from(schema.execute(request).await))
        });
    
    // GraphiQL playground
    let graphiql = warp::path("graphiql")
        .and(warp::get())
        .map(|| {
            HttpResponse::builder()
                .header("content-type", "text/html")
                .body(GraphiQLSource::build().endpoint("/graphql").finish())
        });
    
    // Combine routes
    let routes = graphql_post
        .or(graphiql)
        .with(warp::cors().allow_any_origin());
    
    // Start the server
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
} 