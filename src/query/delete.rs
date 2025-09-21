use crate::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, Iterable,
    PrimaryKeyToColumn, QueryFilter, QueryTrait,
};
use core::marker::PhantomData;
use sea_query::DeleteStatement;

/// Defines the structure for a delete operation
#[derive(Clone, Debug)]
pub struct Delete;

/// A request to delete an [`ActiveModel`](ActiveModelTrait).
///
/// The primary key must be set.
/// Otherwise, it's impossible to generate the SQL condition and find the record.
/// In that case, [`exec`][Self::exec] will return an error and not send any queries to the database.
///
/// If you want to use [`QueryTrait`] and access the generated SQL query,
/// you need to convert into [`ValidatedDeleteOne`] first.
#[derive(Clone, Debug)]
pub struct DeleteOne<A: ActiveModelTrait>(pub(crate) Result<ValidatedDeleteOne<A>, DbErr>);

/// A validated [`DeleteOne`] request, where the primary key is set
/// and it's possible to generate the right SQL condition.
#[derive(Clone, Debug)]
pub struct ValidatedDeleteOne<A: ActiveModelTrait> {
    pub(crate) query: DeleteStatement,
    pub(crate) model: A,
}

impl<A: ActiveModelTrait> TryFrom<DeleteOne<A>> for ValidatedDeleteOne<A> {
    type Error = DbErr;

    fn try_from(value: DeleteOne<A>) -> Result<Self, Self::Error> {
        value.0
    }
}

impl<A: ActiveModelTrait> DeleteOne<A> {
    /// Check whether the primary key is set and we can proceed with the operation.
    pub fn validate(self) -> Result<ValidatedDeleteOne<A>, DbErr> {
        self.try_into()
    }
}

/// Perform a delete operation on multiple models
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
    /// use sea_orm::{DbBackend, entity::*, query::*, tests_cfg::cake};
    ///
    /// assert_eq!(
    ///     Delete::one(cake::Model {
    ///         id: 1,
    ///         name: "Apple Pie".to_owned(),
    ///     })
    ///     .validate()
    ///     .unwrap()
    ///     .build(DbBackend::Postgres)
    ///     .to_string(),
    ///     r#"DELETE FROM "cake" WHERE "cake"."id" = 1"#,
    /// );
    /// ```
    /// ActiveModel
    /// ```
    /// use sea_orm::{DbBackend, entity::*, query::*, tests_cfg::cake};
    ///
    /// assert_eq!(
    ///     Delete::one(cake::ActiveModel {
    ///         id: ActiveValue::set(1),
    ///         name: ActiveValue::set("Apple Pie".to_owned()),
    ///     })
    ///     .validate()
    ///     .unwrap()
    ///     .build(DbBackend::Postgres)
    ///     .to_string(),
    ///     r#"DELETE FROM "cake" WHERE "cake"."id" = 1"#,
    /// );
    /// ```
    //
    // (non-doc comment for maintainers)
    // Ideally, we would make this method fallible instead of stashing and delaying the error.
    // But that's a bigger breaking change.
    pub fn one<E, A, M>(model: M) -> DeleteOne<A>
    where
        E: EntityTrait,
        A: ActiveModelTrait<Entity = E>,
        M: IntoActiveModel<A>,
    {
        let mut myself = ValidatedDeleteOne {
            query: DeleteStatement::new()
                .from_table(A::Entity::default().table_ref())
                .to_owned(),
            model: model.into_active_model(),
        };
        // Build the SQL condition from the primary key columns.
        for key in <A::Entity as EntityTrait>::PrimaryKey::iter() {
            let col = key.into_column();
            let av = myself.model.get(col);
            match av {
                ActiveValue::Set(value) | ActiveValue::Unchanged(value) => {
                    myself = myself.filter(col.eq(value));
                }
                ActiveValue::NotSet => {
                    return DeleteOne(Err(DbErr::PrimaryKeyNotSet { ctx: "DeleteOne" }));
                }
            }
        }
        DeleteOne(Ok(myself))
    }

    /// Delete many ActiveModel
    ///
    /// ```
    /// use sea_orm::{DbBackend, entity::*, query::*, tests_cfg::fruit};
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
        }
    }
}

impl<A> QueryFilter for ValidatedDeleteOne<A>
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

impl<A> QueryTrait for ValidatedDeleteOne<A>
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
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::{cake, fruit};
    use crate::{DbBackend, entity::*, query::*};

    #[test]
    fn delete_1() {
        assert_eq!(
            Delete::one(cake::Model {
                id: 1,
                name: "Apple Pie".to_owned(),
            })
            .validate()
            .unwrap()
            .build(DbBackend::Postgres)
            .to_string(),
            r#"DELETE FROM "cake" WHERE "cake"."id" = 1"#,
        );
        assert_eq!(
            Delete::one(cake::ActiveModel {
                id: ActiveValue::set(1),
                name: ActiveValue::set("Apple Pie".to_owned()),
            })
            .validate()
            .unwrap()
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
