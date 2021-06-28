use crate::{
    ActiveModelTrait, ColumnTrait, Delete, DeleteMany, DeleteOne, FromQueryResult, Insert,
    ModelTrait, PrimaryKeyToColumn, PrimaryKeyTrait, QueryFilter, Related, RelationBuilder,
    RelationTrait, RelationType, Select, Update, UpdateMany, UpdateOne,
};
use sea_query::{Iden, IntoValueTuple};
use std::fmt::Debug;
pub use strum::IntoEnumIterator as Iterable;

pub trait IdenStatic: Iden + Copy + Debug + 'static {
    fn as_str(&self) -> &str;
}

pub trait EntityName: IdenStatic + Default {
    fn table_name(&self) -> &str;

    fn module_name(&self) -> &str {
        Self::table_name(self)
    }
}

pub trait EntityTrait: EntityName {
    type Model: ModelTrait<Entity = Self> + FromQueryResult;

    type Column: ColumnTrait;

    type Relation: RelationTrait;

    type PrimaryKey: PrimaryKeyTrait + PrimaryKeyToColumn<Column = Self::Column>;

    fn belongs_to<R>(related: R) -> RelationBuilder<Self, R>
    where
        R: EntityTrait,
    {
        RelationBuilder::new(RelationType::HasOne, Self::default(), related)
    }

    fn has_one<R>(_: R) -> RelationBuilder<Self, R>
    where
        R: EntityTrait + Related<Self>,
    {
        RelationBuilder::from_rel(RelationType::HasOne, R::to().rev())
    }

    fn has_many<R>(_: R) -> RelationBuilder<Self, R>
    where
        R: EntityTrait + Related<Self>,
    {
        RelationBuilder::from_rel(RelationType::HasMany, R::to().rev())
    }

