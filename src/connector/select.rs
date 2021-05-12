use crate::{Connection, Database, EntityTrait, FromQueryResult, QueryErr, Select, Statement};
use sea_query::{QueryBuilder, SelectStatement};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct SingleSelect<M>
where
    M: FromQueryResult,
{
    query: SelectStatement,
    model: PhantomData<M>,
}

impl<E: 'static> Select<E>
where
    E: EntityTrait,
{
    pub fn into_model<M>(self) -> SingleSelect<M>
    where
        M: FromQueryResult,
    {
        SingleSelect {
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

impl<M> SingleSelect<M>
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
        Ok(M::from_query_result(row, "")?)
    }

    pub async fn all(self, db: &Database) -> Result<Vec<M>, QueryErr> {
        let builder = db.get_query_builder_backend();
        let rows = db.get_connection().query_all(self.build(builder)).await?;
        let mut models = Vec::new();
        for row in rows.into_iter() {
            models.push(M::from_query_result(row, "")?);
        }
        Ok(models)
    }
}
