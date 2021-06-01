use crate::Statement;
use sea_query::{QueryBuilder, QueryStatementBuilder};

pub trait QueryTrait {
    type QueryStatement: QueryStatementBuilder;

    /// Get a mutable ref to the query builder
    fn query(&mut self) -> &mut Self::QueryStatement;

    /// Get an immutable ref to the query builder
    fn as_query(&self) -> &Self::QueryStatement;

    /// Take ownership of the query builder
    fn into_query(self) -> Self::QueryStatement;

    /// Build the query as [`Statement`]
    fn build<B>(&self, builder: B) -> Statement
    where
        B: QueryBuilder,
    {
        self.as_query().build(builder).into()
    }
}
