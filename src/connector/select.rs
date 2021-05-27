use crate::{Connection, Database, EntityTrait, FromQueryResult, JsonValue, Paginator, QueryErr, QueryResult, Select, SelectTwo, Statement, TypeErr, query::combine};
use sea_query::{QueryBuilder, SelectStatement};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct Selector<S>
where
    S: SelectorTrait,
{
    query: SelectStatement,
    selector: S,
}

pub trait SelectorTrait {
    type Item: Sized;

    fn from_raw_query_result(res: QueryResult) -> Result<Self::Item, TypeErr>;
}

pub struct SelectModel<M>
where
    M: FromQueryResult,
{
    model: PhantomData<M>,
}

#[derive(Clone, Debug)]
pub struct SelectTwoModel<M, N>
where
    M: FromQueryResult,
    N: FromQueryResult,
{
    model: PhantomData<(M, N)>,
}

impl<M> SelectorTrait for SelectModel<M>
where
    M: FromQueryResult + Sized,
{
    type Item = M;

    fn from_raw_query_result(res: QueryResult) -> Result<Self::Item, TypeErr> {
        Ok(M::from_query_result(&res, "")?)
    }
}

impl<M, N> SelectorTrait for SelectTwoModel<M, N>
where
    M: FromQueryResult + Sized,
    N: FromQueryResult + Sized,
{
    type Item = (M, N);

    fn from_raw_query_result(res: QueryResult) -> Result<Self::Item, TypeErr> {
        Ok((
            M::from_query_result(&res, combine::SELECT_A)?,
            N::from_query_result(&res, combine::SELECT_B)?,
        ))
    }
}

impl<E> Select<E>
where
    E: EntityTrait,
{
    pub fn into_model<M>(self) -> Selector<SelectModel<M>>
    where
        M: FromQueryResult,
    {
        Selector {
            query: self.query,
            selector: SelectModel { model: PhantomData },
        }
    }

    #[cfg(feature = "with-json")]
    pub fn into_json(self) -> Selector<SelectModel<JsonValue>> {
        Selector {
            query: self.query,
            selector: SelectModel { model: PhantomData },
        }
    }

    pub async fn one(self, db: &Database) -> Result<E::Model, QueryErr> {
        self.into_model::<E::Model>().one(db).await
    }

    pub async fn all(self, db: &Database) -> Result<Vec<E::Model>, QueryErr> {
        self.into_model::<E::Model>().all(db).await
    }

    pub fn paginate<'db>(self, db: &'db Database, page_size: usize) -> Paginator<'db, SelectModel<E::Model>> {
        self.into_model::<E::Model>().paginate(db, page_size)
    }
}

impl<E, F> SelectTwo<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
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

    #[cfg(feature = "with-json")]
    pub fn into_json(self) -> Selector<SelectTwoModel<JsonValue, JsonValue>> {
        Selector {
            query: self.query,
            selector: SelectTwoModel { model: PhantomData },
        }
    }

    pub async fn one(self, db: &Database) -> Result<(E::Model, F::Model), QueryErr> {
        self.into_model::<E::Model, F::Model>().one(db).await
    }

    pub async fn all(self, db: &Database) -> Result<Vec<(E::Model, F::Model)>, QueryErr> {
        self.into_model::<E::Model, F::Model>().all(db).await
    }
}

impl<S> Selector<S>
where
    S: SelectorTrait,
{
    pub fn build<B>(&self, builder: B) -> Statement
    where
        B: QueryBuilder,
    {
        self.query.build(builder).into()
    }

    pub async fn one(mut self, db: &Database) -> Result<S::Item, QueryErr> {
        let builder = db.get_query_builder_backend();
        self.query.limit(1);
        let row = db.get_connection().query_one(self.build(builder)).await?;
        Ok(S::from_raw_query_result(row)?)
    }

    pub async fn all(self, db: &Database) -> Result<Vec<S::Item>, QueryErr> {
        let builder = db.get_query_builder_backend();
        let rows = db.get_connection().query_all(self.build(builder)).await?;
        let mut models = Vec::new();
        for row in rows.into_iter() {
            models.push(S::from_raw_query_result(row)?);
        }
        Ok(models)
    }

    pub fn paginate<'db>(self, db: &'db Database, page_size: usize) -> Paginator<'db, S> {
        Paginator {
            query: self.query,
            page: 0,
            page_size,
            db,
            selector: PhantomData,
        }
    }
}
