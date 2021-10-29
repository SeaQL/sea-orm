use crate::{
    ActiveModelTrait, ColumnTrait, DbBackend, EntityTrait, IntoActiveModel, Iterable, ModelTrait,
    PrimaryKeyToColumn, QueryFilter, QueryTrait, Statement,
};
use core::marker::PhantomData;
use sea_query::{DeleteStatement, Expr, IntoIden, UpdateStatement};

#[derive(Clone, Debug)]
pub struct Delete;

#[derive(Clone, Debug)]
pub struct DeleteOne<A>
where
    A: ActiveModelTrait,
{
    pub(crate) query: DeleteStatement,
    pub(crate) model: A,
}

#[derive(Clone, Debug)]
pub struct DeleteMany<E>
where
    E: EntityTrait,
{
    pub(crate) query: DeleteStatement,
    pub(crate) entity: PhantomData<E>,
}

impl Delete {
    /// Delete one Model or ActiveModel
    ///
    /// Model
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     Delete::one(cake::Model {
    ///         id: 1,
    ///         name: "Apple Pie".to_owned(),
    ///     })
    ///     .build(DbBackend::Postgres)
    ///     .to_string(),
    ///     r#"DELETE FROM "cake" WHERE "cake"."id" = 1"#,
    /// );
    /// ```
    /// ActiveModel
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     Delete::one(cake::ActiveModel {
    ///         id: ActiveValue::set(1),
    ///         name: ActiveValue::set("Apple Pie".to_owned()),
    ///     })
    ///     .build(DbBackend::Postgres)
    ///     .to_string(),
    ///     r#"DELETE FROM "cake" WHERE "cake"."id" = 1"#,
    /// );
    /// ```
    pub fn one<E, A, M>(model: M) -> DeleteOne<A>
    where
        E: EntityTrait,
        A: ActiveModelTrait<Entity = E>,
        M: IntoActiveModel<A>,
    {
        let myself = DeleteOne {
            query: DeleteStatement::new()
                .from_table(A::Entity::default().table_ref())
                .to_owned(),
            model: model.into_active_model(),
        };
        myself.prepare()
    }

    /// Delete many ActiveModel
    ///
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit, DbBackend};
    ///
    /// assert_eq!(
    ///     Delete::many(fruit::Entity)
    ///         .filter(fruit::Column::Name.contains("Apple"))
    ///         .build(DbBackend::Postgres)
    ///         .to_string(),
    ///     r#"DELETE FROM "fruit" WHERE "fruit"."name" LIKE '%Apple%'"#,
    /// );
    /// ```
    pub fn many<E>(entity: E) -> DeleteMany<E>
    where
        E: EntityTrait,
    {
        DeleteMany {
            query: DeleteStatement::new()
                .from_table(entity.into_iden())
                .to_owned(),
            entity: PhantomData,
        }
    }
}

impl<A> DeleteOne<A>
where
    A: ActiveModelTrait,
{
    pub(crate) fn prepare(mut self) -> Self {
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
}

impl<A> QueryFilter for DeleteOne<A>
where
    A: ActiveModelTrait,
{
    type QueryStatement = DeleteStatement;

    fn query(&mut self) -> &mut DeleteStatement {
        &mut self.query
    }
}

impl<E> QueryFilter for DeleteMany<E>
where
    E: EntityTrait,
{
    type QueryStatement = DeleteStatement;

    fn query(&mut self) -> &mut DeleteStatement {
        &mut self.query
    }
}

impl<A> QueryTrait for DeleteOne<A>
where
    A: ActiveModelTrait,
{
    type QueryStatement = DeleteStatement;

    fn query(&mut self) -> &mut DeleteStatement {
        &mut self.query
    }

    fn as_query(&self) -> &DeleteStatement {
        &self.query
    }

    fn into_query(self) -> DeleteStatement {
        self.query
    }

    fn build(&self, db_backend: DbBackend) -> Statement {
        build_delete_stmt::<A::Entity>(self.as_query(), db_backend)
    }
}

impl<E> QueryTrait for DeleteMany<E>
where
    E: EntityTrait,
{
    type QueryStatement = DeleteStatement;

    fn query(&mut self) -> &mut DeleteStatement {
        &mut self.query
    }

    fn as_query(&self) -> &DeleteStatement {
        &self.query
    }

    fn into_query(self) -> DeleteStatement {
        self.query
    }

    fn build(&self, db_backend: DbBackend) -> Statement {
        build_delete_stmt::<E>(self.as_query(), db_backend)
    }
}

fn build_delete_stmt<E>(delete_stmt: &DeleteStatement, db_backend: DbBackend) -> Statement
where
    E: EntityTrait,
{
    let query_builder = db_backend.get_query_builder();
    match <<E as EntityTrait>::Model as ModelTrait>::soft_delete_column() {
        Some(soft_delete_column) => {
            let mut delete_stmt = delete_stmt.clone();
            let value = <E::Model as ModelTrait>::soft_delete_column_value();
            let update_stmt = UpdateStatement::new()
                .table(E::default())
                .col_expr(soft_delete_column, Expr::value(value))
                .set_conditions(delete_stmt.take_conditions())
                .to_owned();
            Statement::from_string_values_tuple(
                db_backend,
                update_stmt.build_any(query_builder.as_ref()),
            )
        }
        None => Statement::from_string_values_tuple(
            db_backend,
            delete_stmt.build_any(query_builder.as_ref()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::{cake, fruit};
    use crate::{entity::*, query::*, DbBackend};

    #[test]
    fn delete_1() {
        assert_eq!(
            Delete::one(cake::Model {
                id: 1,
                name: "Apple Pie".to_owned(),
            })
            .build(DbBackend::Postgres)
            .to_string(),
            r#"DELETE FROM "cake" WHERE "cake"."id" = 1"#,
        );
        assert_eq!(
            Delete::one(cake::ActiveModel {
                id: ActiveValue::set(1),
                name: ActiveValue::set("Apple Pie".to_owned()),
            })
            .build(DbBackend::Postgres)
            .to_string(),
            r#"DELETE FROM "cake" WHERE "cake"."id" = 1"#,
        );
    }

    #[test]
    fn delete_2() {
        assert_eq!(
            Delete::many(fruit::Entity)
                .filter(fruit::Column::Name.contains("Cheese"))
                .build(DbBackend::Postgres)
                .to_string(),
            r#"DELETE FROM "fruit" WHERE "fruit"."name" LIKE '%Cheese%'"#,
        );
    }
}
