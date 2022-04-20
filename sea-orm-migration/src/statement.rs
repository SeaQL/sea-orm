//! Convert SQL statement

use crate::into_orm_db_backend;
use sea_schema::migration;

pub(crate) fn into_orm_stmt(stmt: migration::Statement) -> sea_orm::Statement {
    sea_orm::Statement {
        sql: stmt.sql,
        values: stmt.values,
        db_backend: into_orm_db_backend(stmt.db_backend),
    }
}
