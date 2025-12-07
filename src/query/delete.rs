use crate::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, Iterable,
    PrimaryKeyToColumn, QueryFilter, QueryTrait,
};
use core::marker::PhantomData;
use sea_query::DeleteStatement;

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
    pub(crate) model: A,
    pub(crate) persistent: Option<bool>,
}

/// Perform a delete operation on multiple models
#[derive(Clone, Debug)]
pub struct DeleteMany<E>
where
    E: EntityTrait,
{
    pub(crate) query: DeleteStatement,
    pub(crate) entity: PhantomData<E>,
    pub(crate) persistent: Option<bool>,
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
            persistent: None,
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
                .from_table(entity.table_ref())
                .to_owned(),
            entity: PhantomData,
            persistent: None,
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
            match av {
                ActiveValue::Set(value) | ActiveValue::Unchanged(value) => {
                    self = self.filter(col.eq(value));
                }
                ActiveValue::NotSet => panic!("PrimaryKey is not set"),
            }
        }
        self
    }

    /// Set whether to use prepared statement caching
    ///
    /// # Examples
    ///
    /// ```
    /// # use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    /// #
    /// let res = Delete::one(cake::Model {
    ///     id: 1,
    ///     name: "Apple Pie".to_owned(),
    /// })
    /// .persistent(false);
    /// ```
    pub fn persistent(mut self, value: bool) -> Self {
        self.persistent = Some(value);
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

    fn build(&self, db_backend: crate::DbBackend) -> crate::Statement {
        let query_builder = db_backend.get_query_builder();
        let mut statement = crate::Statement::from_string_values_tuple(
            db_backend,
            self.as_query().build_any(query_builder.as_ref()),
        );
        statement.persistent = self.persistent;
        statement
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

    fn build(&self, db_backend: crate::DbBackend) -> crate::Statement {
        let query_builder = db_backend.get_query_builder();
        let mut statement = crate::Statement::from_string_values_tuple(
            db_backend,
            self.as_query().build_any(query_builder.as_ref()),
        );
        statement.persistent = self.persistent;
        statement
    }
}

impl<E> DeleteMany<E>
where
    E: EntityTrait,
{
    /// Set whether to use prepared statement caching
    ///
    /// # Examples
    ///
    /// ```
    /// # use sea_orm::{entity::*, query::*, tests_cfg::fruit, DbBackend};
    /// #
    /// let res = Delete::many(fruit::Entity)
    ///     .filter(fruit::Column::Name.contains("Apple"))
    ///     .persistent(false);
    /// ```
    pub fn persistent(mut self, value: bool) -> Self {
        self.persistent = Some(value);
        self
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
