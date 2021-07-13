use sea_orm::{Database, DatabaseConnection};
pub mod schema;
pub use schema::*;

pub async fn setup() -> DatabaseConnection {
    // let db = Database::connect("sqlite::memory:").await.unwrap();
    let db = Database::connect("mysql://sea:sea@localhost/seaorm_test")
        .await
        .unwrap();

    assert!(schema::create_bakery_table(&db).await.is_ok());
    assert!(schema::create_baker_table(&db).await.is_ok());
    assert!(schema::create_customer_table(&db).await.is_ok());
    assert!(schema::create_order_table(&db).await.is_ok());
    assert!(schema::create_cake_table(&db).await.is_ok());
    assert!(schema::create_cakes_bakers_table(&db).await.is_ok());
    assert!(schema::create_lineitem_table(&db).await.is_ok());
    db
}
