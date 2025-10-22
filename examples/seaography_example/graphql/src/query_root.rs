use crate::entities::*;
use async_graphql::dynamic::*;
use sea_orm::DatabaseConnection;
use seaography::{async_graphql, lazy_static::lazy_static, Builder, BuilderContext};

lazy_static! {
    static ref CONTEXT: BuilderContext = BuilderContext::default();
}

pub fn schema(
    database: DatabaseConnection,
    depth: Option<usize>,
    complexity: Option<usize>,
) -> Result<Schema, SchemaError> {
    schema_builder(&CONTEXT, database, depth, complexity).finish()
}

pub fn schema_builder(
    context: &'static BuilderContext,
    database: DatabaseConnection,
    depth: Option<usize>,
    complexity: Option<usize>,
) -> SchemaBuilder {
    let mut builder = Builder::new(context, database.clone());
    builder = register_entity_modules(builder);
    builder
        .set_depth_limit(depth)
        .set_complexity_limit(complexity)
        .schema_builder()
        .data(database)
}
