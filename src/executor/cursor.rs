use crate::{
    ConnectionTrait, DbErr, EntityTrait, FromQueryResult, Identity, IntoIdentity, QueryOrder,
    Select, SelectModel, SelectorTrait,
};
use sea_query::{
    Condition, DynIden, Expr, IntoValueTuple, Order, OrderedStatement, SeaRc, SelectStatement,
    SimpleExpr, Value, ValueTuple,
};
use std::marker::PhantomData;

#[cfg(feature = "with-json")]
use crate::JsonValue;

/// Cursor pagination
#[derive(Debug, Clone)]
pub struct Cursor<S>
where
    S: SelectorTrait,
{
    pub(crate) query: SelectStatement,
    pub(crate) table: DynIden,
    pub(crate) order_columns: Identity,
    pub(crate) last: bool,
    pub(crate) phantom: PhantomData<S>,
}

impl<S> Cursor<S>
where
    S: SelectorTrait,
{
    /// Initialize a cursor
    pub fn new<C>(query: SelectStatement, table: DynIden, order_columns: C) -> Self
    where
        C: IntoIdentity,
    {
        Self {
            query,
            table,
            order_columns: order_columns.into_identity(),
            last: false,
            phantom: PhantomData,
        }
    }

    /// Filter paginated result with corresponding column less than the input value
    pub fn before<V>(&mut self, values: V) -> &mut Self
    where
        V: IntoValueTuple,
    {
        let condition = self.apply_filter(values, |c, v| {
            Expr::tbl(SeaRc::clone(&self.table), SeaRc::clone(c)).lt(v)
        });
        self.query.cond_where(condition);
        self
    }

    /// Filter paginated result with corresponding column greater than the input value
    pub fn after<V>(&mut self, values: V) -> &mut Self
    where
        V: IntoValueTuple,
    {
        let condition = self.apply_filter(values, |c, v| {
            Expr::tbl(SeaRc::clone(&self.table), SeaRc::clone(c)).gt(v)
        });
        self.query.cond_where(condition);
        self
    }

    fn apply_filter<V, F>(&self, values: V, f: F) -> Condition
    where
        V: IntoValueTuple,
        F: Fn(&DynIden, Value) -> SimpleExpr,
    {
        match (&self.order_columns, values.into_value_tuple()) {
            (Identity::Unary(c1), ValueTuple::One(v1)) => Condition::all().add(f(c1, v1)),
            (Identity::Binary(c1, c2), ValueTuple::Two(v1, v2)) => {
                Condition::all().add(f(c1, v1)).add(f(c2, v2))
            }
            (Identity::Ternary(c1, c2, c3), ValueTuple::Three(v1, v2, v3)) => Condition::all()
                .add(f(c1, v1))
                .add(f(c2, v2))
                .add(f(c3, v3)),
            _ => panic!("column arity mismatch"),
        }
    }

    /// Limit result set to only first N rows in ascending order of the order by column
    pub fn first(&mut self, num_rows: u64) -> &mut Self {
        self.query.limit(num_rows).clear_order_by();
        let table = SeaRc::clone(&self.table);
        self.apply_order_by(|query, col| {
            query.order_by((SeaRc::clone(&table), SeaRc::clone(col)), Order::Asc);
        });
        self.last = false;
        self
    }

    /// Limit result set to only last N rows in ascending order of the order by column
    pub fn last(&mut self, num_rows: u64) -> &mut Self {
        self.query.limit(num_rows).clear_order_by();
        let table = SeaRc::clone(&self.table);
        self.apply_order_by(|query, col| {
            query.order_by((SeaRc::clone(&table), SeaRc::clone(col)), Order::Desc);
        });
        self.last = true;
        self
    }

    fn apply_order_by<F>(&mut self, f: F)
    where
        F: Fn(&mut SelectStatement, &DynIden),
    {
        let query = &mut self.query;
        match &self.order_columns {
            Identity::Unary(c1) => {
                f(query, c1);
            }
            Identity::Binary(c1, c2) => {
                f(query, c1);
                f(query, c2);
            }
            Identity::Ternary(c1, c2, c3) => {
                f(query, c1);
                f(query, c2);
                f(query, c3);
            }
        }
    }

    /// Fetch the paginated result
    pub async fn all<C>(&mut self, db: &C) -> Result<Vec<S::Item>, DbErr>
    where
        C: ConnectionTrait,
    {
        let stmt = db.get_database_backend().build(&self.query);
        let rows = db.query_all(stmt).await?;
        let mut buffer = Vec::with_capacity(rows.len());
        for row in rows.into_iter() {
            buffer.push(S::from_raw_query_result(row)?);
        }
        if self.last {
            buffer.reverse()
        }
        Ok(buffer)
    }

    /// Construct a [Cursor] that fetch any custom struct
    pub fn into_model<M>(self) -> Cursor<SelectModel<M>>
    where
        M: FromQueryResult,
    {
        Cursor {
            query: self.query,
            table: self.table,
            order_columns: self.order_columns,
            last: self.last,
            phantom: PhantomData,
        }
    }

    /// Construct a [Cursor] that fetch JSON value
    #[cfg(feature = "with-json")]
    pub fn into_json(self) -> Cursor<SelectModel<JsonValue>> {
        Cursor {
            query: self.query,
            table: self.table,
            order_columns: self.order_columns,
            last: self.last,
            phantom: PhantomData,
        }
    }
}

impl<S> QueryOrder for Cursor<S>
where
    S: SelectorTrait,
{
    type QueryStatement = SelectStatement;

    fn query(&mut self) -> &mut SelectStatement {
        &mut self.query
    }
}

/// A trait for any type that can be turn into a cursor
pub trait CursorTrait {
    /// Select operation
    type Selector: SelectorTrait + Send + Sync;

