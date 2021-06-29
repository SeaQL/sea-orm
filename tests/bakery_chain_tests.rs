use sea_orm::DbConn;

pub mod bakery_chain;
mod setup;
pub use bakery_chain::*;
mod table_creation;

#[async_std::test]
// cargo test --test bakery_chain_tests -- --nocapture
async fn main() {
    let db: DbConn = setup::setup().await;
    setup_schema(&db).await;
}

async fn setup_schema(db: &DbConn) {
    assert!(table_creation::create_bakery_table(db).await.is_ok());
    assert!(table_creation::create_baker_table(db).await.is_ok());
    assert!(table_creation::create_customer_table(db).await.is_ok());
    assert!(table_creation::create_order_table(db).await.is_ok());
    assert!(table_creation::create_lineitem_table(db).await.is_ok());
    assert!(table_creation::create_cakes_bakers_table(db).await.is_ok());
}
