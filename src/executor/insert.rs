use crate::{ActiveModelTrait, DatabaseConnection, DbConnection, EntityTrait, Insert, PrimaryKeyTrait, Statement, TryFromU64, error::*};
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
            use crate::{sea_query::Query, Iterable};
            if <A::Entity as EntityTrait>::PrimaryKey::iter().count() > 0 {
                query.returning(
                    Query::select()
                        .columns(<A::Entity as EntityTrait>::PrimaryKey::iter())
                        .take(),
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
    type PrimaryKey<A> = <<A as ActiveModelTrait>::Entity as EntityTrait>::PrimaryKey;
    type ValueTypeOf<A> = <PrimaryKey<A> as PrimaryKeyTrait>::ValueType;
    let last_insert_id = match db {
        #[cfg(feature = "sqlx-postgres")]
        DatabaseConnection::SqlxPostgresPoolConnection(conn) => {
            use crate::{sea_query::Iden, Iterable};
            let cols = PrimaryKey::<A>::iter()
                .map(|col| col.to_string())
                .collect::<Vec<_>>();
            let res = conn.query_one(statement).await?.unwrap();
            res.try_get_many("", cols.as_ref()).unwrap_or_default()
        }
        _ => {
            let last_insert_id = db.execute(statement).await?.last_insert_id();
            ValueTypeOf::<A>::try_from_u64(last_insert_id)
                .ok()
                .unwrap_or_default()
        }
    };
    Ok(InsertResult { last_insert_id })
}
