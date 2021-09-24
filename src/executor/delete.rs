use crate::{error::*, ActiveModelTrait, DatabaseConnection, DeleteMany, DeleteOne, EntityTrait, Statement, IntoDbBackend};
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
        db: &'a DatabaseConnection,
    ) -> impl Future<Output = Result<DeleteResult, DbErr>> + 'a {
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
        db: &'a DatabaseConnection,
    ) -> impl Future<Output = Result<DeleteResult, DbErr>> + 'a {
        // so that self is dropped before entering await
        exec_delete_only(self.query, db)
    }
}

impl Deleter {
    pub fn new(query: DeleteStatement) -> Self {
        Self { query }
    }

    pub fn exec(
        self,
        db: &DatabaseConnection,
    ) -> impl Future<Output = Result<DeleteResult, DbErr>> + '_ {
        let builder = db.get_database_backend();
        exec_delete(builder.build(&self.query), db)
    }
}

async fn exec_delete_only(
    query: DeleteStatement,
    db: &DatabaseConnection,
) -> Result<DeleteResult, DbErr> {
    Deleter::new(query).exec(db).await
}

// Only Statement impl Send
async fn exec_delete(statement: Statement, db: &DatabaseConnection) -> Result<DeleteResult, DbErr> {
    let result = db.execute(statement).await?;
    Ok(DeleteResult {
        rows_affected: result.rows_affected(),
    })
}
