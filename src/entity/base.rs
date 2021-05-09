use crate::{
    ColumnTrait, ModelTrait, PrimaryKeyTrait, RelationBuilder, RelationTrait, RelationType, Select,
};
use sea_query::{Expr, Iden, IntoIden, Value};
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
        Select::<Self>::new()
    }

    /// Find a model by primary key
    /// ```
    /// use sea_orm::{ColumnTrait, EntityTrait, tests_cfg::cake, sea_query::MysqlQueryBuilder};
    ///
    /// assert_eq!(
    ///     cake::Entity::find_by(11)
    ///         .build(MysqlQueryBuilder)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` = 11"
    /// );
    /// ```
    fn find_by<V>(v: V) -> Select<Self>
    where
        V: Into<Value>,
    {
        let mut select = Self::find();
        if let Some(key) = Self::PrimaryKey::iter().next() {
            // TODO: supporting composite primary key
            select = select.filter(Expr::tbl(Self::default(), key).eq(v));
        } else {
            panic!("undefined primary key");
        }
        select
    }
}
