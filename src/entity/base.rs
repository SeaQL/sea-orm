use crate::{
    ActiveModelOf, ActiveModelTrait, ColumnTrait, FromQueryResult, Insert, ModelTrait,
    OneOrManyActiveModel, PrimaryKeyToColumn, PrimaryKeyTrait, QueryFilter, RelationBuilder,
    RelationTrait, RelationType, Select,
};
use sea_query::{Iden, IntoValueTuple};
use std::fmt::Debug;
pub use strum::IntoEnumIterator as Iterable;

pub trait IdenStatic: Iden + Copy + Debug + 'static {
    fn as_str(&self) -> &str;
}

pub trait EntityName: IdenStatic + Default {}

pub trait EntityTrait: EntityName {
    type Model: ModelTrait<Entity = Self> + FromQueryResult;

    type Column: ColumnTrait;

    type Relation: RelationTrait;

    type PrimaryKey: PrimaryKeyTrait + PrimaryKeyToColumn<Self>;

    fn has_one<R>(entity: R) -> RelationBuilder<Self, R>
    where
        R: EntityTrait,
    {
        RelationBuilder::new(RelationType::HasOne, Self::default(), entity)
    }

    fn has_many<R>(entity: R) -> RelationBuilder<Self, R>
    where
        R: EntityTrait,
    {
        RelationBuilder::new(RelationType::HasMany, Self::default(), entity)
    }

    /// ```
    /// use sea_orm::{ColumnTrait, EntityTrait, QueryTrait, tests_cfg::cake, sea_query::PostgresQueryBuilder};
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
    /// use sea_orm::{ColumnTrait, EntityTrait, QueryTrait, tests_cfg::cake, sea_query::PostgresQueryBuilder};
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
    /// use sea_orm::{ColumnTrait, EntityTrait, QueryTrait, tests_cfg::cake_filling, sea_query::PostgresQueryBuilder};
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
        Self::PrimaryKey: PrimaryKeyToColumn<Self>,
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

    fn insert<A, C>(models: C) -> Insert<A>
    where
        A: ActiveModelTrait + ActiveModelOf<Self>,
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

    fn insert_one<A>(model: A) -> Insert<A>
    where
        A: ActiveModelTrait + ActiveModelOf<Self>,
    {
        Insert::new().one(model)
    }

    fn insert_many<A, I>(models: I) -> Insert<A>
    where
        A: ActiveModelTrait + ActiveModelOf<Self>,
        I: IntoIterator<Item = A>,
    {
        Insert::new().many(models)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::cake;
    use crate::{EntityTrait, QueryTrait, Val};
    use sea_query::PostgresQueryBuilder;

    #[test]
    fn insert_one() {
        let apple = cake::ActiveModel {
            name: Val::set("Apple Pie".to_owned()),
            ..Default::default()
        };
        assert_eq!(
            cake::Entity::insert(apple.clone())
                .build(PostgresQueryBuilder)
                .to_string(),
            r#"INSERT INTO "cake" ("name") VALUES ('Apple Pie')"#,
        );
        assert_eq!(
            cake::Entity::insert_one(apple)
                .build(PostgresQueryBuilder)
                .to_string(),
            r#"INSERT INTO "cake" ("name") VALUES ('Apple Pie')"#,
        );
    }

    #[test]
    fn insert_many() {
        let apple = cake::ActiveModel {
            name: Val::set("Apple Pie".to_owned()),
            ..Default::default()
        };
        let orange = cake::ActiveModel {
            name: Val::set("Orange Scone".to_owned()),
            ..Default::default()
        };
        let fruits = vec![apple, orange];
        assert_eq!(
            cake::Entity::insert(fruits.clone())
                .build(PostgresQueryBuilder)
                .to_string(),
            r#"INSERT INTO "cake" ("name") VALUES ('Apple Pie'), ('Orange Scone')"#,
        );
        assert_eq!(
            cake::Entity::insert_many(fruits)
                .build(PostgresQueryBuilder)
                .to_string(),
            r#"INSERT INTO "cake" ("name") VALUES ('Apple Pie'), ('Orange Scone')"#,
        );
    }
}
