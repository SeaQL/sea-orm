use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "sea_orm_role_role")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub role_id: i64,
    #[sea_orm(primary_key)]
    pub super_role_id: i64,
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
        belongs_to = "super::role::Entity",
        from = "Column::SuperRoleId",
        to = "super::role::Column::Id"
    )]
    SuperRole,
}

impl ActiveModelBehavior for ActiveModel {}
