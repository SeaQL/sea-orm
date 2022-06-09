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
    Force { query: DeleteStatement, model: A },
    Soft { query: UpdateStatement, model: A },
}

/// Perform a delete operation on multiple models
#[derive(Clone, Debug)]
pub enum DeleteMany<E>
where
    E: EntityTrait,
{
    Force {
        query: DeleteStatement,
        entity: PhantomData<E>,
    },
    Soft {
        query: UpdateStatement,
        entity: PhantomData<E>,
    },
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

    ///
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

    ///
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

impl<A> ConditionalStatement for DeleteOne<A>
where
    A: ActiveModelTrait,
{
    fn and_or_where(&mut self, condition: LogicalChainOper) -> &mut Self {
        match self {
            DeleteOne::Force { query, .. } => {
                query.and_or_where(condition);
            }
            DeleteOne::Soft { query, .. } => {
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
            DeleteOne::Force { query, .. } => {
                query.cond_where(condition);
            }
            DeleteOne::Soft { query, .. } => {
                query.cond_where(condition);
            }
        }
        self
    }
}

impl<A> QueryFilter for DeleteOne<A>
where
    A: ActiveModelTrait,
{
    type QueryStatement = Self;

    fn query(&mut self) -> &mut Self {
        self
    }
}

impl<E> ConditionalStatement for DeleteMany<E>
where
    E: EntityTrait,
{
    fn and_or_where(&mut self, condition: LogicalChainOper) -> &mut Self {
        match self {
            DeleteMany::Force { query, .. } => {
                query.and_or_where(condition);
            }
            DeleteMany::Soft { query, .. } => {
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
            DeleteMany::Force { query, .. } => {
                query.cond_where(condition);
            }
            DeleteMany::Soft { query, .. } => {
                query.cond_where(condition);
            }
        }
        self
    }
}

impl<E> QueryFilter for DeleteMany<E>
where
    E: EntityTrait,
{
    type QueryStatement = Self;

    fn query(&mut self) -> &mut Self {
        self
    }
}

impl<A> QueryStatementBuilder for DeleteOne<A>
where
    A: ActiveModelTrait,
{
    fn build_collect_any_into(
        &self,
        query_builder: &dyn sea_query::QueryBuilder,
        sql: &mut sea_query::SqlWriter,
        collector: &mut dyn FnMut(sea_query::Value),
    ) {
        match self {
            DeleteOne::Force { query, .. } => {
                query.build_collect_any_into(query_builder, sql, collector);
            }
            DeleteOne::Soft { query, .. } => {
                query.build_collect_any_into(query_builder, sql, collector);
            }
        }
    }

    fn into_sub_query_statement(self) -> sea_query::SubQueryStatement {
        match self {
            DeleteOne::Force { query, .. } => query.into_sub_query_statement(),
            DeleteOne::Soft { query, .. } => query.into_sub_query_statement(),
        }
    }
}

impl<A> QueryTrait for DeleteOne<A>
where
    A: ActiveModelTrait,
{
    type QueryStatement = Self;

    fn query(&mut self) -> &mut Self {
        self
    }

    fn as_query(&self) -> &Self {
        &self
    }

    fn into_query(self) -> Self {
        self
    }
}

impl<E> QueryStatementBuilder for DeleteMany<E>
where
    E: EntityTrait,
{
    fn build_collect_any_into(
        &self,
        query_builder: &dyn sea_query::QueryBuilder,
        sql: &mut sea_query::SqlWriter,
        collector: &mut dyn FnMut(sea_query::Value),
    ) {
        match self {
            DeleteMany::Force { query, .. } => {
                query.build_collect_any_into(query_builder, sql, collector);
            }
            DeleteMany::Soft { query, .. } => {
                query.build_collect_any_into(query_builder, sql, collector);
            }
        }
    }

    fn into_sub_query_statement(self) -> sea_query::SubQueryStatement {
        match self {
            DeleteMany::Force { query, .. } => query.into_sub_query_statement(),
            DeleteMany::Soft { query, .. } => query.into_sub_query_statement(),
        }
    }
}

impl<E> QueryTrait for DeleteMany<E>
where
    E: EntityTrait,
{
    type QueryStatement = Self;

    fn query(&mut self) -> &mut Self {
        self
    }

    fn as_query(&self) -> &Self {
        &self
    }

    fn into_query(self) -> Self {
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
