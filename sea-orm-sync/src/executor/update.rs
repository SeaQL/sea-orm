use super::{ReturningSelector, returning_all_columns};
use crate::{
    ActiveModelTrait, ConnectionTrait, EntityTrait, ExecResult, IntoActiveModel, PrimaryKeyTrait,
    SelectModel, UpdateMany, UpdateOne, ValidatedUpdateOne, error::*,
};
use sea_query::{FromValueTuple, UpdateStatement};

/// Lower-level executor that runs a raw `sea_query` [`UpdateStatement`].
/// Most code shouldn't need it directly — prefer
/// [`EntityTrait::update`](crate::EntityTrait::update) /
/// [`update_many`](crate::EntityTrait::update_many), which return
/// strongly-typed builders.
#[derive(Clone, Debug)]
pub struct Updater {
    query: UpdateStatement,
    check_record_exists: bool,
}

/// Result of an `UPDATE` that doesn't return rows: how many rows were
/// modified.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct UpdateResult {
    /// Number of rows touched by the statement.
    pub rows_affected: u64,
}

impl<A> ValidatedUpdateOne<A>
where
    A: ActiveModelTrait,
{
    /// Execute an UPDATE operation without a RETURNING clause, yielding an
    /// [`UpdateResult`] instead of the updated model. Returns
    /// [`DbErr::RecordNotUpdated`] if no row matches.
    pub fn exec_without_returning<C>(self, db: &C) -> Result<UpdateResult, DbErr>
    where
        C: ConnectionTrait,
    {
        Updater::new(self.query)
            // If nothing is updated, return RecordNotUpdated error
            .check_record_exists()
            .exec(db)
    }

    /// Execute an UPDATE operation on an ActiveModel
    pub fn exec<C>(self, db: &C) -> Result<<A::Entity as EntityTrait>::Model, DbErr>
    where
        <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
        C: ConnectionTrait,
    {
        Updater::new(self.query).exec_update_and_return_updated(self.model, db)
    }
}

impl<A> UpdateOne<A>
where
    A: ActiveModelTrait,
{
    /// Execute an UPDATE operation without a RETURNING clause, yielding an
    /// [`UpdateResult`] instead of the updated model. Returns
    /// [`DbErr::RecordNotUpdated`] if no row matches.
    pub fn exec_without_returning<C>(self, db: &C) -> Result<UpdateResult, DbErr>
    where
        C: ConnectionTrait,
    {
        let task = self.0.map(|update| update.exec_without_returning(db));
        #[cfg(not(feature = "sync"))]
        {
            ready(task).try_flatten()
        }
        #[cfg(feature = "sync")]
        task?
    }

    /// Execute an UPDATE operation on an ActiveModel
    pub fn exec<C>(self, db: &C) -> Result<<A::Entity as EntityTrait>::Model, DbErr>
    where
        <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
        C: ConnectionTrait,
    {
        let task = self.0.map(|update| update.exec(db));
        #[cfg(not(feature = "sync"))]
        {
            ready(task).try_flatten()
        }
        #[cfg(feature = "sync")]
        task?
    }
}

