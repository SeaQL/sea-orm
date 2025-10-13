use crate as sea_orm;
use sea_orm::entity::prelude::*;

#[cfg(feature = "with-json")]
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[cfg_attr(feature = "with-json", derive(Serialize, Deserialize))]
#[sea_orm(table_name = "ingredient")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub filling_id: Option<i32>,
    #[sea_orm(belongs_to, from = "FillingId", to = "Id")]
    pub filling: Option<super::filling::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
