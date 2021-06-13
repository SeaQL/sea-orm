use crate::{
    ActiveModelTrait, Database, ExecErr, Insert, QueryBuilderBackend, QueryTrait, Statement,
};
use sea_query::InsertStatement;
use std::future::Future;

#[derive(Clone, Debug)]
pub struct Inserter {
    query: InsertStatement,
}

#[derive(Clone, Debug)]
pub struct InsertResult {
    pub last_insert_id: u64,
}

impl<A> Insert<A>
where
    A: ActiveModelTrait,
{
    pub fn exec(self, db: &Database) -> impl Future<Output = Result<InsertResult, ExecErr>> + '_ {
        // so that self is dropped before entering await
        Inserter::new(self.into_query()).exec(db)
    }
}

impl Inserter {
    pub fn new(query: InsertStatement) -> Self {
        Self { query }
    }

    pub fn build(&self, builder: QueryBuilderBackend) -> Statement {
        builder.build_insert_statement(&self.query)
    }

    pub fn exec(self, db: &Database) -> impl Future<Output = Result<InsertResult, ExecErr>> + '_ {
        let builder = db.get_query_builder_backend();
        exec_insert(self.build(builder), db)
    }
}

// Only Statement impl Send
async fn exec_insert(statement: Statement, db: &Database) -> Result<InsertResult, ExecErr> {
    let result = db.get_connection().execute(statement).await?;
    // TODO: Postgres instead use query_one + returning clause
    Ok(InsertResult {
        last_insert_id: result.last_insert_id(),
    })
}
