use crate::{ActiveModelTrait, DbConnection, EntityTrait, Statement, UpdateMany, UpdateOne, error::*};
use sea_query::UpdateStatement;
use std::future::Future;

#[derive(Clone, Debug)]
pub struct Updater {
    query: UpdateStatement,
}

#[derive(Clone, Debug)]
pub struct UpdateResult {
    pub rows_affected: u64,
}

impl<'a, A: 'a> UpdateOne<A>
where
    A: ActiveModelTrait,
{
    pub fn exec<C>(self, db: &'a C) -> impl Future<Output = Result<A, DbErr>> + 'a
    where C: DbConnection {
        // so that self is dropped before entering await
        exec_update_and_return_original(self.query, self.model, db)
    }
}

impl<'a, E> UpdateMany<E>
where
    E: EntityTrait,
{
    pub fn exec<C>(
        self,
        db: &'a C,
    ) -> impl Future<Output = Result<UpdateResult, DbErr>> + 'a
    where C: DbConnection {
        // so that self is dropped before entering await
        exec_update_only(self.query, db)
    }
}

impl Updater {
    pub fn new(query: UpdateStatement) -> Self {
        Self { query }
    }

    pub fn exec<C>(
        self,
        db: &C,
    ) -> impl Future<Output = Result<UpdateResult, DbErr>> + '_
    where C: DbConnection {
        let builder = db.get_database_backend();
        exec_update(builder.build(&self.query), db)
    }
}

async fn exec_update_only<C>(
    query: UpdateStatement,
    db: &C,
) -> Result<UpdateResult, DbErr>
where C: DbConnection {
    Updater::new(query).exec(db).await
}

async fn exec_update_and_return_original<A, C>(
    query: UpdateStatement,
    model: A,
    db: &C,
) -> Result<A, DbErr>
where
    A: ActiveModelTrait,
    C: DbConnection,
{
    Updater::new(query).exec(db).await?;
    Ok(model)
}

// Only Statement impl Send
async fn exec_update<C>(statement: Statement, db: &C) -> Result<UpdateResult, DbErr>
where C: DbConnection {
    let result = db.execute(statement).await?;
    Ok(UpdateResult {
        rows_affected: result.rows_affected(),
    })
}
