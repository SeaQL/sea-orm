use crate::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DbErr, EntityTrait, Iterable, PrimaryKeyToColumn,
    QueryFilter, QueryTrait,
};
use core::marker::PhantomData;
use sea_query::{Expr, IntoIden, SimpleExpr, UpdateStatement};

/// Defines a structure to perform UPDATE query operations on a ActiveModel
#[derive(Clone, Debug)]
pub struct Update;

/// A request to update an [`ActiveModel`](ActiveModelTrait).
///
/// The primary key must be set.
/// Otherwise, it's impossible to generate the SQL condition and find the record.
/// In that case, [`exec`][Self::exec] will return an error and not send any queries to the database.
///
/// If you want to use [`QueryTrait`] and access the generated SQL query,
/// you need to convert into [`ValidatedUpdateOne`] first.
#[derive(Clone, Debug)]
pub struct UpdateOne<A: ActiveModelTrait>(pub(crate) Result<ValidatedUpdateOne<A>, DbErr>);

/// A validated [`UpdateOne`] request, where the primary key is set
/// and it's possible to generate the right SQL condition.
#[derive(Clone, Debug)]
pub struct ValidatedUpdateOne<A: ActiveModelTrait> {
    pub(crate) query: UpdateStatement,
    pub(crate) model: A,
}

impl<A: ActiveModelTrait> TryFrom<UpdateOne<A>> for ValidatedUpdateOne<A> {
    type Error = DbErr;

    fn try_from(value: UpdateOne<A>) -> Result<Self, Self::Error> {
        value.0
    }
}

impl<A: ActiveModelTrait> UpdateOne<A> {
    /// Check whether the primary key is set and we can proceed with the operation.
    pub fn validate(self) -> Result<ValidatedUpdateOne<A>, DbErr> {
        self.try_into()
    }
}

/// Defines an UPDATE operation on multiple ActiveModels
#[derive(Clone, Debug)]
pub struct UpdateMany<E>
where
    E: EntityTrait,
{
    pub(crate) query: UpdateStatement,
    pub(crate) entity: PhantomData<E>,
}

impl Update {
    /// Update one ActiveModel
    ///
    /// ```
    /// use sea_orm::{DbBackend, entity::*, query::*, tests_cfg::cake};
    ///
    /// assert_eq!(
    ///     Update::one(cake::ActiveModel {
    ///         id: ActiveValue::set(1),
    ///         name: ActiveValue::set("Apple Pie".to_owned()),
    ///     })
    ///     .validate()
    ///     .unwrap()
    ///     .build(DbBackend::Postgres)
    ///     .to_string(),
    ///     r#"UPDATE "cake" SET "name" = 'Apple Pie' WHERE "cake"."id" = 1"#,
    /// );
    /// ```
    //
    // (non-doc comment for maintainers)
    // Ideally, we would make this method fallible instead of stashing and delaying the error.
    // But that's a bigger breaking change.
    pub fn one<E, A>(model: A) -> UpdateOne<A>
    where
        E: EntityTrait,
        A: ActiveModelTrait<Entity = E>,
    {
        let mut myself = ValidatedUpdateOne {
            query: UpdateStatement::new()
                .table(A::Entity::default().table_ref())
                .to_owned(),
            model,
        };
        // Build the SQL condition from the primary key columns.
        for key in <A::Entity as EntityTrait>::PrimaryKey::iter() {
            let col = key.into_column();
            match myself.model.get(col) {
                ActiveValue::Set(value) | ActiveValue::Unchanged(value) => {
                    myself = myself.filter(col.eq(value));
                }
                ActiveValue::NotSet => {
                    return UpdateOne(Err(DbErr::PrimaryKeyNotSet { ctx: "UpdateOne" }));
                }
            }
        }
        // Set the values to update (from the other columns).
        for col in <A::Entity as EntityTrait>::Column::iter() {
            if <A::Entity as EntityTrait>::PrimaryKey::from_column(col).is_some() {
                continue;
            }
            match myself.model.get(col) {
                ActiveValue::Set(value) => {
                    let expr = col.save_as(Expr::val(value));
                    myself.query.value(col, expr);
                }
                ActiveValue::Unchanged(_) | ActiveValue::NotSet => {}
            }
        }
        UpdateOne(Ok(myself))
    }

    /// Update many ActiveModel
    ///
    /// ```
    /// use sea_orm::{DbBackend, entity::*, query::*, sea_query::Expr, tests_cfg::fruit};
    ///
    /// assert_eq!(
    ///     Update::many(fruit::Entity)
    ///         .col_expr(fruit::Column::Name, Expr::value("Golden Apple"))
    ///         .filter(fruit::Column::Name.contains("Apple"))
    ///         .build(DbBackend::Postgres)
    ///         .to_string(),
    ///     r#"UPDATE "fruit" SET "name" = 'Golden Apple' WHERE "fruit"."name" LIKE '%Apple%'"#,
    /// );
    /// ```
    pub fn many<E>(entity: E) -> UpdateMany<E>
    where
        E: EntityTrait,
    {
        UpdateMany {
            query: UpdateStatement::new().table(entity.table_ref()).to_owned(),
            entity: PhantomData,
        }
    }
}

impl<A> QueryFilter for ValidatedUpdateOne<A>
where
    A: ActiveModelTrait,
{
    type QueryStatement = UpdateStatement;

    fn query(&mut self) -> &mut UpdateStatement {
        &mut self.query
    }
}

impl<E> QueryFilter for UpdateMany<E>
where
    E: EntityTrait,
{
    type QueryStatement = UpdateStatement;

    fn query(&mut self) -> &mut UpdateStatement {
        &mut self.query
    }
}

