#![allow(unused_imports, dead_code)]

pub mod common;
mod crud;

pub use common::{TestContext, bakery_chain::*, setup::*};
pub use crud::*;
use sea_orm::DatabaseConnection;

// Run the test locally:
// DATABASE_URL="sqlite::memory:" cargo test --features rusqlite --test crud_tests
// DATABASE_URL="sqlite::memory:" cargo test --features sqlx-sqlite,runtime-tokio --test crud_tests
// DATABASE_URL="mysql://root:root@localhost" cargo test --features sqlx-mysql,runtime-tokio-native-tls --test crud_tests
// DATABASE_URL="postgres://sea:sea@localhost" cargo test --features sqlx-postgres,runtime-tokio-native-tls --test crud_tests
#[sea_orm_macros::test]
fn main() {
    let ctx = TestContext::new("bakery_chain_schema_crud_tests");
    create_tables(&ctx.db).unwrap();
    create_entities(&ctx.db);
    ctx.delete();
}

pub fn create_entities(db: &DatabaseConnection) {
    test_create_bakery(db);
    test_create_baker(db);
    test_create_customer(db);
    test_create_cake(db);
    test_create_lineitem(db);
    test_create_order(db);

    test_update_cake(db);
    test_update_bakery(db);
    test_update_deleted_customer(db);

    test_delete_cake(db);
    test_cake_error_sqlx(db);
    test_delete_bakery(db);
}
