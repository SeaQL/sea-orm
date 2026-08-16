use crate::{DbBackend, Statement, StatementBuilder};

/// Common operations on a SeaORM query builder: borrow the underlying
/// `sea_query` statement, build it into a backend-specific [`Statement`], or
/// apply optional modifications via [`apply_if`](Self::apply_if).
///
/// Implemented by [`Select`](crate::Select), [`Insert`](crate::Insert),
/// [`Update`](crate::Update), [`Delete`](crate::Delete), and their multi-row
/// variants.
pub trait QueryTrait {
    /// The underlying `sea_query` statement type this builder produces.
    type QueryStatement: StatementBuilder;

    /// Mutable access to the underlying statement.
    fn query(&mut self) -> &mut Self::QueryStatement;

    /// Shared access to the underlying statement.
    fn as_query(&self) -> &Self::QueryStatement;

    /// Consume the builder and return the underlying statement.
    fn into_query(self) -> Self::QueryStatement;

    /// Render the query for `db_backend` as a [`Statement`] (SQL + bound
    /// parameters). Useful for inspecting generated SQL in tests.
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