    /// Convert current type into a cursor
    fn cursor_by<C>(self, order_columns: C) -> Cursor<Self::Selector>
    where
        C: IntoIdentity;
}

impl<E, M> CursorTrait for Select<E>
where
    E: EntityTrait<Model = M>,
    M: FromQueryResult + Sized + Send + Sync,
{
    type Selector = SelectModel<M>;

    fn cursor_by<C>(self, order_columns: C) -> Cursor<Self::Selector>
    where
        C: IntoIdentity,
    {
        Cursor::new(self.query, SeaRc::new(E::default()), order_columns)
    }
}

#[cfg(test)]
#[cfg(feature = "mock")]
mod tests {
    use super::*;
    use crate::entity::prelude::*;
    use crate::tests_cfg::*;
    use crate::{DbBackend, MockDatabase, Statement, Transaction};
    use pretty_assertions::assert_eq;

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
            Entity::find()
                .cursor_by(Column::Id)
                .before(10)
                .first(2)
                .all(&db)
                .await?,
            models
        );

        assert_eq!(
            db.into_transaction_log(),
            vec![Transaction::many(vec![Statement::from_sql_and_values(
                DbBackend::Postgres,
                [
                    r#"SELECT "fruit"."id", "fruit"."name", "fruit"."cake_id""#,
                    r#"FROM "fruit""#,
                    r#"WHERE "fruit"."id" < $1"#,
                    r#"ORDER BY "fruit"."id" ASC"#,
                    r#"LIMIT $2"#,
                ]
                .join(" ")
                .as_str(),
                vec![10_i32.into(), 2_u64.into()]
            ),])]
        );

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
            Entity::find()
                .cursor_by(Column::Id)
                .after(10)
                .last(2)
                .all(&db)
                .await?,
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

        assert_eq!(
            db.into_transaction_log(),
            vec![Transaction::many(vec![Statement::from_sql_and_values(
                DbBackend::Postgres,
                [
                    r#"SELECT "fruit"."id", "fruit"."name", "fruit"."cake_id""#,
                    r#"FROM "fruit""#,
                    r#"WHERE "fruit"."id" > $1"#,
                    r#"ORDER BY "fruit"."id" DESC"#,
                    r#"LIMIT $2"#,
                ]
                .join(" ")
                .as_str(),
                vec![10_i32.into(), 2_u64.into()]
            ),])]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn last_2_after_25_before_30() -> Result<(), DbErr> {
        use fruit::*;

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results(vec![vec![
                Model {
                    id: 27,
                    name: "Rasberry".into(),
                    cake_id: Some(1),
                },
                Model {
                    id: 26,
                    name: "Blueberry".into(),
                    cake_id: Some(1),
                },
            ]])
            .into_connection();

        assert_eq!(
            Entity::find()
                .cursor_by(Column::Id)
                .after(25)
                .before(30)
                .last(2)
                .all(&db)
                .await?,
            vec![
                Model {
                    id: 26,
                    name: "Blueberry".into(),
                    cake_id: Some(1),
                },
                Model {
                    id: 27,
                    name: "Rasberry".into(),
                    cake_id: Some(1),
                },
            ]
        );

        assert_eq!(
            db.into_transaction_log(),
            vec![Transaction::many(vec![Statement::from_sql_and_values(
                DbBackend::Postgres,
                [
                    r#"SELECT "fruit"."id", "fruit"."name", "fruit"."cake_id""#,
                    r#"FROM "fruit""#,
                    r#"WHERE "fruit"."id" > $1"#,
                    r#"AND "fruit"."id" < $2"#,
                    r#"ORDER BY "fruit"."id" DESC"#,
                    r#"LIMIT $3"#,
                ]
                .join(" ")
                .as_str(),
                vec![25_i32.into(), 30_i32.into(), 2_u64.into()]
            ),])]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn composite_keys() -> Result<(), DbErr> {
        use cake_filling::*;

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results(vec![vec![
                Model {
                    cake_id: 1,
                    filling_id: 2,
                },
                Model {
                    cake_id: 1,
                    filling_id: 3,
                },
                Model {
                    cake_id: 2,
                    filling_id: 3,
                },
            ]])
            .into_connection();

        assert_eq!(
            Entity::find()
                .cursor_by((Column::CakeId, Column::FillingId))
                .after((0, 1))
                .before((10, 11))
                .first(3)
                .all(&db)
                .await?,
            vec![
                Model {
                    cake_id: 1,
                    filling_id: 2,
                },
                Model {
                    cake_id: 1,
                    filling_id: 3,
                },
                Model {
                    cake_id: 2,
                    filling_id: 3,
                },
            ]
        );

        assert_eq!(
            db.into_transaction_log(),
            vec![Transaction::many(vec![Statement::from_sql_and_values(
                DbBackend::Postgres,
                [
                    r#"SELECT "cake_filling"."cake_id", "cake_filling"."filling_id""#,
                    r#"FROM "cake_filling""#,
                    r#"WHERE "cake_filling"."cake_id" > $1"#,
                    r#"AND "cake_filling"."filling_id" > $2"#,
                    r#"AND ("cake_filling"."cake_id" < $3"#,
                    r#"AND "cake_filling"."filling_id" < $4)"#,
                    r#"ORDER BY "cake_filling"."cake_id" ASC, "cake_filling"."filling_id" ASC"#,
                    r#"LIMIT $5"#,
                ]
                .join(" ")
                .as_str(),
                vec![
                    0_i32.into(),
                    1_i32.into(),
                    10_i32.into(),
                    11_i32.into(),
                    3_u64.into()
                ]
            ),])]
        );

        Ok(())
    }
}
