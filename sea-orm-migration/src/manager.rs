use sea_orm::sea_query::{
    extension::postgres::{TypeAlterStatement, TypeCreateStatement, TypeDropStatement},
    Alias, Expr, ForeignKeyCreateStatement, ForeignKeyDropStatement, IndexCreateStatement,
    IndexDropStatement, Query, TableAlterStatement, TableCreateStatement, TableDropStatement,
    TableRenameStatement, TableTruncateStatement,
};
use sea_orm::{Condition, ConnectionTrait, DbBackend, DbConn, DbErr, Statement, StatementBuilder};

use super::query_tables;

/// Helper struct for writing migration scripts in migration file
pub struct SchemaManager<'c> {
    conn: &'c DbConn,
}

impl<'c> SchemaManager<'c> {
    pub fn new(conn: &'c DbConn) -> Self {
        Self { conn }
    }

    pub async fn exec_stmt<S>(&self, stmt: S) -> Result<(), DbErr>
    where
        S: StatementBuilder,
    {
        let builder = self.conn.get_database_backend();
        self.conn.execute(builder.build(&stmt)).await.map(|_| ())
    }

    pub fn get_database_backend(&self) -> DbBackend {
        self.conn.get_database_backend()
    }

    pub fn get_connection(&self) -> &'c DbConn {
        self.conn
    }
}

/// Schema Creation
impl<'c> SchemaManager<'c> {
    pub async fn create_table(&self, stmt: TableCreateStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn create_index(&self, stmt: IndexCreateStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn create_foreign_key(&self, stmt: ForeignKeyCreateStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn create_type(&self, stmt: TypeCreateStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }
}

/// Schema Mutation
impl<'c> SchemaManager<'c> {
    pub async fn alter_table(&self, stmt: TableAlterStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn drop_table(&self, stmt: TableDropStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn rename_table(&self, stmt: TableRenameStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn truncate_table(&self, stmt: TableTruncateStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn drop_index(&self, stmt: IndexDropStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn drop_foreign_key(&self, stmt: ForeignKeyDropStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn alter_type(&self, stmt: TypeAlterStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }

    pub async fn drop_type(&self, stmt: TypeDropStatement) -> Result<(), DbErr> {
        self.exec_stmt(stmt).await
    }
}

/// Schema Inspection
impl<'c> SchemaManager<'c> {
    pub async fn has_table<T>(&self, table: T) -> Result<bool, DbErr>
    where
        T: AsRef<str>,
    {
        let mut stmt = Query::select();
        let mut subquery = query_tables(self.conn);
        subquery.cond_where(Expr::col(Alias::new("table_name")).eq(table.as_ref()));
        stmt.expr_as(Expr::cust("COUNT(*)"), Alias::new("rows"))
            .from_subquery(subquery, Alias::new("subquery"));

        let builder = self.conn.get_database_backend();
        let res = self
            .conn
            .query_one(builder.build(&stmt))
            .await?
            .ok_or_else(|| DbErr::Custom("Fail to check table exists".to_owned()))?;
        let rows: i64 = res.try_get("", "rows")?;

        Ok(rows > 0)
    }

    pub async fn has_column<T, C>(&self, table: T, column: C) -> Result<bool, DbErr>
    where
        T: AsRef<str>,
        C: AsRef<str>,
    {
        let db_backend = self.conn.get_database_backend();
        let found = match db_backend {
            DbBackend::MySql | DbBackend::Postgres => {
                let schema_name = match db_backend {
                    DbBackend::MySql => "DATABASE()",
                    DbBackend::Postgres => "CURRENT_SCHEMA()",
                    DbBackend::Sqlite => unreachable!(),
                };
                let mut stmt = Query::select();
                stmt.expr_as(Expr::cust("COUNT(*)"), Alias::new("rows"))
                    .from((Alias::new("information_schema"), Alias::new("columns")))
                    .cond_where(
                        Condition::all()
                            .add(
                                Expr::expr(Expr::cust(schema_name))
                                    .equals(Alias::new("columns"), Alias::new("table_schema")),
                            )
                            .add(Expr::col(Alias::new("table_name")).eq(table.as_ref()))
                            .add(Expr::col(Alias::new("column_name")).eq(column.as_ref())),
                    );

                let res = self
                    .conn
                    .query_one(db_backend.build(&stmt))
                    .await?
                    .ok_or_else(|| DbErr::Custom("Fail to check column exists".to_owned()))?;
                let rows: i64 = res.try_get("", "rows")?;
                rows > 0
            }
            DbBackend::Sqlite => {
                let stmt = Statement::from_string(
                    db_backend,
                    format!("PRAGMA table_info({})", table.as_ref()),
                );
                let results = self.conn.query_all(stmt).await?;
                let mut found = false;
                for res in results {
                    let name: String = res.try_get("", "name")?;
                    if name.as_str() == column.as_ref() {
                        found = true;
                    }
                }
                found
            }
        };
        Ok(found)
    }
}
