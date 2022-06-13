use crate::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, Iterable, PrimaryKeyToColumn,
    QueryFilter, QueryTrait, SoftDeleteTrait,
};
use core::marker::PhantomData;
use sea_query::{
    ConditionalStatement, DeleteStatement, IntoCondition, LogicalChainOper, QueryStatementBuilder,
    UpdateStatement,
};

/// Defines the structure for a delete operation
#[derive(Clone, Debug)]
pub struct Delete;

/// Perform a delete operation on a model
#[derive(Clone, Debug)]
pub enum DeleteOne<A>
where
    A: ActiveModelTrait,
{
    /// Force delete
    Force {
        /// Delete statement
        query: DeleteStatement,
        /// Active model
        model: A,
    },
    /// Soft delete
    Soft {
        /// Update statement
        query: UpdateStatement,
        /// Active model
        model: A,
    },
}

/// Perform a delete operation on multiple models
#[derive(Clone, Debug)]
pub enum DeleteMany<E>
where
    E: EntityTrait,
{
    /// Force delete
    Force {
        /// Delete statement
        query: DeleteStatement,
        /// Phantom
        entity: PhantomData<E>,
    },
    /// Soft delete
    Soft {
        /// Update statement
        query: UpdateStatement,
        /// Phantom
        entity: PhantomData<E>,
    },
}

impl Delete {
    /// Force delete one Model or ActiveModel
    ///
    /// # Example
    ///
    /// Model
    ///
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     Delete::one_force(cake::Model {
    ///         id: 1,
    ///         name: "Apple Pie".to_owned(),
    ///     })
    ///     .build(DbBackend::Postgres)
    ///     .to_string(),
    ///     r#"DELETE FROM "cake" WHERE "cake"."id" = 1"#,
    /// );
    /// ```
    ///
    /// ActiveModel
    ///
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     Delete::one_force(cake::ActiveModel {
    ///         id: ActiveValue::set(1),
    ///         name: ActiveValue::set("Apple Pie".to_owned()),
    ///     })
    ///     .build(DbBackend::Postgres)
    ///     .to_string(),
    ///     r#"DELETE FROM "cake" WHERE "cake"."id" = 1"#,
    /// );
    /// ```
    pub fn one_force<E, A, M>(model: M) -> DeleteOne<A>
    where
        E: EntityTrait,
        A: ActiveModelTrait<Entity = E>,
        M: IntoActiveModel<A>,
    {
        DeleteOne::Force {
            query: DeleteStatement::new()
                .from_table(A::Entity::default().table_ref())
                .to_owned(),
            model: model.into_active_model(),
        }
        .prepare()
    }

    /// Delete one Model or ActiveModel
    ///   - Marking the target row in the database as deleted if soft delete is enabled
    ///   - Otherwise, deleting the target row from the database
    ///
    /// # Example (without soft delete)
    ///
    /// Model
    ///
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
    ///
    /// ActiveModel
    ///
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
    ///
    /// # Example (with soft delete)
    ///
    /// Model
    ///
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::vendor, DbBackend};
    ///
    /// assert_eq!(
    ///     Delete::one(vendor::Model {
    ///         id: 1,
    ///         name: "Vendor A".to_owned(),
    ///         deleted_at: None,
    ///     })
    ///     .build(DbBackend::Postgres)
    ///     .to_string(),
    ///     r#"UPDATE "vendor" SET "deleted_at" = CURRENT_TIMESTAMP WHERE "vendor"."id" = 1"#,
    /// );
    /// ```
    ///
    /// ActiveModel
    ///
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::vendor, DbBackend};
    ///
    /// assert_eq!(
    ///     Delete::one(vendor::ActiveModel {
    ///         id: ActiveValue::set(1),
    ///         name: ActiveValue::set("Vendor A".to_owned()),
    ///         deleted_at: ActiveValue::set(None),
    ///     })
    ///     .build(DbBackend::Postgres)
    ///     .to_string(),
    ///     r#"UPDATE "vendor" SET "deleted_at" = CURRENT_TIMESTAMP WHERE "vendor"."id" = 1"#,
    /// );
    /// ```
    pub fn one<E, A, M>(model: M) -> DeleteOne<A>
    where
        E: EntityTrait,
        A: ActiveModelTrait<Entity = E>,
        M: IntoActiveModel<A>,
    {
        match <E::Column as SoftDeleteTrait>::soft_delete_column() {
            Some(col) => DeleteOne::Soft {
                query: UpdateStatement::new()
                    .table(A::Entity::default().table_ref())
                    .col_expr(col, <E::Column as SoftDeleteTrait>::soft_delete_expr())
                    .to_owned(),
                model: model.into_active_model(),
            }
            .prepare(),
            None => Self::one_force(model),
        }
    }

    /// Force delete many ActiveModel
    ///
    /// # Example
    ///
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit, DbBackend};
    ///
    /// assert_eq!(
    ///     Delete::many_force(fruit::Entity)
    ///         .filter(fruit::Column::Name.contains("Apple"))
    ///         .build(DbBackend::Postgres)
    ///         .to_string(),
    ///     r#"DELETE FROM "fruit" WHERE "fruit"."name" LIKE '%Apple%'"#,
    /// );
    /// ```
    pub fn many_force<E>(entity: E) -> DeleteMany<E>
    where
        E: EntityTrait,
    {
        DeleteMany::Force {
            query: DeleteStatement::new()
                .from_table(entity.table_ref())
                .to_owned(),
            entity: PhantomData,
        }
    }

    /// Delete many ActiveModel
    ///   - Marking the target rows in the database as deleted if soft delete is enabled
    ///   - Otherwise, deleting the target rows from the database
    ///
    /// # Example (without soft delete)
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
    ///
    /// # Example (with soft delete)
    ///
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::vendor, DbBackend};
    ///
    /// assert_eq!(
    ///     Delete::many(vendor::Entity)
    ///         .filter(vendor::Column::Name.contains("Vendor"))
    ///         .build(DbBackend::Postgres)
    ///         .to_string(),
    ///     r#"UPDATE "vendor" SET "deleted_at" = CURRENT_TIMESTAMP WHERE "vendor"."name" LIKE '%Vendor%'"#,
    /// );
    /// ```
    pub fn many<E>(entity: E) -> DeleteMany<E>
    where
        E: EntityTrait,
    {
        match <E::Column as SoftDeleteTrait>::soft_delete_column() {
            Some(col) => DeleteMany::Soft {
                query: UpdateStatement::new()
                    .table(entity.table_ref())
                    .col_expr(col, <E::Column as SoftDeleteTrait>::soft_delete_expr())
                    .to_owned(),
                entity: PhantomData,
            },
            None => Self::many_force(entity),
        }
    }
}

