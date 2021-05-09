use crate::{
    ColumnTrait, Connection, Database, ModelTrait, QueryErr, RelationBuilder,
    RelationTrait, RelationType, Select, PrimaryKeyTrait
};
use async_trait::async_trait;
use sea_query::{Expr, Iden, IntoIden, Value};
use std::fmt::Debug;
pub use strum::IntoEnumIterator as Iterable;

pub trait IdenStatic: Iden + Copy + Debug + 'static {
    fn as_str(&self) -> &str;
}

#[async_trait]
pub trait EntityTrait: IdenStatic + Default {
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
        Select::<Self>::new()
    }

    async fn find_one<V>(db: &Database, v: V) -> Result<Self::Model, QueryErr>
    where
        V: Into<Value> + Send,
    {
        let builder = db.get_query_builder_backend();
        let stmt = {
            let mut select = Self::find();
            if let Some(key) = Self::PrimaryKey::iter().next() {
                // TODO: supporting composite primary key
                select = select.filter(Expr::tbl(Self::default(), key).eq(v));
            } else {
                panic!("undefined primary key");
            }
            select.build(builder)
        };
        let row = db.get_connection().query_one(stmt).await?;
        Ok(Self::Model::from_query_result(row)?)
    }
}
