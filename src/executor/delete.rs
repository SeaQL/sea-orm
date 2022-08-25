use crate::{
    error::*, ActiveModelTrait, ConnectionTrait, DeleteMany, DeleteOne, EntityTrait, Statement,
};
use sea_query::{DeleteStatement, UpdateStatement};
use std::future::Future;

/// Handles DELETE operations in a ActiveModel using [DeleteStatement]
#[derive(Clone, Debug)]
pub enum Deleter {
    /// Force delete
    Force {
        /// Delete statement
        query: DeleteStatement,
    },
    /// Soft delete
    Soft {
        /// Update statement
        query: UpdateStatement,
    },
}

/// The result of a DELETE operation
#[derive(Clone, Debug, PartialEq, Eq)]
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
        C: ConnectionTrait,
    {
        // so that self is dropped before entering await
        match self {
            DeleteOne::Force { query, .. } => Deleter::force(query).exec(db),
            DeleteOne::Soft { query, .. } => Deleter::soft(query).exec(db),
        }
    }
}

impl<'a, E> DeleteMany<E>
where
    E: EntityTrait,
{
    /// Execute a DELETE operation on many ActiveModels
    pub fn exec<C>(self, db: &'a C) -> impl Future<Output = Result<DeleteResult, DbErr>> + '_
    where
        C: ConnectionTrait,
    {
        // so that self is dropped before entering await
        match self {
            DeleteMany::Force { query, .. } => Deleter::force(query).exec(db),
            DeleteMany::Soft { query, .. } => Deleter::soft(query).exec(db),
        }
    }
}

impl Deleter {
    /// Instantiate a new [Deleter] by passing it a [DeleteStatement]
    pub fn new(query: DeleteStatement) -> Self {
        Self::force(query)
    }

    /// Instantiate a force deleter
    pub fn force(query: DeleteStatement) -> Self {
        Self::Force { query }
    }

    /// Instantiate a soft deleter
    pub fn soft(query: UpdateStatement) -> Self {
        Self::Soft { query }
    }

    /// Execute a DELETE operation
    pub fn exec<'a, C>(self, db: &'a C) -> impl Future<Output = Result<DeleteResult, DbErr>> + '_
    where
        C: ConnectionTrait,
    {
        let builder = db.get_database_backend();
        let stmt = match self {
            Deleter::Force { query } => builder.build(&query),
            Deleter::Soft { query } => builder.build(&query),
        };
        exec_delete(stmt, db)
    }
}

async fn exec_delete<C>(statement: Statement, db: &C) -> Result<DeleteResult, DbErr>
where
    C: ConnectionTrait,
{
    let result = db.execute(statement).await?;
    Ok(DeleteResult {
        rows_affected: result.rows_affected(),
    })
}
