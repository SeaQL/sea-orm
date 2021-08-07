use sea_orm::DatabaseConnection;

pub mod common;
pub use common::{bakery_chain::*, setup::*, TestContext};

mod crud;

// Run the test locally:
// DATABASE_URL="mysql://root:@localhost" cargo test --features sqlx-mysql,runtime-async-std --test bakery_chain_tests
#[cfg_attr(feature = "runtime-async-std", async_std::test)]
#[cfg_attr(feature = "runtime-actix", actix_rt::test)]
#[cfg_attr(feature = "runtime-tokio", tokio::test)]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() {
    let ctx = TestContext::new("bakery_chain_schema_crud_tests").await;
    create_entities(&ctx.db).await;
    ctx.delete().await;
}

async fn create_entities(db: &DatabaseConnection) {
    crud::test_create_bakery(db).await;
    crud::create_baker::test_create_baker(db).await;
    crud::test_create_customer(db).await;
    crud::create_cake::test_create_cake(db).await;
    crud::create_lineitem::test_create_lineitem(db).await;
    crud::create_order::test_create_order(db).await;

    crud::updates::test_update_cake(db).await;
    crud::updates::test_update_bakery(db).await;
    crud::updates::test_update_deleted_customer(db).await;

    crud::deletes::test_delete_cake(db).await;
    crud::deletes::test_delete_bakery(db).await;
}
