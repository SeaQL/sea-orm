use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "bills")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: Option<i32> ,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction",
    )]
    Users,
    #[sea_orm(has_many = "super::users_saved_bills::Entity")]
    UsersSavedBills,
    #[sea_orm(has_many = "super::users_votes::Entity")]
    UsersVotes,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::users::Entity", to = "Relation::Users.def()")]
    Users,
    #[sea_orm(
        entity = "super::users_saved_bills::Entity",
        to = "Relation::UsersSavedBills.def()"
    )]
    UsersSavedBills,
    #[sea_orm(
        entity = "super::users_votes::Entity",
        to = "Relation::UsersVotes.def()"
    )]
    UsersVotes
}

impl ActiveModelBehavior for ActiveModel {}
