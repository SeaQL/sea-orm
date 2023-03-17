//! The `cake_filling` entity.

use sea_orm::entity::prelude::*;

/// CakeFilling entity
#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "cake_filling"
    }
}

/// CakeFilling model
#[derive(Clone, Debug, PartialEq, Eq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    /// cake_id field
    pub cake_id: i32,
    /// filling_id field
    pub filling_id: i32,
}

/// CakeFilling column
#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    /// CakeId column
    CakeId,
    /// FillingId column
    FillingId,
}

/// CakeFilling primary key
#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    /// CakeId primary key
    CakeId,
    /// FillingId primary key
    FillingId,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = (i32, i32);

    fn auto_increment() -> bool {
        false
    }
}

/// CakeFilling relation
#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    /// Cake relation
    Cake,
    /// Filling relation
    Filling,
}

impl ColumnTrait for Column {
    type EntityName = Entity;

    fn def(&self) -> ColumnDef {
        match self {
            Self::CakeId => ColumnType::Integer.def(),
            Self::FillingId => ColumnType::Integer.def(),
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
            Self::Filling => Entity::belongs_to(super::filling::Entity)
                .from(Column::FillingId)
                .to(super::filling::Column::Id)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
