use async_graphql::dynamic::*;
use axum_example_service::sea_orm;
use axum_example_service::sea_orm::DatabaseConnection;
use entity::post;
use seaography::{Builder, BuilderContext};

seaography::register_entity_modules!([post]);

lazy_static::lazy_static! {
    static ref CONTEXT: BuilderContext = BuilderContext::default();
}

pub fn schema(
    database: DatabaseConnection,
    depth: Option<usize>,
    complexity: Option<usize>,
) -> Result<Schema, SchemaError> {
    // Construct GraphQL schema
    let builder = Builder::new(&CONTEXT, database.clone());
    let builder = register_entity_modules(builder);
    let schema = builder.schema_builder();
    // Maximum depth of the constructed query
    let schema = if let Some(depth) = depth {
        schema.limit_depth(depth)
    } else {
        schema
    };
    // Maximum complexity of the constructed query
    let schema = if let Some(complexity) = complexity {
        schema.limit_complexity(complexity)
    } else {
        schema
    };
    // GraphQL schema with database connection
    schema.data(database).finish()
}
