use crate::{
    ActiveModelTrait, Connection, Database, EntityTrait, ExecErr, Statement, UpdateMany, UpdateOne,
};
use sea_query::{QueryBuilder, UpdateStatement};
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
    pub fn exec(self, db: &'a Database) -> impl Future<Output = Result<A, ExecErr>> + 'a {
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
        db: &'a Database,
    ) -> impl Future<Output = Result<UpdateResult, ExecErr>> + 'a {
        // so that self is dropped before entering await
        exec_update_only(self.query, db)
    }
}

impl Updater {
    pub fn new(query: UpdateStatement) -> Self {
        Self { query }
    }

    pub fn build<B>(&self, builder: B) -> Statement
    where
        B: QueryBuilder,
    {
        self.query.build(builder).into()
    }

    pub fn exec(self, db: &Database) -> impl Future<Output = Result<UpdateResult, ExecErr>> + '_ {
        let builder = db.get_query_builder_backend();
        exec_update(self.build(builder), db)
    }
}

async fn exec_update_only(query: UpdateStatement, db: &Database) -> Result<UpdateResult, ExecErr> {
    Updater::new(query).exec(db).await
}

async fn exec_update_and_return_original<A>(
    query: UpdateStatement,
    model: A,
    db: &Database,
) -> Result<A, ExecErr>
where
    A: ActiveModelTrait,
{
    Updater::new(query).exec(db).await?;
    Ok(model)
}

// Only Statement impl Send
async fn exec_update(statement: Statement, db: &Database) -> Result<UpdateResult, ExecErr> {
    let result = db.get_connection().execute(statement).await?;
    Ok(UpdateResult {
        rows_affected: result.rows_affected(),
    })
}
