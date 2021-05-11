use crate::{
    ColumnTrait, ModelTrait, PrimaryKeyOfModel, PrimaryKeyTrait, RelationBuilder, RelationTrait,
    RelationType, Select,
};
use sea_query::{Iden, IntoIden, Value};
use std::fmt::Debug;
pub use strum::IntoEnumIterator as Iterable;

pub trait IdenStatic: Iden + Copy + Debug + 'static {
    fn as_str(&self) -> &str;
}

pub trait EntityName: IdenStatic + Default {}

pub trait EntityTrait: EntityName {
    type Model: ModelTrait;

    type Column: ColumnTrait + Iterable;

    type Relation: RelationTrait + Iterable;

    type PrimaryKey: PrimaryKeyTrait + Iterable;

    fn auto_increment() -> bool {
        true
    }

    fn has_one<E>(entity: E) -> RelationBuilder
    where
        E: IntoIden,
    {
        RelationBuilder::new(RelationType::HasOne, Self::default(), entity)
    }

    fn has_many<E>(entity: E) -> RelationBuilder
    where
        E: IntoIden,
    {
        RelationBuilder::new(RelationType::HasMany, Self::default(), entity)
    }

    /// ```
    /// use sea_orm::{ColumnTrait, EntityTrait, tests_cfg::cake, sea_query::PostgresQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .build(PostgresQueryBuilder)
    ///         .to_string(),
    ///     r#"SELECT "cake"."id", "cake"."name" FROM "cake""#
    /// );
    /// ```
    fn find() -> Select<Self> {
        Select::<Self>::new()
    }

    /// Find a model by primary key
    /// ```
    /// use sea_orm::{ColumnTrait, EntityTrait, tests_cfg::cake, sea_query::PostgresQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake::Entity::find_by(11)
    ///         .build(PostgresQueryBuilder)
    ///         .to_string(),
    ///     r#"SELECT "cake"."id", "cake"."name" FROM "cake" WHERE "cake"."id" = 11"#
    /// );
    /// ```
    fn find_by<V>(v: V) -> Select<Self>
    where
        V: Into<Value>,
        Self::PrimaryKey: PrimaryKeyOfModel<Self::Model>,
    {
        let mut select = Self::find();
        if let Some(key) = Self::PrimaryKey::iter().next() {
            // TODO: supporting composite primary key
            let col = key.into_column();
            select = select.filter(col.eq(v));
        } else {
            panic!("undefined primary key");
        }
        select
    }
}
