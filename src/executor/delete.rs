use super::{ReturningSelector, SelectModel};
use crate::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DeleteMany, DeleteOne, EntityTrait, Iterable,
    error::*,
};
use sea_query::{DeleteStatement, Query};
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

impl<A> DeleteOne<A>
where
    A: ActiveModelTrait,
{
    /// Execute a DELETE operation on one ActiveModel
    pub async fn exec<C>(self, db: &C) -> Result<DeleteResult, DbErr>
    where
        C: ConnectionTrait,
    {
        if let Some(err) = self.error {
            return Err(err);
        }
        exec_delete_only(self.query, db).await
    }

    /// Execute an delete operation and return the deleted model
    pub async fn exec_with_returning<C>(
        self,
        db: &C,
    ) -> Result<Option<<A::Entity as EntityTrait>::Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        if let Some(err) = self.error {
            return Err(err);
        }
        exec_delete_with_returning_one::<A::Entity, _>(self.query, db).await
    }
}

impl<'a, E> DeleteMany<E>
where
    E: EntityTrait,
{
    /// Execute a DELETE operation on many ActiveModels
    pub fn exec<C>(self, db: &'a C) -> impl Future<Output = Result<DeleteResult, DbErr>> + 'a
    where
        C: ConnectionTrait,
    {
        // so that self is dropped before entering await
        exec_delete_only(self.query, db)
    }

    /// Execute an delete operation and return the deleted model
    pub fn exec_with_returning<C>(
        self,
        db: &C,
    ) -> impl Future<Output = Result<Vec<E::Model>, DbErr>> + '_
    where
        E: EntityTrait,
        C: ConnectionTrait,
    {
        exec_delete_with_returning_many::<E, _>(self.query, db)
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
        exec_delete(self.query, db)
    }

    /// Execute an delete operation and return the deleted model
    pub fn exec_with_returning<E, C>(
        self,
        db: &C,
    ) -> impl Future<Output = Result<Vec<E::Model>, DbErr>> + '_
    where
        E: EntityTrait,
        C: ConnectionTrait,
    {
        exec_delete_with_returning_many::<E, _>(self.query, db)
    }
}

async fn exec_delete_only<C>(query: DeleteStatement, db: &C) -> Result<DeleteResult, DbErr>
where
    C: ConnectionTrait,
{
    Deleter::new(query).exec(db).await
}

async fn exec_delete<C>(query: DeleteStatement, db: &C) -> Result<DeleteResult, DbErr>
where
    C: ConnectionTrait,
{
    let result = db.execute(&query).await?;
    Ok(DeleteResult {
        rows_affected: result.rows_affected(),
    })
}

async fn exec_delete_with_returning_one<E, C>(
    mut query: DeleteStatement,
    db: &C,
) -> Result<Option<E::Model>, DbErr>
where
    E: EntityTrait,
    C: ConnectionTrait,
{
    let db_backend = db.get_database_backend();
    match db.support_returning() {
        true => {
            query.returning_all();
            ReturningSelector::<SelectModel<<E>::Model>, _>::from_query(query)
                .one(db)
                .await
        }
        false => Err(DbErr::BackendNotSupported {
            db: db_backend.as_str(),
            ctx: "DELETE RETURNING",
        }),
    }
}

async fn exec_delete_with_returning_many<E, C>(
    mut query: DeleteStatement,
    db: &C,
) -> Result<Vec<E::Model>, DbErr>
where
    E: EntityTrait,
    C: ConnectionTrait,
{
    let db_backend = db.get_database_backend();
    match db.support_returning() {
        true => {
            let returning = Query::returning().exprs(
                E::Column::iter().map(|c| c.select_enum_as(c.into_returning_expr(db_backend))),
            );
            query.returning(returning);
            ReturningSelector::<SelectModel<<E>::Model>, _>::from_query(query)
                .all(db)
                .await
        }
        false => Err(DbErr::BackendNotSupported {
            db: db_backend.as_str(),
            ctx: "DELETE RETURNING",
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::cake;

    #[smol_potat::test]
    async fn delete_error() {
        use crate::{DbBackend, DbErr, Delete, EntityTrait, MockDatabase};

        let db = MockDatabase::new(DbBackend::MySql).into_connection();

        assert!(matches!(
            Delete::one(cake::ActiveModel {
                ..Default::default()
            })
            .exec(&db)
            .await,
            Err(DbErr::PrimaryKeyNotSet { .. })
        ));

        assert!(matches!(
            cake::Entity::delete(cake::ActiveModel::default())
                .exec(&db)
                .await,
            Err(DbErr::PrimaryKeyNotSet { .. })
        ));
    }
}
