#![warn(clippy::unused_async_trait_impl)]
use crate as sea_orm;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "cake")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}

impl ActiveModelBehavior for ActiveModel {}
