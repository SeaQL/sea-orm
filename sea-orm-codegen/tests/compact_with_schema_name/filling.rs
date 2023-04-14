//! SeaORM Entity. Generated by sea-orm-codegen 0.1.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(schema_name = "schema_name", table_name = "filling")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm (entity = "super::cake::Entity", to = "super::cake_filling::Relation::Cake.def()", via = "Some(super::cake_filling::Relation::Filling.def().rev())")]
    Cake
}


impl ActiveModelBehavior for ActiveModel {}
