use sea_orm::DbConn;

pub mod bakery_chain;
mod setup;
pub use bakery_chain::*;
mod crud;
mod schema;

#[async_std::test]
// cargo test --test bakery_chain_tests -- --nocapture
async fn main() {
    let db: DbConn = setup::setup().await;
    setup_schema(&db).await;
    create_entities(&db).await;
}

async fn setup_schema(db: &DbConn) {
    assert!(schema::create_bakery_table(db).await.is_ok());
    assert!(schema::create_baker_table(db).await.is_ok());
    assert!(schema::create_customer_table(db).await.is_ok());
    assert!(schema::create_order_table(db).await.is_ok());
    assert!(schema::create_lineitem_table(db).await.is_ok());
    assert!(schema::create_cake_table(db).await.is_ok());
    assert!(schema::create_cakes_bakers_table(db).await.is_ok());
}

async fn create_entities(db: &DbConn) {
    crud::test_create_bakery(db).await;
    crud::test_create_baker(db).await;
    crud::test_create_customer(db).await;
}
