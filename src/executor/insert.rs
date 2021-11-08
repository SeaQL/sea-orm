use crate::{
    error::*, ActiveModelTrait, ConnectionTrait, EntityTrait, Insert, IntoActiveModel, Iterable,
    PrimaryKeyTrait, SelectModel, SelectorRaw, Statement, TryFromU64,
};
use sea_query::{FromValueTuple, Iden, InsertStatement, IntoColumnRef, Returning, ValueTuple};
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
        if db.support_returning() && <A::Entity as EntityTrait>::PrimaryKey::iter().count() > 0 {
            query.returning(Returning::Columns(
                <A::Entity as EntityTrait>::PrimaryKey::iter()
                    .map(|c| c.into_column_ref())
                    .collect(),
            ));
        }
        Inserter::<A>::new(self.primary_key, query).exec(db)
    }

    /// Execute an insert operation and return the inserted model (use `RETURNING` syntax if database supported)
    pub fn exec_with_returning<'a, C>(
        self,
        db: &'a C,
    ) -> impl Future<Output = Result<A, DbErr>> + '_
    where
        <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
        C: ConnectionTrait<'a>,
        A: 'a,
    {
        Inserter::<A>::new(self.primary_key, self.query).exec_with_returning(db)
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

    /// Execute an insert operation and return the inserted model (use `RETURNING` syntax if database supported)
    pub fn exec_with_returning<'a, C>(
        self,
        db: &'a C,
    ) -> impl Future<Output = Result<A, DbErr>> + '_
    where
        <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
        C: ConnectionTrait<'a>,
        A: 'a,
    {
        exec_insert_with_returning(self.primary_key, self.query, db)
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
    let last_insert_id_opt = match db.support_returning() {
        true => {
            let cols = PrimaryKey::<A>::iter()
                .map(|col| col.to_string())
                .collect::<Vec<_>>();
            let res = db.query_one(statement).await?.unwrap();
            res.try_get_many("", cols.as_ref()).ok()
        }
        false => {
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

async fn exec_insert_with_returning<'a, A, C>(
    primary_key: Option<ValueTuple>,
    mut insert_statement: InsertStatement,
    db: &'a C,
) -> Result<A, DbErr>
where
    <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
    C: ConnectionTrait<'a>,
    A: ActiveModelTrait,
{
    let db_backend = db.get_database_backend();
    let found = match db.support_returning() {
        true => {
            insert_statement.returning(Returning::Columns(
                <A::Entity as EntityTrait>::Column::iter()
                    .map(|c| c.into_column_ref())
                    .collect(),
            ));
            SelectorRaw::<SelectModel<<A::Entity as EntityTrait>::Model>>::from_statement(
                db_backend.build(&insert_statement),
            )
            .one(db)
            .await?
        }
        false => {
            let insert_res =
                exec_insert::<A, _>(primary_key, db_backend.build(&insert_statement), db).await?;
            <A::Entity as EntityTrait>::find_by_id(insert_res.last_insert_id)
                .one(db)
                .await?
        }
    };
    match found {
        Some(model) => Ok(model.into_active_model()),
        None => Err(DbErr::Exec("Failed to find inserted item".to_owned())),
    }
}
