pub use super::super::bakery_chain::*;

use super::*;
use crate::common::setup::{create_table, create_enum};
use sea_orm::{
    error::*, sea_query, ConnectionTrait, DatabaseConnection, DbBackend, DbConn, ExecResult,
};
use sea_query::{extension::postgres::Type, Alias, ColumnDef, ForeignKeyCreateStatement};

pub async fn create_tables(db: &DatabaseConnection) -> Result<(), DbErr> {
    create_log_table(db).await?;
    create_metadata_table(db).await?;
    create_repository_table(db).await?;
    create_self_join_table(db).await?;
    create_active_enum_table(db).await?;

    Ok(())
}

pub async fn create_log_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(applog::Entity)
        .col(
            ColumnDef::new(applog::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(applog::Column::Action).string().not_null())
        .col(ColumnDef::new(applog::Column::Json).json().not_null())
        .col(
            ColumnDef::new(applog::Column::CreatedAt)
                .timestamp_with_time_zone()
                .not_null(),
        )
        .to_owned();

    create_table(db, &stmt, Applog).await
}

pub async fn create_metadata_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(metadata::Entity)
        .col(
            ColumnDef::new(metadata::Column::Uuid)
                .uuid()
                .not_null()
                .primary_key(),
        )
        .col(ColumnDef::new(metadata::Column::Type).string().not_null())
        .col(ColumnDef::new(metadata::Column::Key).string().not_null())
        .col(ColumnDef::new(metadata::Column::Value).string().not_null())
        .col(ColumnDef::new(metadata::Column::Bytes).binary().not_null())
        .col(ColumnDef::new(metadata::Column::Date).date())
        .col(ColumnDef::new(metadata::Column::Time).time())
        .to_owned();

    create_table(db, &stmt, Metadata).await
}

pub async fn create_repository_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(repository::Entity)
        .col(
            ColumnDef::new(repository::Column::Id)
                .string()
                .not_null()
                .primary_key(),
        )
        .col(
            ColumnDef::new(repository::Column::Owner)
                .string()
                .not_null(),
        )
        .col(ColumnDef::new(repository::Column::Name).string().not_null())
        .col(ColumnDef::new(repository::Column::Description).string())
        .to_owned();

    create_table(db, &stmt, Repository).await
}

pub async fn create_self_join_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(self_join::Entity)
        .col(
            ColumnDef::new(self_join::Column::Uuid)
                .uuid()
                .not_null()
                .primary_key(),
        )
        .col(ColumnDef::new(self_join::Column::UuidRef).uuid())
        .col(ColumnDef::new(self_join::Column::Time).time())
        .foreign_key(
            ForeignKeyCreateStatement::new()
                .name("fk-self_join-self_join")
                .from_tbl(SelfJoin)
                .from_col(self_join::Column::UuidRef)
                .to_tbl(SelfJoin)
                .to_col(self_join::Column::Uuid),
        )
        .to_owned();

    create_table(db, &stmt, SelfJoin).await
}

pub async fn create_active_enum_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let db_backend = db.get_database_backend();
    let tea_enum = Alias::new("tea");

    let create_enum_stmts = match db_backend {
        DbBackend::MySql | DbBackend::Sqlite => Vec::new(),
        DbBackend::Postgres => vec![Type::create()
            .as_enum(tea_enum.clone())
            .values(vec![Alias::new("EverydayTea"), Alias::new("BreakfastTea")])
            .to_owned()],
    };

    create_enum(db, &create_enum_stmts, ActiveEnum).await?;

    let mut tea_col = ColumnDef::new(active_enum::Column::Tea);
    match db_backend {
        DbBackend::MySql => tea_col.custom(Alias::new("ENUM('EverydayTea', 'BreakfastTea')")),
        DbBackend::Sqlite => tea_col.text(),
        DbBackend::Postgres => tea_col.custom(tea_enum),
    };
    let create_table_stmt = sea_query::Table::create()
        .table(active_enum::Entity)
        .col(
            ColumnDef::new(active_enum::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(active_enum::Column::Category).string_len(1))
        .col(ColumnDef::new(active_enum::Column::Color).integer())
        .col(&mut tea_col)
        .to_owned();

    create_table(db, &create_table_stmt, ActiveEnum).await
}
