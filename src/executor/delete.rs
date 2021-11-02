use crate::{
    error::*, ActiveModelTrait, ConnectionTrait, DeleteMany, DeleteOne, EntityTrait, ModelTrait,
    Statement, StatementBuilder,
};
use sea_query::{ConditionalStatement, DeleteStatement, Expr, UpdateStatement};
use std::future::Future;

/// Handles DELETE operations in a ActiveModel using [DeleteStatement]
#[derive(Clone, Debug)]
pub struct Deleter<Q>
where
    Q: StatementBuilder,
{
    query: Q,
}

/// The result of a DELETE operation
#[derive(Clone, Debug)]
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
        C: ConnectionTrait<'a>,
    {
        // so that self is dropped before entering await
        exec_delete_only::<_, A::Entity>(self.query, self.force_delete, db)
    }
}

impl<'a, E> DeleteMany<E>
where
    E: EntityTrait,
{
    /// Execute a DELETE operation on many ActiveModels
    pub fn exec<C>(self, db: &'a C) -> impl Future<Output = Result<DeleteResult, DbErr>> + '_
    where
        C: ConnectionTrait<'a>,
    {
        // so that self is dropped before entering await
        exec_delete_only::<_, E>(self.query, self.force_delete, db)
    }
}

impl<Q> Deleter<Q>
where
    Q: StatementBuilder,
{
    /// Instantiate a new [Deleter] by passing it a [DeleteStatement]
    pub fn new(query: Q) -> Self {
        Self { query }
    }

    /// Execute a DELETE operation
    pub fn exec<'a, C>(self, db: &'a C) -> impl Future<Output = Result<DeleteResult, DbErr>> + '_
    where
        C: ConnectionTrait<'a>,
    {
        let builder = db.get_database_backend();
        exec_delete(builder.build(&self.query), db)
    }
}

async fn exec_delete_only<'a, C, E>(
    delete_stmt: DeleteStatement,
    force_delete: bool,
    db: &'a C,
) -> Result<DeleteResult, DbErr>
where
    C: ConnectionTrait<'a>,
    E: EntityTrait,
{
    match <<E as EntityTrait>::Model as ModelTrait>::soft_delete_column() {
        Some(soft_delete_column) if !force_delete => {
            let update_stmt = convert_to_soft_delete::<E>(delete_stmt, soft_delete_column);
            Deleter::new(update_stmt).exec(db).await
        }
        _ => Deleter::new(delete_stmt).exec(db).await,
    }
}

pub(crate) fn convert_to_soft_delete<E>(
    delete_stmt: DeleteStatement,
    soft_delete_column: E::Column,
) -> UpdateStatement
where
    E: EntityTrait,
{
    let mut delete_stmt = delete_stmt;
    let value = <E::Model as ModelTrait>::soft_delete_column_value();
    UpdateStatement::new()
        .table(E::default())
        .col_expr(soft_delete_column, Expr::value(value))
        .extend_conditions(delete_stmt.take_conditions())
        .to_owned()
}

async fn exec_delete<'a, C>(statement: Statement, db: &'a C) -> Result<DeleteResult, DbErr>
where
    C: ConnectionTrait<'a>,
{
    let result = db.execute(statement).await?;
    Ok(DeleteResult {
        rows_affected: result.rows_affected(),
    })
}
