use crate::query::combine;
use crate::{
    Connection, Database, EntityTrait, FromQueryResult, QueryErr, Select, SelectTwo, Statement, ModelTrait, QueryResult
};
use sea_query::{QueryBuilder, SelectStatement};
use std::marker::PhantomData;
use serde_json::Value as JsonValue;

#[derive(Clone, Debug)]
pub struct SelectJson<M>
where
    M: ModelTrait,
{
    pub(crate) query: SelectStatement,
    pub(crate) model: PhantomData<M>,
}

impl<M> SelectJson<M>
where
    M: ModelTrait,
{
    pub fn build<B>(&self, builder: B) -> Statement
    where
        B: QueryBuilder,
    {
        self.query.build(builder).into()
    }

    fn as_json_value(row: &QueryResult) -> Result<JsonValue, QueryErr> {
        let model = M::from_query_result(&row, "")?;
        let json_value = serde_json::to_value(model).map_err(|e| QueryErr)?; // TODO: Error handling
        Ok(json_value)
    }

    pub async fn one(mut self, db: &Database) -> Result<JsonValue, QueryErr> {
        let builder = db.get_query_builder_backend();
        self.query.limit(1);
        let row = db.get_connection().query_one(self.build(builder)).await?;
        Self::as_json_value(&row)
    }

    pub async fn all(self, db: &Database) -> Result<JsonValue, QueryErr> {
        let builder = db.get_query_builder_backend();
        let rows = db.get_connection().query_all(self.build(builder)).await?;
        let mut json_values = Vec::new();
        for row in rows.into_iter() {
            json_values.push(Self::as_json_value(&row)?);
        }
        Ok(JsonValue::Array(json_values))
    }
}
