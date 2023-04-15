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
    #[sea_orm(has_many = "super::users_votes::Entity")]
    UsersVotes,
}

impl Related<super::users_votes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UsersVotes.def()
    }
}

impl Related<super::bills::Entity> for Entity {
    fn to() -> RelationDef {
        super::users_votes::Relation::Bills.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::users_votes::Relation::Users.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
