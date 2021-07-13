pub mod setup;
use sea_orm::DatabaseConnection;
pub mod bakery_chain;
pub use bakery_chain::*;

pub struct TestContext {
    base_url: String,
    db_name: String,
    pub db: DatabaseConnection,
}

impl TestContext {
    pub async fn new(base_url: &str, db_name: &str) -> Self {
        let db: DatabaseConnection = setup::setup(base_url, db_name).await;

        Self {
            base_url: base_url.to_string(),
            db_name: db_name.to_string(),
            db,
        }
    }

    pub async fn delete(&self) {
        setup::tear_down(&self.base_url, &self.db_name).await;
    }
}
