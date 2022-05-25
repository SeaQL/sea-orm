use crate::{
    ColumnTrait, ConnectionTrait, DbErr, EntityTrait, FromQueryResult, QueryOrder, Select,
    SelectModel, SelectorTrait,
};
use sea_query::{OrderedStatement, SelectStatement, Value};
use std::marker::PhantomData;

/// Cursor pagination
///
/// To ensure proper ordering of the paginated result, the select statement must have order by expression.
#[derive(Debug, Clone)]
pub struct Cursor<S>
where
    S: SelectorTrait,
{
    pub(crate) query: SelectStatement,
    pub(crate) last: bool,
    pub(crate) phantom: PhantomData<S>,
}

impl<S> Cursor<S>
where
    S: SelectorTrait,
{
    /// Initialize a cursor
    pub fn new(query: SelectStatement) -> Self {
        Self {
            query,
            last: false,
            phantom: PhantomData,
        }
    }

    /// Filter paginated rows with column value less than the input value
    pub fn before<C, V>(&mut self, col: C, val: V) -> &mut Self
    where
        C: ColumnTrait,
        V: Into<Value>,
    {
        self.query.and_where(col.lt(val));
        self
    }

    /// Filter paginated rows with column value greater than the input value
    pub fn after<C, V>(&mut self, col: C, val: V) -> &mut Self
    where
        C: ColumnTrait,
        V: Into<Value>,
    {
        self.query.and_where(col.gt(val));
        self
    }

    fn reverse_ordering(&mut self) {
        self.query.orders_mut_for_each(|order_expr| {
            order_expr.reverse_ordering();
        });
    }

    /// Limit result set to only first N rows in ascending order of the paginated query
    pub fn first(&mut self, num_rows: u64) -> &mut Self {
        self.query.limit(num_rows);
        if self.last {
            self.reverse_ordering();
        }
        self.last = false;
        self
    }

    /// Limit result set to only last N rows in ascending order of the paginated query
    pub fn last(&mut self, num_rows: u64) -> &mut Self {
        self.query.limit(num_rows);
        if !self.last {
            self.reverse_ordering();
        }
        self.last = true;
        self
    }

    /// Fetch the rows
    pub async fn all<C>(&self, db: &C) -> Result<Vec<S::Item>, DbErr>
    where
        C: ConnectionTrait,
    {
        let builder = db.get_database_backend();
        let stmt = builder.build(&self.query);
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
    fn cursor(self) -> Cursor<Self::Selector>;
}

impl<E, M> CursorTrait for Select<E>
where
    E: EntityTrait<Model = M>,
    M: FromQueryResult + Sized + Send + Sync,
{
    type Selector = SelectModel<M>;

    fn cursor(self) -> Cursor<Self::Selector> {
        Cursor::new(self.query)
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
                .cursor()
                .order_by_asc(Column::Id)
                .before(Column::Id, 10)
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
                .order_by_asc(Column::Id)
                .cursor()
                .after(Column::Id, 10)
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
                .order_by_asc(Column::Id)
                .cursor()
                .after(Column::Id, 25)
                .before(Column::Id, 30)
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
}
