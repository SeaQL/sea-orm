//! An entity definition for testing table index creation.
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "indexes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub indexes_id: i32,
    #[sea_orm(unique)]
    pub unique_attr: i32,
    #[sea_orm(unique_key = "my_unique")]
    pub unique_key_a: String,
    #[sea_orm(unique_key = "my_unique")]
    pub unique_key_b: String,
}

impl ActiveModelBehavior for ActiveModel {}
