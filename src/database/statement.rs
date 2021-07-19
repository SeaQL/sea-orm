use crate::DbBackend;
use sea_query::{
    inject_parameters, MysqlQueryBuilder, PostgresQueryBuilder, SqliteQueryBuilder,
};
pub use sea_query::{Value, Values};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub sql: String,
    pub values: Option<Values>,
    pub db_backend: DbBackend,
}

pub trait StatementBuilder {
    fn build(&self, db_backend: &DbBackend) -> Statement;
}

impl Statement {
    pub fn from_string(db_backend: DbBackend, stmt: String) -> Statement {
        Statement {
            sql: stmt,
            values: None,
            db_backend,
        }
    }

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

macro_rules! build_any_stmt {
    ($stmt: expr, $db_backend: expr) => {
        match $db_backend {
            DbBackend::MySql => $stmt.build(MysqlQueryBuilder),
            DbBackend::Postgres => $stmt.build(PostgresQueryBuilder),
            DbBackend::Sqlite => $stmt.build(SqliteQueryBuilder),
        }
    };
}

macro_rules! build_query_stmt {
    ($stmt: ty) => {
        impl StatementBuilder for $stmt {
            fn build(&self, db_backend: &DbBackend) -> Statement {
                let stmt = build_any_stmt!(self, db_backend);
                Statement::from_string_values_tuple(*db_backend, stmt)
            }
        }
    };
}

build_query_stmt!(sea_query::InsertStatement);
build_query_stmt!(sea_query::SelectStatement);
build_query_stmt!(sea_query::UpdateStatement);
build_query_stmt!(sea_query::DeleteStatement);

macro_rules! build_schema_stmt {
    ($stmt: ty) => {
        impl StatementBuilder for $stmt {
            fn build(&self, db_backend: &DbBackend) -> Statement {
                let stmt = build_any_stmt!(self, db_backend);
                Statement::from_string(*db_backend, stmt)
            }
        }
    };
}

build_schema_stmt!(sea_query::TableCreateStatement);
build_schema_stmt!(sea_query::TableDropStatement);
build_schema_stmt!(sea_query::TableAlterStatement);
build_schema_stmt!(sea_query::TableRenameStatement);
build_schema_stmt!(sea_query::TableTruncateStatement);
