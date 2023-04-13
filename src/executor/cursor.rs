use crate::{
    ConnectionTrait, DbErr, EntityTrait, FromQueryResult, Identity, IntoIdentity, QueryOrder,
    Select, SelectModel, SelectorTrait,
};
use sea_query::{
    Condition, DynIden, Expr, IntoValueTuple, Order, SeaRc, SelectStatement, SimpleExpr, Value,
    ValueTuple,
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
            Expr::col((SeaRc::clone(&self.table), SeaRc::clone(c))).lt(v)
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
            Expr::col((SeaRc::clone(&self.table), SeaRc::clone(c))).gt(v)
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
            (Identity::Binary(c1, c2), ValueTuple::Two(v1, v2)) => Condition::any()
                .add(
                    Condition::all()
                        .add(
                            Expr::col((SeaRc::clone(&self.table), SeaRc::clone(c1))).eq(v1.clone()),
                        )
                        .add(f(c2, v2)),
                )
                .add(f(c1, v1)),
            (Identity::Ternary(c1, c2, c3), ValueTuple::Three(v1, v2, v3)) => Condition::any()
                .add(
                    Condition::all()
                        .add(
                            Expr::col((SeaRc::clone(&self.table), SeaRc::clone(c1))).eq(v1.clone()),
                        )
                        .add(
                            Expr::col((SeaRc::clone(&self.table), SeaRc::clone(c2))).eq(v2.clone()),
                        )
                        .add(f(c3, v3)),
                )
                .add(
                    Condition::all()
                        .add(
                            Expr::col((SeaRc::clone(&self.table), SeaRc::clone(c1))).eq(v1.clone()),
                        )
                        .add(f(c2, v2)),
                )
                .add(f(c1, v1)),
            (Identity::Many(col_vec), ValueTuple::Many(val_vec))
                if col_vec.len() == val_vec.len() =>
            {
                // The length of `col_vec` and `val_vec` should be equal and is denoted by "n".
                //
                // The elements of `col_vec` and `val_vec` are denoted by:
                //   - `col_vec`: "col_1", "col_2", ..., "col_n-1", "col_n"
                //   - `val_vec`: "val_1", "val_2", ..., "val_n-1", "val_n"
                //
                // The general form of the where condition should have "n" number of inner-AND-condition chained by an outer-OR-condition.
                // The "n"-th inner-AND-condition should have exactly "n" number of column value expressions,
                // to construct the expression we take the first "n" number of column and value from the respected vector.
                //   - if it's not the last element, then we construct a "col_1 = val_1" equal expression
                //   - otherwise, for the last element, we should construct a "col_n > val_n" greater than or "col_n < val_n" less than expression.
                // i.e.
                // WHERE
                //   (col_1 = val_1 AND col_2 = val_2 AND ... AND col_n > val_n)
                //   OR (col_1 = val_1 AND col_2 = val_2 AND ... AND col_n-1 > val_n-1)
                //   OR (col_1 = val_1 AND col_2 = val_2 AND ... AND col_n-2 > val_n-2)
                //   OR ...
                //   OR (col_1 = val_1 AND col_2 > val_2)
                //   OR (col_1 > val_1)

                // Counting from 1 to "n" (inclusive) but in reverse, i.e. n, n-1, ..., 2, 1
                (1..=col_vec.len())
                    .rev()
                    .fold(Condition::any(), |cond_any, n| {
                        // Construct the inner-AND-condition
                        let inner_cond_all =
                            // Take the first "n" elements from the column and value vector respectively
                            col_vec.iter().zip(val_vec.iter()).enumerate().take(n).fold(
                                Condition::all(),
                                |inner_cond_all, (i, (col, val))| {
                                    let val = val.clone();
                                    // Construct a equal expression,
                                    // except for the last one being greater than or less than expression
                                    let expr = if i != (n - 1) {
                                        Expr::col((SeaRc::clone(&self.table), SeaRc::clone(col)))
                                            .eq(val)
                                    } else {
                                        f(col, val)
                                    };
                                    // Chain it with AND operator
                                    inner_cond_all.add(expr)
                                },
                            );
                        // Chain inner-AND-condition with OR operator
                        cond_any.add(inner_cond_all)
                    })
            }
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
            Identity::Many(vec) => {
                for col in vec.iter() {
                    f(query, col);
                }
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
    use crate::{
        entity::prelude::*, sea_query::IntoIden, tests_cfg::*, DbBackend, MockDatabase, Statement,
        Transaction,
    };
    use pretty_assertions::assert_eq;

    #[smol_potat::test]
    async fn first_2_before_10() -> Result<(), DbErr> {
        use fruit::*;

        let models = [
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
            .append_query_results([models.clone()])
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

        let stmt = Statement::from_sql_and_values(
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
            [10_i32.into(), 2_u64.into()],
        );

        assert_eq!(db.into_transaction_log(), [Transaction::one(stmt.clone())]);

        assert_eq!(
            DbBackend::Postgres.build(
                &Entity::find()
                    .cursor_by(Identity::Many(vec![Column::Id.into_iden()]))
                    .before(ValueTuple::Many(vec![10.into()]))
                    .first(2)
                    .query
            ),
            stmt
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn last_2_after_10() -> Result<(), DbErr> {
        use fruit::*;

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
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
            [
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

        let stmt = Statement::from_sql_and_values(
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
            [10_i32.into(), 2_u64.into()],
        );

        assert_eq!(db.into_transaction_log(), [Transaction::one(stmt.clone())]);

        assert_eq!(
            DbBackend::Postgres.build(
                &Entity::find()
                    .cursor_by(Identity::Many(vec![Column::Id.into_iden()]))
                    .after(ValueTuple::Many(vec![10.into()]))
                    .last(2)
                    .query
            ),
            stmt
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn last_2_after_25_before_30() -> Result<(), DbErr> {
        use fruit::*;

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[
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
            [
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

        let stmt = Statement::from_sql_and_values(
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
            [25_i32.into(), 30_i32.into(), 2_u64.into()],
        );

        assert_eq!(db.into_transaction_log(), [Transaction::one(stmt.clone())]);

        assert_eq!(
            DbBackend::Postgres.build(
                &Entity::find()
                    .cursor_by(Identity::Many(vec![Column::Id.into_iden()]))
                    .after(ValueTuple::Many(vec![25.into()]))
                    .before(ValueTuple::Many(vec![30.into()]))
                    .last(2)
                    .query
            ),
            stmt
        );

        Ok(())
    }

    mod test_entity {
        use crate as sea_orm;
        use crate::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "example")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            #[sea_orm(primary_key)]
            pub category: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    mod xyz_entity {
        use crate as sea_orm;
        use crate::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "m")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub x: i32,
            #[sea_orm(primary_key)]
            pub y: String,
            #[sea_orm(primary_key)]
            pub z: i64,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    #[smol_potat::test]
    async fn composite_keys_1() -> Result<(), DbErr> {
        use test_entity::*;

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[Model {
                id: 1,
                category: "CAT".into(),
            }]])
            .into_connection();

        assert!(!Entity::find()
            .cursor_by((Column::Category, Column::Id))
            .first(3)
            .all(&db)
            .await?
            .is_empty());

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::many([Statement::from_sql_and_values(
                DbBackend::Postgres,
                [
                    r#"SELECT "example"."id", "example"."category""#,
                    r#"FROM "example""#,
                    r#"ORDER BY "example"."category" ASC, "example"."id" ASC"#,
                    r#"LIMIT $1"#,
                ]
                .join(" ")
                .as_str(),
                [3_u64.into()]
            ),])]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn composite_keys_2() -> Result<(), DbErr> {
        use test_entity::*;

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[Model {
                id: 1,
                category: "CAT".into(),
            }]])
            .into_connection();

        assert!(!Entity::find()
            .cursor_by((Column::Category, Column::Id))
            .after(("A".to_owned(), 2))
            .first(3)
            .all(&db)
            .await?
            .is_empty());

        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            [
                r#"SELECT "example"."id", "example"."category""#,
                r#"FROM "example""#,
                r#"WHERE ("example"."category" = $1 AND "example"."id" > $2)"#,
                r#"OR "example"."category" > $3"#,
                r#"ORDER BY "example"."category" ASC, "example"."id" ASC"#,
                r#"LIMIT $4"#,
            ]
            .join(" ")
            .as_str(),
            [
                "A".to_string().into(),
                2i32.into(),
                "A".to_string().into(),
                3_u64.into(),
            ],
        );

        assert_eq!(db.into_transaction_log(), [Transaction::one(stmt.clone())]);

        assert_eq!(
            DbBackend::Postgres.build(
                &Entity::find()
                    .cursor_by(Identity::Many(vec![
                        Column::Category.into_iden(),
                        Column::Id.into_iden()
                    ]))
                    .after(ValueTuple::Many(vec!["A".into(), 2.into()]))
                    .first(3)
                    .query
            ),
            stmt
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn composite_keys_3() -> Result<(), DbErr> {
        use test_entity::*;

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[Model {
                id: 1,
                category: "CAT".into(),
            }]])
            .into_connection();

        assert!(!Entity::find()
            .cursor_by((Column::Category, Column::Id))
            .before(("A".to_owned(), 2))
            .last(3)
            .all(&db)
            .await?
            .is_empty());

        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            [
                r#"SELECT "example"."id", "example"."category""#,
                r#"FROM "example""#,
                r#"WHERE ("example"."category" = $1 AND "example"."id" < $2)"#,
                r#"OR "example"."category" < $3"#,
                r#"ORDER BY "example"."category" DESC, "example"."id" DESC"#,
                r#"LIMIT $4"#,
            ]
            .join(" ")
            .as_str(),
            [
                "A".to_string().into(),
                2i32.into(),
                "A".to_string().into(),
                3_u64.into(),
            ],
        );

        assert_eq!(db.into_transaction_log(), [Transaction::one(stmt.clone())]);

        assert_eq!(
            DbBackend::Postgres.build(
                &Entity::find()
                    .cursor_by(Identity::Many(vec![
                        Column::Category.into_iden(),
                        Column::Id.into_iden()
                    ]))
                    .before(ValueTuple::Many(vec!["A".into(), 2.into()]))
                    .last(3)
                    .query
            ),
            stmt
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn composite_keys_4() -> Result<(), DbErr> {
        use xyz_entity::*;

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[Model {
                x: 'x' as i32,
                y: "y".into(),
                z: 'z' as i64,
            }]])
            .into_connection();

        assert!(!Entity::find()
            .cursor_by((Column::X, Column::Y, Column::Z))
            .first(4)
            .all(&db)
            .await?
            .is_empty());

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::many([Statement::from_sql_and_values(
                DbBackend::Postgres,
                [
                    r#"SELECT "m"."x", "m"."y", "m"."z""#,
                    r#"FROM "m""#,
                    r#"ORDER BY "m"."x" ASC, "m"."y" ASC, "m"."z" ASC"#,
                    r#"LIMIT $1"#,
                ]
                .join(" ")
                .as_str(),
                [4_u64.into()]
            ),])]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn composite_keys_5() -> Result<(), DbErr> {
        use xyz_entity::*;

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[Model {
                x: 'x' as i32,
                y: "y".into(),
                z: 'z' as i64,
            }]])
            .into_connection();

        assert!(!Entity::find()
            .cursor_by((Column::X, Column::Y, Column::Z))
            .after(('x' as i32, "y".to_owned(), 'z' as i64))
            .first(4)
            .all(&db)
            .await?
            .is_empty());

        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            [
                r#"SELECT "m"."x", "m"."y", "m"."z""#,
                r#"FROM "m""#,
                r#"WHERE ("m"."x" = $1 AND "m"."y" = $2 AND "m"."z" > $3)"#,
                r#"OR ("m"."x" = $4 AND "m"."y" > $5)"#,
                r#"OR "m"."x" > $6"#,
                r#"ORDER BY "m"."x" ASC, "m"."y" ASC, "m"."z" ASC"#,
                r#"LIMIT $7"#,
            ]
            .join(" ")
            .as_str(),
            [
                ('x' as i32).into(),
                "y".into(),
                ('z' as i64).into(),
                ('x' as i32).into(),
                "y".into(),
                ('x' as i32).into(),
                4_u64.into(),
            ],
        );

        assert_eq!(db.into_transaction_log(), [Transaction::one(stmt.clone())]);

        assert_eq!(
            DbBackend::Postgres.build(
                &Entity::find()
                    .cursor_by(Identity::Many(vec![
                        Column::X.into_iden(),
                        Column::Y.into_iden(),
                        Column::Z.into_iden()
                    ]))
                    .after(ValueTuple::Many(vec![
                        ('x' as i32).into(),
                        "y".into(),
                        ('z' as i64).into()
                    ]))
                    .first(4)
                    .query
            ),
            stmt
        );

        Ok(())
    }

    mod wxyz_entity {
        use crate as sea_orm;
        use crate::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "m")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub w: bool,
            #[sea_orm(primary_key)]
            pub x: i32,
            #[sea_orm(primary_key)]
            pub y: String,
            #[sea_orm(primary_key)]
            pub z: i64,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    #[smol_potat::test]
    async fn composite_keys_6() -> Result<(), DbErr> {
        use wxyz_entity::*;

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[Model {
                w: true,
                x: 'x' as i32,
                y: "y".into(),
                z: 'z' as i64,
            }]])
            .into_connection();

        assert!(!Entity::find()
            .cursor_by((Column::W, Column::X, Column::Y, Column::Z))
            .first(4)
            .all(&db)
            .await?
            .is_empty());

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::many([Statement::from_sql_and_values(
                DbBackend::Postgres,
                [
                    r#"SELECT "m"."w", "m"."x", "m"."y", "m"."z""#,
                    r#"FROM "m""#,
                    r#"ORDER BY "m"."w" ASC, "m"."x" ASC, "m"."y" ASC, "m"."z" ASC"#,
                    r#"LIMIT $1"#,
                ]
                .join(" ")
                .as_str(),
                [4_u64.into()]
            ),])]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn composite_keys_7() -> Result<(), DbErr> {
        use wxyz_entity::*;

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[Model {
                w: true,
                x: 'x' as i32,
                y: "y".into(),
                z: 'z' as i64,
            }]])
            .into_connection();

        assert!(!Entity::find()
            .cursor_by((Column::W, Column::X, Column::Y, Column::Z))
            .after((true, 'x' as i32, "y".to_owned(), 'z' as i64))
            .first(4)
            .all(&db)
            .await?
            .is_empty());

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::many([Statement::from_sql_and_values(
                DbBackend::Postgres,
                [
                    r#"SELECT "m"."w", "m"."x", "m"."y", "m"."z""#,
                    r#"FROM "m""#,
                    r#"WHERE ("m"."w" = $1 AND "m"."x" = $2 AND "m"."y" = $3 AND "m"."z" > $4)"#,
                    r#"OR ("m"."w" = $5 AND "m"."x" = $6 AND "m"."y" > $7)"#,
                    r#"OR ("m"."w" = $8 AND "m"."x" > $9)"#,
                    r#"OR "m"."w" > $10"#,
                    r#"ORDER BY "m"."w" ASC, "m"."x" ASC, "m"."y" ASC, "m"."z" ASC"#,
                    r#"LIMIT $11"#,
                ]
                .join(" ")
                .as_str(),
                [
                    true.into(),
                    ('x' as i32).into(),
                    "y".into(),
                    ('z' as i64).into(),
                    true.into(),
                    ('x' as i32).into(),
                    "y".into(),
                    true.into(),
                    ('x' as i32).into(),
                    true.into(),
                    4_u64.into(),
                ]
            ),])]
        );

        Ok(())
    }

    mod vwxyz_entity {
        use crate as sea_orm;
        use crate::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "m")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub v: f32,
            #[sea_orm(primary_key)]
            pub w: bool,
            #[sea_orm(primary_key)]
            pub x: i32,
            #[sea_orm(primary_key)]
            pub y: String,
            #[sea_orm(primary_key)]
            pub z: i64,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    #[smol_potat::test]
    async fn composite_keys_8() -> Result<(), DbErr> {
        use vwxyz_entity::*;

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[Model {
                v: 16.24,
                w: true,
                x: 'x' as i32,
                y: "y".into(),
                z: 'z' as i64,
            }]])
            .into_connection();

        assert!(!Entity::find()
            .cursor_by((Column::V, Column::W, Column::X, Column::Y, Column::Z))
            .first(4)
            .all(&db)
            .await?
            .is_empty());

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::many([Statement::from_sql_and_values(
                DbBackend::Postgres,
                [
                    r#"SELECT "m"."v", "m"."w", "m"."x", "m"."y", "m"."z""#,
                    r#"FROM "m""#,
                    r#"ORDER BY "m"."v" ASC, "m"."w" ASC, "m"."x" ASC, "m"."y" ASC, "m"."z" ASC"#,
                    r#"LIMIT $1"#,
                ]
                .join(" ")
                .as_str(),
                [4_u64.into()]
            ),])]
        );

        Ok(())
    }

    #[smol_potat::test]
    async fn composite_keys_9() -> Result<(), DbErr> {
        use vwxyz_entity::*;

        let db = MockDatabase::new(DbBackend::Postgres)
            .append_query_results([[Model {
                v: 16.24,
                w: true,
                x: 'x' as i32,
                y: "y".into(),
                z: 'z' as i64,
            }]])
            .into_connection();

        assert!(!Entity::find()
            .cursor_by((Column::V, Column::W, Column::X, Column::Y, Column::Z))
            .after((16.24, true, 'x' as i32, "y".to_owned(), 'z' as i64))
            .first(4)
            .all(&db)
            .await?
            .is_empty());

        assert_eq!(
            db.into_transaction_log(),
            [Transaction::many([Statement::from_sql_and_values(
                DbBackend::Postgres,
                [
                    r#"SELECT "m"."v", "m"."w", "m"."x", "m"."y", "m"."z""#,
                    r#"FROM "m""#,
                    r#"WHERE ("m"."v" = $1 AND "m"."w" = $2 AND "m"."x" = $3 AND "m"."y" = $4 AND "m"."z" > $5)"#,
                    r#"OR ("m"."v" = $6 AND "m"."w" = $7 AND "m"."x" = $8 AND "m"."y" > $9)"#,
                    r#"OR ("m"."v" = $10 AND "m"."w" = $11 AND "m"."x" > $12)"#,
                    r#"OR ("m"."v" = $13 AND "m"."w" > $14)"#,
                    r#"OR "m"."v" > $15"#,
                    r#"ORDER BY "m"."v" ASC, "m"."w" ASC, "m"."x" ASC, "m"."y" ASC, "m"."z" ASC"#,
                    r#"LIMIT $16"#,
                ]
                .join(" ")
                .as_str(),
                [
                    16.24.into(),
                    true.into(),
                    ('x' as i32).into(),
                    "y".into(),
                    ('z' as i64).into(),
                    16.24.into(),
                    true.into(),
                    ('x' as i32).into(),
                    "y".into(),
                    16.24.into(),
                    true.into(),
                    ('x' as i32).into(),
                    16.24.into(),
                    true.into(),
                    16.24.into(),
                    4_u64.into(),
                ]
            ),])]
        );

        Ok(())
    }
}
