use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    #[sea_orm(unique)]
    pub email: String,
    #[sea_orm(self_ref, via = "user_follower", from = "User", to = "Follower")]
    pub followers: HasMany<Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
