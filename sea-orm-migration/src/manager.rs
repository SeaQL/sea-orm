use super::{IntoSchemaManagerConnection, SchemaManagerConnection};
use sea_orm::sea_query::{
    ForeignKeyCreateStatement, ForeignKeyDropStatement, IndexCreateStatement, IndexDropStatement,
    SelectStatement, TableAlterStatement, TableCreateStatement, TableDropStatement,
    TableRenameStatement, TableTruncateStatement,
    extension::postgres::{TypeAlterStatement, TypeCreateStatement, TypeDropStatement},
};
use sea_orm::{ConnectionTrait, DbBackend, DbErr, StatementBuilder};
#[allow(unused_imports)]
use sea_schema::probe::SchemaProbe;

/// Helper struct for writing migration scripts in migration file
pub struct SchemaManager<'c> {
    conn: SchemaManagerConnection<'c>,
}

impl<'c> SchemaManager<'c> {
    pub fn new<T>(conn: T) -> Self
    where
        T: IntoSchemaManagerConnection<'c>,
    {
        Self {
            conn: conn.into_schema_manager_connection(),
        }
    }

    pub async fn execute<S>(&self, stmt: S) -> Result<(), DbErr>
    where
        S: StatementBuilder,
    {
        self.conn.execute(&stmt).await.map(|_| ())
    }

    #[doc(hidden)]
    pub async fn exec_stmt<S>(&self, stmt: S) -> Result<(), DbErr>
    where
        S: StatementBuilder,
    {
        self.conn.execute(&stmt).await.map(|_| ())
    }

    pub fn get_database_backend(&self) -> DbBackend {
        self.conn.get_database_backend()
    }

    pub fn get_connection(&self) -> &SchemaManagerConnection<'c> {
        &self.conn
    }
}

/// Schema Creation
impl SchemaManager<'_> {
    pub async fn create_table(&self, stmt: TableCreateStatement) -> Result<(), DbErr> {
        self.execute(stmt).await
    }

    pub async fn create_index(&self, stmt: IndexCreateStatement) -> Result<(), DbErr> {
        self.execute(stmt).await
    }

    pub async fn create_foreign_key(&self, stmt: ForeignKeyCreateStatement) -> Result<(), DbErr> {
        self.execute(stmt).await
    }

    pub async fn create_type(&self, stmt: TypeCreateStatement) -> Result<(), DbErr> {
        self.execute(stmt).await
    }
}

/// Schema Mutation
impl SchemaManager<'_> {
    pub async fn alter_table(&self, stmt: TableAlterStatement) -> Result<(), DbErr> {
        self.execute(stmt).await
    }

    pub async fn drop_table(&self, stmt: TableDropStatement) -> Result<(), DbErr> {
        self.execute(stmt).await
    }

    pub async fn rename_table(&self, stmt: TableRenameStatement) -> Result<(), DbErr> {
        self.execute(stmt).await
    }

    pub async fn truncate_table(&self, stmt: TableTruncateStatement) -> Result<(), DbErr> {
        self.execute(stmt).await
    }

    pub async fn drop_index(&self, stmt: IndexDropStatement) -> Result<(), DbErr> {
        self.execute(stmt).await
    }

    pub async fn drop_foreign_key(&self, stmt: ForeignKeyDropStatement) -> Result<(), DbErr> {
        self.execute(stmt).await
    }

    pub async fn alter_type(&self, stmt: TypeAlterStatement) -> Result<(), DbErr> {
        self.execute(stmt).await
    }

    pub async fn drop_type(&self, stmt: TypeDropStatement) -> Result<(), DbErr> {
        self.execute(stmt).await
    }
}

/// Schema Inspection.
impl SchemaManager<'_> {
    pub async fn has_table<T>(&self, table: T) -> Result<bool, DbErr>
    where
        T: AsRef<str>,
    {
        has_table(&self.conn, table).await
    }

    pub async fn has_column<T, C>(&self, _table: T, _column: C) -> Result<bool, DbErr>
    where
        T: AsRef<str>,
        C: AsRef<str>,
    {
        let _stmt: SelectStatement = match self.conn.get_database_backend() {
            #[cfg(feature = "sqlx-mysql")]
            DbBackend::MySql => sea_schema::mysql::MySql.has_column(_table, _column),
            #[cfg(feature = "sqlx-postgres")]
            DbBackend::Postgres => sea_schema::postgres::Postgres.has_column(_table, _column),
            #[cfg(feature = "sqlx-sqlite")]
            DbBackend::Sqlite => sea_schema::sqlite::Sqlite.has_column(_table, _column),
            #[allow(unreachable_patterns)]
            other => {
                return Err(DbErr::BackendNotSupported {
                    db: other.as_str(),
                    ctx: "has_column",
                });
            }
        };

        #[allow(unreachable_code)]
        let res = self
            .conn
            .query_one(&_stmt)
            .await?
            .ok_or_else(|| DbErr::Custom("Failed to check column exists".to_owned()))?;

        res.try_get("", "has_column")
    }

    pub async fn has_index<T, I>(&self, _table: T, _index: I) -> Result<bool, DbErr>
    where
        T: AsRef<str>,
        I: AsRef<str>,
    {
        let _stmt: SelectStatement = match self.conn.get_database_backend() {
            #[cfg(feature = "sqlx-mysql")]
            DbBackend::MySql => sea_schema::mysql::MySql.has_index(_table, _index),
            #[cfg(feature = "sqlx-postgres")]
            DbBackend::Postgres => sea_schema::postgres::Postgres.has_index(_table, _index),
            #[cfg(feature = "sqlx-sqlite")]
            DbBackend::Sqlite => sea_schema::sqlite::Sqlite.has_index(_table, _index),
            #[allow(unreachable_patterns)]
            other => {
                return Err(DbErr::BackendNotSupported {
                    db: other.as_str(),
                    ctx: "has_index",
                });
            }
        };

        #[allow(unreachable_code)]
        let res = self
            .conn
            .query_one(&_stmt)
            .await?
            .ok_or_else(|| DbErr::Custom("Failed to check index exists".to_owned()))?;

        res.try_get("", "has_index")
    }
}

pub(crate) async fn has_table<C, T>(conn: &C, _table: T) -> Result<bool, DbErr>
where
    C: ConnectionTrait,
    T: AsRef<str>,
{
    let _stmt: SelectStatement = match conn.get_database_backend() {
        #[cfg(feature = "sqlx-mysql")]
        DbBackend::MySql => sea_schema::mysql::MySql.has_table(_table),
        #[cfg(feature = "sqlx-postgres")]
        DbBackend::Postgres => sea_schema::postgres::Postgres.has_table(_table),
        #[cfg(feature = "sqlx-sqlite")]
        DbBackend::Sqlite => sea_schema::sqlite::Sqlite.has_table(_table),
        #[allow(unreachable_patterns)]
        other => {
            return Err(DbErr::BackendNotSupported {
                db: other.as_str(),
                ctx: "has_table",
            });
        }
    };

    #[allow(unreachable_code)]
    let res = conn
        .query_one(&_stmt)
        .await?
        .ok_or_else(|| DbErr::Custom("Failed to check table exists".to_owned()))?;

    res.try_get("", "has_table")
}
