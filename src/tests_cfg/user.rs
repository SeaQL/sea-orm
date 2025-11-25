use crate as sea_orm;
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
    #[sea_orm(has_one)]
    pub profile: HasOne<super::profile::Entity>,
    #[sea_orm(has_many)]
    pub posts: HasMany<super::post::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl RelatedSelfVia<super::user_follower::Entity> for Entity {
    fn to() -> RelationDef {
        super::user_follower::Relation::Follower.def()
    }

    fn via() -> RelationDef {
        super::user_follower::Relation::User.def().rev()
    }
}
