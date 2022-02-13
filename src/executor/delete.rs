use crate::{
    error::*, ActiveModelTrait, ConnectionTrait, DeleteMany, DeleteOne, EntityTrait, Statement,
};
use sea_query::DeleteStatement;
use std::future::Future;

/// Handles DELETE operations in a ActiveModel using [DeleteStatement]
#[derive(Clone, Debug)]
pub struct Deleter {
    query: DeleteStatement,
}

/// The result of a DELETE operation
#[derive(Clone, Debug)]
pub struct DeleteResult {
    /// The number of rows affected by the DELETE operation
    pub rows_affected: u64,
}

impl<'a, A: 'a> DeleteOne<A>
where
    A: ActiveModelTrait,
{
    /// Execute a DELETE operation on one ActiveModel
    pub fn exec<C>(self, db: &'a C) -> impl Future<Output = Result<DeleteResult, DbErr>> + '_
    where
        C: ConnectionTrait + ?Sized,
    {
        // so that self is dropped before entering await
        exec_delete_only(self.query, db)
    }
}

impl<'a, E> DeleteMany<E>
where
    E: EntityTrait,
{
    /// Execute a DELETE operation on many ActiveModels
    pub fn exec<C>(self, db: &'a C) -> impl Future<Output = Result<DeleteResult, DbErr>> + '_
    where
        C: ConnectionTrait + ?Sized,
    {
        // so that self is dropped before entering await
        exec_delete_only(self.query, db)
    }
}

impl Deleter {
    /// Instantiate a new [Deleter] by passing it a [DeleteStatement]
    pub fn new(query: DeleteStatement) -> Self {
        Self { query }
    }

    /// Execute a DELETE operation
    pub fn exec<'a, C>(self, db: &'a C) -> impl Future<Output = Result<DeleteResult, DbErr>> + '_
    where
        C: ConnectionTrait + ?Sized,
    {
        let builder = db.get_database_backend();
        exec_delete(builder.build(&self.query), db)
    }
}

async fn exec_delete_only<'a, C>(query: DeleteStatement, db: &'a C) -> Result<DeleteResult, DbErr>
where
    C: ConnectionTrait + ?Sized,
{
    Deleter::new(query).exec(db).await
}

async fn exec_delete<'a, C>(statement: Statement, db: &'a C) -> Result<DeleteResult, DbErr>
where
    C: ConnectionTrait + ?Sized,
{
    let result = db.execute(statement).await?;
    Ok(DeleteResult {
        rows_affected: result.rows_affected(),
    })
}
