//! > Adapted from https://github.com/loco-rs/loco/blob/master/src/schema.rs
//!
//! # Database Table Schema Helpers
//!
//! This module defines functions and helpers for creating database table
//! schemas using the `sea-orm` and `sea-query` libraries.
//!
//! # Example
//!
//! The following example shows how the user migration file should be and using
//! the schema helpers to create the Db fields.
//!
//! ```rust
//! use sea_orm_migration::{prelude::*, schema::*};
//!
//! #[derive(DeriveMigrationName)]
//! pub struct Migration;
//!
//! #[async_trait::async_trait]
//! impl MigrationTrait for Migration {
//!     async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
//!         let table = table_auto(Users::Table)
//!             .col(&mut pk_auto(Users::Id)) // TODO: remove `&mut` after upgraded to sea-query 0.31.0
//!             .col(&mut uuid(Users::Pid))
//!             .col(&mut string_uniq(Users::Email))
//!             .col(&mut string(Users::Password))
//!             .col(&mut string(Users::Name))
//!             .col(&mut string_null(Users::ResetToken))
//!             .col(&mut timestamp_null(Users::ResetSentAt))
//!             .to_owned();
//!         manager.create_table(table).await?;
//!         Ok(())
//!     }
//!
//!     async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
//!         manager
//!             .drop_table(Table::drop().table(Users::Table).to_owned())
//!             .await
//!     }
//! }
//!
//! #[derive(Iden)]
//! pub enum Users {
//!     Table,
//!     Id,
//!     Pid,
//!     Email,
//!     Name,
//!     Password,
//!     ResetToken,
//!     ResetSentAt,
//! }
//! ```

use crate::{prelude::Iden, sea_query};
use sea_orm::sea_query::{ColumnDef, Expr, IntoIden, Table, TableCreateStatement};

#[derive(Iden)]
enum GeneralIds {
    CreatedAt,
    UpdatedAt,
}

/// Wrapping  table schema creation.
pub fn table_auto<T>(name: T) -> TableCreateStatement
where
    T: IntoIden + 'static,
{
    timestamps(Table::create().table(name).if_not_exists().take())
}

/// Create a primary key column with auto-increment feature.
pub fn pk_auto<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name)
        .integer()
        .not_null()
        .auto_increment()
        .primary_key()
        .take()
}

/// Add timestamp columns (`CreatedAt` and `UpdatedAt`) to an existing table.
#[must_use]
pub fn timestamps(t: TableCreateStatement) -> TableCreateStatement {
    let mut t = t;
    t.col(
        ColumnDef::new(GeneralIds::CreatedAt)
            .date_time()
            .not_null()
            .take()
            .default(Expr::current_timestamp()),
    )
    .col(timestamp(GeneralIds::UpdatedAt).default(Expr::current_timestamp()));
    t.take()
}

/// Create a UUID column definition with a unique constraint.
pub fn uuid<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).unique_key().uuid().not_null().take()
}

/// Create a UUID type column definition.
pub fn uuid_col<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).uuid().not_null().take()
}

/// Create a nullable UUID type column definition.
pub fn uuid_col_null<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).uuid().take()
}

/// Create a nullable string column definition.
pub fn string_null<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).string().take()
}

/// Create a nullable timestamptz column definition.
pub fn timestamptz_null<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).timestamp_with_time_zone().take()
}

/// Create a non-nullable timestamptz column definition.
pub fn timestamptz<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name)
        .timestamp_with_time_zone()
        .not_null()
        .take()
}

/// Create a non-nullable string column definition.
pub fn string<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    string_null(name).not_null().take()
}

/// Create a unique string column definition.
pub fn string_uniq<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    string(name).unique_key().take()
}

/// Create a nullable text column definition.
pub fn text_null<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).text().take()
}

/// Create a nullable text column definition.
pub fn text<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).text().take()
}

/// Create a nullable tiny integer column definition.
pub fn tiny_integer_null<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).tiny_integer().take()
}

/// Create a non-nullable tiny integer column definition.
pub fn tiny_integer<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).tiny_integer().not_null().take()
}

/// Create a unique tiny integer column definition.
pub fn tiny_integer_uniq<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).tiny_integer().unique_key().take()
}

/// Create a nullable small integer column definition.
pub fn small_integer_null<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).small_integer().take()
}

/// Create a non-nullable small integer column definition.
pub fn small_integer<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).small_integer().not_null().take()
}

/// Create a unique small integer column definition.
pub fn small_integer_uniq<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).small_integer().unique_key().take()
}

/// Create a nullable integer column definition.
pub fn integer_null<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).integer().take()
}

/// Create a non-nullable integer column definition.
pub fn integer<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).integer().not_null().take()
}

/// Create a unique integer column definition.
pub fn integer_uniq<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).integer().unique_key().take()
}

/// Create a nullable big integer column definition.
pub fn big_integer_null<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).big_integer().take()
}

/// Create a non-nullable big integer column definition.
pub fn big_integer<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).big_integer().not_null().take()
}

/// Create a unique big integer column definition.
pub fn big_integer_uniq<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).big_integer().unique_key().take()
}

/// Create a nullable float column definition.
pub fn float_null<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).float().take()
}

/// Create a non-nullable float column definition.
pub fn float<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).float().not_null().take()
}

/// Create a nullable double column definition.
pub fn double_null<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).double().take()
}

/// Create a non-nullable double column definition.
pub fn double<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).double().not_null().take()
}

/// Create a nullable decimal column definition.
pub fn decimal_null<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).decimal().take()
}

/// Create a non-nullable decimal column definition.
pub fn decimal<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).decimal().not_null().take()
}

/// Create a nullable decimal length column definition with custom precision and
/// scale.
pub fn decimal_len_null<T>(name: T, precision: u32, scale: u32) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).decimal_len(precision, scale).take()
}

/// Create a non-nullable decimal length column definition with custom precision
/// and scale.
pub fn decimal_len<T>(name: T, precision: u32, scale: u32) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name)
        .decimal_len(precision, scale)
        .not_null()
        .take()
}

/// Create a nullable boolean column definition.
pub fn bool_null<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).boolean().take()
}

/// Create a non-nullable boolean column definition.
pub fn bool<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).boolean().not_null().take()
}

/// Create a nullable date column definition.
pub fn date_null<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).date().take()
}

/// Create a non-nullable date column definition.
pub fn date<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).date().not_null().take()
}

/// Create a nullable timestamp column definition.
pub fn timestamp_null<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).date_time().take()
}

/// Create a non-nullable timestamp column definition.
pub fn timestamp<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).date_time().not_null().take()
}

/// Create a non-nullable json column definition.
pub fn json<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).json().not_null().take()
}

/// Create a nullable json column definition.
pub fn json_null<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).json().take()
}

/// Create a non-nullable json binary column definition.
pub fn jsonb<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).json_binary().not_null().take()
}

/// Create a nullable json binary column definition.
pub fn jsonb_null<T>(name: T) -> ColumnDef
where
    T: IntoIden,
{
    ColumnDef::new(name).json_binary().take()
}
