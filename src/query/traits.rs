use crate::{Statement, Syntax};
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
    fn build(&self, syntax: Syntax) -> Statement {
        let query_builder = syntax.get_query_builder();
        Statement::from_string_values_tuple(
            syntax,
            self.as_query().build_any(query_builder.as_ref()),
        )
    }
}

pub trait QueryBuilderWithSyntax: QueryBuilder {
    fn syntax(&self) -> Syntax;
}
