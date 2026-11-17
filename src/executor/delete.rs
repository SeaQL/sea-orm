use super::{ReturningSelector, SelectModel};
use crate::{
    ColumnTrait, ConnectionTrait, DeleteMany, DeleteOne, EntityTrait, Iterable, ValidatedDeleteOne,
    error::*,
};
use futures_util::FutureExt;
use futures_util::TryFutureExt;
use futures_util::future::ready;
use sea_query::{DeleteStatement, Query};
use std::future::Future;

/// Lower-level executor that runs a raw `sea_query` [`DeleteStatement`] and
/// collects the result. Most code shouldn't need it directly — prefer
/// [`EntityTrait::delete`](crate::EntityTrait::delete) /
/// [`delete_many`](crate::EntityTrait::delete_many) /
/// [`delete_by_id`](crate::EntityTrait::delete_by_id), which return
/// strongly-typed builders.
#[derive(Clone, Debug)]
pub struct Deleter {
    query: DeleteStatement,
}

/// Result of a `DELETE`: how many rows were removed.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DeleteResult {
    /// Number of rows removed by the statement.
    pub rows_affected: u64,
}

impl<E> ValidatedDeleteOne<E>
where
    E: EntityTrait,
{
    /// Execute a DELETE operation on one ActiveModel
    pub fn exec<C>(self, db: &C) -> impl Future<Output = Result<DeleteResult, DbErr>>
    where
        C: ConnectionTrait,
    {
        Deleter::new(self.query).exec(db)
    }

    /// Execute an delete operation and return the deleted model
    pub fn exec_with_returning<C>(
        mut self,
        db: &C,
    ) -> impl Future<Output = Result<Option<E::Model>, DbErr>>
    where
        C: ConnectionTrait,
    {
        let db_backend = db.get_database_backend();
        if db.support_returning() {
            let returning = Query::returning().exprs(
                E::Column::iter().map(|c| c.select_enum_as(c.into_returning_expr(db_backend))),
            );
            self.query.returning(returning);
            ReturningSelector::<SelectModel<<E>::Model>, _>::from_query(self.query)
                .one(db)
                .left_future()
        } else {
            async move {
                Err(DbErr::BackendNotSupported {
                    db: db_backend.as_str(),
                    ctx: "DELETE RETURNING",
                })
            }
            .right_future()
        }
    }
}

impl<E> DeleteOne<E>
where
    E: EntityTrait,
{
    /// Execute a DELETE operation on one ActiveModel
    pub fn exec<C>(self, db: &C) -> impl Future<Output = Result<DeleteResult, DbErr>>
    where
        C: ConnectionTrait,
    {
        let task = self.0.map(|delete| delete.exec(db));
        #[cfg(not(feature = "sync"))]
        {
            ready(task).try_flatten()
        }
        #[cfg(feature = "sync")]
        task?
    }

    /// Execute an delete operation and return the deleted model
    pub fn exec_with_returning<C>(
        self,
        db: &C,
    ) -> impl Future<Output = Result<Option<E::Model>, DbErr>>
    where
        C: ConnectionTrait,
    {
        let task = self.0.map(|delete| delete.exec_with_returning(db));
        #[cfg(not(feature = "sync"))]
        {
            ready(task).try_flatten()
        }
        #[cfg(feature = "sync")]
        task?
    }
}

impl<'a, E> DeleteMany<E>
where
    E: EntityTrait,
{
    /// Execute a DELETE operation on many ActiveModels
    pub fn exec<C>(self, db: &'a C) -> impl Future<Output = Result<DeleteResult, DbErr>>
    where
        C: ConnectionTrait,
    {
        Deleter::new(self.query).exec(db)
    }

    /// Execute an delete operation and return the deleted model
    pub fn exec_with_returning<C>(
        self,
        db: &C,
    ) -> impl Future<Output = Result<Vec<E::Model>, DbErr>>
    where
        E: EntityTrait,
        C: ConnectionTrait,
    {
        Deleter::new(self.query).exec_with_returning::<E, _>(db)
    }
}

impl Deleter {
    /// Instantiate a new [Deleter] by passing it a [DeleteStatement]
    pub fn new(query: DeleteStatement) -> Self {
        Self { query }
    }

    /// Execute a DELETE operation
    pub fn exec<C>(self, db: &C) -> impl Future<Output = Result<DeleteResult, DbErr>>
    where
        C: ConnectionTrait,
    {
        TryFutureExt::map_ok(db.execute(&self.query), |result| DeleteResult {
            rows_affected: result.rows_affected(),
        })
    }

    /// Execute an delete operation and return the deleted model
    pub fn exec_with_returning<E, C>(
        mut self,
        db: &C,
    ) -> impl Future<Output = Result<Vec<E::Model>, DbErr>>
    where
        E: EntityTrait,
        C: ConnectionTrait,
    {
        let db_backend = db.get_database_backend();
        if db.support_returning() {
            let returning = Query::returning().exprs(
                E::Column::iter().map(|c| c.select_enum_as(c.into_returning_expr(db_backend))),
            );
            self.query.returning(returning);
            ReturningSelector::<SelectModel<<E>::Model>, _>::from_query(self.query)
                .all(db)
                .left_future()
        } else {
            async move {
                Err(DbErr::BackendNotSupported {
                    db: db_backend.as_str(),
                    ctx: "DELETE RETURNING",
                })
            }
            .right_future()
        }
    }
}

impl DeleteResult {
    #[doc(hidden)]
    pub fn empty() -> Self {
        Self { rows_affected: 0 }
    }

    #[doc(hidden)]
    pub fn merge(&mut self, other: DeleteResult) {
        self.rows_affected += other.rows_affected;
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::cake;

    #[macro_rules_attribute::apply(smol_macros::test!)]
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
