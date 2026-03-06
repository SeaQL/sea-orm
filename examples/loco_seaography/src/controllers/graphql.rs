use async_graphql::{
    dynamic::Schema,
    http::{GraphQLPlaygroundConfig, playground_source},
};
use async_graphql_axum::GraphQLRequest;
use loco_rs::prelude::*;
use seaography::async_graphql;

// GraphQL playground UI
async fn graphql_playground() -> Result<Response> {
    // The `GraphQLPlaygroundConfig` take one parameter
    // which is the URL of the GraphQL handler: `/api/graphql`
    let res = playground_source(GraphQLPlaygroundConfig::new("/api/graphql"));

    Ok(Response::new(res.into()))
}

async fn graphql_handler(
    // _auth: auth::JWT,
    State(ctx): State<AppContext>,
    gql_req: GraphQLRequest,
) -> Result<async_graphql_axum::GraphQLResponse, (axum::http::StatusCode, &'static str)> {
    let gql_req = gql_req.into_inner();

    let schema: Schema = ctx.shared_store.get().ok_or((
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "GraphQL not setup",
    ))?;
    let res = schema.execute(gql_req).await.into();

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
