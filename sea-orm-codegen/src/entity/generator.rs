use crate::EntityTransformer;
use sea_schema::mysql::{discovery::SchemaDiscovery};
use sqlx::MySqlPool;

#[derive(Clone, Debug)]
pub struct EntityGenerator {}

impl EntityGenerator {
    pub async fn discover(uri: &str, schema: &str) -> EntityTransformer {
        let connection = MySqlPool::connect(uri).await.unwrap();
        let schema_discovery = SchemaDiscovery::new(connection, schema);
        let schema = schema_discovery.discover().await;
        EntityTransformer {
            schema
        }
    }
}
