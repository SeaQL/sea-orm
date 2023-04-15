use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text")]
    pub email: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::bills::Entity")]
    Bills,
    #[sea_orm(has_many = "super::users_saved_bills::Entity")]
    UsersSavedBills,
    #[sea_orm(has_many = "super::users_votes::Entity")]
    UsersVotes,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::bills::Entity", to = "Relation::Bills.def()")]
    Bills,
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