impl<A> QueryTrait for ValidatedUpdateOne<A>
where
    A: ActiveModelTrait,
{
    type QueryStatement = UpdateStatement;

    fn query(&mut self) -> &mut UpdateStatement {
        &mut self.query
    }

    fn as_query(&self) -> &UpdateStatement {
        &self.query
    }

    fn into_query(self) -> UpdateStatement {
        self.query
    }
}

impl<E> QueryTrait for UpdateMany<E>
where
    E: EntityTrait,
{
    type QueryStatement = UpdateStatement;

    fn query(&mut self) -> &mut UpdateStatement {
        &mut self.query
    }

    fn as_query(&self) -> &UpdateStatement {
        &self.query
    }

    fn into_query(self) -> UpdateStatement {
        self.query
    }
}

impl<E> UpdateMany<E>
where
    E: EntityTrait,
{
    /// Add the models to update to Self
    pub fn set<A>(mut self, model: A) -> Self
    where
        A: ActiveModelTrait<Entity = E>,
    {
        for col in E::Column::iter() {
            match model.get(col) {
                ActiveValue::Set(value) => {
                    let expr = col.save_as(Expr::val(value));
                    self.query.value(col, expr);
                }
                ActiveValue::Unchanged(_) | ActiveValue::NotSet => {}
            }
        }
        self
    }

    /// Creates a [SimpleExpr] from a column
    pub fn col_expr<T>(mut self, col: T, expr: SimpleExpr) -> Self
    where
        T: IntoIden,
    {
        self.query.value(col, expr);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::{cake, fruit, lunch_set, sea_orm_active_enums::Tea};
    use crate::{DbBackend, entity::*, query::*};
    use sea_query::{Expr, Value};

    #[test]
    fn update_1() {
        assert_eq!(
            Update::one(cake::ActiveModel {
                id: ActiveValue::set(1),
                name: ActiveValue::set("Apple Pie".to_owned()),
            })
            .validate()
            .unwrap()
            .build(DbBackend::Postgres)
            .to_string(),
            r#"UPDATE "cake" SET "name" = 'Apple Pie' WHERE "cake"."id" = 1"#,
        );
    }

    #[test]
    fn update_2() {
        assert_eq!(
            Update::one(fruit::ActiveModel {
                id: ActiveValue::set(1),
                name: ActiveValue::set("Orange".to_owned()),
                cake_id: ActiveValue::not_set(),
            })
            .validate()
            .unwrap()
            .build(DbBackend::Postgres)
            .to_string(),
            r#"UPDATE "fruit" SET "name" = 'Orange' WHERE "fruit"."id" = 1"#,
        );
    }

    #[test]
    fn update_3() {
        assert_eq!(
            Update::one(fruit::ActiveModel {
                id: ActiveValue::set(2),
                name: ActiveValue::unchanged("Apple".to_owned()),
                cake_id: ActiveValue::set(Some(3)),
            })
            .validate()
            .unwrap()
            .build(DbBackend::Postgres)
            .to_string(),
            r#"UPDATE "fruit" SET "cake_id" = 3 WHERE "fruit"."id" = 2"#,
        );
    }

    #[test]
    fn update_4() {
        assert_eq!(
            Update::many(fruit::Entity)
                .col_expr(fruit::Column::CakeId, Expr::value(Value::Int(None)))
                .filter(fruit::Column::Id.eq(2))
                .build(DbBackend::Postgres)
                .to_string(),
            r#"UPDATE "fruit" SET "cake_id" = NULL WHERE "fruit"."id" = 2"#,
        );
    }

    #[test]
    fn update_5() {
        assert_eq!(
            Update::many(fruit::Entity)
                .set(fruit::ActiveModel {
                    name: ActiveValue::set("Apple".to_owned()),
                    cake_id: ActiveValue::set(Some(3)),
                    ..Default::default()
                })
                .filter(fruit::Column::Id.eq(2))
                .build(DbBackend::Postgres)
                .to_string(),
            r#"UPDATE "fruit" SET "name" = 'Apple', "cake_id" = 3 WHERE "fruit"."id" = 2"#,
        );
    }

    #[test]
    fn update_6() {
        assert_eq!(
            Update::many(fruit::Entity)
                .set(fruit::ActiveModel {
                    id: ActiveValue::set(3),
                    ..Default::default()
                })
                .filter(fruit::Column::Id.eq(2))
                .build(DbBackend::Postgres)
                .to_string(),
            r#"UPDATE "fruit" SET "id" = 3 WHERE "fruit"."id" = 2"#,
        );
    }

    #[test]
    fn update_7() {
        assert_eq!(
            Update::many(lunch_set::Entity)
                .set(lunch_set::ActiveModel {
                    tea: Set(Tea::EverydayTea),
                    ..Default::default()
                })
                .filter(lunch_set::Column::Tea.eq(Tea::BreakfastTea))
                .build(DbBackend::Postgres)
                .to_string(),
            r#"UPDATE "lunch_set" SET "tea" = CAST('EverydayTea' AS "tea") WHERE "lunch_set"."tea" = (CAST('BreakfastTea' AS "tea"))"#,
        );
    }

    #[test]
    fn update_8() {
        assert_eq!(
            Update::one(lunch_set::ActiveModel {
                id: Unchanged(1),
                tea: Set(Tea::EverydayTea),
                ..Default::default()
            })
            .validate()
            .unwrap()
            .build(DbBackend::Postgres)
            .to_string(),
            r#"UPDATE "lunch_set" SET "tea" = CAST('EverydayTea' AS "tea") WHERE "lunch_set"."id" = 1"#,
        );
    }
}
