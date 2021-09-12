use serde::{Deserialize, Serialize};
use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity, Deserialize, Serialize)]
#[serde(crate = "serde")]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "posts"
    }
}

#[derive(
    Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel, Deserialize, Serialize,
)]
#[serde(crate = "serde")]
pub struct Model {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub title: String,
    pub text: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    Title,
    Text,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Id,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = i32;

    fn auto_increment() -> bool {
        true
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl ColumnTrait for Column {
    type EntityName = Entity;

    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::Integer.def(),
            Self::Title => ColumnType::String(None).def(),
            Self::Text => ColumnType::String(None).def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!()
    }
}
impl ActiveModelBehavior for ActiveModel {}
