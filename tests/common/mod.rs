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
    pub db: DatabaseConnection,
}

impl TestContext {
    pub async fn new() -> Self {
        let db: DatabaseConnection = setup::setup().await;

        // let stmt: Statement = Statement::from("SET autocommit=0;\nSTART TRANSACTION;".to_string());
        // let _ = db.execute(stmt).await;

        Self { db }
    }

    pub async fn delete(&self) {
        // let stmt = sea_query::Table::drop()
        //     .table(baker::Entity)
        //     .table(baker::Entity)
        //     .to_owned();

        // let builder = self.db.get_schema_builder_backend();
        // let result = self.db.execute(builder.build(&stmt)).await;

        let _ = self
            .db
            .execute(Statement::from("SET FOREIGN_KEY_CHECKS = 0;".to_string()))
            .await;
        let _ = self
            .db
            .execute(Statement::from("TRUNCATE TABLE `baker`;".to_string()))
            .await;
        let result = self
            .db
            .execute(Statement::from("TRUNCATE TABLE `bakery`;".to_string()))
            .await;
        let result = self
            .db
            .execute(Statement::from("TRUNCATE TABLE `cake`;".to_string()))
            .await;
        let result = self
            .db
            .execute(Statement::from(
                "TRUNCATE TABLE `cakes_bakers`;".to_string(),
            ))
            .await;
        let result = self
            .db
            .execute(Statement::from("TRUNCATE TABLE `customer`;".to_string()))
            .await;
        let result = self
            .db
            .execute(Statement::from("TRUNCATE TABLE `order`;".to_string()))
            .await;
        let result = self
            .db
            .execute(Statement::from("TRUNCATE TABLE `lineitem`;".to_string()))
            .await;
        let _ = self
            .db
            .execute(Statement::from("SET FOREIGN_KEY_CHECKS = 1;".to_string()))
            .await;
        println!("result: {:#?}", result);
    }
}
