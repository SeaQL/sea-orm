pub mod setup;
use sea_orm::{DatabaseConnection, Statement};
pub mod bakery_chain;
pub use bakery_chain::*;

#[macro_export]
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

pub struct TestContext {
    base_url: String,
    db_name: String,
    pub db: DatabaseConnection,
}

impl TestContext {
    pub async fn new(base_url: &str, db_name: &str) -> Self {
        let db: DatabaseConnection = setup::setup().await;

        // let stmt: Statement = Statement::from("BEGIN".to_string());
        // let _ = db.execute(stmt).await;

        Self {
            base_url: base_url.to_string(),
            db_name: db_name.to_string(),
            db,
        }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // println!("dropping context");
        // let stmt: Statement = Statement::from("ROLLBACK".to_string());
        // let _ = self.db.execute(stmt);
    }
}
