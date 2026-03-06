use super::*;
use crate::{
    JoinType, Paginator, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait, Related,
    SelectC, SelectFour, SelectThree, SelectThreeMany, Topology, TopologyChain, TopologyStar,
    combine::prepare_select_col,
};

impl<E, F, G, TOP> SelectThree<E, F, G, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    TOP: Topology,
{
    pub(crate) fn new(query: SelectStatement) -> Self {
        Self::new_without_prepare(query).prepare_select()
    }

    pub(crate) fn new_without_prepare(query: SelectStatement) -> Self {
        Self {
            query,
            entity: PhantomData,
        }
    }

    fn prepare_select(mut self) -> Self {
        prepare_select_col::<G, _, _>(&mut self, SelectC);
        self
    }

    /// Left Join with a Related Entity and select all Entities.
    pub fn find_also<T, H>(self, _: T, _: H) -> SelectFour<E, F, G, H, TopologyStar>
    where
        H: EntityTrait,
        T: EntityTrait + Related<H>,
    {
        SelectFour::new(
            self.join_join(JoinType::LeftJoin, T::to(), T::via())
                .into_query(),
        )
    }
}

macro_rules! impl_query_trait {
    ( $trait: ident ) => {
        impl<E, F, G, TOP> $trait for SelectThree<E, F, G, TOP>
        where
            E: EntityTrait,
            F: EntityTrait,
            G: EntityTrait,
            TOP: Topology,
        {
            type QueryStatement = SelectStatement;

            fn query(&mut self) -> &mut SelectStatement {
                &mut self.query
            }
        }
    };
}

impl<E, F, G, TOP> QueryTrait for SelectThree<E, F, G, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    TOP: Topology,
{
    type QueryStatement = SelectStatement;
    fn query(&mut self) -> &mut SelectStatement {
        &mut self.query
    }
    fn as_query(&self) -> &SelectStatement {
        &self.query
    }
    fn into_query(self) -> SelectStatement {
        self.query
    }
}

impl_query_trait!(QuerySelect);
impl_query_trait!(QueryFilter);
impl_query_trait!(QueryOrder);

impl<M, N, O> SelectorTrait for SelectThreeModel<M, N, O>
where
    M: FromQueryResult + Sized,
    N: FromQueryResult + Sized,
    O: FromQueryResult + Sized,
{
    type Item = (M, Option<N>, Option<O>);

    fn from_raw_query_result(res: QueryResult) -> Result<Self::Item, DbErr> {
        Ok((
            M::from_query_result(&res, SelectA.as_str())?,
            N::from_query_result_optional(&res, SelectB.as_str())?,
            O::from_query_result_optional(&res, SelectC.as_str())?,
        ))
    }
}

impl<E, F, G, TOP> SelectThree<E, F, G, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    TOP: Topology,
{
    /// Perform a conversion into a [SelectThreeModel]
    pub fn into_model<M, N, O>(self) -> Selector<SelectThreeModel<M, N, O>>
    where
        M: FromQueryResult,
        N: FromQueryResult,
        O: FromQueryResult,
    {
        Selector {
            query: self.query,
            selector: PhantomData,
        }
    }

    /// Perform a conversion into a [SelectThreeModel] with [PartialModel](PartialModelTrait)
    pub fn into_partial_model<M, N, O>(self) -> Selector<SelectThreeModel<M, N, O>>
    where
        M: PartialModelTrait,
        N: PartialModelTrait,
        O: PartialModelTrait,
    {
        let select = QuerySelect::select_only(self);
        let select = M::select_cols(select);
        let select = N::select_cols(select);
        let select = O::select_cols(select);
        select.into_model::<M, N, O>()
    }

    /// Convert the Models into JsonValue
    #[cfg(feature = "with-json")]
    pub fn into_json(self) -> Selector<SelectThreeModel<JsonValue, JsonValue, JsonValue>> {
        Selector {
            query: self.query,
            selector: PhantomData,
        }
    }

    /// Get one Model from the Select query
    pub async fn one<C>(
        self,
        db: &C,
    ) -> Result<Option<(E::Model, Option<F::Model>, Option<G::Model>)>, DbErr>
    where
        C: ConnectionTrait,
    {
        self.into_model().one(db).await
    }

    /// Get all Models from the Select query
    pub async fn all<C>(
        self,
        db: &C,
    ) -> Result<Vec<(E::Model, Option<F::Model>, Option<G::Model>)>, DbErr>
    where
        C: ConnectionTrait,
    {
        self.into_model().all(db).await
    }

