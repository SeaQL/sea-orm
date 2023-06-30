use crate::{
    error::*, ConnectionTrait, DbBackend, EntityTrait, FromQueryResult, IdenStatic, Iterable,
    ModelTrait, PartialModelTrait, PrimaryKeyToColumn, QueryResult, QuerySelect, Select, SelectA,
    SelectB, SelectTwo, SelectTwoMany, Statement, StreamTrait, TryGetableMany,
};
use futures::{Stream, TryStreamExt};
use sea_query::SelectStatement;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::pin::Pin;

#[cfg(feature = "with-json")]
use crate::JsonValue;

/// Defines a type to do `SELECT` operations through a [SelectStatement] on a Model
#[derive(Clone, Debug)]
pub struct Selector<S>
where
    S: SelectorTrait,
{
    pub(crate) query: SelectStatement,
    selector: S,
}

/// Performs a raw `SELECT` operation on a model
#[derive(Clone, Debug)]
pub struct SelectorRaw<S>
where
    S: SelectorTrait,
{
    pub(crate) stmt: Statement,
    #[allow(dead_code)]
    selector: S,
}

/// A Trait for any type that can perform SELECT queries
pub trait SelectorTrait {
    #[allow(missing_docs)]
    type Item: Sized;

    /// The method to perform a query on a Model
    fn from_raw_query_result(res: QueryResult) -> Result<Self::Item, DbErr>;
}

/// Get tuple from query result based on a list of column identifiers
#[derive(Debug)]
pub struct SelectGetableValue<T, C>
where
    T: TryGetableMany,
    C: strum::IntoEnumIterator + sea_query::Iden,
{
    columns: PhantomData<C>,
    model: PhantomData<T>,
}

/// Get tuple from query result based on column index
#[derive(Debug)]
pub struct SelectGetableTuple<T>
where
    T: TryGetableMany,
{
    model: PhantomData<T>,
}

/// Defines a type to get a Model
#[derive(Debug)]
pub struct SelectModel<M>
where
    M: FromQueryResult,
{
    model: PhantomData<M>,
}

/// Defines a type to get two Models
#[derive(Clone, Debug)]
pub struct SelectTwoModel<M, N>
where
    M: FromQueryResult,
    N: FromQueryResult,
{
    model: PhantomData<(M, N)>,
}

impl<T, C> SelectorTrait for SelectGetableValue<T, C>
where
    T: TryGetableMany,
    C: strum::IntoEnumIterator + sea_query::Iden,
{
    type Item = T;

    fn from_raw_query_result(res: QueryResult) -> Result<Self::Item, DbErr> {
        let cols: Vec<String> = C::iter().map(|col| col.to_string()).collect();
        T::try_get_many(&res, "", &cols).map_err(Into::into)
    }
}

impl<T> SelectorTrait for SelectGetableTuple<T>
where
    T: TryGetableMany,
{
    type Item = T;

    fn from_raw_query_result(res: QueryResult) -> Result<Self::Item, DbErr> {
        T::try_get_many_by_index(&res).map_err(Into::into)
    }
}

impl<M> SelectorTrait for SelectModel<M>
where
    M: FromQueryResult + Sized,
{
    type Item = M;

    fn from_raw_query_result(res: QueryResult) -> Result<Self::Item, DbErr> {
        M::from_query_result(&res, "")
    }
}

impl<M, N> SelectorTrait for SelectTwoModel<M, N>
where
    M: FromQueryResult + Sized,
    N: FromQueryResult + Sized,
{
    type Item = (M, Option<N>);

    fn from_raw_query_result(res: QueryResult) -> Result<Self::Item, DbErr> {
        Ok((
            M::from_query_result(&res, SelectA.as_str())?,
            N::from_query_result_optional(&res, SelectB.as_str())?,
        ))
    }
}

