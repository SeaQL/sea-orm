use log::LevelFilter;
use std::{
    ops::Deref,
    sync::{Arc, Mutex, MutexGuard, TryLockError},
    time::{Duration, Instant},
};
use tracing::{instrument, warn};

pub use OwnedRow as RusqliteRow;
pub use rusqlite::{Connection as RusqliteConnection, Error as RusqliteError};
use rusqlite::{
    Row, Rows,
    types::{FromSql, FromSqlError, Value},
};
use sea_query_rusqlite::{RusqliteValue, RusqliteValues, rusqlite};

use crate::{
    AccessMode, ColIdx, ConnectOptions, DatabaseConnection, DatabaseConnectionType,
    DatabaseTransaction, IsolationLevel, QueryStream, Statement, TransactionError, debug_print,
    error::*, executor::*,
};

/// A helper class to connect to Rusqlite
#[derive(Debug)]
pub struct RusqliteConnector;

const DEFAULT_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(60);

/// Defines a SQLite connection
#[derive(Clone)]
pub struct RusqliteSharedConnection {
    pub(crate) conn: Arc<Mutex<State>>,
    acquire_timeout: Duration,
    metric_callback: Option<crate::metric::Callback>,
}

#[derive(Debug)]
pub struct RusqliteExecResult {
    pub(crate) rows_affected: u64,
    pub(crate) last_insert_rowid: i64,
}

#[derive(Debug)]
pub struct OwnedRow {
    pub columns: Vec<Arc<str>>,
    pub values: Vec<Value>,
}

#[derive(Debug, Default)]
pub enum State {
    Idle(RusqliteConnection),
    Loaned,
    #[default]
    Disconnected,
}

impl OwnedRow {
    pub fn columns(&self) -> &[Arc<str>] {
        &self.columns
    }

    pub fn from_row(columns: Vec<Arc<str>>, row: &Row) -> OwnedRow {
        let mut values = Vec::new();

        for i in 0..columns.len() {
            let v: Value = row.get_unwrap(i);
            values.push(v);
        }

        OwnedRow { columns, values }
    }

    pub fn try_get<T: FromSql, I: ColIdx>(&self, idx: I) -> Result<T, TryGetError> {
        let (idx, col, value) = if let Some(idx) = idx.as_usize() {
            (*idx, None, &self.values[*idx])
        } else if let Some(name) = idx.as_str() {
            if let Some(idx) = self.columns.iter().position(|c| c.deref() == name) {
                (idx, Some(name), &self.values[idx])
            } else {
                return Err(TryGetError::Null(format!(
                    "column `{name}` does not exist in row"
                )));
            }
        } else {
            unreachable!("ColIdx must be either usize or str")
        };
        FromSql::column_result(value.into())
            .map_err(|err| match err {
                FromSqlError::OutOfRange(i) => RusqliteError::IntegralValueOutOfRange(idx, i),
                FromSqlError::Other(err) => {
                    RusqliteError::FromSqlConversionFailure(idx, value.data_type(), err)
                }
                FromSqlError::InvalidBlobSize { .. } => {
                    RusqliteError::FromSqlConversionFailure(idx, value.data_type(), Box::new(err))
                }
                // FromSqlError::InvalidType
                _ => RusqliteError::InvalidColumnType(
                    idx,
                    col.map(|c| c.to_owned()).unwrap_or_default(),
                    value.data_type(),
                ),
            })
            .map_err(|err| TryGetError::DbErr(query_err(err)))
    }
}

impl std::fmt::Debug for RusqliteSharedConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RusqliteSharedConnection {{ conn: {:?} }}", self.conn)
    }
}

impl From<RusqliteConnection> for RusqliteSharedConnection {
    fn from(conn: RusqliteConnection) -> Self {
        RusqliteSharedConnection {
            conn: Arc::new(Mutex::new(State::Idle(conn))),
            acquire_timeout: DEFAULT_ACQUIRE_TIMEOUT,
            metric_callback: None,
        }
    }
}

