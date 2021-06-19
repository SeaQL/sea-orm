use crate::{ActiveModelTrait, Database, DeleteMany, DeleteOne, EntityTrait, ExecErr, Statement};
use sea_query::DeleteStatement;
use std::future::Future;

#[derive(Clone, Debug)]
pub struct Deleter {
    query: DeleteStatement,
}

#[derive(Clone, Debug)]
pub struct DeleteResult {
    pub rows_affected: u64,
}

impl<'a, A: 'a> DeleteOne<A>
where
    A: ActiveModelTrait,
{
    pub fn exec(
        self,
        db: &'a Database,
    ) -> impl Future<Output = Result<DeleteResult, ExecErr>> + 'a {
        // so that self is dropped before entering await
        exec_delete_only(self.query, db)
    }
}

impl<'a, E> DeleteMany<E>
where
    E: EntityTrait,
{
    pub fn exec(
        self,
        db: &'a Database,
    ) -> impl Future<Output = Result<DeleteResult, ExecErr>> + 'a {
        // so that self is dropped before entering await
        exec_delete_only(self.query, db)
    }
}

impl Deleter {
    pub fn new(query: DeleteStatement) -> Self {
        Self { query }
    }

    pub fn exec(self, db: &Database) -> impl Future<Output = Result<DeleteResult, ExecErr>> + '_ {
        let builder = db.get_query_builder_backend();
        exec_delete(builder.build(&self.query), db)
    }
}

async fn exec_delete_only(query: DeleteStatement, db: &Database) -> Result<DeleteResult, ExecErr> {
    Deleter::new(query).exec(db).await
}

// Only Statement impl Send
async fn exec_delete(statement: Statement, db: &Database) -> Result<DeleteResult, ExecErr> {
    let result = db.get_connection().execute(statement).await?;
    Ok(DeleteResult {
        rows_affected: result.rows_affected(),
    })
}
