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
    #[sea_orm(has_many = "super::users_votes::Entity")]
    UsersVotes,
}

impl Related<super::users_votes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UsersVotes.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        super::users_votes::Relation::Users.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::users_votes::Relation::Bills.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
