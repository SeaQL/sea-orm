use crate::{Connection, Database, EntityTrait, FromQueryResult, QueryErr, Select, Statement};
use sea_query::{QueryBuilder, SelectStatement};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct ModelSelect<M>
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
    pub fn into_model<M>(self) -> ModelSelect<M>
    where
        M: FromQueryResult,
    {
        ModelSelect {
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

impl<M> ModelSelect<M>
where
    M: FromQueryResult,
{
    /// Get a mutable ref to the query builder
    pub fn query(&mut self) -> &mut SelectStatement {
        &mut self.query
    }

    /// Get an immutable ref to the query builder
    pub fn as_query(&self) -> &SelectStatement {
        &self.query
    }

    /// Take ownership of the query builder
    pub fn into_query(self) -> SelectStatement {
        self.query
    }

    /// Build the query as [`Statement`]
    pub fn build<B>(&self, builder: B) -> Statement
    where
        B: QueryBuilder,
    {
        self.as_query().build(builder).into()
    }

    pub async fn one(mut self, db: &Database) -> Result<M, QueryErr> {
        let builder = db.get_query_builder_backend();
        self.query().limit(1);
        let row = db.get_connection().query_one(self.build(builder)).await?;
        Ok(M::from_query_result(row)?)
    }

    pub async fn all(self, db: &Database) -> Result<Vec<M>, QueryErr> {
        let builder = db.get_query_builder_backend();
        let rows = db.get_connection().query_all(self.build(builder)).await?;
        let mut models = Vec::new();
        for row in rows.into_iter() {
            models.push(M::from_query_result(row)?);
        }
        Ok(models)
    }
}
