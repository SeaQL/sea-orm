use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users_saved_bills")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub bill_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::bills::Entity",
        from = "Column::BillId",
        to = "super::bills::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Bills,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Users,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::bills::Entity", to = "Relation::Bills.def()")]
    Bills,
    #[sea_orm(entity = "super::users::Entity", to = "Relation::Users.def()")]
    Users,
}

impl ActiveModelBehavior for ActiveModel {}
