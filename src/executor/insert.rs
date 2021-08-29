use crate::{
    error::*, ActiveModelTrait, DatabaseConnection, EntityTrait, Insert, PrimaryKeyTrait, Statement,
};
use sea_query::InsertStatement;
use std::{future::Future, marker::PhantomData};

#[derive(Clone, Debug)]
pub struct Inserter<A>
where
    A: ActiveModelTrait,
{
    query: InsertStatement,
    model: PhantomData<A>,
}

#[derive(Debug)]
pub struct InsertResult<A>
where
    A: ActiveModelTrait,
{
    pub last_insert_id: <<<A as ActiveModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
}

impl<A> Insert<A>
where
    A: ActiveModelTrait,
{
    #[allow(unused_mut)]
    pub fn exec<'a>(
        self,
        db: &'a DatabaseConnection,
    ) -> impl Future<Output = Result<InsertResult<A>, DbErr>> + 'a
    where
        A: 'a,
    {
        // so that self is dropped before entering await
        let mut query = self.query;
        #[cfg(feature = "sqlx-postgres")]
        if let DatabaseConnection::SqlxPostgresPoolConnection(_) = db {
            use crate::Iterable;
            use sea_query::{Alias, Expr, Query};
            for key in <A::Entity as EntityTrait>::PrimaryKey::iter() {
                query.returning(
                    Query::select()
                        .expr_as(Expr::col(key), Alias::new("last_insert_id"))
                        .to_owned(),
                );
            }
        }
        Inserter::<A>::new(query).exec(db)
    }
}

impl<A> Inserter<A>
where
    A: ActiveModelTrait,
{
    pub fn new(query: InsertStatement) -> Self {
        Self {
            query,
            model: PhantomData,
        }
    }

    pub fn exec<'a>(
        self,
        db: &'a DatabaseConnection,
    ) -> impl Future<Output = Result<InsertResult<A>, DbErr>> + 'a
    where
        A: 'a,
    {
        let builder = db.get_database_backend();
        exec_insert(builder.build(&self.query), db)
    }
}

// Only Statement impl Send
async fn exec_insert<A>(
    statement: Statement,
    db: &DatabaseConnection,
) -> Result<InsertResult<A>, DbErr>
where
    A: ActiveModelTrait,
{
    // TODO: Postgres instead use query_one + returning clause
    let last_insert_id = match db {
        #[cfg(feature = "sqlx-postgres")]
        DatabaseConnection::SqlxPostgresPoolConnection(conn) => {
            let res = conn.query_one(statement).await?.unwrap();
            res.try_get("", "last_insert_id").unwrap_or_default()
        }
        _ => {
            db.execute(statement).await?
            .last_insert_id()
            .to_string()
            .parse()
            .unwrap_or_default()
        },
    };
    Ok(InsertResult {
        last_insert_id,
    })
}
