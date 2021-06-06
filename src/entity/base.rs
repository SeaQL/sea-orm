use crate::{
    ActiveModelTrait, ColumnTrait, FromQueryResult, Insert, ModelTrait, OneOrManyActiveModel,
    PrimaryKeyToColumn, PrimaryKeyTrait, QueryFilter, RelationBuilder, RelationTrait, RelationType,
    Select, Update,
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

    fn has_one<R>(related: R) -> RelationBuilder<Self, R>
    where
        R: EntityTrait,
    {
        RelationBuilder::new(RelationType::HasOne, Self::default(), related)
    }

    fn has_many<R>(related: R) -> RelationBuilder<Self, R>
    where
        R: EntityTrait,
    {
        RelationBuilder::new(RelationType::HasMany, Self::default(), related)
    }

    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, sea_query::PostgresQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .build(PostgresQueryBuilder)
    ///         .to_string(),
    ///     r#"SELECT "cake"."id", "cake"."name" FROM "cake""#
    /// );
    /// ```
    fn find() -> Select<Self> {
        Select::new()
    }

    /// Find a model by primary key
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, sea_query::PostgresQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake::Entity::find_by(11)
    ///         .build(PostgresQueryBuilder)
    ///         .to_string(),
    ///     r#"SELECT "cake"."id", "cake"."name" FROM "cake" WHERE "cake"."id" = 11"#
    /// );
    /// ```
    /// Find by composite key
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake_filling, sea_query::PostgresQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake_filling::Entity::find_by((2, 3))
    ///         .build(PostgresQueryBuilder)
    ///         .to_string(),
    ///     [
    ///         r#"SELECT "cake_filling"."cake_id", "cake_filling"."filling_id" FROM "cake_filling""#,
    ///         r#"WHERE "cake_filling"."cake_id" = 2 AND "cake_filling"."filling_id" = 3"#,
    ///     ].join(" ")
    /// );
    /// ```
    fn find_by<V>(values: V) -> Select<Self>
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

    /// Insert one
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, sea_query::PostgresQueryBuilder};
    ///
    /// let apple = cake::ActiveModel {
    ///     name: Set("Apple Pie".to_owned()),
    ///     ..Default::default()
    /// };
    /// assert_eq!(
    ///     cake::Entity::insert(apple)
    ///         .build(PostgresQueryBuilder)
    ///         .to_string(),
    ///     r#"INSERT INTO "cake" ("name") VALUES ('Apple Pie')"#,
    /// );
    /// ```
    /// Insert many
    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, sea_query::PostgresQueryBuilder};
    ///
    /// let apple = cake::ActiveModel {
    ///     name: Set("Apple Pie".to_owned()),
    ///     ..Default::default()
    /// };
    /// let orange = cake::ActiveModel {
    ///     name: Set("Orange Scone".to_owned()),
    ///     ..Default::default()
    /// };
    /// assert_eq!(
    ///     cake::Entity::insert(vec![apple, orange])
    ///         .build(PostgresQueryBuilder)
    ///         .to_string(),
    ///     r#"INSERT INTO "cake" ("name") VALUES ('Apple Pie'), ('Orange Scone')"#,
    /// );
    /// ```
    fn insert<A, C>(models: C) -> Insert<A>
    where
        A: ActiveModelTrait<Entity = Self>,
        C: OneOrManyActiveModel<A>,
    {
        if C::is_one() {
            Self::insert_one(models.get_one())
        } else if C::is_many() {
            Self::insert_many(models.get_many())
        } else {
            unreachable!()
        }
    }

    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, sea_query::PostgresQueryBuilder};
    ///
    /// let apple = cake::ActiveModel {
    ///     name: Set("Apple Pie".to_owned()),
    ///     ..Default::default()
    /// };
    /// assert_eq!(
    ///     cake::Entity::insert_one(apple)
    ///         .build(PostgresQueryBuilder)
    ///         .to_string(),
    ///     r#"INSERT INTO "cake" ("name") VALUES ('Apple Pie')"#,
    /// );
    /// ```
    fn insert_one<A>(model: A) -> Insert<A>
    where
        A: ActiveModelTrait<Entity = Self>,
    {
        Insert::new().one(model)
    }

    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, sea_query::PostgresQueryBuilder};
    ///
    /// let apple = cake::ActiveModel {
    ///     name: Set("Apple Pie".to_owned()),
    ///     ..Default::default()
    /// };
    /// let orange = cake::ActiveModel {
    ///     name: Set("Orange Scone".to_owned()),
    ///     ..Default::default()
    /// };
    /// assert_eq!(
    ///     cake::Entity::insert_many(vec![apple, orange])
    ///         .build(PostgresQueryBuilder)
    ///         .to_string(),
    ///     r#"INSERT INTO "cake" ("name") VALUES ('Apple Pie'), ('Orange Scone')"#,
    /// );
    /// ```
    fn insert_many<A, I>(models: I) -> Insert<A>
    where
        A: ActiveModelTrait<Entity = Self>,
        I: IntoIterator<Item = A>,
    {
        Insert::new().many(models)
    }

    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::fruit, sea_query::PostgresQueryBuilder};
    ///
    /// let orange = fruit::ActiveModel {
    ///     id: Set(1),
    ///     name: Set("Orange".to_owned()),
    ///     ..Default::default()
    /// };
    /// assert_eq!(
    ///     fruit::Entity::update(orange)
    ///         .build(PostgresQueryBuilder)
    ///         .to_string(),
    ///     r#"UPDATE "fruit" SET "name" = 'Orange' WHERE "fruit"."id" = 1"#,
    /// );
    /// ```
    fn update<A>(model: A) -> Update<A>
    where
        A: ActiveModelTrait<Entity = Self>,
    {
        Update::new(model)
    }
}
