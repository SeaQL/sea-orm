use super::*;
use crate::{
    JoinType, Paginator, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait, Related,
    SelectC, SelectFive, SelectFour, Topology, TopologyStar,
    combine::{SelectD, prepare_select_col},
};

impl<E, F, G, H, TOP> SelectFour<E, F, G, H, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    H: EntityTrait,
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
        prepare_select_col::<H, _, _>(&mut self, SelectD);
        self
    }

    /// Left Join with a Related Entity and select all Entities.
    pub fn find_also<T, I>(self, _: T, _: I) -> SelectFive<E, F, G, H, I, TopologyStar>
    where
        I: EntityTrait,
        T: EntityTrait + Related<I>,
    {
        SelectFive::new(
            self.join_join(JoinType::LeftJoin, T::to(), T::via())
                .into_query(),
        )
    }
}

macro_rules! impl_query_trait {
    ( $trait: ident ) => {
        impl<E, F, G, H, TOP> $trait for SelectFour<E, F, G, H, TOP>
        where
            E: EntityTrait,
            F: EntityTrait,
            G: EntityTrait,
            H: EntityTrait,
            TOP: Topology,
        {
            type QueryStatement = SelectStatement;

            fn query(&mut self) -> &mut SelectStatement {
                &mut self.query
            }
        }
    };
}

impl<E, F, G, H, TOP> QueryTrait for SelectFour<E, F, G, H, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    H: EntityTrait,
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

impl<M, N, O, P> SelectorTrait for SelectFourModel<M, N, O, P>
where
    M: FromQueryResult + Sized,
    N: FromQueryResult + Sized,
    O: FromQueryResult + Sized,
    P: FromQueryResult + Sized,
{
    type Item = (M, Option<N>, Option<O>, Option<P>);

    fn from_raw_query_result(res: QueryResult) -> Result<Self::Item, DbErr> {
        Ok((
            M::from_query_result(&res, SelectA.as_str())?,
            N::from_query_result_optional(&res, SelectB.as_str())?,
            O::from_query_result_optional(&res, SelectC.as_str())?,
            P::from_query_result_optional(&res, SelectD.as_str())?,
        ))
    }
}

impl<E, F, G, H, TOP> SelectFour<E, F, G, H, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    H: EntityTrait,
    TOP: Topology,
{
    /// Perform a conversion into a [SelectFourModel]
    pub fn into_model<M, N, O, P>(self) -> Selector<SelectFourModel<M, N, O, P>>
    where
        M: FromQueryResult,
        N: FromQueryResult,
        O: FromQueryResult,
        P: FromQueryResult,
    {
        Selector {
            query: self.query,
            selector: PhantomData,
        }
    }

    /// Perform a conversion into a [SelectFourModel] with [PartialModel](PartialModelTrait)
    pub fn into_partial_model<M, N, O, P>(self) -> Selector<SelectFourModel<M, N, O, P>>
    where
        M: PartialModelTrait,
        N: PartialModelTrait,
        O: PartialModelTrait,
        P: PartialModelTrait,
    {
        let select = QuerySelect::select_only(self);
        let select = M::select_cols(select);
        let select = N::select_cols(select);
        let select = O::select_cols(select);
        let select = P::select_cols(select);
        select.into_model::<M, N, O, P>()
    }

    /// Convert the Models into JsonValue
    #[cfg(feature = "with-json")]
    pub fn into_json(
        self,
    ) -> Selector<SelectFourModel<JsonValue, JsonValue, JsonValue, JsonValue>> {
        Selector {
            query: self.query,
            selector: PhantomData,
        }
    }

    /// Get one Model from the Select query
    pub fn one<C>(
        self,
        db: &C,
    ) -> Result<
        Option<(
            E::Model,
            Option<F::Model>,
            Option<G::Model>,
            Option<H::Model>,
        )>,
        DbErr,
    >
    where
        C: ConnectionTrait,
    {
        self.into_model().one(db)
    }

    /// Get all Models from the Select query
    pub fn all<C>(
        self,
        db: &C,
    ) -> Result<
        Vec<(
            E::Model,
            Option<F::Model>,
            Option<G::Model>,
            Option<H::Model>,
        )>,
        DbErr,
    >
    where
        C: ConnectionTrait,
    {
        self.into_model().all(db)
    }

    /// Stream the results of a Select operation on a Model
    pub fn stream<'a: 'b, 'b, C>(
        self,
        db: &'a C,
    ) -> Result<
        impl Iterator<
            Item = Result<
                (
                    E::Model,
                    Option<F::Model>,
                    Option<G::Model>,
                    Option<H::Model>,
                ),
                DbErr,
            >,
        > + 'b,
        DbErr,
    >
    where
        C: ConnectionTrait + StreamTrait,
    {
        self.into_model().stream(db)
    }

    /// Stream the result of the operation with PartialModel
    pub fn stream_partial_model<'a: 'b, 'b, C, M, N, O, P>(
        self,
        db: &'a C,
    ) -> Result<impl Iterator<Item = Result<(M, Option<N>, Option<O>, Option<P>), DbErr>> + 'b, DbErr>
    where
        C: ConnectionTrait + StreamTrait,
        M: PartialModelTrait + 'b,
        N: PartialModelTrait + 'b,
        O: PartialModelTrait + 'b,
        P: PartialModelTrait + 'b,
    {
        self.into_partial_model().stream(db)
    }
}

impl<'db, C, EE, FF, GG, HH, E, F, G, H, TOP> PaginatorTrait<'db, C> for SelectFour<E, F, G, H, TOP>
where
    C: ConnectionTrait,
    E: EntityTrait<Model = EE>,
    F: EntityTrait<Model = FF>,
    G: EntityTrait<Model = GG>,
    H: EntityTrait<Model = HH>,
    EE: FromQueryResult + Sized + 'db,
    FF: FromQueryResult + Sized + 'db,
    GG: FromQueryResult + Sized + 'db,
    HH: FromQueryResult + Sized + 'db,
    TOP: Topology,
{
    type Selector = SelectFourModel<EE, FF, GG, HH>;

    fn paginate(self, db: &'db C, page_size: u64) -> Paginator<'db, C, Self::Selector> {
        self.into_model().paginate(db, page_size)
    }
}