    /// ```
    /// # #[cfg(feature = "mock")]
    /// # use sea_orm::{MockDatabase, Transaction};
    /// # let db = MockDatabase::new().into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// # async_std::task::block_on(async {
    /// cake::Entity::find().one(&db).await;
    /// cake::Entity::find().all(&db).await;
    /// # });
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![
    ///     Transaction::from_sql_and_values(
    ///         r#"SELECT "cake"."id", "cake"."name" FROM "cake" LIMIT $1"#, vec![1u64.into()]
    ///     ),
    ///     Transaction::from_sql_and_values(
    ///         r#"SELECT "cake"."id", "cake"."name" FROM "cake""#, vec![]
    ///     ),
    /// ]);
    /// ```
    fn find() -> Select<Self> {
        Select::new()
    }

    /// Find a model by primary key
    /// ```
    /// # #[cfg(feature = "mock")]
    /// # use sea_orm::{MockDatabase, Transaction};
    /// # let db = MockDatabase::new().into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// # async_std::task::block_on(async {
    /// cake::Entity::find_by_id(11).all(&db).await;
    /// # });
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         r#"SELECT "cake"."id", "cake"."name" FROM "cake" WHERE "cake"."id" = $1"#, vec![11i32.into()]
    ///     )]);
    /// ```
    /// Find by composite key
    /// ```
    /// # #[cfg(feature = "mock")]
    /// # use sea_orm::{MockDatabase, Transaction};
    /// # let db = MockDatabase::new().into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake_filling};
    ///
    /// # async_std::task::block_on(async {
    /// cake_filling::Entity::find_by_id((2, 3)).all(&db).await;
    /// # });
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values([
    ///             r#"SELECT "cake_filling"."cake_id", "cake_filling"."filling_id" FROM "cake_filling""#,
    ///             r#"WHERE "cake_filling"."cake_id" = $1 AND "cake_filling"."filling_id" = $2"#,
    ///         ].join(" ").as_str(),
    ///         vec![2i32.into(), 3i32.into()]
    ///     )]);
    /// ```
    fn find_by_id<V>(values: V) -> Select<Self>
    where
        V: IntoValueTuple,
    {
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

    /// ```
    /// # #[cfg(feature = "mock")]
    /// # use sea_orm::{MockDatabase, Transaction};
    /// # let db = MockDatabase::new().into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake};
    ///
    /// let apple = cake::ActiveModel {
    ///     name: Set("Apple Pie".to_owned()),
    ///     ..Default::default()
    /// };
    ///
    /// # async_std::task::block_on(async {
    /// cake::Entity::insert(apple).exec(&db).await;
    /// # });
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         r#"INSERT INTO "cake" ("name") VALUES ($1)"#, vec!["Apple Pie".into()]
    ///     )]);
    /// ```
    fn insert<A>(model: A) -> Insert<A>
    where
        A: ActiveModelTrait<Entity = Self>,
    {
        Insert::one(model)
    }

    /// ```
    /// # #[cfg(feature = "mock")]
    /// # use sea_orm::{MockDatabase, Transaction};
    /// # let db = MockDatabase::new().into_connection();
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
    /// # async_std::task::block_on(async {
    /// cake::Entity::insert_many(vec![apple, orange]).exec(&db).await;
    /// # });
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         r#"INSERT INTO "cake" ("name") VALUES ($1), ($2)"#,
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

    /// ```
    /// # #[cfg(feature = "mock")]
    /// # use sea_orm::{MockDatabase, Transaction};
    /// # let db = MockDatabase::new().into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit};
    ///
    /// let orange = fruit::ActiveModel {
    ///     id: Set(1),
    ///     name: Set("Orange".to_owned()),
    ///     ..Default::default()
    /// };
    ///
    /// # async_std::task::block_on(async {
    /// fruit::Entity::update(orange).exec(&db).await;
    /// # });
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         r#"UPDATE "fruit" SET "name" = $1 WHERE "fruit"."id" = $2"#, vec!["Orange".into(), 1i32.into()]
    ///     )]);
    /// ```
    fn update<A>(model: A) -> UpdateOne<A>
    where
        A: ActiveModelTrait<Entity = Self>,
    {
        Update::one(model)
    }

    /// ```
    /// # #[cfg(feature = "mock")]
    /// # use sea_orm::{MockDatabase, Transaction};
    /// # let db = MockDatabase::new().into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit, sea_query::{Expr, Value}};
    ///
    /// # async_std::task::block_on(async {
    /// fruit::Entity::update_many()
    ///     .col_expr(fruit::Column::CakeId, Expr::value(Value::Null))
    ///     .filter(fruit::Column::Name.contains("Apple"))
    ///     .exec(&db)
    ///     .await;
    /// # });
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         r#"UPDATE "fruit" SET "cake_id" = $1 WHERE "fruit"."name" LIKE $2"#, vec![Value::Null, "%Apple%".into()]
    ///     )]);
    /// ```
    fn update_many() -> UpdateMany<Self> {
        Update::many(Self::default())
    }

    /// ```
    /// # #[cfg(feature = "mock")]
    /// # use sea_orm::{MockDatabase, Transaction};
    /// # let db = MockDatabase::new().into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit};
    ///
    /// let orange = fruit::ActiveModel {
    ///     id: Set(3),
    ///     ..Default::default()
    /// };
    ///
    /// # async_std::task::block_on(async {
    /// fruit::Entity::delete(orange).exec(&db).await;
    /// # });
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         r#"DELETE FROM "fruit" WHERE "fruit"."id" = $1"#, vec![3i32.into()]
    ///     )]);
    /// ```
    fn delete<A>(model: A) -> DeleteOne<A>
    where
        A: ActiveModelTrait<Entity = Self>,
    {
        Delete::one(model)
    }

    /// ```
    /// # #[cfg(feature = "mock")]
    /// # use sea_orm::{MockDatabase, Transaction};
    /// # let db = MockDatabase::new().into_connection();
    /// #
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit};
    ///
    /// # async_std::task::block_on(async {
    /// fruit::Entity::delete_many()
    ///     .filter(fruit::Column::Name.contains("Apple"))
    ///     .exec(&db)
    ///     .await;
    /// # });
    ///
    /// assert_eq!(
    ///     db.into_transaction_log(),
    ///     vec![Transaction::from_sql_and_values(
    ///         r#"DELETE FROM "fruit" WHERE "fruit"."name" LIKE $1"#, vec!["%Apple%".into()]
    ///     )]);
    /// ```
    fn delete_many() -> DeleteMany<Self> {
        Delete::many(Self::default())
    }
}
