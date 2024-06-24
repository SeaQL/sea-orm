use async_graphql::dynamic::*;
use sea_orm::DatabaseConnection;
use seaography::{Builder, BuilderContext};

use crate::models::_entities::*;

lazy_static::lazy_static! { static ref CONTEXT: BuilderContext = BuilderContext::default(); }

pub fn schema(
    database: DatabaseConnection,
    depth: Option<usize>,
    complexity: Option<usize>,
) -> Result<Schema, SchemaError> {
    // Builder of Seaography query root
    let mut builder = Builder::new(&CONTEXT, database.clone());
    // Register SeaORM entities
    seaography::register_entities!(
        builder,
        // List all models we want to include in the GraphQL endpoint here
        [files, notes, users]
    );
    // Configure async GraphQL limits
    let schema = builder.schema_builder();
    let schema = if let Some(depth) = depth {
        schema.limit_depth(depth)
    } else {
        schema
    };
    let schema = if let Some(complexity) = complexity {
        schema.limit_complexity(complexity)
    } else {
        schema
    };
    // Finish up with including SeaORM database connection
    schema.data(database).finish()
}
