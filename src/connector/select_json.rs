use crate::query::combine;
use crate::{Connection, Database, QueryErr, Statement};
use sea_query::{QueryBuilder, SelectStatement};
use serde_json::Value as JsonValue;

#[derive(Clone, Debug)]
pub struct SelectJson {
    pub(crate) query: SelectStatement,
}

impl SelectJson {
    pub fn build<B>(&self, builder: B) -> Statement
    where
        B: QueryBuilder,
    {
        self.query.build(builder).into()
    }

    pub async fn one(mut self, db: &Database) -> Result<JsonValue, QueryErr> {
        let builder = db.get_query_builder_backend();
        self.query.limit(1);
        // TODO: Error handling
        db.get_connection().query_one(self.build(builder)).await?.as_json("").map_err(|_e| QueryErr)
    }

    pub async fn all(self, db: &Database) -> Result<JsonValue, QueryErr> {
        let builder = db.get_query_builder_backend();
        let rows = db.get_connection().query_all(self.build(builder)).await?;
        let mut values = Vec::new();
        for row in rows.into_iter() {
            // TODO: Error handling
            values.push(row.as_json("").map_err(|_e| QueryErr)?);
        }
        Ok(JsonValue::Array(values))
    }
}

#[derive(Clone, Debug)]
pub struct SelectTwoJson {
    pub(crate) query: SelectStatement,
}

impl SelectTwoJson {
    pub fn build<B>(&self, builder: B) -> Statement
    where
        B: QueryBuilder,
    {
        self.query.build(builder).into()
    }

    pub async fn one(mut self, db: &Database) -> Result<JsonValue, QueryErr> {
        let builder = db.get_query_builder_backend();
        self.query.limit(1);
        let row = db.get_connection().query_one(self.build(builder)).await?;
        Ok(JsonValue::Array(vec![
            // TODO: Error handling
            row.as_json(combine::SELECT_A).map_err(|_e| QueryErr)?,
            row.as_json(combine::SELECT_B).map_err(|_e| QueryErr)?,
        ]))
    }

    pub async fn all(self, db: &Database) -> Result<JsonValue, QueryErr> {
        let builder = db.get_query_builder_backend();
        let rows = db.get_connection().query_all(self.build(builder)).await?;
        let mut json_values = Vec::new();
        for row in rows.into_iter() {
            json_values.push(JsonValue::Array(vec![
                // TODO: Error handling
                row.as_json(combine::SELECT_A).map_err(|_e| QueryErr)?,
                row.as_json(combine::SELECT_B).map_err(|_e| QueryErr)?,
            ]));
        }
        Ok(JsonValue::Array(json_values))
    }
}