impl<E> Select<E>
where
    E: EntityTrait,
{
    /// Perform a Select operation on a Model using a [Statement]
    #[allow(clippy::wrong_self_convention)]
    pub fn from_raw_sql(self, stmt: Statement) -> SelectorRaw<SelectModel<E::Model>> {
        SelectorRaw {
            stmt,
            selector: SelectModel { model: PhantomData },
        }
    }

    /// Return a [Selector] from `Self` that wraps a [SelectModel]
    pub fn into_model<M>(self) -> Selector<SelectModel<M>>
    where
        M: FromQueryResult,
    {
        Selector {
            query: self.query,
            selector: SelectModel { model: PhantomData },
        }
    }

    /// Return a [Selector] from `Self` that wraps a [SelectModel] with a [PartialModel](PartialModelTrait)
    ///
    /// ```
    /// # #[cfg(feature = "macros")]
    /// # {
    /// use sea_orm::{
    ///     entity::*,
    ///     query::*,
    ///     tests_cfg::cake::{self, Entity as Cake},
    ///     DbBackend, DerivePartialModel, FromQueryResult,
    /// };
    /// use sea_query::{Expr, Func, SimpleExpr};
    ///
    /// #[derive(DerivePartialModel, FromQueryResult)]
    /// #[sea_orm(entity = "Cake")]
    /// struct PartialCake {
    ///     name: String,
    ///     #[sea_orm(
    ///         from_expr = r#"SimpleExpr::FunctionCall(Func::upper(Expr::col((Cake, cake::Column::Name))))"#
    ///     )]
    ///     name_upper: String,
    /// }
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .into_partial_model::<PartialCake>()
    ///         .into_statement(DbBackend::Sqlite)
    ///         .to_string(),
    ///     r#"SELECT "cake"."name", UPPER("cake"."name") AS "name_upper" FROM "cake""#
    /// );
    /// # }
    /// ```
    pub fn into_partial_model<M>(self) -> Selector<SelectModel<M>>
    where
        M: PartialModelTrait,
    {
        M::select_cols(QuerySelect::select_only(self)).into_model::<M>()
    }

    /// Get a selectable Model as a [JsonValue] for SQL JSON operations
    #[cfg(feature = "with-json")]
    pub fn into_json(self) -> Selector<SelectModel<JsonValue>> {
        Selector {
            query: self.query,
            selector: SelectModel { model: PhantomData },
        }
    }

    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(all(feature = "mock", feature = "macros"))]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results([[
    /// #         maplit::btreemap! {
    /// #             "cake_name" => Into::<Value>::into("Chocolate Forest"),
    /// #         },
    /// #         maplit::btreemap! {
    /// #             "cake_name" => Into::<Value>::into("New York Cheese"),
    /// #         },
    /// #     ]])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DeriveColumn, EnumIter};
    ///
    /// #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
    /// enum QueryAs {
    ///     CakeName,
    /// }
    ///
    /// let res: Vec<String> = cake::Entity::find()
    ///     .select_only()
    ///     .column_as(cake::Column::Name, QueryAs::CakeName)
    ///     .into_values::<_, QueryAs>()
    ///     .all(&db)
    ///     .await?;
    ///
    /// assert_eq!(
    ///     res,
    ///     ["Chocolate Forest".to_owned(), "New York Cheese".to_owned()]
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     [Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"SELECT "cake"."name" AS "cake_name" FROM "cake""#,
    ///         []
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(all(feature = "mock", feature = "macros"))]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results([[
    /// #         maplit::btreemap! {
    /// #             "cake_name" => Into::<Value>::into("Chocolate Forest"),
    /// #             "num_of_cakes" => Into::<Value>::into(2i64),
    /// #         },
    /// #     ]])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DeriveColumn, EnumIter};
    ///
    /// #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
    /// enum QueryAs {
    ///     CakeName,
    ///     NumOfCakes,
    /// }
    ///
    /// let res: Vec<(String, i64)> = cake::Entity::find()
    ///     .select_only()
    ///     .column_as(cake::Column::Name, QueryAs::CakeName)
    ///     .column_as(cake::Column::Id.count(), QueryAs::NumOfCakes)
    ///     .group_by(cake::Column::Name)
    ///     .into_values::<_, QueryAs>()
    ///     .all(&db)
    ///     .await?;
    ///
    /// assert_eq!(res, [("Chocolate Forest".to_owned(), 2i64)]);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     [Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         [
    ///             r#"SELECT "cake"."name" AS "cake_name", COUNT("cake"."id") AS "num_of_cakes""#,
    ///             r#"FROM "cake" GROUP BY "cake"."name""#,
    ///         ]
    ///         .join(" ")
    ///         .as_str(),
    ///         []
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_values<T, C>(self) -> Selector<SelectGetableValue<T, C>>
    where
        T: TryGetableMany,
        C: strum::IntoEnumIterator + sea_query::Iden,
    {
        Selector::<SelectGetableValue<T, C>>::with_columns(self.query)
    }

    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(all(feature = "mock", feature = "macros"))]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results(vec![vec![
    /// #         maplit::btreemap! {
    /// #             "cake_name" => Into::<Value>::into("Chocolate Forest"),
    /// #         },
    /// #         maplit::btreemap! {
    /// #             "cake_name" => Into::<Value>::into("New York Cheese"),
    /// #         },
    /// #     ]])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// let res: Vec<String> = cake::Entity::find()
    ///     .select_only()
    ///     .column(cake::Column::Name)
    ///     .into_tuple()
    ///     .all(&db)
    ///     .await?;
    ///
    /// assert_eq!(
    ///     res,
    ///     vec!["Chocolate Forest".to_owned(), "New York Cheese".to_owned()]
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"SELECT "cake"."name" FROM "cake""#,
    ///         vec![]
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(all(feature = "mock", feature = "macros"))]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results(vec![vec![
    /// #         maplit::btreemap! {
    /// #             "cake_name" => Into::<Value>::into("Chocolate Forest"),
    /// #             "num_of_cakes" => Into::<Value>::into(2i64),
    /// #         },
    /// #     ]])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// let res: Vec<(String, i64)> = cake::Entity::find()
    ///     .select_only()
    ///     .column(cake::Column::Name)
    ///     .column(cake::Column::Id)
    ///     .group_by(cake::Column::Name)
    ///     .into_tuple()
    ///     .all(&db)
    ///     .await?;
    ///
    /// assert_eq!(res, vec![("Chocolate Forest".to_owned(), 2i64)]);
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         vec![
    ///             r#"SELECT "cake"."name", "cake"."id""#,
    ///             r#"FROM "cake" GROUP BY "cake"."name""#,
    ///         ]
    ///         .join(" ")
    ///         .as_str(),
    ///         vec![]
    ///     )]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_tuple<T>(self) -> Selector<SelectGetableTuple<T>>
    where
        T: TryGetableMany,
    {
        Selector::<SelectGetableTuple<T>>::into_tuple(self.query)
    }

    /// Get one Model from the SELECT query
    pub async fn one<'a, C>(self, db: &C) -> Result<Option<E::Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        self.into_model().one(db).await
    }

    /// Get all Models from the SELECT query
    pub async fn all<'a, C>(self, db: &C) -> Result<Vec<E::Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        self.into_model().all(db).await
    }

    /// Stream the results of a SELECT operation on a Model
    pub async fn stream<'a: 'b, 'b, C>(
        self,
        db: &'a C,
    ) -> Result<impl Stream<Item = Result<E::Model, DbErr>> + 'b + Send, DbErr>
    where
        C: ConnectionTrait + StreamTrait + Send,
    {
        self.into_model().stream(db).await
    }

    /// Stream the result of the operation with PartialModel
    pub async fn stream_partial_model<'a: 'b, 'b, C, M>(
        self,
        db: &'a C,
    ) -> Result<impl Stream<Item = Result<M, DbErr>> + 'b + Send, DbErr>
    where
        C: ConnectionTrait + StreamTrait + Send,
        M: PartialModelTrait + Send + 'b,
    {
        self.into_partial_model().stream(db).await
    }
}

impl<E, F> SelectTwo<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
    /// Perform a conversion into a [SelectTwoModel]
    pub fn into_model<M, N>(self) -> Selector<SelectTwoModel<M, N>>
    where
        M: FromQueryResult,
        N: FromQueryResult,
    {
        Selector {
            query: self.query,
            selector: SelectTwoModel { model: PhantomData },
        }
    }

    /// Perform a conversion into a [SelectTwoModel] with [PartialModel](PartialModelTrait)
    pub fn into_partial_model<M, N>(self) -> Selector<SelectTwoModel<M, N>>
    where
        M: PartialModelTrait,
        N: PartialModelTrait,
    {
        let select = QuerySelect::select_only(self);
        let select = M::select_cols(select);
        let select = N::select_cols(select);
        select.into_model::<M, N>()
    }

    /// Convert the Models into JsonValue
    #[cfg(feature = "with-json")]
    pub fn into_json(self) -> Selector<SelectTwoModel<JsonValue, JsonValue>> {
        Selector {
            query: self.query,
            selector: SelectTwoModel { model: PhantomData },
        }
    }

    /// Get one Model from the Select query
    pub async fn one<'a, C>(self, db: &C) -> Result<Option<(E::Model, Option<F::Model>)>, DbErr>
    where
        C: ConnectionTrait,
    {
        self.into_model().one(db).await
    }

    /// Get all Models from the Select query
    pub async fn all<'a, C>(self, db: &C) -> Result<Vec<(E::Model, Option<F::Model>)>, DbErr>
    where
        C: ConnectionTrait,
    {
        self.into_model().all(db).await
    }

    /// Stream the results of a Select operation on a Model
    pub async fn stream<'a: 'b, 'b, C>(
        self,
        db: &'a C,
    ) -> Result<impl Stream<Item = Result<(E::Model, Option<F::Model>), DbErr>> + 'b, DbErr>
    where
        C: ConnectionTrait + StreamTrait + Send,
    {
        self.into_model().stream(db).await
    }

    /// Stream the result of the operation with PartialModel
    pub async fn stream_partial_model<'a: 'b, 'b, C, M, N>(
        self,
        db: &'a C,
    ) -> Result<impl Stream<Item = Result<(M, Option<N>), DbErr>> + 'b + Send, DbErr>
    where
        C: ConnectionTrait + StreamTrait + Send,
        M: PartialModelTrait + Send + 'b,
        N: PartialModelTrait + Send + 'b,
    {
        self.into_partial_model().stream(db).await
    }
}

impl<E, F> SelectTwoMany<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
    /// Performs a conversion to [Selector]
    fn into_model<M, N>(self) -> Selector<SelectTwoModel<M, N>>
    where
        M: FromQueryResult,
        N: FromQueryResult,
    {
        Selector {
            query: self.query,
            selector: SelectTwoModel { model: PhantomData },
        }
    }

    /// Performs a conversion to [Selector] with partial model
    fn into_partial_model<M, N>(self) -> Selector<SelectTwoModel<M, N>>
    where
        M: PartialModelTrait,
        N: PartialModelTrait,
    {
        let select = self.select_only();
        let select = M::select_cols(select);
        let select = N::select_cols(select);
        select.into_model()
    }

    /// Convert the results to JSON
    #[cfg(feature = "with-json")]
    pub fn into_json(self) -> Selector<SelectTwoModel<JsonValue, JsonValue>> {
        Selector {
            query: self.query,
            selector: SelectTwoModel { model: PhantomData },
        }
    }

    /// Stream the result of the operation
    pub async fn stream<'a: 'b, 'b, C>(
        self,
        db: &'a C,
    ) -> Result<impl Stream<Item = Result<(E::Model, Option<F::Model>), DbErr>> + 'b + Send, DbErr>
    where
        C: ConnectionTrait + StreamTrait + Send,
    {
        self.into_model().stream(db).await
    }

    /// Stream the result of the operation with PartialModel
    pub async fn stream_partial_model<'a: 'b, 'b, C, M, N>(
        self,
        db: &'a C,
    ) -> Result<impl Stream<Item = Result<(M, Option<N>), DbErr>> + 'b + Send, DbErr>
    where
        C: ConnectionTrait + StreamTrait + Send,
        M: PartialModelTrait + Send + 'b,
        N: PartialModelTrait + Send + 'b,
    {
        self.into_partial_model().stream(db).await
    }

    /// Get all Models from the select operation
    ///
    /// > `SelectTwoMany::one()` method has been dropped (#486)
    /// >
    /// > You can get `(Entity, Vec<RelatedEntity>)` by first querying a single model from Entity,
    /// > then use [`ModelTrait::find_related`] on the model.
    /// >
    /// > See https://www.sea-ql.org/SeaORM/docs/basic-crud/select#lazy-loading for details.
    pub async fn all<'a, C>(self, db: &C) -> Result<Vec<(E::Model, Vec<F::Model>)>, DbErr>
    where
        C: ConnectionTrait,
    {
        let rows = self.into_model().all(db).await?;
        Ok(consolidate_query_result::<E, F>(rows))
    }

    // pub fn paginate()
    // we could not implement paginate easily, if the number of children for a
    // parent is larger than one page, then we will end up splitting it in two pages
    // so the correct way is actually perform query in two stages
    // paginate the parent model and then populate the children

    // pub fn count()
    // we should only count the number of items of the parent model
}

