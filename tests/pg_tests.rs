use sea_orm::{
    entity::prelude::*, Database, DatabaseBackend, DatabaseConnection, DbErr, ExecResult, Set,
    Statement,
};

pub mod common;
pub use common::bakery_chain::*;

use sea_query::{ColumnDef, TableCreateStatement};

// cargo test --test pg_tests -- --nocapture
#[cfg_attr(feature = "runtime-async-std", async_std::main)]
#[cfg_attr(feature = "runtime-actix", actix_rt::main)]
#[cfg_attr(feature = "runtime-tokio", tokio::main)]
#[cfg(feature = "sqlx-postgres")]
async fn main() {
    let base_url = "postgres://root:root@localhost";
    let db_name = "bakery_chain_schema_crud_tests";

    let db = setup(base_url, db_name).await;
    setup_schema(&db).await;
    create_entities(&db).await;
}

pub async fn setup(base_url: &str, db_name: &str) -> DatabaseConnection {
    let url = format!("{}/postgres", base_url);
    let db = Database::connect(&url).await.unwrap();

    let _drop_db_result = db
        .execute(Statement::from_string(
            DatabaseBackend::Postgres,
            format!("DROP DATABASE IF EXISTS \"{}\";", db_name),
        ))
        .await
        .unwrap();

    let _create_db_result = db
        .execute(Statement::from_string(
            DatabaseBackend::Postgres,
            format!("CREATE DATABASE \"{}\";", db_name),
        ))
        .await
        .unwrap();

    let url = format!("{}/{}", base_url, db_name);
    Database::connect(&url).await.unwrap()
}

async fn setup_schema(db: &DatabaseConnection) {
    assert!(create_bakery_table(db).await.is_ok());
}

async fn create_table(
    db: &DatabaseConnection,
    stmt: &TableCreateStatement,
) -> Result<ExecResult, DbErr> {
    let builder = db.get_database_backend();
    db.execute(builder.build(stmt)).await
}

pub async fn create_bakery_table(db: &DatabaseConnection) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(bakery::Entity)
        .if_not_exists()
        .col(
            ColumnDef::new(bakery::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(bakery::Column::Name).string())
        .col(ColumnDef::new(bakery::Column::ProfitMargin).double())
        .to_owned();

    create_table(db, &stmt).await
}

async fn create_entities(db: &DatabaseConnection) {
    test_create_bakery(db).await;
}

pub async fn test_create_bakery(db: &DatabaseConnection) {
    let seaside_bakery = bakery::ActiveModel {
        name: Set("SeaSide Bakery".to_owned()),
        profit_margin: Set(10.4),
        ..Default::default()
    };
    let res = Bakery::insert(seaside_bakery)
        .exec(db)
        .await
        .expect("could not insert bakery");

    let bakery = Bakery::find_by_id(res.last_insert_id)
        .one(db)
        .await
        .expect("could not find bakery");

    assert!(bakery.is_some());
    let bakery_model = bakery.unwrap();
    assert_eq!(bakery_model.name, "SeaSide Bakery");
    assert_eq!(bakery_model.profit_margin, 10.4);
}
