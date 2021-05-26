use crate::query::combine;
use crate::{
    Connection, Database, EntityTrait, FromQueryResult, QueryErr, Select, SelectTwo, Statement,
};
use sea_query::{QueryBuilder, SelectStatement};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct SelectModel<M>
where
    M: FromQueryResult,
{
    query: SelectStatement,
    model: PhantomData<M>,
}

#[derive(Clone, Debug)]
pub struct SelectTwoModel<M, N>
where
    M: FromQueryResult,
    N: FromQueryResult,
{
    query: SelectStatement,
    model: PhantomData<(M, N)>,
}

impl<E> Select<E>
where
    E: EntityTrait,
{
    pub fn into_model<M>(self) -> SelectModel<M>
    where
        M: FromQueryResult,
    {
        SelectModel {
            query: self.query,
            model: PhantomData,
        }
    }

    #[cfg(feature = "with-json")]
    pub fn into_json(self) -> SelectModel<serde_json::Value> {
        SelectModel {
            query: self.query,
            model: PhantomData,
        }
    }

    pub async fn one(self, db: &Database) -> Result<E::Model, QueryErr> {
        self.into_model::<E::Model>().one(db).await
    }

    pub async fn all(self, db: &Database) -> Result<Vec<E::Model>, QueryErr> {
        self.into_model::<E::Model>().all(db).await
    }
}

impl<E, F> SelectTwo<E, F>
where
    E: EntityTrait,
    F: EntityTrait,
{
    fn into_model<M, N>(self) -> SelectTwoModel<M, N>
    where
        M: FromQueryResult,
        N: FromQueryResult,
    {
        SelectTwoModel {
            query: self.query,
            model: PhantomData,
        }
    }

    #[cfg(feature = "with-json")]
    pub fn into_json(self) -> SelectTwoModel<serde_json::Value, serde_json::Value> {
        SelectTwoModel {
            query: self.query,
            model: PhantomData,
        }
    }

    pub async fn one(self, db: &Database) -> Result<(E::Model, F::Model), QueryErr> {
        self.into_model::<E::Model, F::Model>().one(db).await
    }

    pub async fn all(self, db: &Database) -> Result<Vec<(E::Model, F::Model)>, QueryErr> {
        self.into_model::<E::Model, F::Model>().all(db).await
    }
}

impl<M> SelectModel<M>
where
    M: FromQueryResult,
{
    pub fn build<B>(&self, builder: B) -> Statement
    where
        B: QueryBuilder,
    {
        self.query.build(builder).into()
    }

    pub async fn one(mut self, db: &Database) -> Result<M, QueryErr> {
        let builder = db.get_query_builder_backend();
        self.query.limit(1);
        let row = db.get_connection().query_one(self.build(builder)).await?;
        Ok(M::from_query_result(&row, "")?)
    }

    pub async fn all(self, db: &Database) -> Result<Vec<M>, QueryErr> {
        let builder = db.get_query_builder_backend();
        let rows = db.get_connection().query_all(self.build(builder)).await?;
        let mut models = Vec::new();
        for row in rows.into_iter() {
            models.push(M::from_query_result(&row, "")?);
        }
        Ok(models)
    }
}

impl<M, N> SelectTwoModel<M, N>
where
    M: FromQueryResult,
    N: FromQueryResult,
{
    pub fn build<B>(&self, builder: B) -> Statement
    where
        B: QueryBuilder,
    {
        self.query.build(builder).into()
    }

    pub async fn one(mut self, db: &Database) -> Result<(M, N), QueryErr> {
        let builder = db.get_query_builder_backend();
        self.query.limit(1);
        let row = db.get_connection().query_one(self.build(builder)).await?;
        Ok((
            M::from_query_result(&row, combine::SELECT_A)?,
            N::from_query_result(&row, combine::SELECT_B)?,
        ))
    }

    pub async fn all(self, db: &Database) -> Result<Vec<(M, N)>, QueryErr> {
        let builder = db.get_query_builder_backend();
        let rows = db.get_connection().query_all(self.build(builder)).await?;
        let mut models = Vec::new();
        for row in rows.into_iter() {
            models.push((
                M::from_query_result(&row, combine::SELECT_A)?,
                N::from_query_result(&row, combine::SELECT_B)?,
            ));
        }
        Ok(models)
    }
}
