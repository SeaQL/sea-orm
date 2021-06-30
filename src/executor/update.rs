use crate::{
    error::*, ActiveModelTrait, DatabaseConnection, EntityTrait, Statement, UpdateMany, UpdateOne,
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
    pub fn exec(
        self,
        db: &'a DatabaseConnection,
    ) -> impl Future<Output = Result<A, SeaErr>> + 'a {
        // so that self is dropped before entering await
        exec_update_and_return_original(self.query, self.model, db)
    }
}

impl<'a, E> UpdateMany<E>
where
    E: EntityTrait,
{
    pub fn exec(
        self,
        db: &'a DatabaseConnection,
    ) -> impl Future<Output = Result<UpdateResult, SeaErr>> + 'a {
        // so that self is dropped before entering await
        exec_update_only(self.query, db)
    }
}

impl Updater {
    pub fn new(query: UpdateStatement) -> Self {
        Self { query }
    }

    pub fn exec(
        self,
        db: &DatabaseConnection,
    ) -> impl Future<Output = Result<UpdateResult, SeaErr>> + '_ {
        let builder = db.get_query_builder_backend();
        exec_update(builder.build(&self.query), db)
    }
}

async fn exec_update_only(
    query: UpdateStatement,
    db: &DatabaseConnection,
) -> Result<UpdateResult, SeaErr> {
    Updater::new(query).exec(db).await
}

async fn exec_update_and_return_original<A>(
    query: UpdateStatement,
    model: A,
    db: &DatabaseConnection,
) -> Result<A, SeaErr>
where
    A: ActiveModelTrait,
{
    Updater::new(query).exec(db).await?;
    Ok(model)
}

// Only Statement impl Send
async fn exec_update(
    statement: Statement,
    db: &DatabaseConnection,
) -> Result<UpdateResult, SeaErr> {
    let result = db.execute(statement).await?;
    Ok(UpdateResult {
        rows_affected: result.rows_affected(),
    })
}
