use async_graphql::{EmptySubscription, Schema, Object, SimpleObject, Context, http::GraphiQLSource};
use async_graphql_warp::GraphQLResponse;
use investment_management::portfolio::model::{ModelPortfolioService};
use investment_management::portfolio::factor::FactorModelApi;
use std::sync::Arc;
use warp::{Filter, http::Response as HttpResponse};

// Define a simple model for testing
#[derive(SimpleObject)]
struct TestModel {
    id: String,
    name: String,
}

struct Query;

#[Object]
impl Query {
    async fn test_models(&self, _ctx: &Context<'_>) -> Vec<TestModel> {
        // Return some test data
        vec![
            TestModel {
                id: "model-1".to_string(),
                name: "Test Model 1".to_string(),
            },
            TestModel {
                id: "model-2".to_string(),
                name: "Test Model 2".to_string(),
            },
        ]
    }
}

struct Mutation;

#[Object]
impl Mutation {
    async fn create_test_model(&self, _name: String) -> TestModel {
        // Just return a mock model
        TestModel {
            id: format!("model-{}", uuid::Uuid::new_v4()),
            name: _name,
        }
    }
}

type TestSchema = Schema<Query, Mutation, EmptySubscription>;

#[tokio::main]
async fn main() {
    // Create the model portfolio service (not used in this simple example)
    let factor_model_api = FactorModelApi::new();
    let _service = Arc::new(ModelPortfolioService::new(factor_model_api));
    
    // Create the GraphQL schema
    let schema = TestSchema::build(Query, Mutation, EmptySubscription)
        .finish();
    
    println!("GraphQL server starting at http://localhost:8000/graphql");
    println!("GraphiQL playground available at http://localhost:8000/graphiql");
    
    // GraphQL endpoint
    let graphql_post = async_graphql_warp::graphql(schema.clone())
        .and_then(|(schema, request): (TestSchema, async_graphql::Request)| async move {
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