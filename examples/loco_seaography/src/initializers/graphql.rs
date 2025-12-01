use crate::graphql::query_root;
use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::prelude::*;

// Maximum depth of the constructed query
const DEPTH: Option<usize> = None;
// Maximum complexity of the constructed query
const COMPLEXITY: Option<usize> = None;

pub struct GraphQLInitializer;

#[async_trait]
impl Initializer for GraphQLInitializer {
    fn name(&self) -> String {
        "graphql".to_string()
    }

    async fn after_routes(&self, router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
        let schema = query_root::schema(ctx.db.clone(), DEPTH, COMPLEXITY)
            .expect("Failed to build GraphQL schema");
        ctx.shared_store.insert(schema);

        Ok(router)
    }
}
