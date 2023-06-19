use crate::{
    ActiveModelTrait, ColumnTrait, DbBackend, IntoActiveModel, IntoIdentity, IntoSimpleExpr,
    QuerySelect, Statement,
};
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

    /// Apply an operation on the [QueryTrait::QueryStatement] if the given `Option<T>` is `Some(_)`
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

/// Select specific column for partial model queries
pub trait SelectColumns {
    /// Add a select column
    ///
    /// For more detail, please visit [QuerySelect::column]
    fn select_column<C: ColumnTrait>(self, col: C) -> Self;

    /// Add a select column with alias
    ///
    /// For more detail, please visit [QuerySelect::column_as]
    fn select_column_as<C, I>(self, col: C, alias: I) -> Self
    where
        C: IntoSimpleExpr,
        I: IntoIdentity;
}

impl<S> SelectColumns for S
where
    S: QuerySelect,
{
    fn select_column<C: ColumnTrait>(self, col: C) -> Self {
        QuerySelect::column(self, col)
    }

    fn select_column_as<C, I>(self, col: C, alias: I) -> Self
    where
        C: IntoSimpleExpr,
        I: IntoIdentity,
    {
        QuerySelect::column_as(self, col, alias)
    }
}

/// Insert query Trait
pub trait InsertTrait<A>: Sized
where
    A: ActiveModelTrait,
{
    /// required function for new self
    fn new() -> Self;

    /// required function for add value
    fn add<M>(self, m: M) -> Self
    where
        M: IntoActiveModel<A>;

    /// Insert one Model or ActiveModel
    ///
    /// Model
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     Insert::one(cake::Model {
    ///         id: 1,
    ///         name: "Apple Pie".to_owned(),
    ///     })
    ///     .build(DbBackend::Postgres)
    ///     .to_string(),
    ///     r#"INSERT INTO "cake" ("id", "name") VALUES (1, 'Apple Pie')"#,
    /// );
    /// ```
    /// ActiveModel
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     Insert::one(cake::ActiveModel {
    ///         id: NotSet,
    ///         name: Set("Apple Pie".to_owned()),
    ///     })
    ///     .build(DbBackend::Postgres)
    ///     .to_string(),
    ///     r#"INSERT INTO "cake" ("name") VALUES ('Apple Pie')"#,
    /// );
    /// ```
    fn one<M>(m: M) -> Self
    where
        M: IntoActiveModel<A>,
    {
        Self::new().add(m)
    }

    /// Insert many Model or ActiveModel
    ///
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     Insert::many([
    ///         cake::Model {
    ///             id: 1,
    ///             name: "Apple Pie".to_owned(),
    ///         },
    ///         cake::Model {
    ///             id: 2,
    ///             name: "Orange Scone".to_owned(),
    ///         }
    ///     ])
    ///     .build(DbBackend::Postgres)
    ///     .to_string(),
    ///     r#"INSERT INTO "cake" ("id", "name") VALUES (1, 'Apple Pie'), (2, 'Orange Scone')"#,
    /// );
    /// ```
    fn many<M, I>(models: I) -> Self
    where
        M: IntoActiveModel<A>,
        I: IntoIterator<Item = M>,
    {
        Self::new().add_many(models)
    }

    /// Add many Models to Self
    fn add_many<M, I>(mut self, models: I) -> Self
    where
        M: IntoActiveModel<A>,
        I: IntoIterator<Item = M>,
    {
        for model in models.into_iter() {
            self = self.add(model);
        }
        self
    }
}
