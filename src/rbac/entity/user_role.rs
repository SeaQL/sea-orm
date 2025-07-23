use crate as sea_orm;
use sea_orm::entity::prelude::*;

use super::{role::RoleId, user::UserId};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "sea_orm_user_role")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_id: UserId,
    #[sea_orm(primary_key)]
    pub role_id: RoleId,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::role::Entity",
        from = "Column::RoleId",
        to = "super::role::Column::Id"
    )]
    Role,
}

impl ActiveModelBehavior for ActiveModel {}