impl<'a, E> UpdateMany<E>
where
    E: EntityTrait,
{
    /// Execute an update operation on multiple ActiveModels
    pub fn exec<C>(self, db: &'a C) -> Result<UpdateResult, DbErr>
    where
        C: ConnectionTrait,
    {
        Updater::new(self.query).exec(db)
    }

    /// Execute an update operation and return the updated model (use `RETURNING` syntax if supported)
    pub fn exec_with_returning<C>(self, db: &'a C) -> Result<Vec<E::Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        Updater::new(self.query).exec_update_with_returning::<E, _>(db)
    }
}

impl Updater {
    /// Instantiate an update using an [UpdateStatement]
    fn new(query: UpdateStatement) -> Self {
        Self {
            query,
            check_record_exists: false,
        }
    }

    fn check_record_exists(mut self) -> Self {
        self.check_record_exists = true;
        self
    }

    /// Execute an update operation
    pub fn exec<C>(self, db: &C) -> Result<UpdateResult, DbErr>
    where
        C: ConnectionTrait,
    {
        let task = if self.is_noop() {
            None
        } else {
            Some(db.execute(&self.query))
        };
        let check_record_exists = self.check_record_exists;
        let convert = move |result: Option<Result<ExecResult, DbErr>>| {
            let Some(result) = result.transpose()? else {
                return Ok(UpdateResult::default());
            };
            if check_record_exists && result.rows_affected() == 0 {
                return Err(DbErr::RecordNotUpdated);
            }
            Ok(UpdateResult {
                rows_affected: result.rows_affected(),
            })
        };
        #[cfg(not(feature = "sync"))]
        {
            FutureExt::map(OptionFuture::from(task), convert)
        }
        #[cfg(feature = "sync")]
        convert(task)
    }

    fn exec_update_and_return_updated<A, C>(
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

        if !self.is_noop() && db.support_returning() {
            let db_backend = db.get_database_backend();
            let returning = returning_all_columns::<Entity<A>>(db_backend);
            self.query.returning(returning);
            let task =
                ReturningSelector::<SelectModel<Model<A>>, _>::from_query(self.query).one(db);
            let convert = move |result: Result<Option<Model<A>>, DbErr>| {
                let _model = model;
                // If we got `None` then we are updating a row that does not exist.
                result?.ok_or(DbErr::RecordNotUpdated)
            };
            #[cfg(not(feature = "sync"))]
            {
                FutureExt::map(task, convert)
            }
            #[cfg(feature = "sync")]
            convert(task)
        } else {
            let task = self.check_record_exists().exec(db);
            // If we updating a row that does not exist then an error will be thrown here.
            #[cfg(not(feature = "sync"))]
            {
                task.and_then(move |_| find_updated_model_by_id(model, db))
            }
            #[cfg(feature = "sync")]
            {
                task?;
                find_updated_model_by_id(model, db)
            }
        }
    }

    fn exec_update_with_returning<E, C>(mut self, db: &C) -> Result<Vec<E::Model>, DbErr>
    where
        E: EntityTrait,
        C: ConnectionTrait,
    {
        if self.is_noop() {
            Ok(vec![])
        } else {
            let db_backend = db.get_database_backend();
            let task = if db.support_returning() {
                let returning = returning_all_columns::<E>(db_backend);
                self.query.returning(returning);
                ReturningSelector::<SelectModel<E::Model>, _>::from_query(self.query).all(db)
            } else {
                {
                    Err(DbErr::BackendNotSupported {
                        db: db_backend.as_str(),
                        ctx: "UPDATE RETURNING",
                    })
                }
            };
            task
        }
    }

    fn is_noop(&self) -> bool {
        self.query.get_values().is_empty()
    }
}

fn find_updated_model_by_id<A, C>(
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
        None => return Err(DbErr::UpdateGetPrimaryKey),
    };
    let found = Entity::<A>::find_by_id(primary_key_value).one(db)?;
    // If we cannot select the updated row from db by the cached primary key
    match found {
        Some(model) => Ok(model),
        None => Err(DbErr::RecordNotFound(
            "Failed to find updated item".to_owned(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ColumnTrait, DbBackend, DbErr, EntityTrait, IntoActiveModel, MockDatabase, MockExecResult,
        QueryFilter, Set, Transaction, Update, UpdateResult, tests_cfg::cake,
    };
    use pretty_assertions::assert_eq;
    use sea_query::Expr;

    #[test]
    fn update_record_not_found_1() -> Result<(), DbErr> {
        use crate::ActiveModelTrait;

        let updated_cake = cake::Model {
            id: 1,
            name: "Cheese Cake".to_owned(),
        };

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([
                vec![updated_cake.clone()],
                vec![],
                vec![],
                vec![],
                vec![updated_cake.clone()],
                vec![updated_cake.clone()],
                vec![updated_cake.clone()],
            ])
            .append_exec_results([MockExecResult {
                last_insert_id: 0,
                rows_affected: 0,
            }])
            .into_connection();

        let model = cake::Model {
            id: 1,
            name: "New York Cheese".to_owned(),
        };

        assert_eq!(
            cake::ActiveModel {
                name: Set("Cheese Cake".to_owned()),
                ..model.clone().into_active_model()
            }
            .update(&db)?,
            cake::Model {
                id: 1,
                name: "Cheese Cake".to_owned(),
            }
        );

        let model = cake::Model {
            id: 2,
            name: "New York Cheese".to_owned(),
        };

        assert_eq!(
            cake::ActiveModel {
                name: Set("Cheese Cake".to_owned()),
                ..model.clone().into_active_model()
            }
            .update(&db),
            Err(DbErr::RecordNotUpdated)
        );

        assert_eq!(
            cake::Entity::update(cake::ActiveModel {
                name: Set("Cheese Cake".to_owned()),
                ..model.clone().into_active_model()
            })
            .exec(&db),
            Err(DbErr::RecordNotUpdated)
        );

        assert_eq!(
            Update::one(cake::ActiveModel {
                name: Set("Cheese Cake".to_owned()),
                ..model.clone().into_active_model()
            })
            .exec(&db),
            Err(DbErr::RecordNotUpdated)
        );

        assert_eq!(
            Update::many(cake::Entity)
                .col_expr(cake::Column::Name, Expr::value("Cheese Cake".to_owned()))
                .filter(cake::Column::Id.eq(2))
                .exec(&db),
            Ok(UpdateResult { rows_affected: 0 })
        );

        assert_eq!(
            updated_cake.clone().into_active_model().save(&db)?,
            updated_cake.clone().into_active_model()
        );

        assert_eq!(
            updated_cake.clone().into_active_model().update(&db)?,
            updated_cake
        );

        assert_eq!(
            cake::Entity::update(updated_cake.clone().into_active_model()).exec(&db)?,
            updated_cake
        );

        assert_eq!(cake::Entity::update_many().exec(&db)?.rows_affected, 0);

        assert_eq!(
            db.into_transaction_log(),
            [
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"UPDATE "cake" SET "name" = $1 WHERE "cake"."id" = $2 RETURNING "id", "name""#,
                    ["Cheese Cake".into(), 1i32.into()]
                ),
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"UPDATE "cake" SET "name" = $1 WHERE "cake"."id" = $2 RETURNING "id", "name""#,
                    ["Cheese Cake".into(), 2i32.into()]
                ),
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"UPDATE "cake" SET "name" = $1 WHERE "cake"."id" = $2 RETURNING "id", "name""#,
                    ["Cheese Cake".into(), 2i32.into()]
                ),
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"UPDATE "cake" SET "name" = $1 WHERE "cake"."id" = $2 RETURNING "id", "name""#,
                    ["Cheese Cake".into(), 2i32.into()]
                ),
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"UPDATE "cake" SET "name" = $1 WHERE "cake"."id" = $2"#,
                    ["Cheese Cake".into(), 2i32.into()]
                ),
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "cake"."id", "cake"."name" FROM "cake" WHERE "cake"."id" = $1 LIMIT $2"#,
                    [1.into(), 1u64.into()]
                ),
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "cake"."id", "cake"."name" FROM "cake" WHERE "cake"."id" = $1 LIMIT $2"#,
                    [1.into(), 1u64.into()]
                ),
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"SELECT "cake"."id", "cake"."name" FROM "cake" WHERE "cake"."id" = $1 LIMIT $2"#,
                    [1.into(), 1u64.into()]
                ),
            ]
        );

        Ok(())
    }

    #[test]
    fn update_error() {
        use crate::{DbBackend, DbErr, MockDatabase};

        let db = MockDatabase::new(DbBackend::MySql).into_connection();

        assert!(matches!(
            Update::one(cake::ActiveModel {
                ..Default::default()
            })
            .exec(&db),
            Err(DbErr::PrimaryKeyNotSet { .. })
        ));

        assert!(matches!(
            cake::Entity::update(cake::ActiveModel::default()).exec(&db),
            Err(DbErr::PrimaryKeyNotSet { .. })
        ));
    }
}
