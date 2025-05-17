use crate::entities::*;
use async_graphql::dynamic::*;
use sea_orm::DatabaseConnection;
use seaography::{Builder, BuilderContext, async_graphql, lazy_static};

lazy_static::lazy_static! { static ref CONTEXT : BuilderContext = BuilderContext :: default () ; }

pub fn schema(
    database: DatabaseConnection,
    depth: Option<usize>,
    complexity: Option<usize>,
) -> Result<Schema, SchemaError> {
    let mut builder = Builder::new(&CONTEXT, database.clone());
    seaography::register_entities!(builder, [baker, bakery, cake, cake_baker,]);
    builder
        .set_depth_limit(depth)
        .set_complexity_limit(complexity)
        .schema_builder()
        .data(database)
        .finish()
}
