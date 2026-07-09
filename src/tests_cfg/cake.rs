use crate as sea_orm;
use sea_orm::entity::prelude::*;

#[cfg(feature = "with-json")]
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[cfg_attr(feature = "with-json", derive(Serialize, Deserialize))]
#[sea_orm(table_name = "cake")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    #[sea_orm(has_one)]
    pub fruit: HasOne<super::fruit::Entity>,
    #[sea_orm(has_many, via = "cake_filling")]
    pub fillings: HasMany<super::filling::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
