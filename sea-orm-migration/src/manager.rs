//! Schema manager

use crate::{into_orm_db_err, DatabaseConnection};
use sea_orm::sea_query::{
    extension::postgres::{TypeAlterStatement, TypeCreateStatement, TypeDropStatement},
    ForeignKeyCreateStatement, ForeignKeyDropStatement, IndexCreateStatement, IndexDropStatement,
    TableAlterStatement, TableCreateStatement, TableDropStatement, TableRenameStatement,
    TableTruncateStatement,
};
use sea_orm::{ConnectionTrait, DbBackend, DbConn, DbErr, StatementBuilder};
use sea_schema::migration;

/// Helper struct for writing migration scripts in migration file
#[derive(Debug)]
pub struct SchemaManager<'c> {
    conn: DatabaseConnection<'c>,
}

impl<'c> SchemaManager<'c> {
    /// Initialize [`SchemaManager`]
    pub fn new(conn: &'c DbConn) -> Self {
        Self {
            conn: DatabaseConnection { conn },
        }
    }

    /// Execute any statement that implemented [`StatementBuilder`]
    pub async fn exec_stmt<S>(&self, stmt: S) -> Result<(), DbErr>
    where
        S: StatementBuilder,
    {
        let builder = self.conn.conn.get_database_backend();
        self.conn
            .conn
            .execute(builder.build(&stmt))
            .await
            .map(|_| ())
    }

    /// Get database backend
    pub fn get_database_backend(&self) -> DbBackend {
        self.conn.conn.get_database_backend()
    }

    /// Borrow database connection
    pub fn get_connection(&self) -> &'c DbConn {
        self.conn.conn
    }
}

/// Schema Creation
impl<'c> SchemaManager<'c> {
    /// Create table
    pub async fn create_table(&self, stmt: TableCreateStatement) -> Result<(), DbErr> {
        migration::SchemaManager::create_table(stmt, &self.conn)
            .await
            .map_err(into_orm_db_err)
    }

    /// Create index
    pub async fn create_index(&self, stmt: IndexCreateStatement) -> Result<(), DbErr> {
        migration::SchemaManager::create_index(stmt, &self.conn)
            .await
            .map_err(into_orm_db_err)
    }

    /// Create foreign key
    pub async fn create_foreign_key(&self, stmt: ForeignKeyCreateStatement) -> Result<(), DbErr> {
        migration::SchemaManager::create_foreign_key(stmt, &self.conn)
            .await
            .map_err(into_orm_db_err)
    }

    /// Create type
    pub async fn create_type(&self, stmt: TypeCreateStatement) -> Result<(), DbErr> {
        migration::SchemaManager::create_type(stmt, &self.conn)
            .await
            .map_err(into_orm_db_err)
    }
}

/// Schema Mutation
impl<'c> SchemaManager<'c> {
    /// Alter table
    pub async fn alter_table(&self, stmt: TableAlterStatement) -> Result<(), DbErr> {
        migration::SchemaManager::alter_table(stmt, &self.conn)
            .await
            .map_err(into_orm_db_err)
    }

    /// Drop table
    pub async fn drop_table(&self, stmt: TableDropStatement) -> Result<(), DbErr> {
        migration::SchemaManager::drop_table(stmt, &self.conn)
            .await
            .map_err(into_orm_db_err)
    }

    /// Rename table
    pub async fn rename_table(&self, stmt: TableRenameStatement) -> Result<(), DbErr> {
        migration::SchemaManager::rename_table(stmt, &self.conn)
            .await
            .map_err(into_orm_db_err)
    }

    /// Truncate table
    pub async fn truncate_table(&self, stmt: TableTruncateStatement) -> Result<(), DbErr> {
        migration::SchemaManager::truncate_table(stmt, &self.conn)
            .await
            .map_err(into_orm_db_err)
    }

    /// Drop index
    pub async fn drop_index(&self, stmt: IndexDropStatement) -> Result<(), DbErr> {
        migration::SchemaManager::drop_index(stmt, &self.conn)
            .await
            .map_err(into_orm_db_err)
    }

    /// Drop foreign key
    pub async fn drop_foreign_key(&self, stmt: ForeignKeyDropStatement) -> Result<(), DbErr> {
        migration::SchemaManager::drop_foreign_key(stmt, &self.conn)
            .await
            .map_err(into_orm_db_err)
    }

    /// Alter type
    pub async fn alter_type(&self, stmt: TypeAlterStatement) -> Result<(), DbErr> {
        migration::SchemaManager::alter_type(stmt, &self.conn)
            .await
            .map_err(into_orm_db_err)
    }

    /// Drop type
    pub async fn drop_type(&self, stmt: TypeDropStatement) -> Result<(), DbErr> {
        migration::SchemaManager::drop_type(stmt, &self.conn)
            .await
            .map_err(into_orm_db_err)
    }
}

/// Schema Inspection
impl<'c> SchemaManager<'c> {
    /// Check if a table exists in the database
    pub async fn has_table<T>(&self, table: T) -> Result<bool, DbErr>
    where
        T: AsRef<str>,
    {
        migration::SchemaManager::has_table(table, &self.conn)
            .await
            .map_err(into_orm_db_err)
    }

    /// Check if a column exists in a specific database table
    pub async fn has_column<T, C>(&self, table: T, column: C) -> Result<bool, DbErr>
    where
        T: AsRef<str>,
        C: AsRef<str>,
    {
        migration::SchemaManager::has_column(table, column, &self.conn)
            .await
            .map_err(into_orm_db_err)
    }
}
