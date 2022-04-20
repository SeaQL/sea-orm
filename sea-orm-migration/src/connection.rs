//! Manage migration connection

use crate::{into_migration_db_backend, into_migration_err, into_orm_stmt, QueryResult};
use sea_orm::ConnectionTrait;
use sea_schema::migration::{self, MigrationErr, QueryResultTrait};

#[derive(Debug)]
pub(crate) struct DatabaseConnection<'c> {
    pub(crate) conn: &'c sea_orm::DatabaseConnection,
}

#[async_trait::async_trait]
impl migration::ConnectionTrait for DatabaseConnection<'_> {
    fn get_database_backend(&self) -> migration::DbBackend {
        into_migration_db_backend(ConnectionTrait::get_database_backend(self.conn))
    }

    async fn execute(&self, stmt: migration::Statement) -> Result<(), MigrationErr> {
        ConnectionTrait::execute(self.conn, into_orm_stmt(stmt))
            .await
            .map(|_| ())
            .map_err(|e| MigrationErr(e.to_string()))
    }

    async fn query_one(
        &self,
        stmt: migration::Statement,
    ) -> Result<Option<Box<dyn QueryResultTrait>>, MigrationErr> {
        ConnectionTrait::query_one(self.conn, into_orm_stmt(stmt))
            .await
            .map(|res| res.map(|res| Box::new(QueryResult { res }) as Box<dyn QueryResultTrait>))
            .map_err(into_migration_err)
    }

    async fn query_all(
        &self,
        stmt: migration::Statement,
    ) -> Result<Vec<Box<dyn QueryResultTrait>>, MigrationErr> {
        ConnectionTrait::query_all(self.conn, into_orm_stmt(stmt))
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|res| Box::new(QueryResult { res }) as Box<dyn QueryResultTrait>)
                    .collect()
            })
            .map_err(into_migration_err)
    }
}
