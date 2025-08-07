use crate as sea_orm;
use sea_orm::entity::prelude::*;

use super::{permission::PermissionId, resource::ResourceId, role::RoleId};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "sea_orm_role_permission")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub role_id: RoleId,
    #[sea_orm(primary_key)]
    pub permission_id: PermissionId,
    #[sea_orm(primary_key)]
    pub resource_id: ResourceId,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::role::Entity",
        from = "Column::RoleId",
        to = "super::role::Column::Id"
    )]
    Role,
    #[sea_orm(
        belongs_to = "super::permission::Entity",
        from = "Column::PermissionId",
        to = "super::permission::Column::Id"
    )]
    Permission,
    #[sea_orm(
        belongs_to = "super::resource::Entity",
        from = "Column::ResourceId",
        to = "super::resource::Column::Id"
    )]
    Resource,
}

impl ActiveModelBehavior for ActiveModel {}
