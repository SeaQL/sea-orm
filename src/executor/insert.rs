use crate::{
    error::*, ActiveModelTrait, ConnectionTrait, DbBackend, EntityTrait, Insert, PrimaryKeyTrait,
    Statement, TryFromU64,
};
use sea_query::{FromValueTuple, InsertStatement, ValueTuple};
use std::{future::Future, marker::PhantomData};

/// Defines a structure to perform INSERT operations in an ActiveModel
#[derive(Debug)]
pub struct Inserter<A>
where
    A: ActiveModelTrait,
{
    primary_key: Option<ValueTuple>,
    query: InsertStatement,
    model: PhantomData<A>,
}

/// The result of an INSERT operation on an ActiveModel
#[derive(Debug)]
pub struct InsertResult<A>
where
    A: ActiveModelTrait,
{
    /// The id performed when AUTOINCREMENT was performed on the PrimaryKey
    pub last_insert_id: <<<A as ActiveModelTrait>::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
}

impl<A> Insert<A>
where
    A: ActiveModelTrait,
{
    /// Execute an insert operation
    #[allow(unused_mut)]
    pub fn exec<'a, C>(self, db: &'a C) -> impl Future<Output = Result<InsertResult<A>, DbErr>> + '_
    where
        C: ConnectionTrait<'a>,
        A: 'a,
    {
        // so that self is dropped before entering await
        let mut query = self.query;
        if db.get_database_backend() == DbBackend::Postgres {
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
    }
}

impl<A> Inserter<A>
where
    A: ActiveModelTrait,
{
    /// Instantiate a new insert operation
    pub fn new(primary_key: Option<ValueTuple>, query: InsertStatement) -> Self {
        Self {
            primary_key,
            query,
            model: PhantomData,
        }
    }

    /// Execute an insert operation
    pub fn exec<'a, C>(self, db: &'a C) -> impl Future<Output = Result<InsertResult<A>, DbErr>> + '_
    where
        C: ConnectionTrait<'a>,
        A: 'a,
    {
        let builder = db.get_database_backend();
        exec_insert(self.primary_key, builder.build(&self.query), db)
    }
}

async fn exec_insert<'a, A, C>(
    primary_key: Option<ValueTuple>,
    statement: Statement,
    db: &'a C,
) -> Result<InsertResult<A>, DbErr>
where
    C: ConnectionTrait<'a>,
    A: ActiveModelTrait,
{
    type PrimaryKey<A> = <<A as ActiveModelTrait>::Entity as EntityTrait>::PrimaryKey;
    type ValueTypeOf<A> = <PrimaryKey<A> as PrimaryKeyTrait>::ValueType;
    let last_insert_id_opt = match db.get_database_backend() {
        DbBackend::Postgres => {
            use crate::{sea_query::Iden, Iterable};
            let cols = PrimaryKey::<A>::iter()
                .map(|col| col.to_string())
                .collect::<Vec<_>>();
            let res = db.query_one(statement).await?.unwrap();
            res.try_get_many("", cols.as_ref()).ok()
        }
        _ => {
            let last_insert_id = db.execute(statement).await?.last_insert_id();
            ValueTypeOf::<A>::try_from_u64(last_insert_id).ok()
        }
    };
    let last_insert_id = match primary_key {
        Some(value_tuple) => FromValueTuple::from_value_tuple(value_tuple),
        None => match last_insert_id_opt {
            Some(last_insert_id) => last_insert_id,
            None => return Err(DbErr::Exec("Fail to unpack last_insert_id".to_owned())),
        },
    };
    Ok(InsertResult { last_insert_id })
}
