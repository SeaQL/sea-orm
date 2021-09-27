use crate::{
    error::*, ActiveModelTrait, DatabaseConnection, EntityTrait, Insert, PrimaryKeyTrait,
    Statement, TryFromU64,
};
use sea_query::InsertStatement;
use std::{future::Future, marker::PhantomData};

#[derive(Debug)]
pub struct Inserter<A>
where
    A: ActiveModelTrait,
{
    primary_key: Option<<<<A as ActiveModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>,
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
        // TODO: extract primary key's value from query
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
        Inserter::<A>::new(self.primary_key, query).exec(db)
        // TODO: return primary key if extracted before, otherwise use InsertResult
    }
}

impl<A> Inserter<A>
where
    A: ActiveModelTrait,
{
    pub fn new(
        primary_key: Option<<<<A as ActiveModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>,
        query: InsertStatement,
    ) -> Self {
        Self {
            primary_key,
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
        exec_insert(self.primary_key, builder.build(&self.query), db)
    }
}

// Only Statement impl Send
async fn exec_insert<A>(
    primary_key: Option<<<<A as ActiveModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    statement: Statement,
    db: &DatabaseConnection,
) -> Result<InsertResult<A>, DbErr>
where
    A: ActiveModelTrait,
{
    type PrimaryKey<A> = <<A as ActiveModelTrait>::Entity as EntityTrait>::PrimaryKey;
    type ValueTypeOf<A> = <PrimaryKey<A> as PrimaryKeyTrait>::ValueType;
    let last_insert_id_opt = match db {
        #[cfg(feature = "sqlx-postgres")]
        DatabaseConnection::SqlxPostgresPoolConnection(conn) => {
            use crate::{sea_query::Iden, Iterable};
            let cols = PrimaryKey::<A>::iter()
                .map(|col| col.to_string())
                .collect::<Vec<_>>();
            let res = conn.query_one(statement).await?.unwrap();
            Some(res.try_get_many("", cols.as_ref()).unwrap_or_default())
        }
        _ => {
            let last_insert_id = db.execute(statement).await?.last_insert_id();
            ValueTypeOf::<A>::try_from_u64(last_insert_id).ok()
        }
    };
    let last_insert_id = match last_insert_id_opt {
        Some(last_insert_id) => last_insert_id,
        None => match primary_key {
            Some(primary_key) => primary_key,
            None => return Err(DbErr::Exec("Fail to unpack last_insert_id".to_owned())),
        },
    };
    Ok(InsertResult { last_insert_id })
}
