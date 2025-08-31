use crate as sea_orm;
use sea_orm::DeriveValueType;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "sea_orm_permission")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: PermissionId,
    #[sea_orm(unique)]
    pub action: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, DeriveValueType)]
pub struct PermissionId(pub i64);

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
