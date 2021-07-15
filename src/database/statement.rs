use crate::DatabaseBackend;
use sea_query::{
    inject_parameters, MysqlQueryBuilder, PostgresQueryBuilder, SqliteQueryBuilder, Values,
};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub sql: String,
    pub values: Option<Values>,
    pub db_backend: DatabaseBackend,
}

pub trait IntoStatement {
    fn into_statement(&self, db_backend: &DatabaseBackend) -> Statement;
}

impl Statement {
    pub fn from_string(db_backend: DatabaseBackend, stmt: String) -> Statement {
        Statement {
            sql: stmt,
            values: None,
            db_backend,
        }
    }

    pub fn from_string_values_tuple(
        db_backend: DatabaseBackend,
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
            DatabaseBackend::MySql => $stmt.build(MysqlQueryBuilder),
            DatabaseBackend::Postgres => $stmt.build(PostgresQueryBuilder),
            DatabaseBackend::Sqlite => $stmt.build(SqliteQueryBuilder),
        }
    };
}

macro_rules! build_query_stmt {
    ($stmt: ty) => {
        impl IntoStatement for $stmt {
            fn into_statement(&self, db_backend: &DatabaseBackend) -> Statement {
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
        impl IntoStatement for $stmt {
            fn into_statement(&self, db_backend: &DatabaseBackend) -> Statement {
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
