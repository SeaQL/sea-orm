use crate::{EntityTransformer, Error};
use sea_schema::mysql::discovery::SchemaDiscovery;
use sqlx::MySqlPool;

#[derive(Clone, Debug)]
pub struct EntityGenerator {}

impl EntityGenerator {
    pub async fn discover(uri: &str, schema: &str) -> Result<EntityTransformer, Error> {
        let connection = MySqlPool::connect(uri).await?;
        let schema_discovery = SchemaDiscovery::new(connection, schema);
        let schema = schema_discovery.discover().await;
        Ok(EntityTransformer { schema })
    }
}
