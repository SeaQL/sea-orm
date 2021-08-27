use crate::{error::*, ActiveModelTrait, DatabaseConnection, Insert, Statement};
use sea_query::InsertStatement;
use std::future::Future;

#[derive(Clone, Debug)]
pub struct Inserter {
    query: InsertStatement,
}

#[cfg(any(feature = "sqlx-mysql", feature = "sqlx-sqlite", feature = "mock"))]
#[derive(Clone, Debug)]
pub struct InsertResult {
    pub last_insert_id: u64,
}

#[cfg(feature = "sqlx-postgres")]
#[derive(Clone, Debug)]
pub struct InsertResult<T> {
    pub last_insert_id: Option<T>,
}

impl<A> Insert<A>
where
    A: ActiveModelTrait,
{
    #[cfg(any(feature = "sqlx-mysql", feature = "sqlx-sqlite", feature = "mock"))]
    #[allow(unused_mut)]
    pub fn exec(
        self,
        db: &DatabaseConnection,
    ) -> impl Future<Output = Result<InsertResult, DbErr>> + '_ {
        // so that self is dropped before entering await
        let query = self.query;
        Inserter::new(query).exec(db)
    }

    #[cfg(feature = "sqlx-postgres")]
    #[allow(unused_mut)]
    pub fn exec(
        self,
        db: &DatabaseConnection,
    ) -> impl Future<
        Output = Result<
            InsertResult<<<A::Entity as crate::EntityTrait>::PrimaryKey as crate::PrimaryKeyTrait>::ValueType>,
            DbErr,
        >,
    > + '_{
        // so that self is dropped before entering await
        let mut query = self.query;
        if let DatabaseConnection::SqlxPostgresPoolConnection(_) = db {
            use crate::{EntityTrait, Iterable};
            use sea_query::{Alias, Expr, Query};
            for key in <A::Entity as EntityTrait>::PrimaryKey::iter() {
                query.returning(
                    Query::select()
                        .expr_as(Expr::col(key), Alias::new("last_insert_id"))
                        .to_owned(),
                );
            }
        }
        Inserter::new(query).exec(db)
    }
}

impl Inserter {
    pub fn new(query: InsertStatement) -> Self {
        Self { query }
    }

    #[cfg(any(feature = "sqlx-mysql", feature = "sqlx-sqlite", feature = "mock"))]
    pub fn exec(
        self,
        db: &DatabaseConnection,
    ) -> impl Future<Output = Result<InsertResult, DbErr>> + '_ {
        let builder = db.get_database_backend();
        exec_insert(builder.build(&self.query), db)
    }

    #[cfg(feature = "sqlx-postgres")]
    pub fn exec<'a, T: 'a>(
        self,
        db: &'a DatabaseConnection,
    ) -> impl Future<Output = Result<InsertResult<T>, DbErr>> + 'a
    where
        T: TryGetable + Clone,
    {
        let builder = db.get_database_backend();
        exec_insert(builder.build(&self.query), db)
    }
}

// Only Statement impl Send
#[cfg(any(feature = "sqlx-mysql", feature = "sqlx-sqlite", feature = "mock"))]
async fn exec_insert(statement: Statement, db: &DatabaseConnection) -> Result<InsertResult, DbErr> {
    // TODO: Postgres instead use query_one + returning clause
    let result = db.execute(statement).await?;
    Ok(InsertResult {
        last_insert_id: result.last_insert_id(),
    })
}

#[cfg(feature = "sqlx-postgres")]
async fn exec_insert<T>(
    statement: Statement,
    db: &DatabaseConnection,
) -> Result<InsertResult<T>, DbErr>
where
    T: TryGetable + Clone,
{
    // TODO: Postgres instead use query_one + returning clause
    let result = {
        let res = db.query_one(statement).await?.unwrap();
        crate::query_result_into_exec_result(res)?
    };
    Ok(InsertResult {
        last_insert_id: result.last_insert_id().clone(),
    })
}
