use sea_orm::{Database, DatabaseConnection, Statement};
pub mod schema;
pub use schema::*;

pub async fn setup(base_url: &str, db_name: &str) -> DatabaseConnection {
    let url = format!("{}/mysql", base_url);
    let db = Database::connect(&url).await.unwrap();
    let _create_db_result = db
        .execute(Statement::from(format!(
            "CREATE DATABASE IF NOT EXISTS `{}`;",
            db_name
        )))
        .await;

    let url = format!("{}/{}", base_url, db_name);
    let db = Database::connect(&url).await.unwrap();

    assert!(schema::create_bakery_table(&db).await.is_ok());
    assert!(schema::create_baker_table(&db).await.is_ok());
    assert!(schema::create_customer_table(&db).await.is_ok());
    assert!(schema::create_order_table(&db).await.is_ok());
    assert!(schema::create_cake_table(&db).await.is_ok());
    assert!(schema::create_cakes_bakers_table(&db).await.is_ok());
    assert!(schema::create_lineitem_table(&db).await.is_ok());
    db
}

pub async fn tear_down(base_url: &str, db_name: &str) {
    let url = format!("{}/mysql", base_url);
    let db = Database::connect(&url).await.unwrap();
    let _drop_db_result = db
        .execute(Statement::from(format!(
            "DROP DATABASE IF EXISTS `{}`;",
            db_name
        )))
        .await;
}
