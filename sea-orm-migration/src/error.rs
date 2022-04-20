//! Convert migration error

use sea_schema::migration;

pub(crate) fn into_orm_db_err(err: migration::MigrationErr) -> sea_orm::DbErr {
    sea_orm::DbErr::Migration(err.to_string())
}

pub(crate) fn into_migration_err(err: sea_orm::DbErr) -> migration::MigrationErr {
    migration::MigrationErr(err.to_string())
}
