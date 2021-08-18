pub mod bakery_chain;
pub mod runtime;
pub mod setup;

pub use bakery_chain::*;
use sea_orm::DatabaseConnection;
use std::env;

pub struct TestContext {
    base_url: String,
    db_name: String,
    pub db: DatabaseConnection,
}

impl TestContext {
    pub async fn new(test_name: &str) -> Self {
        let base_url =
            env::var("DATABASE_URL").expect("Enviroment variable 'DATABASE_URL' not set");
        let db: DatabaseConnection = setup::setup(&base_url, test_name).await;

        Self {
            base_url: base_url,
            db_name: test_name.to_string(),
            db,
        }
    }

    pub async fn delete(&self) {
        setup::tear_down(&self.base_url, &self.db_name).await;
    }
}