impl From<RusqliteSharedConnection> for DatabaseConnection {
    fn from(conn: RusqliteSharedConnection) -> Self {
        DatabaseConnectionType::RusqliteSharedConnection(conn).into()
    }
}

impl RusqliteConnector {
    /// Check if the URI provided corresponds to `sqlite:` for a SQLite database
    pub fn accepts(string: &str) -> bool {
        string.starts_with("sqlite:")
    }

    /// Add configuration options for the SQLite database
    #[instrument(level = "trace")]
    pub fn connect(options: ConnectOptions) -> Result<DatabaseConnection, DbErr> {
        let acquire_timeout = options.acquire_timeout.unwrap_or(DEFAULT_ACQUIRE_TIMEOUT);
        // TODO handle disable_statement_logging
        let after_conn = options.after_connect;

        let conn = RusqliteConnection::open(
            options
                .url
                .trim_start_matches("sqlite://")
                .trim_start_matches("sqlite:"),
        )
        .map_err(conn_err)?;

        let conn = RusqliteSharedConnection {
            conn: Arc::new(Mutex::new(State::Idle(conn))),
            acquire_timeout,
            metric_callback: None,
        };

        #[cfg(feature = "sqlite-use-returning-for-3_35")]
        {
            let version = get_version(&conn)?;
            super::sqlite::ensure_returning_version(&version)?;
        }

        let conn: DatabaseConnection = conn.into();

        if let Some(cb) = after_conn {
            cb(conn.clone())?;
        }

        Ok(conn)
    }
}

impl RusqliteConnector {
    /// Convert a Rusqlite connection to a [DatabaseConnection]
    pub fn from_rusqlite_connection(conn: RusqliteConnection) -> DatabaseConnection {
        let conn: RusqliteSharedConnection = conn.into();
        conn.into()
    }
}

