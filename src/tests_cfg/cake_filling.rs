use crate as sea_orm;
use crate::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
#[table = "cake_filling"]
pub struct Entity;

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub cake_id: i32,
    pub filling_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    CakeId,
    FillingId,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    CakeId,
    FillingId,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Cake,
    Filling,
}

impl ColumnTrait for Column {
    type EntityName = Entity;

    fn def(&self) -> ColumnType {
        match self {
            Self::CakeId => ColumnType::Integer(None),
            Self::FillingId => ColumnType::Integer(None),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Cake => Entity::has_one(super::cake::Entity)
                .from(Column::CakeId)
                .to(super::cake::Column::Id)
                .into(),
            Self::Filling => Entity::has_one(super::filling::Entity)
                .from(Column::FillingId)
                .to(super::filling::Column::Id)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {
    type Entity = Entity;
}
