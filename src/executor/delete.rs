use crate::{ActiveModelTrait, ConnectionTrait, DeleteMany, DeleteOne, EntityTrait, Statement, error::*};
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
    pub fn exec<C>(
        self,
        db: &'a C,
    ) -> impl Future<Output = Result<DeleteResult, DbErr>> + 'a
    where C: ConnectionTrait {
        // so that self is dropped before entering await
        exec_delete_only(self.query, db)
    }
}

impl<'a, E> DeleteMany<E>
where
    E: EntityTrait,
{
    pub fn exec<C>(
        self,
        db: &'a C,
    ) -> impl Future<Output = Result<DeleteResult, DbErr>> + 'a
    where C: ConnectionTrait {
        // so that self is dropped before entering await
        exec_delete_only(self.query, db)
    }
}

impl Deleter {
    pub fn new(query: DeleteStatement) -> Self {
        Self { query }
    }

    pub fn exec<C>(
        self,
        db: &C,
    ) -> impl Future<Output = Result<DeleteResult, DbErr>> + '_
    where C: ConnectionTrait {
        let builder = db.get_database_backend();
        exec_delete(builder.build(&self.query), db)
    }
}

async fn exec_delete_only<C>(
    query: DeleteStatement,
    db: &C,
) -> Result<DeleteResult, DbErr>
where C: ConnectionTrait {
    Deleter::new(query).exec(db).await
}

// Only Statement impl Send
async fn exec_delete<C>(statement: Statement, db: &C) -> Result<DeleteResult, DbErr>
where C: ConnectionTrait {
    let result = db.execute(statement).await?;
    Ok(DeleteResult {
        rows_affected: result.rows_affected(),
    })
}
