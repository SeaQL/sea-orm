use crate::{
    convert_to_soft_delete, ActiveModelTrait, ColumnTrait, DbBackend, EntityTrait, IntoActiveModel,
    Iterable, ModelTrait, PrimaryKeyToColumn, QueryFilter, QueryTrait, Statement,
};
use core::marker::PhantomData;
use sea_query::{DeleteStatement, IntoIden};

/// Defines the structure for a delete operation
#[derive(Clone, Debug)]
pub struct Delete;

/// Perform a delete operation on a model
#[derive(Clone, Debug)]
pub struct DeleteOne<A>
where
    A: ActiveModelTrait,
{
    pub(crate) query: DeleteStatement,
    pub(crate) force_delete: bool,
    pub(crate) model: A,
}

/// Perform a delete operation on multiple models
#[derive(Clone, Debug)]
pub struct DeleteMany<E>
where
    E: EntityTrait,
{
    pub(crate) query: DeleteStatement,
    pub(crate) force_delete: bool,
    pub(crate) entity: PhantomData<E>,
}

impl Delete {
    /// Delete one Model or ActiveModel, soft delete will be performed if it's enabled
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
            force_delete: false,
            model: model.into_active_model(),
        };
        myself.prepare()
    }

    /// Force delete one Model or ActiveModel even when soft delete is enabled
    pub fn one_forcefully<E, A, M>(model: M) -> DeleteOne<A>
    where
        E: EntityTrait,
        A: ActiveModelTrait<Entity = E>,
        M: IntoActiveModel<A>,
    {
        let mut delete_one = Self::one(model);
        delete_one.force_delete = true;
        delete_one
    }

    /// Delete many ActiveModel, soft delete will be performed if it's enabled
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
            force_delete: false,
            entity: PhantomData,
        }
    }

    /// Force delete many ActiveModel even when soft delete is enabled
    pub fn many_forcefully<E>(entity: E) -> DeleteMany<E>
    where
        E: EntityTrait,
    {
        let mut delete_many = Self::many(entity);
        delete_many.force_delete = true;
        delete_many
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
        build_delete_stmt::<A::Entity>(self.as_query(), self.force_delete, db_backend)
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
        build_delete_stmt::<E>(self.as_query(), self.force_delete, db_backend)
    }
}

fn build_delete_stmt<E>(
    delete_stmt: &DeleteStatement,
    force_delete: bool,
    db_backend: DbBackend,
) -> Statement
where
    E: EntityTrait,
{
    let query_builder = db_backend.get_query_builder();
    match <<E as EntityTrait>::Model as ModelTrait>::soft_delete_column() {
        Some(soft_delete_column) if !force_delete => {
            let update_stmt = convert_to_soft_delete::<E>(delete_stmt.clone(), soft_delete_column);
            Statement::from_string_values_tuple(
                db_backend,
                update_stmt.build_any(query_builder.as_ref()),
            )
        }
        _ => Statement::from_string_values_tuple(
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
