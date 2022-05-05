use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "serde_json_value")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub json: Json,
    pub json_value: JsonValue<KeyValue>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KeyValue {
    pub id: i32,
    pub name: String,
    pub price: f32,
    pub notes: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
