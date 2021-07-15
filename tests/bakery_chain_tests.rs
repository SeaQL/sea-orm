use sea_orm::DatabaseConnection;

pub mod common;
pub use common::{bakery_chain::*, setup::*, TestContext};

mod crud;

#[async_std::test]
// cargo test --test bakery_chain_tests -- --nocapture
async fn main() {
    let base_url = "mysql://root:@localhost";
    let db_name = "bakery_chain_schema_crud_tests";

    let db: DatabaseConnection = common::setup::setup(base_url, db_name).await;
    create_entities(&db).await;
    common::setup::tear_down(base_url, db_name).await;
}

async fn create_entities(db: &DatabaseConnection) {
    crud::test_create_bakery(db).await;
    crud::test_create_baker(db).await;
    crud::test_create_customer(db).await;
    crud::create_cake::test_create_cake(db).await;
    crud::create_lineitem::test_create_lineitem(db).await;
    crud::create_order::test_create_order(db).await;

    crud::updates::test_update_cake(db).await;
    crud::updates::test_update_bakery(db).await;

    crud::deletes::test_delete_cake(db).await;
    crud::deletes::test_delete_bakery(db).await;
}
