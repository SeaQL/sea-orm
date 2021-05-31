use crate::{Connection, Database, ExecErr, Statement};
use sea_query::{InsertStatement, QueryBuilder};

#[derive(Clone, Debug)]
pub struct Inserter {
    query: InsertStatement,
}

#[derive(Clone, Debug)]
pub struct InsertResult {
    pub last_insert_id: u64,
}

impl Inserter {
    pub fn new(query: InsertStatement) -> Self {
        Self { query }
    }

    pub fn build<B>(&self, builder: B) -> Statement
    where
        B: QueryBuilder,
    {
        self.query.build(builder).into()
    }

    pub async fn exec(self, db: &Database) -> Result<InsertResult, ExecErr> {
        let builder = db.get_query_builder_backend();
        let result = db.get_connection().execute(self.build(builder)).await?;
        // TODO: Postgres instead use query_one + returning clause
        Ok(InsertResult {
            last_insert_id: result.last_insert_id(),
        })
    }
}
