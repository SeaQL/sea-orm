//! An entity definition for testing table index creation.
use crate as sea_orm;
use crate::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(schema_name = "public", table_name = "indexes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub indexes_id: i32,
    #[sea_orm(unique)]
    pub unique_attr: i32,
    #[sea_orm(indexed)]
    pub index1_attr: i32,
    #[sea_orm(unique, indexed)]
    pub index2_attr: i32,
    #[sea_orm(unique_key = "my_unique")]
    pub unique_key_a: String,
    #[sea_orm(unique_key = "my_unique")]
    pub unique_key_b: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
