use crate::{
    ColumnTrait, ConnectionTrait, DbErr, EntityTrait, FromQueryResult, Iterable,
    PrimaryKeyToColumn, PrimaryKeyTrait, Select, SelectModel, SelectorTrait,
};
use sea_query::{Condition, IntoValueTuple, Order, SelectStatement, SimpleExpr, Value};
use std::marker::PhantomData;

/// Limit number of rows to be fetched from the cursor
#[derive(Debug)]
pub enum CursorLimit {
    /// Fetch first N rows in ascending order of primary key
    First(u32),
    /// Fetch last N rows in descending order of primary key
    Last(u32),
}

/// Cursor pagination
#[derive(Debug)]
pub struct Cursor<E, S>
where
    E: EntityTrait,
    S: SelectorTrait,
{
    pub(crate) query: SelectStatement,
    pub(crate) condition: Option<Condition>,
    pub(crate) limit: Option<CursorLimit>,
    pub(crate) phantom: PhantomData<(E, S)>,
}

impl<E, S> Cursor<E, S>
where
    E: EntityTrait,
    S: SelectorTrait,
{
    /// Initialize a cursor
    pub fn new(query: SelectStatement) -> Self {
        Self {
            query,
            condition: None,
            limit: None,
            phantom: PhantomData,
        }
    }

    /// Filter rows with primary key value less than the input value
    pub fn before(&mut self, values: <E::PrimaryKey as PrimaryKeyTrait>::ValueType) -> &mut Self {
        self.condition = Some(get_condition::<E, _>(values, |c: E::Column, v: Value| {
            c.lt(v)
        }));
        self
    }

    /// Filter rows with primary key value greater than the input value
    pub fn after(&mut self, values: <E::PrimaryKey as PrimaryKeyTrait>::ValueType) -> &mut Self {
        self.condition = Some(get_condition::<E, _>(values, |c: E::Column, v: Value| {
            c.gt(v)
        }));
        self
    }

    /// Limit result set to only first N rows in ascending order of the primary key
    pub fn first(&mut self, num_rows: u32) -> &mut Self {
        self.limit = Some(CursorLimit::First(num_rows));
        self
    }

    /// Limit result set to only last N rows in ascending order of the primary key
    pub fn last(&mut self, num_rows: u32) -> &mut Self {
        self.limit = Some(CursorLimit::Last(num_rows));
        self
    }

    /// Fetch the rows
    pub async fn all<C>(&self, db: &C) -> Result<Vec<S::Item>, DbErr>
    where
        C: ConnectionTrait,
    {
        let mut query = self.query.clone();
        if let Some(condition) = self.condition.clone() {
            query.cond_where(condition);
        }
        if let Some(limit) = &self.limit {
            let (order, limit) = match limit {
                CursorLimit::First(limit) => (Order::Asc, limit),
                CursorLimit::Last(limit) => (Order::Desc, limit),
            };
            for key in E::PrimaryKey::iter() {
                query.order_by(key, order.clone());
            }
            query.limit(*limit as u64);
        }
        let builder = db.get_database_backend();
        let stmt = builder.build(&query);
        let rows = db.query_all(stmt).await?;
        let mut buffer = Vec::with_capacity(rows.len());
        for row in rows.into_iter() {
            buffer.push(S::from_raw_query_result(row)?);
        }
        if let Some(CursorLimit::Last(_)) = &self.limit {
            buffer.reverse()
        }
        Ok(buffer)
    }
}

fn get_condition<E, F>(
    values: <E::PrimaryKey as PrimaryKeyTrait>::ValueType,
    filter_fn: F,
) -> Condition
where
    E: EntityTrait,
    F: Fn(E::Column, Value) -> SimpleExpr,
{
    let mut condition = Condition::all();
    let mut keys = E::PrimaryKey::iter();
    for v in values.into_value_tuple() {
        if let Some(key) = keys.next() {
            let col = key.into_column();
            condition = condition.add(filter_fn(col, v));
        } else {
            panic!("primary key arity mismatch");
        }
    }
    if keys.next().is_some() {
        panic!("primary key arity mismatch");
    }
    condition
}

/// A trait for any type that can be turn into a cursor
pub trait CursorTrait<E>
where
    E: EntityTrait,
{
    /// Select operation
    type Selector: SelectorTrait + Send + Sync;

    /// Convert current type into a cursor
    fn cursor(self) -> Cursor<E, Self::Selector>;
}

impl<E, M> CursorTrait<E> for Select<E>
where
    E: EntityTrait<Model = M>,
    M: FromQueryResult + Sized + Send + Sync,
{
    type Selector = SelectModel<M>;

    fn cursor(self) -> Cursor<E, Self::Selector> {
        Cursor::new(self.query)
    }
}

#[cfg(test)]
#[cfg(feature = "mock")]
mod tests {
    use super::*;
    use crate::entity::prelude::*;
    use crate::{tests_cfg::*, ConnectionTrait};
    use crate::{DbBackend, MockDatabase, Transaction};
    use pretty_assertions::assert_eq;
    use sea_query::{Expr, SelectStatement};

    #[smol_potat::test]
    async fn first_2_before_10() -> Result<(), DbErr> {
        use fruit::*;

        let models = vec![
            Model {
                id: 1,
                name: "Blueberry".into(),
                cake_id: Some(1),
            },
            Model {
                id: 2,
                name: "Rasberry".into(),
                cake_id: Some(1),
            },
        ];

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results(vec![models.clone()])
            .into_connection();

        assert_eq!(
            Entity::find().cursor().before(10).first(2).all(&db).await?,
            models
        );

        let select = SelectStatement::new()
            .exprs(vec![
                Expr::tbl(Entity, Column::Id),
                Expr::tbl(Entity, Column::Name),
                Expr::tbl(Entity, Column::CakeId),
            ])
            .from(Entity)
            .and_where(Column::Id.lt(10))
            .order_by(Column::Id, Order::Asc)
            .limit(2)
            .to_owned();

        let query_builder = db.get_database_backend();
        let stmts = vec![query_builder.build(&select)];

        assert_eq!(db.into_transaction_log(), Transaction::wrap(stmts));

        Ok(())
    }

    #[smol_potat::test]
    async fn last_2_after_10() -> Result<(), DbErr> {
        use fruit::*;

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results(vec![vec![
                Model {
                    id: 22,
                    name: "Rasberry".into(),
                    cake_id: Some(1),
                },
                Model {
                    id: 21,
                    name: "Blueberry".into(),
                    cake_id: Some(1),
                },
            ]])
            .into_connection();

        assert_eq!(
            Entity::find().cursor().after(10).last(2).all(&db).await?,
            vec![
                Model {
                    id: 21,
                    name: "Blueberry".into(),
                    cake_id: Some(1),
                },
                Model {
                    id: 22,
                    name: "Rasberry".into(),
                    cake_id: Some(1),
                },
            ]
        );

        let select = SelectStatement::new()
            .exprs(vec![
                Expr::tbl(Entity, Column::Id),
                Expr::tbl(Entity, Column::Name),
                Expr::tbl(Entity, Column::CakeId),
            ])
            .from(Entity)
            .and_where(Column::Id.gt(10))
            .order_by(Column::Id, Order::Desc)
            .limit(2)
            .to_owned();

        let query_builder = db.get_database_backend();
        let stmts = vec![query_builder.build(&select)];

        assert_eq!(db.into_transaction_log(), Transaction::wrap(stmts));

        Ok(())
    }
}
