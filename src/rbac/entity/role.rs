use crate as sea_orm;
use sea_orm::DeriveValueType;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "sea_orm_role")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: RoleId,
    #[sea_orm(unique)]
    pub role: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, DeriveValueType)]
pub struct RoleId(pub i64);

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
