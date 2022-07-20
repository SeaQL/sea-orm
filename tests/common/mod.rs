pub mod bakery_chain;
pub mod features;
pub mod runtime;
pub mod setup;

use sea_orm::{ConnectOptions, DatabaseConnection};

pub struct TestContext {
    base_url: String,
    db_name: String,
    pub db: DatabaseConnection,
}

impl TestContext {
    pub async fn new(test_name: &str) -> Self {
        let fn_conn_opt = |url: &str| ConnectOptions::new(url.to_string());
        Self::new_with_opt(test_name, fn_conn_opt).await
    }

    pub async fn new_with_opt<F>(test_name: &str, fn_conn_opt: F) -> Self
    where
        F: Fn(&str) -> ConnectOptions,
    {
        let base_url =
            std::env::var("DATABASE_URL").expect("Enviroment variable 'DATABASE_URL' not set");
        let db: DatabaseConnection = setup::setup(&base_url, test_name, fn_conn_opt).await;

        Self {
            base_url,
            db_name: test_name.to_string(),
            db,
        }
    }

    pub async fn delete(&self) {
        setup::tear_down(&self.base_url, &self.db_name).await;
    }
}
