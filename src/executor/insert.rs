use crate::{error::*, ActiveModelTrait, DatabaseConnection, Insert, Statement};
use sea_query::InsertStatement;
use std::future::Future;

#[derive(Clone, Debug)]
pub struct Inserter {
    query: InsertStatement,
}

#[derive(Clone, Debug)]
pub struct InsertResult {
    pub last_insert_id: u64,
}

impl<A> Insert<A>
where
    A: ActiveModelTrait,
{
    #[allow(unused_mut)]
    pub fn exec(
        self,
        db: &DatabaseConnection,
    ) -> impl Future<Output = Result<InsertResult, DbErr>> + '_ {
        // so that self is dropped before entering await
        let mut query = self.query;
        #[cfg(feature = "sqlx-postgres")]
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

    pub fn exec(
        self,
        db: &DatabaseConnection,
    ) -> impl Future<Output = Result<InsertResult, DbErr>> + '_ {
        let builder = db.get_database_backend();
        exec_insert(builder.build(&self.query), db)
    }
}

// Only Statement impl Send
async fn exec_insert(statement: Statement, db: &DatabaseConnection) -> Result<InsertResult, DbErr> {
    // TODO: Postgres instead use query_one + returning clause
    let result = match db {
        #[cfg(feature = "sqlx-postgres")]
        DatabaseConnection::SqlxPostgresPoolConnection(conn) => {
            let res = conn.query_one(statement).await?.unwrap();
            crate::query_result_into_exec_result(res)?
        }
        _ => db.execute(statement).await?,
    };
    Ok(InsertResult {
        last_insert_id: result.last_insert_id(),
    })
}
