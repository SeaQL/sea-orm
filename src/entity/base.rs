use crate::{
    ColumnTrait, Connection, Database, Identity, ModelTrait, QueryErr, RelationBuilder,
    RelationTrait, RelationType, Select,
};
use async_trait::async_trait;
use sea_query::{Expr, Iden, IntoIden, Value};
use std::fmt::Debug;
pub use strum::IntoEnumIterator as Iterable;

#[async_trait]
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
        Select::<Self>::new()
    }

    async fn find_one<V>(db: &Database, v: V) -> Result<Self::Model, QueryErr>
    where
        V: Into<Value> + Send,
    {
        let builder = db.get_query_builder_backend();
        let stmt = {
            let mut select = Self::find();
            match Self::primary_key() {
                Identity::Unary(iden) => {
                    select = select.filter(Expr::tbl(Self::default(), iden).eq(v));
                }
            }
            select.build(builder)
        };
        let row = db.get_connection().query_one(stmt).await?;
        Ok(Self::Model::from_query_result(row)?)
    }
}