impl RusqliteSharedConnection {
    pub fn acquire(&self) -> Result<MutexGuard<'_, State>, DbErr> {
        let deadline = Instant::now() + self.acquire_timeout;
        loop {
            match self.conn.try_lock() {
                Ok(state) => match *state {
                    State::Idle(_) => return Ok(state),
                    State::Loaned => (), // transaction in progress, wait
                    State::Disconnected => {
                        return Err(DbErr::ConnectionAcquire(ConnAcquireErr::ConnectionClosed));
                    }
                },
                Err(TryLockError::WouldBlock) => (),
                Err(TryLockError::Poisoned(_)) => {
                    return Err(DbErr::ConnectionAcquire(ConnAcquireErr::ConnectionClosed));
                }
            }
            if Instant::now() >= deadline {
                return Err(DbErr::ConnectionAcquire(ConnAcquireErr::Timeout));
            }
            std::thread::yield_now();
        }
    }

    /// Execute a [Statement] on a SQLite backend
    #[instrument(level = "trace")]
    pub fn execute(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        debug_print!("{}", stmt);

        let values = sql_values(&stmt);
        let conn = self.acquire()?;
        let conn = conn.conn();
        crate::metric::metric!(self.metric_callback, &stmt, {
            match conn.execute(&stmt.sql, &*values.as_params()) {
                Ok(rows_affected) => Ok(RusqliteExecResult {
                    rows_affected: rows_affected as u64,
                    last_insert_rowid: conn.last_insert_rowid(),
                }
                .into()),
                Err(err) => Err(exec_err(err)),
            }
        })
    }

    /// Execute an unprepared SQL statement on a SQLite backend
    #[instrument(level = "trace")]
    pub fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        debug_print!("{}", sql);

        let conn = self.acquire()?;
        let conn = conn.conn();
        match conn.execute_batch(sql) {
            Ok(()) => Ok(RusqliteExecResult {
                rows_affected: conn.changes(),
                last_insert_rowid: conn.last_insert_rowid(),
            }
            .into()),
            Err(err) => Err(exec_err(err)),
        }
    }

    /// Get one result from a SQL query. Returns [Option::None] if no match was found
    #[instrument(level = "trace")]
    pub fn query_one(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        debug_print!("{}", stmt);

        let values = sql_values(&stmt);
        let conn = self.acquire()?;
        let conn = conn.conn();
        let mut sql = conn.prepare_cached(&stmt.sql).map_err(query_err)?;
        let columns: Vec<Arc<str>> = sql
            .column_names()
            .into_iter()
            .map(|r| Arc::from(r))
            .collect();

        crate::metric::metric!(self.metric_callback, &stmt, {
            match sql.query(&*values.as_params()) {
                Ok(mut rows) => {
                    let mut out = None;
                    if let Some(row) = rows.next().map_err(query_err)? {
                        out = Some(OwnedRow::from_row(columns.clone(), row).into());
                    }
                    Ok(out)
                }
                Err(err) => Err(query_err(err)),
            }
        })
    }

    /// Get the results of a query returning them as a Vec<[QueryResult]>
    #[instrument(level = "trace")]
    pub fn query_all(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        debug_print!("{}", stmt);

        let values = sql_values(&stmt);
        let conn = self.acquire()?;
        let conn = conn.conn();
        let mut sql = conn.prepare_cached(&stmt.sql).map_err(query_err)?;
        let columns: Vec<Arc<str>> = sql
            .column_names()
            .into_iter()
            .map(|r| Arc::from(r))
            .collect();

        crate::metric::metric!(self.metric_callback, &stmt, {
            match sql.query(&*values.as_params()) {
                Ok(mut rows) => {
                    let mut out = Vec::new();
                    while let Some(row) = rows.next().map_err(query_err)? {
                        out.push(OwnedRow::from_row(columns.clone(), row).into());
                    }
                    Ok(out)
                }
                Err(err) => Err(query_err(err)),
            }
        })
    }

    /// Stream the results of executing a SQL query
    #[instrument(level = "trace")]
    pub fn stream(&self, stmt: Statement) -> Result<QueryStream, DbErr> {
        debug_print!("{}", stmt);

        todo!()
        // let conn = self.acquire()?;
        // let conn = conn.conn();
        // Ok(QueryStream::from((
        //     conn,
        //     stmt,
        //     self.metric_callback.clone(),
        // )))
    }

    /// Bundle a set of SQL statements that execute together.
    #[instrument(level = "trace")]
    pub fn begin(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<DatabaseTransaction, DbErr> {
        todo!()
        // let conn = self.acquire()?;
        // let conn = conn.conn();
        // DatabaseTransaction::new_sqlite(
        //     conn,
        //     self.metric_callback.clone(),
        //     isolation_level,
        //     access_mode,
        // )
    }

    /// Create a SQLite transaction
    #[instrument(level = "trace", skip(callback))]
    pub fn transaction<F, T, E>(
        &self,
        callback: F,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<T, TransactionError<E>>
    where
        F: for<'b> FnOnce(&'b DatabaseTransaction) -> Result<T, E>,
        E: std::fmt::Display + std::fmt::Debug,
    {
        todo!()
        // let conn = self.acquire()?;
        // let conn = conn.conn();
        // let transaction = DatabaseTransaction::new_sqlite(
        //     conn,
        //     self.metric_callback.clone(),
        //     isolation_level,
        //     access_mode,
        // )
        // .map_err(|e| TransactionError::Connection(e))?;
        // transaction.run(callback)
    }

    pub(crate) fn set_metric_callback<F>(&mut self, callback: F)
    where
        F: Fn(&crate::metric::Info<'_>) + 'static,
    {
        self.metric_callback = Some(Arc::new(callback));
    }

    /// Checks if a connection to the database is still valid.
    pub fn ping(&self) -> Result<(), DbErr> {
        let conn = self.acquire()?;
        let conn = conn.conn();

        match conn.execute("SELECT 1", ()) {
            Ok(_) => Ok(()),
            Err(err) => Err(conn_err(err)),
        }
    }

    /// Explicitly close the SQLite connection.
    /// See [`Self::close_by_ref`] for usage with references.
    pub fn close(self) -> Result<(), DbErr> {
        self.close_by_ref()
    }

    /// Explicitly close the SQLite connection
    pub fn close_by_ref(&self) -> Result<(), DbErr> {
        let mut conn = self.acquire()?;
        *conn = State::Disconnected;
        Ok(())
    }
}

impl State {
    fn conn(&self) -> &RusqliteConnection {
        match self {
            State::Idle(conn) => conn,
            _ => panic!("No connection"),
        }
    }
}

impl From<OwnedRow> for QueryResult {
    fn from(row: OwnedRow) -> QueryResult {
        QueryResult {
            row: QueryResultRow::Rusqlite(row),
        }
    }
}

impl From<RusqliteExecResult> for ExecResult {
    fn from(result: RusqliteExecResult) -> ExecResult {
        ExecResult {
            result: ExecResultHolder::Rusqlite(result),
        }
    }
}

pub(crate) fn sql_values(stmt: &Statement) -> RusqliteValues {
    let values = match &stmt.values {
        Some(values) => values.iter().cloned().map(RusqliteValue).collect(),
        None => Vec::new(),
    };
    RusqliteValues(values)
}

pub(crate) fn set_transaction_config(
    _conn: RusqliteSharedConnection,
    isolation_level: Option<IsolationLevel>,
    access_mode: Option<AccessMode>,
) -> Result<(), DbErr> {
    if isolation_level.is_some() {
        warn!("Setting isolation level in a SQLite transaction isn't supported");
    }
    if access_mode.is_some() {
        warn!("Setting access mode in a SQLite transaction isn't supported");
    }
    Ok(())
}

#[cfg(feature = "sqlite-use-returning-for-3_35")]
fn get_version(conn: &RusqliteSharedConnection) -> Result<String, DbErr> {
    let stmt = Statement {
        sql: "SELECT sqlite_version()".to_string(),
        values: None,
        db_backend: crate::DbBackend::Sqlite,
    };
    conn.query_one(stmt)?
        .ok_or_else(|| {
            DbErr::Conn(RuntimeErr::Internal(
                "Error reading SQLite version".to_string(),
            ))
        })?
        .try_get_by(0)
}

fn conn_err(err: RusqliteError) -> DbErr {
    DbErr::Conn(RuntimeErr::Rusqlite(err.into()))
}

fn exec_err(err: RusqliteError) -> DbErr {
    DbErr::Exec(RuntimeErr::Rusqlite(err.into()))
}

fn query_err(err: RusqliteError) -> DbErr {
    DbErr::Query(RuntimeErr::Rusqlite(err.into()))
}

// impl
//     From<(
//         PoolConnection<sqlx::Sqlite>,
//         Statement,
//         Option<crate::metric::Callback>,
//     )> for crate::QueryStream
// {
//     fn from(
//         (conn, stmt, metric_callback): (
//             PoolConnection<sqlx::Sqlite>,
//             Statement,
//             Option<crate::metric::Callback>,
//         ),
//     ) -> Self {
//         crate::QueryStream::build(stmt, crate::InnerConnection::Sqlite(conn), metric_callback)
//     }
// }

// impl crate::DatabaseTransaction {
//     pub(crate) fn new_sqlite(
//         inner: PoolConnection<sqlx::Sqlite>,
//         metric_callback: Option<crate::metric::Callback>,
//         isolation_level: Option<IsolationLevel>,
//         access_mode: Option<AccessMode>,
//     ) -> Result<crate::DatabaseTransaction, DbErr> {
//         Self::begin(
//             Arc::new(Mutex::new(crate::InnerConnection::Sqlite(inner))),
//             crate::DbBackend::Sqlite,
//             metric_callback,
//             isolation_level,
//             access_mode,
//         )
//     }
// }
