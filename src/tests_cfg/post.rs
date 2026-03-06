use crate as sea_orm;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "post")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    #[sea_orm(belongs_to, from = "user_id", to = "id")]
    pub author: HasOne<super::user::Entity>,
    #[sea_orm(has_many)]
    pub comments: HasMany<super::comment::Entity>,
    #[sea_orm(has_many, via = "post_tag")]
    pub tags: HasMany<super::tag::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
