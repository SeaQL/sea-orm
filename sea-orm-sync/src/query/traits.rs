use crate::{DbBackend, Statement, StatementBuilder};

/// A Trait for any type performing queries on a Model or ActiveModel
pub trait QueryTrait {
    /// Constrain the QueryStatement to [StatementBuilder] trait
    type QueryStatement: StatementBuilder;

    /// Get a mutable ref to the query builder
    fn query(&mut self) -> &mut Self::QueryStatement;

    /// Get an immutable ref to the query builder
    fn as_query(&self) -> &Self::QueryStatement;

    /// Take ownership of the query builder
    fn into_query(self) -> Self::QueryStatement;

    /// Build the query as [`Statement`]
    fn build(&self, db_backend: DbBackend) -> Statement {
        StatementBuilder::build(self.as_query(), &db_backend)
    }

    /// Apply an operation on the [QueryTrait::QueryStatement] if the given `Option<T>` is `Some(_)`
    ///
    /// # Example
    ///
    /// ```
    /// use sea_orm::{DbBackend, entity::*, query::*, tests_cfg::cake};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .apply_if(Some(3), |query, v| { query.filter(cake::Column::Id.eq(v)) })
    ///         .apply_if(Some(100), QuerySelect::limit)
    ///         .apply_if(None, QuerySelect::offset::<Option<u64>>) // no-op
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