impl<A> DeleteOne<A>
where
    A: ActiveModelTrait,
{
    pub(crate) fn prepare(mut self) -> Self {
        let model = match self {
            DeleteOne::Force { ref model, .. } | DeleteOne::Soft { ref model, .. } => model.clone(),
        };
        for key in <A::Entity as EntityTrait>::PrimaryKey::iter() {
            let col = key.into_column();
            let av = model.get(col);
            if av.is_set() || av.is_unchanged() {
                self = self.filter(col.eq(av.unwrap()));
            } else {
                panic!("PrimaryKey is not set");
            }
        }
        self
    }
}

macro_rules! impl_traits {
    ($ty: ident, $t: ident) => {
        impl<T> QueryTrait for $ty<T>
        where
            T: $t,
        {
            type QueryStatement = Self;

            fn query(&mut self) -> &mut Self {
                self
            }

            fn as_query(&self) -> &Self {
                self
            }

            fn into_query(self) -> Self {
                self
            }
        }

        impl<T> QueryFilter for $ty<T>
        where
            T: $t,
        {
            type QueryStatement = Self;

            fn query(&mut self) -> &mut Self {
                self
            }
        }

        impl<T> ConditionalStatement for $ty<T>
        where
            T: $t,
        {
            fn and_or_where(&mut self, condition: LogicalChainOper) -> &mut Self {
                match self {
                    $ty::Force { query, .. } => {
                        query.and_or_where(condition);
                    }
                    $ty::Soft { query, .. } => {
                        query.and_or_where(condition);
                    }
                }
                self
            }

            fn cond_where<C>(&mut self, condition: C) -> &mut Self
            where
                C: IntoCondition,
            {
                match self {
                    $ty::Force { query, .. } => {
                        query.cond_where(condition);
                    }
                    $ty::Soft { query, .. } => {
                        query.cond_where(condition);
                    }
                }
                self
            }
        }

        impl<T> QueryStatementBuilder for $ty<T>
        where
            T: $t,
        {
            fn build_collect_any_into(
                &self,
                query_builder: &dyn sea_query::QueryBuilder,
                sql: &mut sea_query::SqlWriter,
                collector: &mut dyn FnMut(sea_query::Value),
            ) {
                match self {
                    $ty::Force { query, .. } => {
                        query.build_collect_any_into(query_builder, sql, collector);
                    }
                    $ty::Soft { query, .. } => {
                        query.build_collect_any_into(query_builder, sql, collector);
                    }
                }
            }

            fn into_sub_query_statement(self) -> sea_query::SubQueryStatement {
                match self {
                    $ty::Force { query, .. } => query.into_sub_query_statement(),
                    $ty::Soft { query, .. } => query.into_sub_query_statement(),
                }
            }
        }
    };
}

impl_traits!(DeleteOne, ActiveModelTrait);
impl_traits!(DeleteMany, EntityTrait);

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
