pub mod common;
pub use common::{bakery_chain::*, setup::*, TestContext};
use pretty_assertions::assert_eq;
use rust_decimal_macros::dec;
use sea_orm::ConnectionTrait;
pub use sea_orm::{
    entity::*, error::*, tests_cfg, DatabaseConnection, DbBackend, EntityName, ExecResult,
};
use uuid::Uuid;

#[sea_orm_macros::test]
#[cfg(any(
    feature = "sqlx-mysql",
    feature = "sqlx-sqlite",
    feature = "sqlx-postgres"
))]
async fn main() {
    let ctx = TestContext::new("bakery_chain_sql_err_tests").await;
    create_tables(&ctx.db).await.unwrap();
    test_error(&ctx.db).await;
    ctx.delete().await;
}

pub async fn test_error(db: &DatabaseConnection) {
    let mud_cake = cake::ActiveModel {
        name: Set("Moldy Cake".to_owned()),
        price: Set(dec!(10.25)),
        gluten_free: Set(false),
        serial: Set(Uuid::new_v4()),
        bakery_id: Set(None),
        ..Default::default()
    };

    let cake = mud_cake.save(db).await.expect("could not insert cake");

    // if compiling without sqlx, this assignment will complain,
    // but the whole test is useless in that case anyway.
    #[allow(unused_variables)]
    let error: DbErr = cake
        .into_active_model()
        .insert(db)
        .await
        .expect_err("inserting should fail due to duplicate primary key");

    let mut error_message: &str = "";
    if db.get_database_backend() == DbBackend::MySql {
        error_message = "Duplicate entry '1' for key 'cake.PRIMARY'"
    } else if db.get_database_backend() == DbBackend::Postgres {
        error_message = "duplicate key value violates unique constraint \"cake_pkey\""
    } else if db.get_database_backend() == DbBackend::Sqlite {
        error_message = "UNIQUE constraint failed: cake.id"
    }

    assert_eq!(
        error.sql_err(),
        Some(SqlErr::UniqueConstraintViolation(String::from(
            error_message
        )))
    );

    let fk_cake = cake::ActiveModel {
        name: Set("fk error Cake".to_owned()),
        price: Set(dec!(10.25)),
        gluten_free: Set(false),
        serial: Set(Uuid::new_v4()),
        bakery_id: Set(Some(1000)),
        ..Default::default()
    };

    let fk_error = fk_cake
        .insert(db)
        .await
        .expect_err("create foreign key should fail with non-primary key");

    if db.get_database_backend() == DbBackend::MySql {
        error_message = "Cannot add or update a child row: a foreign key constraint fails (`bakery_chain_sql_err_tests`.`cake`, CONSTRAINT `fk-cake-bakery_id` FOREIGN KEY (`bakery_id`) REFERENCES `bakery` (`id`) ON DELETE CASCADE ON UPDATE CASCADE)"
    } else if db.get_database_backend() == DbBackend::Postgres {
        error_message = "insert or update on table \"cake\" violates foreign key constraint \"fk-cake-bakery_id\""
    } else if db.get_database_backend() == DbBackend::Sqlite {
        error_message = "FOREIGN KEY constraint failed"
    }

    assert_eq!(
        fk_error.sql_err(),
        Some(SqlErr::ForeignKeyConstraintViolation(String::from(
            error_message
        )))
    );
}
