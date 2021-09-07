use crate::{
    ActiveModelTrait, ColumnTrait, Delete, DeleteMany, DeleteOne, FromQueryResult, Insert,
    ModelTrait, PrimaryKeyToColumn, PrimaryKeyTrait, QueryFilter, Related, RelationBuilder,
    RelationTrait, RelationType, Select, Update, UpdateMany, UpdateOne,
};
use sea_query::{Alias, Iden, IntoIden, IntoTableRef, IntoValueTuple, TableRef};
pub use sea_strum::IntoEnumIterator as Iterable;
use std::fmt::Debug;

pub trait IdenStatic: Iden + Copy + Debug + 'static {
    fn as_str(&self) -> &str;
}

pub trait EntityName: IdenStatic + Default {
    fn schema_name(&self) -> Option<&str> {
        None
    }

    fn table_name(&self) -> &str;

    fn module_name(&self) -> &str {
        self.table_name()
    }

    fn table_ref(&self) -> TableRef {
        match self.schema_name() {
            Some(schema) => (Alias::new(schema).into_iden(), self.into_iden()).into_table_ref(),
            None => self.into_table_ref(),
        }
    }
}

/// An Entity implementing `EntityTrait` represents a table in a database.
///
/// This trait provides an API for you to inspect it's properties
/// - Column (implemented [`ColumnTrait`])
/// - Relation (implemented [`RelationTrait`])
/// - Primary Key (implemented [`PrimaryKeyTrait`] and [`PrimaryKeyToColumn`])
///
/// This trait also provides an API for CRUD actions
/// - Select: `find`, `find_*`
/// - Insert: `insert`, `insert_*`
/// - Update: `update`, `update_*`
/// - Delete: `delete`, `delete_*`
pub trait EntityTrait: EntityName {
    type Model: ModelTrait<Entity = Self> + FromQueryResult;

    type Column: ColumnTrait;

    type Relation: RelationTrait;

    type PrimaryKey: PrimaryKeyTrait + PrimaryKeyToColumn<Column = Self::Column>;

    fn belongs_to<R>(related: R) -> RelationBuilder<Self, R>
    where
        R: EntityTrait,
    {
        RelationBuilder::new(RelationType::HasOne, Self::default(), related, false)
    }

    fn has_one<R>(_: R) -> RelationBuilder<Self, R>
    where
        R: EntityTrait + Related<Self>,
    {
        RelationBuilder::from_rel(RelationType::HasOne, R::to().rev(), true)
    }

    fn has_many<R>(_: R) -> RelationBuilder<Self, R>
    where
        R: EntityTrait + Related<Self>,
    {
        RelationBuilder::from_rel(RelationType::HasMany, R::to().rev(), true)
    }