    /// Stream the results of a Select operation on a Model
    pub async fn stream<'a: 'b, 'b, C>(
        self,
        db: &'a C,
    ) -> Result<
        impl Stream<Item = Result<(E::Model, Option<F::Model>, Option<G::Model>), DbErr>> + 'b,
        DbErr,
    >
    where
        C: ConnectionTrait + StreamTrait + Send,
    {
        self.into_model().stream(db).await
    }

    /// Stream the result of the operation with PartialModel
    pub async fn stream_partial_model<'a: 'b, 'b, C, M, N, O>(
        self,
        db: &'a C,
    ) -> Result<impl Stream<Item = Result<(M, Option<N>, Option<O>), DbErr>> + 'b + Send, DbErr>
    where
        C: ConnectionTrait + StreamTrait + Send,
        M: PartialModelTrait + Send + 'b,
        N: PartialModelTrait + Send + 'b,
        O: PartialModelTrait + Send + 'b,
    {
        self.into_partial_model().stream(db).await
    }

    /// Consolidate query result by first / second model depending on join topology
    /// ```
    /// # use sea_orm::{tests_cfg::*, *};
    /// # async fn function(db: &DbConn) -> Result<(), DbErr> {
    /// // fruit -> cake -> filling
    /// let items: Vec<(fruit::Model, Vec<(cake::Model, Vec<filling::Model>)>)> = fruit::Entity::find()
    ///     .find_also_related(cake::Entity)
    ///     .and_also_related(filling::Entity)
    ///     .consolidate()
    ///     .all(db)
    ///     .await?;
    ///
    /// // cake -> fruit
    /// //      -> filling
    /// let items: Vec<(cake::Model, Vec<fruit::Model>, Vec<filling::Model>)> = cake::Entity::find()
    ///     .find_also_related(fruit::Entity)
    ///     .find_also_related(filling::Entity)
    ///     .consolidate()
    ///     .all(db)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn consolidate(self) -> SelectThreeMany<E, F, G, TOP> {
        SelectThreeMany {
            query: self.query,
            entity: self.entity,
        }
    }
}

impl<E, F, G, TOP> SelectThreeMany<E, F, G, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    TOP: Topology,
{
    /// Performs a conversion to [Selector]
    fn into_model<M, N, O>(self) -> Selector<SelectThreeModel<M, N, O>>
    where
        M: FromQueryResult,
        N: FromQueryResult,
        O: FromQueryResult,
    {
        Selector {
            query: self.query,
            selector: PhantomData,
        }
    }
}

impl<E, F, G> SelectThreeMany<E, F, G, TopologyStar>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
{
    /// Execute query and consolidate rows by E
    pub async fn all<C>(
        self,
        db: &C,
    ) -> Result<Vec<(E::Model, Vec<F::Model>, Vec<G::Model>)>, DbErr>
    where
        C: ConnectionTrait,
    {
        let rows = self.into_model().all(db).await?;
        Ok(consolidate_query_result_tee::<E, F, G>(rows))
    }
}

impl<E, F, G> SelectThreeMany<E, F, G, TopologyChain>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
{
    /// Execute query and consolidate rows in two passes, first by E, then by F
    pub async fn all<C>(
        self,
        db: &C,
    ) -> Result<Vec<(E::Model, Vec<(F::Model, Vec<G::Model>)>)>, DbErr>
    where
        C: ConnectionTrait,
    {
        let rows = self.into_model().all(db).await?;
        Ok(consolidate_query_result_chain::<E, F, G>(rows))
    }
}

impl<'db, C, M, N, O, E, F, G, TOP> PaginatorTrait<'db, C> for SelectThree<E, F, G, TOP>
where
    C: ConnectionTrait,
    E: EntityTrait<Model = M>,
    F: EntityTrait<Model = N>,
    G: EntityTrait<Model = O>,
    M: FromQueryResult + Sized + Send + Sync + 'db,
    N: FromQueryResult + Sized + Send + Sync + 'db,
    O: FromQueryResult + Sized + Send + Sync + 'db,
    TOP: Topology,
{
    type Selector = SelectThreeModel<M, N, O>;

    fn paginate(self, db: &'db C, page_size: u64) -> Paginator<'db, C, Self::Selector> {
        self.into_model().paginate(db, page_size)
    }
}
