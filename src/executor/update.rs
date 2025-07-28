use super::ReturningSelector;
use crate::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel, Iterable,
    PrimaryKeyTrait, SelectModel, UpdateMany, UpdateOne, error::*,
};
use sea_query::{FromValueTuple, Query, UpdateStatement};

/// Defines an update operation
#[derive(Clone, Debug)]
pub struct Updater {
    query: UpdateStatement,
    check_record_exists: bool,
}

/// The result of an update operation on an ActiveModel
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct UpdateResult {
    /// The rows affected by the update operation
    pub rows_affected: u64,
}

impl<A> UpdateOne<A>
where
    A: ActiveModelTrait,
{
    /// Execute an UPDATE operation on an ActiveModel
    pub async fn exec<C>(self, db: &C) -> Result<<A::Entity as EntityTrait>::Model, DbErr>
    where
        <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
        C: ConnectionTrait,
    {
        if let Some(err) = self.error {
            return Err(err);
        }
        Updater::new(self.query)
            .exec_update_and_return_updated(self.model, db)
            .await
    }
}

impl<'a, E> UpdateMany<E>
where
    E: EntityTrait,
{
    /// Execute an update operation on multiple ActiveModels
    pub async fn exec<C>(self, db: &'a C) -> Result<UpdateResult, DbErr>
    where
        C: ConnectionTrait,
    {
        Updater::new(self.query).exec(db).await
    }

    /// Execute an update operation and return the updated model (use `RETURNING` syntax if supported)
    pub async fn exec_with_returning<C>(self, db: &'a C) -> Result<Vec<E::Model>, DbErr>
    where
        C: ConnectionTrait,
    {
        Updater::new(self.query)
            .exec_update_with_returning::<E, _>(db)
            .await
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

    /// Execute an update operation
    pub async fn exec<C>(self, db: &C) -> Result<UpdateResult, DbErr>
    where
        C: ConnectionTrait,
    {
        if self.is_noop() {
            return Ok(UpdateResult::default());
        }
        let result = db.execute(&self.query).await?;
        if self.check_record_exists && result.rows_affected() == 0 {
            return Err(DbErr::RecordNotUpdated);
        }
        Ok(UpdateResult {
            rows_affected: result.rows_affected(),
        })
    }

    async fn exec_update_and_return_updated<A, C>(
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

        if self.is_noop() {
            return find_updated_model_by_id(model, db).await;
        }

        match db.support_returning() {
            true => {
                let db_backend = db.get_database_backend();
                let returning = Query::returning().exprs(
                    Column::<A>::iter().map(|c| c.select_as(c.into_returning_expr(db_backend))),
                );
                self.query.returning(returning);
                let found: Option<Model<A>> =
                    ReturningSelector::<SelectModel<Model<A>>, _>::from_query(self.query)
                        .one(db)
                        .await?;
                // If we got `None` then we are updating a row that does not exist.
                match found {
                    Some(model) => Ok(model),
                    None => Err(DbErr::RecordNotUpdated),
                }
            }
            false => {
                // If we updating a row that does not exist then an error will be thrown here.
                self.check_record_exists = true;
                self.exec(db).await?;
                find_updated_model_by_id(model, db).await
            }
        }
    }

    async fn exec_update_with_returning<E, C>(mut self, db: &C) -> Result<Vec<E::Model>, DbErr>
    where
        E: EntityTrait,
        C: ConnectionTrait,
    {
        if self.is_noop() {
            return Ok(vec![]);
        }

        let db_backend = db.get_database_backend();
        match db.support_returning() {
            true => {
                let returning = Query::returning().exprs(
                    E::Column::iter().map(|c| c.select_as(c.into_returning_expr(db_backend))),
                );
                self.query.returning(returning);
                let models: Vec<E::Model> =
                    ReturningSelector::<SelectModel<E::Model>, _>::from_query(self.query)
                        .all(db)
                        .await?;
                Ok(models)
            }
            false => Err(DbErr::BackendNotSupported {
                db: db_backend.as_str(),
                ctx: "UPDATE RETURNING",
            }),
        }
    }

    fn is_noop(&self) -> bool {
        self.query.get_values().is_empty()
    }
}

async fn find_updated_model_by_id<A, C>(
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
    let found = Entity::<A>::find_by_id(primary_key_value).one(db).await?;
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

    #[smol_potat::test]
    async fn update_record_not_found_1() -> Result<(), DbErr> {
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
            .update(&db)
            .await?,
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
            .update(&db)
            .await,
            Err(DbErr::RecordNotUpdated)
        );

        assert_eq!(
            cake::Entity::update(cake::ActiveModel {
                name: Set("Cheese Cake".to_owned()),
                ..model.clone().into_active_model()
            })
            .exec(&db)
            .await,
            Err(DbErr::RecordNotUpdated)
        );

        assert_eq!(
            Update::one(cake::ActiveModel {
                name: Set("Cheese Cake".to_owned()),
                ..model.clone().into_active_model()
            })
            .exec(&db)
            .await,
            Err(DbErr::RecordNotUpdated)
        );

        assert_eq!(
            Update::many(cake::Entity)
                .col_expr(cake::Column::Name, Expr::value("Cheese Cake".to_owned()))
                .filter(cake::Column::Id.eq(2))
                .exec(&db)
                .await,
            Ok(UpdateResult { rows_affected: 0 })
        );

        assert_eq!(
            updated_cake.clone().into_active_model().save(&db).await?,
            updated_cake.clone().into_active_model()
        );

        assert_eq!(
            updated_cake.clone().into_active_model().update(&db).await?,
            updated_cake
        );

        assert_eq!(
            cake::Entity::update(updated_cake.clone().into_active_model())
                .exec(&db)
                .await?,
            updated_cake
        );

        assert_eq!(
            cake::Entity::update_many().exec(&db).await?.rows_affected,
            0
        );

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

    #[smol_potat::test]
    async fn update_error() {
        use crate::{DbBackend, DbErr, MockDatabase};

        let db = MockDatabase::new(DbBackend::MySql).into_connection();

        assert!(matches!(
            Update::one(cake::ActiveModel {
                ..Default::default()
            })
            .exec(&db)
            .await,
            Err(DbErr::PrimaryKeyNotSet { .. })
        ));

        assert!(matches!(
            cake::Entity::update(cake::ActiveModel::default())
                .exec(&db)
                .await,
            Err(DbErr::PrimaryKeyNotSet { .. })
        ));
    }
}
