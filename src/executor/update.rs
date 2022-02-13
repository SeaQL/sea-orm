use crate::{
    error::*, ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel,
    Iterable, SelectModel, SelectorRaw, Statement, UpdateMany, UpdateOne,
};
use sea_query::{Alias, Expr, FromValueTuple, Query, UpdateStatement};
use std::future::Future;

/// Defines an update operation
#[derive(Clone, Debug)]
pub struct Updater {
    query: UpdateStatement,
    check_record_exists: bool,
}

/// The result of an update operation on an ActiveModel
#[derive(Clone, Debug, PartialEq)]
pub struct UpdateResult {
    /// The rows affected by the update operation
    pub rows_affected: u64,
}

impl<'a, A: 'a> UpdateOne<A>
where
    A: ActiveModelTrait,
{
    /// Execute an update operation on an ActiveModel
    pub async fn exec<'b, C>(self, db: &'b C) -> Result<<A::Entity as EntityTrait>::Model, DbErr>
    where
        <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
        C: ConnectionTrait + ?Sized,
    {
        // so that self is dropped before entering await
        exec_update_and_return_updated(self.query, self.model, db).await
    }
}

impl<'a, E> UpdateMany<E>
where
    E: EntityTrait,
{
    /// Execute an update operation on multiple ActiveModels
    pub fn exec<C>(self, db: &'a C) -> impl Future<Output = Result<UpdateResult, DbErr>> + '_
    where
        C: ConnectionTrait + ?Sized,
    {
        // so that self is dropped before entering await
        exec_update_only(self.query, db)
    }
}

impl Updater {
    /// Instantiate an update using an [UpdateStatement]
    pub fn new(query: UpdateStatement) -> Self {
        Self {
            query,
            check_record_exists: false,
        }
    }

    /// Check if a record exists on the ActiveModel to perform the update operation on
    pub fn check_record_exists(mut self) -> Self {
        self.check_record_exists = true;
        self
    }

    /// Execute an update operation
    pub fn exec<'a, C>(self, db: &'a C) -> impl Future<Output = Result<UpdateResult, DbErr>> + '_
    where
        C: ConnectionTrait + ?Sized,
    {
        let builder = db.get_database_backend();
        exec_update(builder.build(&self.query), db, self.check_record_exists)
    }
}

async fn exec_update_only<'a, C>(query: UpdateStatement, db: &'a C) -> Result<UpdateResult, DbErr>
where
    C: ConnectionTrait + ?Sized,
{
    Updater::new(query).exec(db).await
}

async fn exec_update_and_return_updated<'a, A, C>(
    mut query: UpdateStatement,
    model: A,
    db: &'a C,
) -> Result<<A::Entity as EntityTrait>::Model, DbErr>
where
    A: ActiveModelTrait,
    C: ConnectionTrait + ?Sized,
{
    match db.support_returning() {
        true => {
            let mut returning = Query::select();
            returning.exprs(<A::Entity as EntityTrait>::Column::iter().map(|c| {
                let col = Expr::col(c);
                let col_def = c.def();
                let col_type = col_def.get_column_type();
                match col_type.get_enum_name() {
                    Some(_) => col.as_enum(Alias::new("text")),
                    None => col.into(),
                }
            }));
            query.returning(returning);
            let db_backend = db.get_database_backend();
            let found: Option<<A::Entity as EntityTrait>::Model> =
                SelectorRaw::<SelectModel<<A::Entity as EntityTrait>::Model>>::from_statement(
                    db_backend.build(&query),
                )
                .one(db)
                .await?;
            // If we got `None` then we are updating a row that does not exist.
            match found {
                Some(model) => Ok(model),
                None => Err(DbErr::RecordNotFound(
                    "None of the database rows are affected".to_owned(),
                )),
            }
        }
        false => {
            // If we updating a row that does not exist then an error will be thrown here.
            Updater::new(query).check_record_exists().exec(db).await?;
            let primary_key_value = match model.get_primary_key_value() {
                Some(val) => FromValueTuple::from_value_tuple(val),
                None => return Err(DbErr::Exec("Fail to get primary key from model".to_owned())),
            };
            let found = <A::Entity as EntityTrait>::find_by_id(primary_key_value)
                .one(db)
                .await?;
            // If we cannot select the updated row from db by the cached primary key
            match found {
                Some(model) => Ok(model),
                None => Err(DbErr::Exec("Failed to find inserted item".to_owned())),
            }
        }
    }
}

async fn exec_update<'a, C>(
    statement: Statement,
    db: &'a C,
    check_record_exists: bool,
) -> Result<UpdateResult, DbErr>
where
    C: ConnectionTrait + ?Sized,
{
    let result = db.execute(statement).await?;
    if check_record_exists && result.rows_affected() == 0 {
        return Err(DbErr::RecordNotFound(
            "None of the database rows are affected".to_owned(),
        ));
    }
    Ok(UpdateResult {
        rows_affected: result.rows_affected(),
    })
}

#[cfg(test)]
mod tests {
    use crate::{entity::prelude::*, tests_cfg::*, *};
    use pretty_assertions::assert_eq;
    use sea_query::Expr;

    #[smol_potat::test]
    async fn update_record_not_found_1() -> Result<(), DbErr> {
        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results(vec![
                vec![cake::Model {
                    id: 1,
                    name: "Cheese Cake".to_owned(),
                }],
                vec![],
                vec![],
                vec![],
            ])
            .append_exec_results(vec![MockExecResult {
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
                ..model.into_active_model()
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
            Err(DbErr::RecordNotFound(
                "None of the database rows are affected".to_owned()
            ))
        );

        assert_eq!(
            cake::Entity::update(cake::ActiveModel {
                name: Set("Cheese Cake".to_owned()),
                ..model.clone().into_active_model()
            })
            .exec(&db)
            .await,
            Err(DbErr::RecordNotFound(
                "None of the database rows are affected".to_owned()
            ))
        );

        assert_eq!(
            Update::one(cake::ActiveModel {
                name: Set("Cheese Cake".to_owned()),
                ..model.into_active_model()
            })
            .exec(&db)
            .await,
            Err(DbErr::RecordNotFound(
                "None of the database rows are affected".to_owned()
            ))
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
            db.into_transaction_log(),
            vec![
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"UPDATE "cake" SET "name" = $1 WHERE "cake"."id" = $2 RETURNING "id", "name""#,
                    vec!["Cheese Cake".into(), 1i32.into()]
                ),
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"UPDATE "cake" SET "name" = $1 WHERE "cake"."id" = $2 RETURNING "id", "name""#,
                    vec!["Cheese Cake".into(), 2i32.into()]
                ),
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"UPDATE "cake" SET "name" = $1 WHERE "cake"."id" = $2 RETURNING "id", "name""#,
                    vec!["Cheese Cake".into(), 2i32.into()]
                ),
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"UPDATE "cake" SET "name" = $1 WHERE "cake"."id" = $2 RETURNING "id", "name""#,
                    vec!["Cheese Cake".into(), 2i32.into()]
                ),
                Transaction::from_sql_and_values(
                    DbBackend::Postgres,
                    r#"UPDATE "cake" SET "name" = $1 WHERE "cake"."id" = $2"#,
                    vec!["Cheese Cake".into(), 2i32.into()]
                ),
            ]
        );

        Ok(())
    }
}
