use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use axum::{body::Body, extract::Request};
use loco_rs::prelude::*;
use tower_service::Service;

use crate::graphql::query_root;

// GraphQL playground UI
async fn graphql_playground() -> Result<Response> {
    // The `GraphQLPlaygroundConfig` take one parameter
    // which is the URL of the GraphQL handler: `/api/graphql`
    let res = playground_source(GraphQLPlaygroundConfig::new("/api/graphql"));

    Ok(Response::new(res.into()))
}

async fn graphql_handler(
    _auth: auth::JWT,
    State(ctx): State<AppContext>,
    req: Request<Body>,
) -> Result<Response> {
    const DEPTH: usize = 10;
    const COMPLEXITY: usize = 100;
    // Construct the the GraphQL query root
    let schema = query_root::schema(ctx.db.clone(), DEPTH, COMPLEXITY).unwrap();
    // GraphQL handler
    let mut graphql_handler = async_graphql_axum::GraphQL::new(schema);
    // Execute GraphQL request and fetch the results
    let res = graphql_handler.call(req).await.unwrap();

    Ok(res)
}

pub fn routes() -> Routes {
    // Define route
    Routes::new()
        // We put all GraphQL route behind `graphql` prefix
        .prefix("graphql")
        // GraphQL playground page is a GET request
        .add("/", get(graphql_playground))
        // GraphQL handler is a POST request
        .add("/", post(graphql_handler))
}