    /// Construct select statement to find one / all models
    ///
    /// - To select columns, join tables and group by expressions, see [`QuerySelect`](crate::query::QuerySelect)
    /// - To apply where conditions / filters, see [`QueryFilter`](crate::query::QueryFilter)
    /// - To apply order by expressions, see [`QueryOrder`](crate::query::QueryOrder)
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "mock")]
    /// # use sea_orm::{error::*, tests_cfg::*, MockDatabase, Transaction, DbBackend};
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results(vec![
    /// #         vec![
    /// #             cake::Model {
    /// #                 id: 1,
    /// #                 name: "New York Cheese".to_owned(),
    /// #             },
    /// #         ],
    /// #         vec![
    /// #             cake::Model {
    /// #                 id: 1,
    /// #                 name: "New York Cheese".to_owned(),
    /// #             },
    /// #             cake::Model {
    /// #                 id: 2,
    /// #                 name: "Chocolate Forest".to_owned(),
    /// #             },
    /// #         ],
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// # let _: Result<(), DbErr> = smol::block_on(async {
    /// #
    /// assert_eq!(
    ///     cake::Entity::find().one(&db).await?,
    ///     Some(cake::Model {
    ///         id: 1,
    ///         name: "New York Cheese".to_owned(),
    ///     })
    /// );
    ///
    /// assert_eq!(
    ///     cake::Entity::find().all(&db).await?,
    ///     vec![
    ///         cake::Model {
    ///             id: 1,
    ///             name: "New York Cheese".to_owned(),
    ///         },
    ///         cake::Model {
    ///             id: 2,
    ///             name: "Chocolate Forest".to_owned(),
    ///         },
    ///     ]
    /// );
    /// #
    /// # Ok(())
    /// # });
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![
    ///         Transaction::from_sql_and_values(
    ///             DbBackend::Postgres,
    ///             r#"SELECT "cake"."id", "cake"."name" FROM "cake" LIMIT $1"#,
    ///             vec![1u64.into()]
    ///         ),
    ///         Transaction::from_sql_and_values(
    ///             DbBackend::Postgres,
    ///             r#"SELECT "cake"."id", "cake"."name" FROM "cake""#,
    ///             vec![]
    ///         ),
    ///     ]
    /// );
    /// ```
    fn find() -> Select<Self> {
        Select::new()
    }

    /// Find a model by primary key
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "mock")]
    /// # use sea_orm::{error::*, tests_cfg::*, MockDatabase, Transaction, DbBackend};
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results(vec![
    /// #         vec![
    /// #             cake::Model {
    /// #                 id: 11,
    /// #                 name: "Sponge Cake".to_owned(),
    /// #             },
    /// #         ],
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// # let _: Result<(), DbErr> = smol::block_on(async {
    /// #
    /// assert_eq!(
    ///     cake::Entity::find_by_id(11).all(&db).await?,
    ///     vec![cake::Model {
    ///         id: 11,
    ///         name: "Sponge Cake".to_owned(),
    ///     }]
    /// );
    /// #
    /// # Ok(())
    /// # });
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         r#"SELECT "cake"."id", "cake"."name" FROM "cake" WHERE "cake"."id" = $1"#,
    ///         vec![11i32.into()]
    ///     )]
    /// );
    /// ```
    /// Find by composite key
    /// ```
    /// # #[cfg(feature = "mock")]
    /// # use sea_orm::{error::*, tests_cfg::*, MockDatabase, Transaction, DbBackend};
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_query_results(vec![
    /// #         vec![
    /// #             cake_filling::Model {
    /// #                 cake_id: 2,
    /// #                 filling_id: 3,
    /// #             },
    /// #         ],
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake_filling};
    ///
    /// # let _: Result<(), DbErr> = smol::block_on(async {
    /// #
    /// assert_eq!(
    ///     cake_filling::Entity::find_by_id((2, 3)).all(&db).await?,
    ///     vec![cake_filling::Model {
    ///         cake_id: 2,
    ///         filling_id: 3,
    ///     }]
    /// );
    /// #
    /// # Ok(())
    /// # });
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres,
    ///         [
    ///             r#"SELECT "cake_filling"."cake_id", "cake_filling"."filling_id" FROM "cake_filling""#,
    ///             r#"WHERE "cake_filling"."cake_id" = $1 AND "cake_filling"."filling_id" = $2"#,
    ///         ].join(" ").as_str(),
    ///         vec![2i32.into(), 3i32.into()]
    ///     )]);
    /// ```
    fn find_by_id(values: <Self::PrimaryKey as PrimaryKeyTrait>::ValueType) -> Select<Self> {
        let mut select = Self::find();
        let mut keys = Self::PrimaryKey::iter();
        for v in values.into_value_tuple() {
            if let Some(key) = keys.next() {
                let col = key.into_column();
                select = select.filter(col.eq(v));
            } else {
                panic!("primary key arity mismatch");
            }
        }
        if keys.next().is_some() {
            panic!("primary key arity mismatch");
        }
        select
    }

    /// Insert an model into database
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "mock")]
    /// # use sea_orm::{error::*, tests_cfg::*, MockDatabase, MockExecResult, Transaction, DbBackend};
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 15,
    /// #             rows_affected: 1,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// let apple = cake::ActiveModel {
    ///     name: Set("Apple Pie".to_owned()),
    ///     ..Default::default()
    /// };
    ///
    /// # let _: Result<(), DbErr> = smol::block_on(async {
    /// #
    /// let insert_result = cake::Entity::insert(apple).exec(&db).await?;
    ///
    /// assert_eq!(insert_result.last_insert_id, 15);
    /// // assert_eq!(insert_result.rows_affected, 1);
    /// #
    /// # Ok(())
    /// # });
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres, r#"INSERT INTO "cake" ("name") VALUES ($1)"#, vec!["Apple Pie".into()]
    ///     )]);
    /// ```
    fn insert<A>(model: A) -> Insert<A>
    where
        A: ActiveModelTrait<Entity = Self>,
    {
        Insert::one(model)
    }

    /// Insert many models into database
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "mock")]
    /// # use sea_orm::{error::*, tests_cfg::*, MockDatabase, MockExecResult, Transaction, DbBackend};
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 28,
    /// #             rows_affected: 2,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// let apple = cake::ActiveModel {
    ///     name: Set("Apple Pie".to_owned()),
    ///     ..Default::default()
    /// };
    /// let orange = cake::ActiveModel {
    ///     name: Set("Orange Scone".to_owned()),
    ///     ..Default::default()
    /// };
    ///
    /// # let _: Result<(), DbErr> = smol::block_on(async {
    /// #
    /// let insert_result = cake::Entity::insert_many(vec![apple, orange]).exec(&db).await?;
    ///
    /// assert_eq!(insert_result.last_insert_id, 28);
    /// // assert_eq!(insert_result.rows_affected, 2);
    /// #
    /// # Ok(())
    /// # });
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres, r#"INSERT INTO "cake" ("name") VALUES ($1), ($2)"#,
    ///         vec!["Apple Pie".into(), "Orange Scone".into()]
    ///     )]);
    /// ```
    fn insert_many<A, I>(models: I) -> Insert<A>
    where
        A: ActiveModelTrait<Entity = Self>,
        I: IntoIterator<Item = A>,
    {
        Insert::many(models)
    }

    /// Update an model in database
    ///
    /// - To apply where conditions / filters, see [`QueryFilter`](crate::query::QueryFilter)
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "mock")]
    /// # use sea_orm::{error::*, tests_cfg::*, MockDatabase, MockExecResult, Transaction, DbBackend};
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 0,
    /// #             rows_affected: 1,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit};
    ///
    /// let orange = fruit::ActiveModel {
    ///     id: Set(1),
    ///     name: Set("Orange".to_owned()),
    ///     ..Default::default()
    /// };
    ///
    /// # let _: Result<(), DbErr> = smol::block_on(async {
    /// #
    /// assert_eq!(
    ///     orange.clone().update(&db).await?, // Clone here because we need to assert_eq
    ///     orange
    /// );
    /// #
    /// # Ok(())
    /// # });
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres, r#"UPDATE "fruit" SET "name" = $1 WHERE "fruit"."id" = $2"#, vec!["Orange".into(), 1i32.into()]
    ///     )]);
    /// ```
    fn update<A>(model: A) -> UpdateOne<A>
    where
        A: ActiveModelTrait<Entity = Self>,
    {
        Update::one(model)
    }

    /// Update many models in database
    ///
    /// - To apply where conditions / filters, see [`QueryFilter`](crate::query::QueryFilter)
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "mock")]
    /// # use sea_orm::{error::*, tests_cfg::*, MockDatabase, MockExecResult, Transaction, DbBackend};
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 0,
    /// #             rows_affected: 5,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit, sea_query::{Expr, Value}};
    ///
    /// # let _: Result<(), DbErr> = smol::block_on(async {
    /// #
    /// let update_result = fruit::Entity::update_many()
    ///     .col_expr(fruit::Column::CakeId, Expr::value(Value::Int(None)))
    ///     .filter(fruit::Column::Name.contains("Apple"))
    ///     .exec(&db)
    ///     .await?;
    ///
    /// assert_eq!(update_result.rows_affected, 5);
    /// #
    /// # Ok(())
    /// # });
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres, r#"UPDATE "fruit" SET "cake_id" = $1 WHERE "fruit"."name" LIKE $2"#, vec![Value::Int(None), "%Apple%".into()]
    ///     )]);
    /// ```
    fn update_many() -> UpdateMany<Self> {
        Update::many(Self::default())
    }

    /// Delete an model from database
    ///
    /// - To apply where conditions / filters, see [`QueryFilter`](crate::query::QueryFilter)
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "mock")]
    /// # use sea_orm::{error::*, tests_cfg::*, MockDatabase, MockExecResult, Transaction, DbBackend};
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 0,
    /// #             rows_affected: 1,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit};
    ///
    /// let orange = fruit::ActiveModel {
    ///     id: Set(3),
    ///     ..Default::default()
    /// };
    ///
    /// # let _: Result<(), DbErr> = smol::block_on(async {
    /// #
    /// let delete_result = fruit::Entity::delete(orange).exec(&db).await?;
    ///
    /// assert_eq!(delete_result.rows_affected, 1);
    /// #
    /// # Ok(())
    /// # });
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres, r#"DELETE FROM "fruit" WHERE "fruit"."id" = $1"#, vec![3i32.into()]
    ///     )]);
    /// ```
    fn delete<A>(model: A) -> DeleteOne<A>
    where
        A: ActiveModelTrait<Entity = Self>,
    {
        Delete::one(model)
    }

    /// Delete many models from database
    ///
    /// - To apply where conditions / filters, see [`QueryFilter`](crate::query::QueryFilter)
    ///
    /// # Example
    ///
    /// ```
    /// # #[cfg(feature = "mock")]
    /// # use sea_orm::{error::*, tests_cfg::*, MockDatabase, MockExecResult, Transaction, DbBackend};
    /// #
    /// # let db = MockDatabase::new(DbBackend::Postgres)
    /// #     .append_exec_results(vec![
    /// #         MockExecResult {
    /// #             last_insert_id: 0,
    /// #             rows_affected: 5,
    /// #         },
    /// #     ])
    /// #     .into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit};
    ///
    /// # let _: Result<(), DbErr> = smol::block_on(async {
    /// #
    /// let delete_result = fruit::Entity::delete_many()
    ///     .filter(fruit::Column::Name.contains("Apple"))
    ///     .exec(&db)
    ///     .await?;
    ///
    /// assert_eq!(delete_result.rows_affected, 5);
    /// #
    /// # Ok(())
    /// # });
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         DbBackend::Postgres, r#"DELETE FROM "fruit" WHERE "fruit"."name" LIKE $1"#, vec!["%Apple%".into()]
    ///     )]);
    /// ```
    fn delete_many() -> DeleteMany<Self> {
        Delete::many(Self::default())
    }
}
