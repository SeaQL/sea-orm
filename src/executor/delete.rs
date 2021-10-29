use crate::{
    error::*, ActiveModelTrait, ConnectionTrait, DeleteMany, DeleteOne, EntityTrait, Statement,
    StatementBuilder,
};
use std::future::Future;

#[derive(Clone, Debug)]
pub struct Deleter<Q>
where
    Q: StatementBuilder,
{
    query: Q,
}

#[derive(Clone, Debug)]
pub struct DeleteResult {
    pub rows_affected: u64,
}

impl<'a, A: 'a> DeleteOne<A>
where
    A: ActiveModelTrait,
{
    pub fn exec<C>(self, db: &'a C) -> impl Future<Output = Result<DeleteResult, DbErr>> + '_
    where
        C: ConnectionTrait<'a>,
    {
        // so that self is dropped before entering await
        exec_delete_only(self.query, db)
    }
}

impl<'a, E> DeleteMany<E>
where
    E: EntityTrait,
{
    pub fn exec<C>(self, db: &'a C) -> impl Future<Output = Result<DeleteResult, DbErr>> + '_
    where
        C: ConnectionTrait<'a>,
    {
        // so that self is dropped before entering await
        exec_delete_only(self.query, db)
    }
}

impl<Q> Deleter<Q>
where
    Q: StatementBuilder,
{
    pub fn new(query: Q) -> Self {
        Self { query }
    }

    pub fn exec<'a, C>(self, db: &'a C) -> impl Future<Output = Result<DeleteResult, DbErr>> + '_
    where
        C: ConnectionTrait<'a>,
    {
        let builder = db.get_database_backend();
        exec_delete(builder.build(&self.query), db)
    }
}

async fn exec_delete_only<'a, C, Q>(query: Q, db: &'a C) -> Result<DeleteResult, DbErr>
where
    C: ConnectionTrait<'a>,
    Q: StatementBuilder,
{
    Deleter::new(query).exec(db).await
}

async fn exec_delete<'a, C>(statement: Statement, db: &'a C) -> Result<DeleteResult, DbErr>
where
    C: ConnectionTrait<'a>,
{
    let result = db.execute(statement).await?;
    Ok(DeleteResult {
        rows_affected: result.rows_affected(),
    })
}
