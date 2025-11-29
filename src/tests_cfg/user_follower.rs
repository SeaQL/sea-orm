use crate as sea_orm;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user_follower")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_id: i32,
    #[sea_orm(primary_key)]
    pub follower_id: i32,
    #[sea_orm(belongs_to, from = "user_id", to = "id")]
    pub user: Option<super::user::Entity>,
    #[sea_orm(
        belongs_to,
        relation_enum = "Follower",
        from = "follower_id",
        to = "id"
    )]
    pub follower: Option<super::user::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
