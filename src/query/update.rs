use crate::{
    ActiveModelTrait, ColumnTrait, EntityTrait, Iterable, PrimaryKeyToColumn, QueryFilter,
    QueryTrait,
};
use core::marker::PhantomData;
use sea_query::{Alias, Expr, IntoIden, SimpleExpr, UpdateStatement};

/// Defines a structure to perform UPDATE query operations on a ActiveModel
#[derive(Clone, Debug)]
pub struct Update;

/// Defines an UPDATE operation on one ActiveModel
#[derive(Clone, Debug)]
pub struct UpdateOne<A>
where
    A: ActiveModelTrait,
{
    pub(crate) query: UpdateStatement,
    pub(crate) model: A,
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
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     Update::one(cake::ActiveModel {
    ///         id: ActiveValue::set(1),
    ///         name: ActiveValue::set("Apple Pie".to_owned()),
    ///     })
    ///     .build(DbBackend::Postgres)
    ///     .to_string(),
    ///     r#"UPDATE "cake" SET "name" = 'Apple Pie' WHERE "cake"."id" = 1"#,
    /// );
    /// ```
    pub fn one<E, A>(model: A) -> UpdateOne<A>
    where
        E: EntityTrait,
        A: ActiveModelTrait<Entity = E>,
    {
        UpdateOne {
            query: UpdateStatement::new()
                .table(A::Entity::default().table_ref())
                .to_owned(),
            model,
        }
        .prepare_filters()
        .prepare_values()
    }

    /// Update many ActiveModel
    ///
    /// ```
    /// use sea_orm::{entity::*, query::*, sea_query::Expr, tests_cfg::fruit, DbBackend};
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

impl<A> UpdateOne<A>
where
    A: ActiveModelTrait,
{
    fn prepare_filters(mut self) -> Self {
        for key in <A::Entity as EntityTrait>::PrimaryKey::iter() {
            let col = key.into_column();
            let av = self.model.get(col);
            if av.is_set() || av.is_unchanged() {
                self = self.filter(col.eq(av.unwrap()));
            } else {
                panic!("PrimaryKey is not set");
            }
        }
        self
    }

    fn prepare_values(mut self) -> Self {
        for col in <A::Entity as EntityTrait>::Column::iter() {
            if <A::Entity as EntityTrait>::PrimaryKey::from_column(col).is_some() {
                continue;
            }
            let av = self.model.get(col);
            if av.is_set() {
                let val = Expr::val(av.into_value().unwrap());
                let col_def = col.def();
                let col_type = col_def.get_column_type();
                let expr = match col_type.get_enum_name() {
                    Some(enum_name) => val.as_enum(Alias::new(enum_name)),
                    None => val.into(),
                };
                self.query.value_expr(col, expr);
            }
        }
        self
    }
}

impl<A> QueryFilter for UpdateOne<A>
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

impl<A> QueryTrait for UpdateOne<A>
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
            let av = model.get(col);
            if av.is_set() {
                self.query.value(col, av.unwrap());
            }
        }
        self
    }

    /// Creates a [SimpleExpr] from a column
    pub fn col_expr<T>(mut self, col: T, expr: SimpleExpr) -> Self
    where
        T: IntoIden,
    {
        self.query.col_expr(col, expr);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::{cake, fruit};
    use crate::{entity::*, query::*, DbBackend};
    use sea_query::{Expr, Value};

    #[test]
    fn update_1() {
        assert_eq!(
            Update::one(cake::ActiveModel {
                id: ActiveValue::set(1),
                name: ActiveValue::set("Apple Pie".to_owned()),
            })
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
}
