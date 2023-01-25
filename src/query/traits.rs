use crate::{DbBackend, Statement};
use sea_query::QueryStatementBuilder;

/// A Trait for any type performing queries on a Model or ActiveModel
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

    /// Perform some operations on the [QueryTrait::QueryStatement] with the given `Option<T>` value
    ///
    /// # Example
    ///
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .apply_if(Some(3), |mut query, v| {
    ///             query.filter(cake::Column::Id.eq(v))
    ///         })
    ///         .apply_if(Some(100), QuerySelect::limit)
    ///         .apply_if(None, QuerySelect::offset) // no-op
    ///         .build(DbBackend::Postgres)
    ///         .to_string(),
    ///     r#"SELECT "cake"."id", "cake"."name" FROM "cake" WHERE "cake"."id" = 3 LIMIT 100"#
    /// );
    /// ```
    fn apply_if<T, F>(self, val: Option<T>, if_some: F) -> Self
    where
        Self: Sized,
        F: FnOnce(Self, T) -> Self,
    {
        if let Some(val) = val {
            if_some(self, val)
        } else {
            self
        }
    }
}
