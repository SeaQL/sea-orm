use super::{ColumnTrait, Identity, ModelTrait, RelationBuilder, RelationTrait, RelationType};
use crate::Select;
use sea_query::{Expr, Iden, IntoIden, Value};
use std::fmt::Debug;
pub use strum::IntoEnumIterator as Iterable;

pub trait EntityTrait: Iden + Default + Debug + 'static {
    type Model: ModelTrait;

    type Column: ColumnTrait + Iterable;

    type Relation: RelationTrait + Iterable;

    fn primary_key() -> Identity;

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

    fn belongs_to<E>(entity: E) -> RelationBuilder
    where
        E: IntoIden,
    {
        RelationBuilder::new(RelationType::BelongsTo, Self::default(), entity)
    }

    /// ```
    /// use sea_orm::{ColumnTrait, EntityTrait, tests_cfg::cake, sea_query::MysqlQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .build(MysqlQueryBuilder)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake`"
    /// );
    /// ```
    fn find() -> Select<Self> {
        Select::new(Self::default())
    }

    /// ```
    /// use sea_orm::{ColumnTrait, EntityTrait, tests_cfg::cake, sea_query::MysqlQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake::Entity::find_one()
    ///         .build(MysqlQueryBuilder)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` LIMIT 1"
    /// );
    /// ```
    fn find_one() -> Select<Self> {
        let mut select = Self::find();
        select.query().limit(1);
        select
    }

    /// ```
    /// use sea_orm::{ColumnTrait, EntityTrait, tests_cfg::cake, sea_query::MysqlQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake::Entity::find_one_by(11)
    ///         .build(MysqlQueryBuilder)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` = 11 LIMIT 1"
    /// );
    /// ```
    fn find_one_by<V>(v: V) -> Select<Self>
    where
        V: Into<Value>,
    {
        let select = Self::find_one();
        let select =
            select.filter(Expr::tbl(Self::default(), Self::primary_key().into_iden()).eq(v));
        select
    }
}