impl<S> Selector<S>
where
    S: SelectorTrait,
{
    /// Create `Selector` from Statement and columns. Executing this `Selector`
    /// will return a type `T` which implement `TryGetableMany`.
    pub fn with_columns<T, C>(query: SelectStatement) -> Selector<SelectGetableValue<T, C>>
    where
        T: TryGetableMany,
        C: strum::IntoEnumIterator + sea_query::Iden,
    {
        Selector {
            query,
            selector: SelectGetableValue {
                columns: PhantomData,
                model: PhantomData,
            },
        }
    }

    /// Get tuple from query result based on column index
    pub fn into_tuple<T>(query: SelectStatement) -> Selector<SelectGetableTuple<T>>
    where
        T: TryGetableMany,
    {
        Selector {
            query,
            selector: SelectGetableTuple { model: PhantomData },
        }
    }

    fn into_selector_raw<C>(self, db: &C) -> SelectorRaw<S>
    where
        C: ConnectionTrait,
    {
        let builder = db.get_database_backend();
        let stmt = builder.build(&self.query);
        SelectorRaw {
            stmt,
            selector: self.selector,
        }
    }

    /// Get the SQL statement
    pub fn into_statement(self, builder: DbBackend) -> Statement {
        builder.build(&self.query)
    }

    /// Get an item from the Select query
    pub async fn one<'a, C>(mut self, db: &C) -> Result<Option<S::Item>, DbErr>
    where
        C: ConnectionTrait,
    {
        self.query.limit(1);
        self.into_selector_raw(db).one(db).await
    }

    /// Get all items from the Select query
    pub async fn all<'a, C>(self, db: &C) -> Result<Vec<S::Item>, DbErr>
    where
        C: ConnectionTrait,
    {
        self.into_selector_raw(db).all(db).await
    }

    /// Stream the results of the Select operation
    pub async fn stream<'a: 'b, 'b, C>(
        self,
        db: &'a C,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<S::Item, DbErr>> + 'b + Send>>, DbErr>
    where
        C: ConnectionTrait + StreamTrait + Send,
        S: 'b,
        S::Item: Send,
    {
        self.into_selector_raw(db).stream(db).await
    }
}

