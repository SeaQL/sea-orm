use crate as sea_orm;
use crate::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "filling"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub id: i32,
    pub name: String,
}

// If your column names are not in snake-case, derive `DeriveCustomColumn` here.
#[derive(Copy, Clone, Debug, EnumIter, DeriveCustomColumn)]
pub enum Column {
    Id,
    Name,
}

// Then, customize each column names here.
impl IdenStatic for Column {
    fn as_str(&self) -> &str {
        match self {
            // Override column names
            Self::Id => "id",
            // Leave all other columns using default snake-case values
            _ => self.default_as_str(),
        }
    }
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
        panic!()
    }
}

impl Related<super::cake::Entity> for Entity {
    fn to() -> RelationDef {
        super::cake_filling::Relation::Cake.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::cake_filling::Relation::Filling.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
