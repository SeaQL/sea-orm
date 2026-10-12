use crate::DbBackend;
#[cfg(feature = "rbac")]
pub use sea_query::audit::{AuditTrait, Error as AuditError, QueryAccessAudit};
use sea_query::{MysqlQueryBuilder, PostgresQueryBuilder, SqliteQueryBuilder, inject_parameters};
pub use sea_query::{Value, Values};
use std::fmt;

/// A SQL string together with its bound parameters, ready to send to a
/// connection. Build one yourself with
/// [`from_sql_and_values`](Self::from_sql_and_values) (or the
/// [`raw_sql!`](crate::raw_sql) macro), or get one out of any query builder
/// via [`QueryTrait::build`](crate::QueryTrait::build).
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    /// The SQL text, with backend-specific placeholders for the values.
    pub sql: String,
    /// Bound parameter values, in the order they appear in `sql`. `None`
    /// means the statement has no parameters.
    pub values: Option<Values>,
    /// Backend this statement was built for; both the SQL dialect and the
    /// placeholder style must match.
    pub db_backend: DbBackend,
    /// Whether SQLx should retain the prepared statement in its connection cache.
    /// Defaults to `true`. See [`Statement::persistent`].
    pub persistent: bool,
}

/// Anything that can be rendered to a backend-specific [`Statement`].
///
/// Implemented by the `sea_query` statement types (`SelectStatement`,
/// `InsertStatement`, etc.) so they can be passed to
/// [`ConnectionTrait::execute`](crate::ConnectionTrait::execute) /
/// [`query_all`](crate::ConnectionTrait::query_all) directly.
pub trait StatementBuilder {
    /// Render `self` into a [`Statement`] for `db_backend`.
    fn build(&self, db_backend: &DbBackend) -> Statement;

    /// Set whether SQLx should retain this prepared statement in its connection cache.
    /// See [`Statement::persistent`] for backend behavior.
    fn persistent(&self, persistent: bool) -> impl StatementBuilder + '_
    where
        Self: Sized,
    {
        StatementWithPersistence {
            inner: self,
            persistent,
        }
    }

    #[cfg(feature = "rbac")]
    /// Inspect the statement and produce the access request that
    /// [`RbacEngine`](crate::rbac::RbacEngine) needs to authorise it.
    fn audit(&self) -> Result<QueryAccessAudit, AuditError>;
}

struct StatementWithPersistence<'a, S> {
    inner: &'a S,
    persistent: bool,
}

impl<S: StatementBuilder> StatementBuilder for StatementWithPersistence<'_, S> {
    fn build(&self, db_backend: &DbBackend) -> Statement {
        self.inner.build(db_backend).persistent(self.persistent)
    }

    #[cfg(feature = "rbac")]
    fn audit(&self) -> Result<QueryAccessAudit, AuditError> {
        self.inner.audit()
    }
}

impl Statement {
    /// Set whether SQLx should retain this prepared statement in its connection cache.
    ///
    /// Defaults to `true`. Set to `false` for queries whose prepared statements
    /// should not be retained, for example when PostgreSQL's cached generic
    /// plan is unsuitable for parameter-dependent workloads. Parameters remain
    /// bound; this does not interpolate them into the SQL or change session settings.
    ///
    /// This forwards SQLx's `persistent` option. It does not evict statements
    /// already cached on a connection; SQLx may reuse those statements. Use
    /// `false` consistently for a given query to avoid that reuse.
    /// Non-SQLx backends ignore this option.
    ///
    /// SQLx 0.9's PostgreSQL driver separates parsing an unnamed statement from
    /// binding it with a protocol `Sync`. Do not use `false` with a
    /// transaction-pooling proxy such as PgBouncer outside a transaction: the
    /// backend can change between those messages, causing errors or wrong results.
    pub fn persistent(mut self, persistent: bool) -> Self {
        self.persistent = persistent;
        self
    }

