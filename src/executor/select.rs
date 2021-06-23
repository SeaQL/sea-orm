use crate::{
    query::combine, DatabaseConnection, EntityTrait, FromQueryResult, Iterable, JsonValue,
    ModelTrait, Paginator, PrimaryKeyToColumn, QueryErr, QueryResult, Select, SelectTwo, TypeErr,
};
use sea_query::SelectStatement;
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
        M::from_query_result(&res, "")
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

    pub async fn one(self, db: &DatabaseConnection) -> Result<Option<E::Model>, QueryErr> {
        self.into_model::<E::Model>().one(db).await
    }

    pub async fn all(self, db: &DatabaseConnection) -> Result<Vec<E::Model>, QueryErr> {
        self.into_model::<E::Model>().all(db).await
    }

    pub fn paginate(
        self,
        db: &DatabaseConnection,
        page_size: usize,
    ) -> Paginator<'_, SelectModel<E::Model>> {
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

    pub async fn one(
        self,
        db: &DatabaseConnection,
    ) -> Result<Option<(E::Model, F::Model)>, QueryErr> {
        self.into_model::<E::Model, F::Model>().one(db).await
    }

    pub async fn all(
        self,
        db: &DatabaseConnection,
    ) -> Result<Vec<(E::Model, Vec<F::Model>)>, QueryErr> {
        let rows = self.into_model::<E::Model, F::Model>().all(db).await?;
        Ok(parse_query_result::<E, _>(rows))
    }
}

impl<S> Selector<S>
where
    S: SelectorTrait,
{
    pub async fn one(mut self, db: &DatabaseConnection) -> Result<Option<S::Item>, QueryErr> {
        let builder = db.get_query_builder_backend();
        self.query.limit(1);
        let row = db.query_one(builder.build(&self.query)).await?;
        match row {
            Some(row) => Ok(Some(S::from_raw_query_result(row)?)),
            None => Ok(None),
        }
    }

    pub async fn all(self, db: &DatabaseConnection) -> Result<Vec<S::Item>, QueryErr> {
        let builder = db.get_query_builder_backend();
        let rows = db.query_all(builder.build(&self.query)).await?;
        let mut models = Vec::new();
        for row in rows.into_iter() {
            models.push(S::from_raw_query_result(row)?);
        }
        Ok(models)
    }

    pub fn paginate(self, db: &DatabaseConnection, page_size: usize) -> Paginator<'_, S> {
        Paginator {
            query: self.query,
            page: 0,
            page_size,
            db,
            selector: PhantomData,
        }
    }
}

fn parse_query_result<L, R>(rows: Vec<(L::Model, R)>) -> Vec<(L::Model, Vec<R>)>
    where
        L: EntityTrait,
    {
        let mut acc = Vec::new();
        for (l_model, r_model) in rows {
            if acc.is_empty() {
                acc.push((l_model, vec![r_model]));
                continue;
            }
            let (last_l, last_r) = acc.last_mut().unwrap();
            let mut l_equal = true;
            for pk_col in <L::PrimaryKey as Iterable>::iter() {
                let col = pk_col.into_column();
                let curr_val = l_model.get(col);
                let last_val = last_l.get(col);
                if !curr_val.eq(&last_val) {
                    l_equal = false;
                    break;
                }
            }
            if l_equal {
                last_r.push(r_model);
            } else {
                acc.push((l_model, vec![r_model]));
            }
        }
        acc
    }
