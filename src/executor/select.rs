use crate::{
    error::*, query::combine, DatabaseConnection, EntityTrait, FromQueryResult, Iterable,
    JsonValue, ModelTrait, Paginator, PrimaryKeyToColumn, QueryResult, Select, SelectTwo,
    SelectTwoMany,
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

    fn from_raw_query_result(res: QueryResult) -> Result<Self::Item, SeaErr>;
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

    fn from_raw_query_result(res: QueryResult) -> Result<Self::Item, SeaErr> {
        M::from_query_result(&res, "")
    }
}

impl<M, N> SelectorTrait for SelectTwoModel<M, N>
where
    M: FromQueryResult + Sized,
    N: FromQueryResult + Sized,
{
    type Item = (M, Option<N>);

    fn from_raw_query_result(res: QueryResult) -> Result<Self::Item, SeaErr> {
        Ok((
            M::from_query_result(&res, combine::SELECT_A)?,
            N::from_query_result_optional(&res, combine::SELECT_B)?,
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

    pub async fn one(self, db: &DatabaseConnection) -> Result<Option<E::Model>, SeaErr> {
        self.into_model::<E::Model>().one(db).await
    }

    pub async fn all(self, db: &DatabaseConnection) -> Result<Vec<E::Model>, SeaErr> {
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
    ) -> Result<Option<(E::Model, Option<F::Model>)>, SeaErr> {
        self.into_model::<E::Model, F::Model>().one(db).await
    }

    pub async fn all(
        self,
        db: &DatabaseConnection,
    ) -> Result<Vec<(E::Model, Option<F::Model>)>, SeaErr> {
        self.into_model::<E::Model, F::Model>().all(db).await
    }
}

impl<E, F> SelectTwoMany<E, F>
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
    ) -> Result<Option<(E::Model, Option<F::Model>)>, SeaErr> {
        self.into_model::<E::Model, F::Model>().one(db).await
    }

    pub async fn all(
        self,
        db: &DatabaseConnection,
    ) -> Result<Vec<(E::Model, Vec<F::Model>)>, SeaErr> {
        let rows = self.into_model::<E::Model, F::Model>().all(db).await?;
        Ok(consolidate_query_result::<E, F>(rows))
    }
}

impl<S> Selector<S>
where
    S: SelectorTrait,
{
    pub async fn one(mut self, db: &DatabaseConnection) -> Result<Option<S::Item>, SeaErr> {
        let builder = db.get_query_builder_backend();
        self.query.limit(1);
        let row = db.query_one(builder.build(&self.query)).await?;
        match row {
            Some(row) => Ok(Some(S::from_raw_query_result(row)?)),
            None => Ok(None),
        }
    }

    pub async fn all(self, db: &DatabaseConnection) -> Result<Vec<S::Item>, SeaErr> {
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

fn consolidate_query_result<L, R>(
    rows: Vec<(L::Model, Option<R::Model>)>,
) -> Vec<(L::Model, Vec<R::Model>)>
where
    L: EntityTrait,
    R: EntityTrait,
{
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
        if r.is_some() {
            acc.push((l, vec![r.unwrap()]));
        } else {
            acc.push((l, vec![]));
        }
    }
    acc
}
