use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "cake"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    Name,
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
            Self::Name => ColumnType::String(None).def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        unreachable!()
    }
}

impl ActiveModelBehavior for ActiveModel {}
