use super::*;
use crate::{
    Paginator, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait, SelectC,
    SelectSix, Topology,
    combine::{SelectD, SelectE, SelectF, prepare_select_col},
};

impl<E, F, G, H, I, J, TOP> SelectSix<E, F, G, H, I, J, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    H: EntityTrait,
    I: EntityTrait,
    J: EntityTrait,
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
        prepare_select_col::<J, _, _>(&mut self, SelectF);
        self
    }
}

macro_rules! impl_query_trait {
    ( $trait: ident ) => {
        impl<E, F, G, H, I, J, TOP> $trait for SelectSix<E, F, G, H, I, J, TOP>
        where
            E: EntityTrait,
            F: EntityTrait,
            G: EntityTrait,
            H: EntityTrait,
            I: EntityTrait,
            J: EntityTrait,
            TOP: Topology,
        {
            type QueryStatement = SelectStatement;

            fn query(&mut self) -> &mut SelectStatement {
                &mut self.query
            }
        }
    };
}

impl<E, F, G, H, I, J, TOP> QueryTrait for SelectSix<E, F, G, H, I, J, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    H: EntityTrait,
    I: EntityTrait,
    J: EntityTrait,
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

impl<M, N, O, P, Q, R> SelectorTrait for SelectSixModel<M, N, O, P, Q, R>
where
    M: FromQueryResult + Sized,
    N: FromQueryResult + Sized,
    O: FromQueryResult + Sized,
    P: FromQueryResult + Sized,
    Q: FromQueryResult + Sized,
    R: FromQueryResult + Sized,
{
    type Item = (M, Option<N>, Option<O>, Option<P>, Option<Q>, Option<R>);

    fn from_raw_query_result(res: QueryResult) -> Result<Self::Item, DbErr> {
        Ok((
            M::from_query_result(&res, SelectA.as_str())?,
            N::from_query_result_optional(&res, SelectB.as_str())?,
            O::from_query_result_optional(&res, SelectC.as_str())?,
            P::from_query_result_optional(&res, SelectD.as_str())?,
            Q::from_query_result_optional(&res, SelectE.as_str())?,
            R::from_query_result_optional(&res, SelectF.as_str())?,
        ))
    }
}

impl<E, F, G, H, I, J, TOP> SelectSix<E, F, G, H, I, J, TOP>
where
    E: EntityTrait,
    F: EntityTrait,
    G: EntityTrait,
    H: EntityTrait,
    I: EntityTrait,
    J: EntityTrait,
    TOP: Topology,
{
    /// Perform a conversion into a [SelectSixModel]
    pub fn into_model<M, N, O, P, Q, R>(self) -> Selector<SelectSixModel<M, N, O, P, Q, R>>
    where
        M: FromQueryResult,
        N: FromQueryResult,
        O: FromQueryResult,
        P: FromQueryResult,
        Q: FromQueryResult,
        R: FromQueryResult,
    {
        Selector {
            query: self.query,
            selector: PhantomData,
        }
    }

    /// Perform a conversion into a [SelectSixModel] with [PartialModel](PartialModelTrait)
    pub fn into_partial_model<M, N, O, P, Q, R>(self) -> Selector<SelectSixModel<M, N, O, P, Q, R>>
    where
        M: PartialModelTrait,
        N: PartialModelTrait,
        O: PartialModelTrait,
        P: PartialModelTrait,
        Q: PartialModelTrait,
        R: PartialModelTrait,
    {
        let select = QuerySelect::select_only(self);
        let select = M::select_cols(select);
        let select = N::select_cols(select);
        let select = O::select_cols(select);
        let select = P::select_cols(select);
        let select = Q::select_cols(select);
        let select = R::select_cols(select);
        select.into_model::<M, N, O, P, Q, R>()
    }

    /// Convert the Models into JsonValue
    #[cfg(feature = "with-json")]
    pub fn into_json(
        self,
    ) -> Selector<SelectSixModel<JsonValue, JsonValue, JsonValue, JsonValue, JsonValue, JsonValue>>
    {
        Selector {
            query: self.query,
            selector: PhantomData,
        }
    }

    /// Get one Model from the Select query
    pub async fn one<C>(
        self,
        db: &C,
    ) -> Result<
        Option<(
            E::Model,
            Option<F::Model>,
            Option<G::Model>,
            Option<H::Model>,
            Option<I::Model>,
            Option<J::Model>,
        )>,
        DbErr,
    >
    where
        C: ConnectionTrait,
    {
        self.into_model().one(db).await
    }

    /// Get all Models from the Select query
    pub async fn all<C>(
        self,
        db: &C,
    ) -> Result<
        Vec<(
            E::Model,
            Option<F::Model>,
            Option<G::Model>,
            Option<H::Model>,
            Option<I::Model>,
            Option<J::Model>,
        )>,
        DbErr,
    >
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
        impl Stream<
            Item = Result<
                (
                    E::Model,
                    Option<F::Model>,
                    Option<G::Model>,
                    Option<H::Model>,
                    Option<I::Model>,
                    Option<J::Model>,
                ),
                DbErr,
            >,
        > + 'b,
        DbErr,
    >
    where
        C: ConnectionTrait + StreamTrait + Send,
    {
        self.into_model().stream(db).await
    }
}

impl<'db, C, EE, FF, GG, HH, II, JJ, E, F, G, H, I, J, TOP> PaginatorTrait<'db, C>
    for SelectSix<E, F, G, H, I, J, TOP>
where
    C: ConnectionTrait,
    E: EntityTrait<Model = EE>,
    F: EntityTrait<Model = FF>,
    G: EntityTrait<Model = GG>,
    H: EntityTrait<Model = HH>,
    I: EntityTrait<Model = II>,
    J: EntityTrait<Model = JJ>,
    EE: FromQueryResult + Sized + Send + Sync + 'db,
    FF: FromQueryResult + Sized + Send + Sync + 'db,
    GG: FromQueryResult + Sized + Send + Sync + 'db,
    HH: FromQueryResult + Sized + Send + Sync + 'db,
    II: FromQueryResult + Sized + Send + Sync + 'db,
    JJ: FromQueryResult + Sized + Send + Sync + 'db,
    TOP: Topology,
{
    type Selector = SelectSixModel<EE, FF, GG, HH, II, JJ>;

    fn paginate(self, db: &'db C, page_size: u64) -> Paginator<'db, C, Self::Selector> {
        self.into_model().paginate(db, page_size)
    }
}
