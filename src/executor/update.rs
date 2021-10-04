use crate::{
    error::*, ActiveModelTrait, ConnectionTrait, EntityTrait, Statement, UpdateMany, UpdateOne,
};
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
    pub async fn exec<'b, C>(self, db: &'b C) -> Result<A, DbErr>
    where
        C: ConnectionTrait<'b>,
    {
        // so that self is dropped before entering await
        exec_update_and_return_original(self.query, self.model, db).await
    }
}

impl<'a, E> UpdateMany<E>
where
    E: EntityTrait,
{
    pub fn exec<C>(self, db: &'a C) -> impl Future<Output = Result<UpdateResult, DbErr>> + 'a
    where
        C: ConnectionTrait<'a>,
    {
        // so that self is dropped before entering await
        exec_update_only(self.query, db)
    }
}

impl Updater {
    pub fn new(query: UpdateStatement) -> Self {
        Self { query }
    }

    pub async fn exec<'a, C>(self, db: &'a C) -> Result<UpdateResult, DbErr>
    where
        C: ConnectionTrait<'a>,
    {
        let builder = db.get_database_backend();
        exec_update(builder.build(&self.query), db).await
    }
}

async fn exec_update_only<'a, C>(query: UpdateStatement, db: &'a C) -> Result<UpdateResult, DbErr>
where
    C: ConnectionTrait<'a>,
{
    Updater::new(query).exec(db).await
}

async fn exec_update_and_return_original<'a, A, C>(
    query: UpdateStatement,
    model: A,
    db: &'a C,
) -> Result<A, DbErr>
where
    A: ActiveModelTrait,
    C: ConnectionTrait<'a>,
{
    Updater::new(query).exec(db).await?;
    Ok(model)
}

// Only Statement impl Send
async fn exec_update<'a, C>(statement: Statement, db: &'a C) -> Result<UpdateResult, DbErr>
where
    C: ConnectionTrait<'a>,
{
    let result = db.execute(statement).await?;
    Ok(UpdateResult {
        rows_affected: result.rows_affected(),
    })
}
