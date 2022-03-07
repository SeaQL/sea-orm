pub use super::super::bakery_chain::*;

use super::*;
use crate::common::setup::{create_enum, create_table, create_table_without_asserts};
use sea_orm::{
    error::*, sea_query, ConnectionTrait, DatabaseConnection, DbBackend, DbConn, EntityName,
    ExecResult, Schema,
};
use sea_query::{extension::postgres::Type, Alias, ColumnDef, ForeignKeyCreateStatement};

pub async fn create_tables(db: &DatabaseConnection) -> Result<(), DbErr> {
    let db_backend = db.get_database_backend();

    create_log_table(db).await?;
    create_metadata_table(db).await?;
    create_repository_table(db).await?;
    create_self_join_table(db).await?;
    create_byte_primary_key_table(db).await?;
    create_satellites_table(db).await?;

    let create_enum_stmts = match db_backend {
        DbBackend::MySql | DbBackend::Sqlite => Vec::new(),
        DbBackend::Postgres => {
            let schema = Schema::new(db_backend);
            let enum_create_stmt = Type::create()
                .as_enum(Alias::new("tea"))
                .values(vec![Alias::new("EverydayTea"), Alias::new("BreakfastTea")])
                .to_owned();
            assert_eq!(
                db_backend.build(&enum_create_stmt),
                db_backend.build(&schema.create_enum_from_active_enum::<Tea>())
            );
            vec![enum_create_stmt]
        }
    };
    create_enum(db, &create_enum_stmts, ActiveEnum).await?;

    create_active_enum_table(db).await?;
    create_active_enum_child_table(db).await?;

    Ok(())
}

pub async fn create_log_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(applog::Entity)
        .col(
            ColumnDef::new(applog::Column::Id)
                .big_integer()
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
                .name("fk-self_join-uuid_ref")
                .from_tbl(SelfJoin)
                .from_col(self_join::Column::UuidRef)
                .to_tbl(SelfJoin)
                .to_col(self_join::Column::Uuid),
        )
        .to_owned();

    create_table(db, &stmt, SelfJoin).await
}

pub async fn create_byte_primary_key_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let mut primary_key_col = ColumnDef::new(byte_primary_key::Column::Id);
    match db.get_database_backend() {
        DbBackend::MySql => primary_key_col.binary_len(3),
        DbBackend::Sqlite | DbBackend::Postgres => primary_key_col.binary(),
    };

    let stmt = sea_query::Table::create()
        .table(byte_primary_key::Entity)
        .col(primary_key_col.not_null().primary_key())
        .col(
            ColumnDef::new(byte_primary_key::Column::Value)
                .string()
                .not_null(),
        )
        .to_owned();

    create_table_without_asserts(db, &stmt).await
}

pub async fn create_active_enum_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let create_table_stmt = sea_query::Table::create()
        .table(active_enum::Entity.table_ref())
        .col(
            ColumnDef::new(active_enum::Column::Id)
                .big_integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(active_enum::Column::Category).string_len(1))
        .col(ColumnDef::new(active_enum::Column::Color).big_integer())
        .col(
            ColumnDef::new(active_enum::Column::Tea)
                .enumeration("tea", vec!["EverydayTea", "BreakfastTea"]),
        )
        .to_owned();

    create_table(db, &create_table_stmt, ActiveEnum).await
}

pub async fn create_active_enum_child_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let create_table_stmt = sea_query::Table::create()
        .table(active_enum_child::Entity.table_ref())
        .col(
            ColumnDef::new(active_enum_child::Column::Id)
                .big_integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(active_enum_child::Column::ParentId)
                .big_integer()
                .not_null(),
        )
        .col(ColumnDef::new(active_enum_child::Column::Category).string_len(1))
        .col(ColumnDef::new(active_enum_child::Column::Color).big_integer())
        .col(
            ColumnDef::new(active_enum_child::Column::Tea)
                .enumeration("tea", vec!["EverydayTea", "BreakfastTea"]),
        )
        .foreign_key(
            ForeignKeyCreateStatement::new()
                .name("fk-active_enum_child-active_enum")
                .from_tbl(ActiveEnumChild)
                .from_col(active_enum_child::Column::ParentId)
                .to_tbl(ActiveEnum)
                .to_col(active_enum::Column::Id),
        )
        .to_owned();

    create_table(db, &create_table_stmt, ActiveEnumChild).await
}

pub async fn create_satellites_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(satellite::Entity)
        .col(
            ColumnDef::new(satellite::Column::Id)
                .big_integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(satellite::Column::SatelliteName)
                .string()
                .not_null(),
        )
        .col(
            ColumnDef::new(satellite::Column::LaunchDate)
                .timestamp_with_time_zone()
                .not_null()
                .default("2022-01-26 16:24:00"),
        )
        .col(
            ColumnDef::new(satellite::Column::DeploymentDate)
                .timestamp_with_time_zone()
                .not_null()
                .default("2022-01-26 16:24:00"),
        )
        .to_owned();

    create_table(db, &stmt, Satellite).await
}
