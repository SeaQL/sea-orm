use crate::{ActiveModelTrait, DatabaseConnection, ExecErr, Insert, QueryTrait, Statement};
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
    pub fn exec(
        self,
        db: &DatabaseConnection,
    ) -> impl Future<Output = Result<InsertResult, ExecErr>> + '_ {
        // so that self is dropped before entering await
        Inserter::new(self.into_query()).exec(db)
    }
}

impl Inserter {
    pub fn new(query: InsertStatement) -> Self {
        Self { query }
    }

    pub fn exec(
        self,
        db: &DatabaseConnection,
    ) -> impl Future<Output = Result<InsertResult, ExecErr>> + '_ {
        let builder = db.get_query_builder_backend();
        exec_insert(builder.build(&self.query), db)
    }
}

// Only Statement impl Send
async fn exec_insert(
    statement: Statement,
    db: &DatabaseConnection,
) -> Result<InsertResult, ExecErr> {
    let result = db.execute(statement).await?;
    // TODO: Postgres instead use query_one + returning clause
    Ok(InsertResult {
        last_insert_id: result.last_insert_id(),
    })
}
