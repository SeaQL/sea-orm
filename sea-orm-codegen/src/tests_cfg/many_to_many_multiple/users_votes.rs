use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users_votes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub bill_id: i32,
    pub user_idd: Option<i32>,
    pub bill_idd: Option<i32>,
    pub vote: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::bills::Entity",
        from = "Column::BillIdd",
        to = "super::bills::Column::Id"
    )]
    Bills2,
    #[sea_orm(
        belongs_to = "super::bills::Entity",
        from = "Column::BillId",
        to = "super::bills::Column::Id"
    )]
    Bills1,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserIdd",
        to = "super::users::Column::Id"
    )]
    Users2,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    Users1,
}

impl ActiveModelBehavior for ActiveModel {}
