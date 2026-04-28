use super::sea_orm_active_enums::*;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[cfg_attr(feature = "sqlx-postgres", sea_orm(schema_name = "my_schema"))]
#[sea_orm(table_name = "schema_enum")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub mood: Option<Mood>,
    pub priority: Option<Priority>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
