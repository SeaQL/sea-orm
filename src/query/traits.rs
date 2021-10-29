use crate::{DbBackend, Statement};
use sea_query::QueryStatementBuilder;

/// Enforces a set of constraints to any type performing queries on a Model or ActiveModel
pub trait QueryTrait {
    /// Constrain the QueryStatement to [QueryStatementBuilder] trait
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
