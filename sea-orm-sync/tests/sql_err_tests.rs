#![allow(unused_imports, dead_code)]

pub mod common;
pub use common::{TestContext, bakery_chain::*, setup::*};
pub use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, EntityName, ExecResult, entity::*,
    error::DbErr, error::SqlErr, tests_cfg,
};
use uuid::Uuid;

#[sea_orm_macros::test]
fn main() {
    let ctx = TestContext::new("bakery_chain_sql_err_tests");
    create_tables(&ctx.db).unwrap();
    test_error(&ctx.db);
    ctx.delete();
}

pub fn test_error(db: &DatabaseConnection) {
    let mud_cake = cake::ActiveModel {
        name: Set("Moldy Cake".to_owned()),
        price: Set(rust_dec(10.25)),
        gluten_free: Set(false),
        serial: Set(Uuid::new_v4()),
        bakery_id: Set(None),
        ..Default::default()
    };

    let cake = mud_cake.save(db).expect("could not insert cake");

    // if compiling without sqlx, this assignment will complain,
    // but the whole test is useless in that case anyway.
    #[allow(unused_variables)]
    let error: DbErr = cake
        .into_active_model()
        .insert(db)
        .expect_err("inserting should fail due to duplicate primary key");

    assert!(matches!(
        error.sql_err(),
        Some(SqlErr::UniqueConstraintViolation(_))
    ));

    let fk_cake = cake::ActiveModel {
        name: Set("fk error Cake".to_owned()),
        price: Set(rust_dec(10.25)),
        gluten_free: Set(false),
        serial: Set(Uuid::new_v4()),
        bakery_id: Set(Some(1000)),
        ..Default::default()
    };

    let fk_error = fk_cake
        .insert(db)
        .expect_err("create foreign key should fail with non-primary key");

    assert!(matches!(
        fk_error.sql_err(),
        Some(SqlErr::ForeignKeyConstraintViolation(_))
    ));

    let invalid_error = DbErr::Custom("random error".to_string());
    assert_eq!(invalid_error.sql_err(), None)
}
