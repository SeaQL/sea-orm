use super::sea_orm_active_enums::*;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, IntoActiveValue};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[cfg_attr(feature = "sqlx-postgres", sea_orm(schema_name = "public"))]
#[sea_orm(table_name = "custom_active_model")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub age: i32,
    pub weight: Option<f32>,
    pub amount: Option<i32>,
    pub tea: Tea,
    pub category: Option<Category>,
    pub color: Option<Color>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, DeriveIntoActiveModel)]
pub struct CustomActiveModel {
    pub age: Option<i32>,
    pub weight: Option<f32>,
    pub amount: Option<Option<i32>>,
    pub tea: Option<Tea>,
    pub category: Option<Category>,
    pub color: Option<Option<Color>>,
}
