//! The `fruit` entity.

use sea_orm::entity::prelude::*;

/// Fruit entity
#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "fruit"
    }
}

/// Fruit model
#[derive(Clone, Debug, PartialEq, Eq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    /// id field
    pub id: i32,
    /// name field
    pub name: String,
    /// cake_id field
    pub cake_id: Option<i32>,
}

/// Fruit column
#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    /// Id column
    Id,
    /// Name column
    Name,
    /// CakeId column
    CakeId,
}

/// Fruit primary key
#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    /// Id primary key
    Id,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = i32;

    fn auto_increment() -> bool {
        true
    }
}

/// Fruit relation
#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    /// Cake relation
    Cake,
}

impl ColumnTrait for Column {
    type EntityName = Entity;

    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::Integer.def(),
            Self::Name => ColumnType::String(None).def(),
            Self::CakeId => ColumnType::Integer.def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Cake => Entity::belongs_to(super::cake::Entity)
                .from(Column::CakeId)
                .to(super::cake::Column::Id)
                .into(),
        }
    }
}

impl Related<super::cake::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Cake.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
