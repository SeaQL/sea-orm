use crate::{
    error::*, ActiveModelTrait, ColumnTrait, ConnectionTrait, DeleteMany, DeleteOne, EntityTrait,
    IntoActiveModel, Iterable, PrimaryKeyTrait, SelectModel, SelectorRaw, Statement,
};
use sea_query::{DeleteStatement, Expr, FromValueTuple, Query};
use std::future::Future;

/// Handles DELETE operations in a ActiveModel using [DeleteStatement]
#[derive(Clone, Debug)]
pub struct Deleter {
    query: DeleteStatement,
}

/// The result of a DELETE operation
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeleteResult {
    /// The number of rows affected by the DELETE operation
    pub rows_affected: u64,
}

impl<'a, A: 'a> DeleteOne<A>
where
    A: ActiveModelTrait,
{
    /// Execute a DELETE operation on one ActiveModel
    pub fn exec<C>(self, db: &'a C) -> impl Future<Output = Result<DeleteResult, DbErr>> + '_
    where
        C: ConnectionTrait,
    {
        // so that self is dropped before entering await
        exec_delete_only(self.query, db)
    }

    /// Execute a DELETE operation on one ActiveModel and return the deleted model (use `RETURNING` syntax if database supported)
    pub async fn exec_with_returning<'b, C>(
        self,
        db: &'b C,
    ) -> Result<<A::Entity as EntityTrait>::Model, DbErr>
    where
        <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
        C: ConnectionTrait,
    {
        Deleter::new(self.query)
            .exec_delete_and_return_deleted(self.model, db)
            .await
    }
}

impl<'a, E> DeleteMany<E>
where
    E: EntityTrait,
{
    /// Execute a DELETE operation on many ActiveModels
    pub fn exec<C>(self, db: &'a C) -> impl Future<Output = Result<DeleteResult, DbErr>> + '_
    where
        C: ConnectionTrait,
    {
        // so that self is dropped before entering await
        exec_delete_only(self.query, db)
    }
}

impl Deleter {
    /// Instantiate a new [Deleter] by passing it a [DeleteStatement]
    pub fn new(query: DeleteStatement) -> Self {
        Self { query }
    }

    /// Execute a DELETE operation
    pub fn exec<C>(self, db: &C) -> impl Future<Output = Result<DeleteResult, DbErr>> + '_
    where
        C: ConnectionTrait,
    {
        let builder = db.get_database_backend();
        exec_delete(builder.build(&self.query), db)
    }

    async fn exec_delete_and_return_deleted<A, C>(
        mut self,
        model: A,
        db: &C,
    ) -> Result<<A::Entity as EntityTrait>::Model, DbErr>
    where
        A: ActiveModelTrait,
        C: ConnectionTrait,
    {
        type Entity<A> = <A as ActiveModelTrait>::Entity;
        type Model<A> = <Entity<A> as EntityTrait>::Model;
        type Column<A> = <Entity<A> as EntityTrait>::Column;
        match db.support_returning() {
            true => {
                let returning = Query::returning()
                    .exprs(Column::<A>::iter().map(|c| c.select_as(Expr::col(c))));
                self.query.returning(returning);
                let db_backend = db.get_database_backend();
                let found: Option<Model<A>> = SelectorRaw::<SelectModel<Model<A>>>::from_statement(
                    db_backend.build(&self.query),
                )
                .one(db)
                .await?;
                // If we got `None` then we are updating a row that does not exist.
                match found {
                    Some(model) => Ok(model),
                    None => Err(DbErr::RecordNotDeleted),
                }
            }
            false => {
                let deleted_item = find_deleted_model_by_id(model, db).await;
                self.exec(db).await?;
                deleted_item
            }
        }
    }
}

async fn exec_delete_only<C>(query: DeleteStatement, db: &C) -> Result<DeleteResult, DbErr>
where
    C: ConnectionTrait,
{
    Deleter::new(query).exec(db).await
}

async fn exec_delete<C>(statement: Statement, db: &C) -> Result<DeleteResult, DbErr>
where
    C: ConnectionTrait,
{
    let result = db.execute(statement).await?;
    Ok(DeleteResult {
        rows_affected: result.rows_affected(),
    })
}

async fn find_deleted_model_by_id<A, C>(
    model: A,
    db: &C,
) -> Result<<A::Entity as EntityTrait>::Model, DbErr>
where
    A: ActiveModelTrait,
    C: ConnectionTrait,
{
    type Entity<A> = <A as ActiveModelTrait>::Entity;
    type ValueType<A> = <<Entity<A> as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType;

    let primary_key_value = match model.get_primary_key_value() {
        Some(val) => ValueType::<A>::from_value_tuple(val),
        None => return Err(DbErr::DeleteGetPrimaryKey),
    };
    let found = Entity::<A>::find_by_id(primary_key_value).one(db).await?;
    // If we cannot select the deleted row from db by the cached primary key
    match found {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound(
            "Failed to find deleted item".to_owned(),
        )),
    }
}