impl<S> SelectorRaw<S>
where
    S: SelectorTrait,
{
    /// Select a custom Model from a raw SQL [Statement].
    pub fn from_statement<M>(stmt: Statement) -> SelectorRaw<SelectModel<M>>
    where
        M: FromQueryResult,
    {
        SelectorRaw {
            stmt,
            selector: SelectModel { model: PhantomData },
        }
    }

    /// Create `SelectorRaw` from Statement and columns. Executing this `SelectorRaw` will
    /// return a type `T` which implement `TryGetableMany`.
    pub fn with_columns<T, C>(stmt: Statement) -> SelectorRaw<SelectGetableValue<T, C>>
    where
        T: TryGetableMany,
        C: strum::IntoEnumIterator + sea_query::Iden,
    {
        SelectorRaw {
            stmt,
            selector: SelectGetableValue {
                columns: PhantomData,
                model: PhantomData,
            },
        }
    }

    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results([[
    /// #         maplit::btreemap! {
    /// #             "name" => Into::<Value>::into("Chocolate Forest"),
    /// #             "num_of_cakes" => Into::<Value>::into(1),
    /// #         },
    /// #         maplit::btreemap! {
    /// #             "name" => Into::<Value>::into("New York Cheese"),
    /// #             "num_of_cakes" => Into::<Value>::into(1),
    /// #         },
    /// #     ]])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, FromQueryResult};
    ///
    /// #[derive(Debug, PartialEq, FromQueryResult)]
    /// struct SelectResult {
    ///     name: String,
    ///     num_of_cakes: i32,
    /// }
    ///
    /// let res: Vec<SelectResult> = cake::Entity::find()
    ///     .from_raw_sql(Statement::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"SELECT "cake"."name", count("cake"."id") AS "num_of_cakes" FROM "cake""#,
    ///         [],
    ///     ))
    ///     .into_model::<SelectResult>()
    ///     .all(&db)
    ///     .await?;
    ///
    /// assert_eq!(
    ///     res,
    ///     [
    ///         SelectResult {
    ///             name: "Chocolate Forest".to_owned(),
    ///             num_of_cakes: 1,
    ///         },
    ///         SelectResult {
    ///             name: "New York Cheese".to_owned(),
    ///             num_of_cakes: 1,
    ///         },
    ///     ]
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     [Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"SELECT "cake"."name", count("cake"."id") AS "num_of_cakes" FROM "cake""#,
    ///         []
    ///     ),]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_model<M>(self) -> SelectorRaw<SelectModel<M>>
    where
        M: FromQueryResult,
    {
        SelectorRaw {
            stmt: self.stmt,
            selector: SelectModel { model: PhantomData },
        }
    }

    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results([[
    /// #         maplit::btreemap! {
    /// #             "name" => Into::<Value>::into("Chocolate Forest"),
    /// #             "num_of_cakes" => Into::<Value>::into(1),
    /// #         },
    /// #         maplit::btreemap! {
    /// #             "name" => Into::<Value>::into("New York Cheese"),
    /// #             "num_of_cakes" => Into::<Value>::into(1),
    /// #         },
    /// #     ]])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// let res: Vec<serde_json::Value> = cake::Entity::find().from_raw_sql(
    ///     Statement::from_sql_and_values(
    ///         DbBackend::Postgres, r#"SELECT "cake"."id", "cake"."name" FROM "cake""#, []
    ///     )
    /// )
    /// .into_json()
    /// .all(&db)
    /// .await?;
    ///
    /// assert_eq!(
    ///     res,
    ///     [
    ///         serde_json::json!({
    ///             "name": "Chocolate Forest",
    ///             "num_of_cakes": 1,
    ///         }),
    ///         serde_json::json!({
    ///             "name": "New York Cheese",
    ///             "num_of_cakes": 1,
    ///         }),
    ///     ]
    /// );
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     [
    ///     Transaction::from_sql_and_values(
    ///             DbBackend::Postgres, r#"SELECT "cake"."id", "cake"."name" FROM "cake""#, []
    ///     ),
    /// ]);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "with-json")]
    pub fn into_json(self) -> SelectorRaw<SelectModel<JsonValue>> {
        SelectorRaw {
            stmt: self.stmt,
            selector: SelectModel { model: PhantomData },
        }
    }

    /// Get the SQL statement
    pub fn into_statement(self) -> Statement {
        self.stmt
    }

    /// Get an item from the Select query
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results([
    /// #         [cake::Model {
    /// #             id: 1,
    /// #             name: "Cake".to_owned(),
    /// #         }],
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// let _: Option<cake::Model> = cake::Entity::find()
    ///     .from_raw_sql(Statement::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"SELECT "cake"."id", "cake"."name" FROM "cake" WHERE "id" = $1"#,
    ///         [1.into()],
    ///     ))
    ///     .one(&db)
    ///     .await?;
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     [Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"SELECT "cake"."id", "cake"."name" FROM "cake" WHERE "id" = $1"#,
    ///         [1.into()]
    ///     ),]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn one<'a, C>(self, db: &C) -> Result<Option<S::Item>, DbErr>
    where
        C: ConnectionTrait,
    {
        let row = db.query_one(self.stmt).await?;
        match row {
            Some(row) => Ok(Some(S::from_raw_query_result(row)?)),
            None => Ok(None),
        }
    }

    /// Get all items from the Select query
    /// ```
    /// # use sea_orm::{error::*, tests_cfg::*, *};
    /// #
    /// # #[smol_potat::main]
    /// # #[cfg(feature = "mock")]
    /// # pub async fn main() -> Result<(), DbErr> {
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results([
    /// #         [cake::Model {
    /// #             id: 1,
    /// #             name: "Cake".to_owned(),
    /// #         }],
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// let _: Vec<cake::Model> = cake::Entity::find()
    ///     .from_raw_sql(Statement::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"SELECT "cake"."id", "cake"."name" FROM "cake""#,
    ///         [],
    ///     ))
    ///     .all(&db)
    ///     .await?;
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     [Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"SELECT "cake"."id", "cake"."name" FROM "cake""#,
    ///         []
    ///     ),]
    /// );
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn all<'a, C>(self, db: &C) -> Result<Vec<S::Item>, DbErr>
    where
        C: ConnectionTrait,
    {
        let rows = db.query_all(self.stmt).await?;
        let mut models = Vec::new();
        for row in rows.into_iter() {
            models.push(S::from_raw_query_result(row)?);
        }
        Ok(models)
    }

    /// Stream the results of the Select operation
    pub async fn stream<'a: 'b, 'b, C>(
        self,
        db: &'a C,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<S::Item, DbErr>> + 'b + Send>>, DbErr>
    where
        C: ConnectionTrait + StreamTrait + Send,
        S: 'b,
        S::Item: Send,
    {
        let stream = db.stream(self.stmt).await?;
        Ok(Box::pin(stream.and_then(|row| {
            futures::future::ready(S::from_raw_query_result(row))
        })))
    }
}

fn consolidate_query_result<L, R>(
    rows: Vec<(L::Model, Option<R::Model>)>,
) -> Vec<(L::Model, Vec<R::Model>)>
where
    L: EntityTrait,
    R: EntityTrait,
{

    let keys: Vec<L::Model> = rows
        .iter()
        .map(|row| row.0.to_owned())
        .collect();

    let col = <L::PrimaryKey as Iterable>::iter().next().unwrap_or_default().into_column();

    let hashmap: HashMap<L::Model, Vec<R::Model>> = rows.into_iter().fold(
        HashMap::<L::Model, Vec<R::Model>>::new(),
        |mut acc: HashMap<L::Model, Vec<R::Model>>,
         value: (L::Model, Option<R::Model>)| {
            {
                let key = value.0.get(L::PrimaryKey);

                acc.insert(format!("{key:?}"), value.1);
            }

            acc
        },
    );


    let mut acc: Vec<(L::Model, Vec<R::Model>)> = Vec::new();
    for (l, r) in rows {
        if let Some((last_l, last_r)) = acc.last_mut() {
            let mut same_l = true;
            for pk_col in <L::PrimaryKey as Iterable>::iter() {
                let col = pk_col.into_column();
                let val = l.get(col);
                let last_val = last_l.get(col);
                if !val.eq(&last_val) {
                    same_l = false;
                    break;
                }
            }
            if same_l {
                if let Some(r) = r {
                    last_r.push(r);
                    continue;
                }
            }
        }
        let rows = match r {
            Some(r) => vec![r],
            None => vec![],
        };
        acc.push((l, rows));
    }
    acc
}