    /// Create a [Statement] from a [crate::DatabaseBackend] and a raw SQL statement
    pub fn from_string<T>(db_backend: DbBackend, stmt: T) -> Statement
    where
        T: Into<String>,
    {
        Statement {
            sql: stmt.into(),
            values: None,
            db_backend,
            persistent: true,
        }
    }

    /// Create a SQL statement from a [crate::DatabaseBackend], a
    /// raw SQL statement and param values
    pub fn from_sql_and_values<I, T>(db_backend: DbBackend, sql: T, values: I) -> Self
    where
        I: IntoIterator<Item = Value>,
        T: Into<String>,
    {
        Self::from_string_values_tuple(db_backend, (sql, Values(values.into_iter().collect())))
    }

    pub(crate) fn from_string_values_tuple<T>(db_backend: DbBackend, stmt: (T, Values)) -> Statement
    where
        T: Into<String>,
    {
        Statement {
            sql: stmt.0.into(),
            values: Some(stmt.1),
            db_backend,
            persistent: true,
        }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.values {
            Some(values) => {
                let string = match self.db_backend {
                    DbBackend::MySql => inject_parameters(&self.sql, &values.0, &MysqlQueryBuilder),
                    DbBackend::Postgres => {
                        inject_parameters(&self.sql, &values.0, &PostgresQueryBuilder)
                    }
                    DbBackend::Sqlite => {
                        inject_parameters(&self.sql, &values.0, &SqliteQueryBuilder)
                    }
                };
                write!(f, "{string}")
            }
            None => {
                write!(f, "{}", self.sql)
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

macro_rules! build_postgres_stmt {
    ($stmt: expr, $db_backend: expr) => {
        match $db_backend {
            DbBackend::Postgres => $stmt.to_string(PostgresQueryBuilder),
            DbBackend::MySql | DbBackend::Sqlite => unimplemented!(),
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

            #[cfg(feature = "rbac")]
            fn audit(&self) -> Result<QueryAccessAudit, AuditError> {
                AuditTrait::audit(self)
            }
        }
    };
}

build_query_stmt!(sea_query::InsertStatement);
build_query_stmt!(sea_query::SelectStatement);
build_query_stmt!(sea_query::UpdateStatement);
build_query_stmt!(sea_query::DeleteStatement);
build_query_stmt!(sea_query::WithQuery);

macro_rules! build_schema_stmt {
    ($stmt: ty) => {
        impl StatementBuilder for $stmt {
            fn build(&self, db_backend: &DbBackend) -> Statement {
                let stmt = build_any_stmt!(self, db_backend);
                Statement::from_string(*db_backend, stmt)
            }

            #[cfg(feature = "rbac")]
            fn audit(&self) -> Result<QueryAccessAudit, AuditError> {
                todo!("Audit not supported for {} yet", stringify!($stmt))
            }
        }
    };
}

build_schema_stmt!(sea_query::TableCreateStatement);
build_schema_stmt!(sea_query::TableDropStatement);
build_schema_stmt!(sea_query::TableAlterStatement);
build_schema_stmt!(sea_query::TableRenameStatement);
build_schema_stmt!(sea_query::TableTruncateStatement);
build_schema_stmt!(sea_query::IndexCreateStatement);
build_schema_stmt!(sea_query::IndexDropStatement);
build_schema_stmt!(sea_query::ForeignKeyCreateStatement);
build_schema_stmt!(sea_query::ForeignKeyDropStatement);

macro_rules! build_type_stmt {
    ($stmt: ty) => {
        impl StatementBuilder for $stmt {
            fn build(&self, db_backend: &DbBackend) -> Statement {
                let stmt = build_postgres_stmt!(self, db_backend);
                Statement::from_string(*db_backend, stmt)
            }

            #[cfg(feature = "rbac")]
            fn audit(&self) -> Result<QueryAccessAudit, AuditError> {
                Err(AuditError::UnsupportedQuery)
            }
        }
    };
}

build_type_stmt!(sea_query::extension::postgres::TypeAlterStatement);
build_type_stmt!(sea_query::extension::postgres::TypeCreateStatement);
build_type_stmt!(sea_query::extension::postgres::TypeDropStatement);
