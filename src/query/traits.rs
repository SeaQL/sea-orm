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
    ///         .select_only()
    ///         // Select column
    ///         .maybe(cake::Column::Id, |mut query, column| {
    ///             if let Some(col) = column.into() {
    ///                 query = query.column_as(col.count(), "count");
    ///             }
    ///             query
    ///         })
    ///         // Limit result to the first 100 rows
    ///         .maybe(Some(100), |mut query, limit| {
    ///             if let Some(n) = limit.into() {
    ///                 query = query.limit(n);
    ///             }
    ///             query
    ///         })
    ///         // Do nothing
    ///         .maybe(None, |mut query, offset| {
    ///             if let Some(n) = offset.into() {
    ///                 query = query.offset(n);
    ///             }
    ///             query
    ///         })
    ///         .build(DbBackend::Postgres)
    ///         .to_string(),
    ///     r#"SELECT COUNT("cake"."id") AS "count" FROM "cake" LIMIT 100"#
    /// );
    /// ```
    fn maybe<T, F>(self, val: T, if_some: F) -> Self
    where
        Self: Sized,
        T: Into<Option<T>>,
        F: FnOnce(Self, T) -> Self,
    {
        if_some(self, val)
    }
}
