use crate::{
    DbBackend, DbErr, ExecResult, QueryResult, Statement, StatementBuilder, TransactionError,
};

/// The generic API for a database connection that can perform query or execute statements.
/// It abstracts database connection and transaction
pub trait ConnectionTrait {
    /// Get the database backend for the connection. This depends on feature flags enabled.
    fn get_database_backend(&self) -> DbBackend;

    /// Execute a [Statement]
    fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr>;

    /// Execute a [QueryStatement]
    fn execute<S: StatementBuilder>(&self, stmt: &S) -> Result<ExecResult, DbErr> {
        let db_backend = self.get_database_backend();
        let stmt = db_backend.build(stmt);
        self.execute_raw(stmt)
    }

    /// Execute a unprepared [Statement]
    fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr>;

    /// Execute a [Statement] and return a single row of `QueryResult`
    fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr>;

    /// Execute a [QueryStatement] and return a single row of `QueryResult`
    fn query_one<S: StatementBuilder>(&self, stmt: &S) -> Result<Option<QueryResult>, DbErr> {
        let db_backend = self.get_database_backend();
        let stmt = db_backend.build(stmt);
        self.query_one_raw(stmt)
    }

    /// Execute a [Statement] and return a vector of `QueryResult`
    fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr>;

    /// Execute a [QueryStatement] and return a vector of `QueryResult`
    fn query_all<S: StatementBuilder>(&self, stmt: &S) -> Result<Vec<QueryResult>, DbErr> {
        let db_backend = self.get_database_backend();
        let stmt = db_backend.build(stmt);
        self.query_all_raw(stmt)
    }

    /// Check if the connection supports `RETURNING` syntax on insert and update
    fn support_returning(&self) -> bool {
        let db_backend = self.get_database_backend();
        db_backend.support_returning()
    }

    /// Check if the connection is a test connection for the Mock database
    fn is_mock_connection(&self) -> bool {
        false
    }
}

/// Stream query results
pub trait StreamTrait {
    /// Create a stream for the [QueryResult]
    type Stream<'a>: Iterator<Item = Result<QueryResult, DbErr>>
    where
        Self: 'a;

    /// Get the database backend for the connection. This depends on feature flags enabled.
    fn get_database_backend(&self) -> DbBackend;

    /// Execute a [Statement] and return a stream of results
    fn stream_raw<'a>(&'a self, stmt: Statement) -> Result<Self::Stream<'a>, DbErr>;

    /// Execute a [QueryStatement] and return a stream of results
    fn stream<'a, S: StatementBuilder>(&'a self, stmt: &S) -> Result<Self::Stream<'a>, DbErr> {
        let db_backend = self.get_database_backend();
        let stmt = db_backend.build(stmt);
        self.stream_raw(stmt)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// Isolation level
pub enum IsolationLevel {
    /// Consistent reads within the same transaction read the snapshot established by the first read.
    RepeatableRead,
    /// Each consistent read, even within the same transaction, sets and reads its own fresh snapshot.
    ReadCommitted,
    /// SELECT statements are performed in a nonlocking fashion, but a possible earlier version of a row might be used.
    ReadUncommitted,
    /// All statements of the current transaction can only see rows committed before the first query or data-modification statement was executed in this transaction.
    Serializable,
}

impl std::fmt::Display for IsolationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IsolationLevel::RepeatableRead => write!(f, "REPEATABLE READ"),
            IsolationLevel::ReadCommitted => write!(f, "READ COMMITTED"),
            IsolationLevel::ReadUncommitted => write!(f, "READ UNCOMMITTED"),
            IsolationLevel::Serializable => write!(f, "SERIALIZABLE"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// Access mode
pub enum AccessMode {
    /// Data can't be modified in this transaction
    ReadOnly,
    /// Data can be modified in this transaction (default)
    ReadWrite,
}

impl std::fmt::Display for AccessMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessMode::ReadOnly => write!(f, "READ ONLY"),
            AccessMode::ReadWrite => write!(f, "READ WRITE"),
        }
    }
}

/// Spawn database transaction
pub trait TransactionTrait {
    /// The concrete type for the transaction
    type Transaction: ConnectionTrait + TransactionTrait + TransactionSession;

    /// Execute SQL `BEGIN` transaction.
    /// Returns a Transaction that can be committed or rolled back
    fn begin(&self) -> Result<Self::Transaction, DbErr>;

    /// Execute SQL `BEGIN` transaction with isolation level and/or access mode.
    /// Returns a Transaction that can be committed or rolled back
    fn begin_with_config(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<Self::Transaction, DbErr>;

    /// Execute the function inside a transaction.
    /// If the function returns an error, the transaction will be rolled back. If it does not return an error, the transaction will be committed.
    fn transaction<F, T, E>(&self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(&'c Self::Transaction) -> Result<T, E>,
        E: std::fmt::Display + std::fmt::Debug;

    /// Execute the function inside a transaction with isolation level and/or access mode.
    /// If the function returns an error, the transaction will be rolled back. If it does not return an error, the transaction will be committed.
    fn transaction_with_config<F, T, E>(
        &self,
        callback: F,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(&'c Self::Transaction) -> Result<T, E>,
        E: std::fmt::Display + std::fmt::Debug;
}

/// Represents an open transaction
pub trait TransactionSession {
    /// Commit a transaction
    fn commit(self) -> Result<(), DbErr>;

    /// Rolls back a transaction explicitly
    fn rollback(self) -> Result<(), DbErr>;
}
