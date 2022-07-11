use crate::DbBackend;
use sea_query::{inject_parameters, MysqlQueryBuilder, PostgresQueryBuilder, SqliteQueryBuilder};
pub use sea_query::{Value, Values};
use std::fmt;

/// Defines an SQL statement
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    /// The SQL query
    pub sql: String,
    /// The values for the SQL statement's parameters
    pub values: Option<Values>,
    /// The database backend this statement is constructed for.
    /// The SQL dialect and values should be valid for the DbBackend.
    pub db_backend: DbBackend,
}

/// Constraints for building a [Statement]
pub trait StatementBuilder: IntoAnyStatement {
    /// Method to call in order to build a [Statement]
    fn build(&self, db_backend: &DbBackend) -> Statement;

    fn build_with_plugins<I, T>(&self, db_backend: &DbBackend, plugins: I) -> Statement
    where
        I: IntoIterator<Item = T>,
        T: StatementBuilderPlugin,
    {
        let stmt = self.build(db_backend);
        let any_stmt = self.into_any_statement();
        for plugin in plugins {
            plugin.run(&any_stmt);
        }
        stmt
    }
}

#[derive(Debug)]
pub enum AnyStatement<'a> {
    Insert(&'a sea_query::InsertStatement),
    Select(&'a sea_query::SelectStatement),
    Update(&'a sea_query::UpdateStatement),
    Delete(&'a sea_query::DeleteStatement),
    TableCreate(&'a sea_query::TableCreateStatement),
    TableDrop(&'a sea_query::TableDropStatement),
    TableAlter(&'a sea_query::TableAlterStatement),
    TableRename(&'a sea_query::TableRenameStatement),
    TableTruncate(&'a sea_query::TableTruncateStatement),
    IndexCreate(&'a sea_query::IndexCreateStatement),
    IndexDrop(&'a sea_query::IndexDropStatement),
    ForeignKeyCreate(&'a sea_query::ForeignKeyCreateStatement),
    ForeignKeyDrop(&'a sea_query::ForeignKeyDropStatement),
    TypeAlter(&'a sea_query::extension::postgres::TypeAlterStatement),
    TypeCreate(&'a sea_query::extension::postgres::TypeCreateStatement),
    TypeDrop(&'a sea_query::extension::postgres::TypeDropStatement),
}

pub trait IntoAnyStatement {
    fn into_any_statement(&self) -> AnyStatement;
}

pub trait StatementBuilderPlugin {
    fn run(&self, stmt: &AnyStatement);
}

impl Statement {
    /// Create a [Statement] from a [crate::DatabaseBackend] and a raw SQL statement
    pub fn from_string(db_backend: DbBackend, stmt: String) -> Statement {
        Statement {
            sql: stmt,
            values: None,
            db_backend,
        }
    }

    /// Create a SQL statement from a [crate::DatabaseBackend], a
    /// raw SQL statement and param values
    pub fn from_sql_and_values<I>(db_backend: DbBackend, sql: &str, values: I) -> Self
    where
        I: IntoIterator<Item = Value>,
    {
        Self::from_string_values_tuple(
            db_backend,
            (sql.to_owned(), Values(values.into_iter().collect())),
        )
    }

    pub(crate) fn from_string_values_tuple(
        db_backend: DbBackend,
        stmt: (String, Values),
    ) -> Statement {
        Statement {
            sql: stmt.0,
            values: Some(stmt.1),
            db_backend,
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.values {
            Some(values) => {
                let string = inject_parameters(
                    &self.sql,
                    values.0.clone(),
                    self.db_backend.get_query_builder().as_ref(),
                );
                write!(f, "{}", &string)
            }
            None => {
                write!(f, "{}", &self.sql)
            }
        }
    }
}

macro_rules! impl_into_any_stmt {
    ($stmt: ty, $variant: ident) => {
        impl IntoAnyStatement for $stmt {
            fn into_any_statement(&self) -> AnyStatement {
                AnyStatement::$variant(self)
            }
        }
    };
}

macro_rules! build_any_stmt {
    ($stmt: expr, $db_backend: expr) => {
        match $db_backend {
            DbBackend::MySql => $stmt.build(MysqlQueryBuilder),
            DbBackend::Postgres => $stmt.build(PostgresQueryBuilder),
            DbBackend::Sqlite => $stmt.build(SqliteQueryBuilder),
        }
    };
}

macro_rules! build_postgres_stmt {
    ($stmt: expr, $db_backend: expr) => {
        match $db_backend {
            DbBackend::Postgres => $stmt.to_string(PostgresQueryBuilder),
            DbBackend::MySql | DbBackend::Sqlite => unimplemented!(),
        }
    };
}

macro_rules! build_query_stmt {
    ($stmt: ty, $variant: ident) => {
        impl StatementBuilder for $stmt {
            fn build(&self, db_backend: &DbBackend) -> Statement {
                let stmt = build_any_stmt!(self, db_backend);
                Statement::from_string_values_tuple(*db_backend, stmt)
            }
        }

        impl_into_any_stmt!($stmt, $variant);
    };
}

build_query_stmt!(sea_query::InsertStatement, Insert);
build_query_stmt!(sea_query::SelectStatement, Select);
build_query_stmt!(sea_query::UpdateStatement, Update);
build_query_stmt!(sea_query::DeleteStatement, Delete);

macro_rules! build_schema_stmt {
    ($stmt: ty, $variant: ident) => {
        impl StatementBuilder for $stmt {
            fn build(&self, db_backend: &DbBackend) -> Statement {
                let stmt = build_any_stmt!(self, db_backend);
                Statement::from_string(*db_backend, stmt)
            }
        }

        impl_into_any_stmt!($stmt, $variant);
    };
}

build_schema_stmt!(sea_query::TableCreateStatement, TableCreate);
build_schema_stmt!(sea_query::TableDropStatement, TableDrop);
build_schema_stmt!(sea_query::TableAlterStatement, TableAlter);
build_schema_stmt!(sea_query::TableRenameStatement, TableRename);
build_schema_stmt!(sea_query::TableTruncateStatement, TableTruncate);
build_schema_stmt!(sea_query::IndexCreateStatement, IndexCreate);
build_schema_stmt!(sea_query::IndexDropStatement, IndexDrop);
build_schema_stmt!(sea_query::ForeignKeyCreateStatement, ForeignKeyCreate);
build_schema_stmt!(sea_query::ForeignKeyDropStatement, ForeignKeyDrop);

macro_rules! build_type_stmt {
    ($stmt: ty, $variant: ident) => {
        impl StatementBuilder for $stmt {
            fn build(&self, db_backend: &DbBackend) -> Statement {
                let stmt = build_postgres_stmt!(self, db_backend);
                Statement::from_string(*db_backend, stmt)
            }
        }

        impl_into_any_stmt!($stmt, $variant);
    };
}

build_type_stmt!(
    sea_query::extension::postgres::TypeAlterStatement,
    TypeAlter
);
build_type_stmt!(
    sea_query::extension::postgres::TypeCreateStatement,
    TypeCreate
);
build_type_stmt!(sea_query::extension::postgres::TypeDropStatement, TypeDrop);
