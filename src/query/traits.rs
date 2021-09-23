use crate::{DatabaseConnection, DbBackend, Statement};
use sea_query::QueryStatementBuilder;

pub trait QueryTrait {
    type QueryStatement: QueryStatementBuilder;

    /// Get a mutable ref to the query builder
    fn query(&mut self) -> &mut Self::QueryStatement;

    /// Get an immutable ref to the query builder
    fn as_query(&self) -> &Self::QueryStatement;

    /// Take ownership of the query builder
    fn into_query(self) -> Self::QueryStatement;

    /// Build the query as [`Statement`]
    fn build(&self, db_backend: DbBackend) -> Statement {
        let query_builder = db_backend.get_query_builder();
        Statement::from_string_values_tuple(
            db_backend,
            self.as_query().build_any(query_builder.as_ref()),
        )
    }
}

/// Make get raw_sql becomes simply. It does not need to specify a specific `DbBackend`,
/// but can be obtained through `get_database_backend` with `DatabaseConnection`.
/// Return a Statement type.
pub fn debug_query(query: &impl QueryTrait, conn: &DatabaseConnection) -> Statement {
    query.build(conn.get_database_backend())
}

/// Use `debug_query` get raw_sql.
pub fn debug_query_fmt(query: &impl QueryTrait, conn: &DatabaseConnection) -> String {
    debug_query(query, conn).to_string()
}
