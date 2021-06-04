use crate as sea_orm;
use crate::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
#[table = "cake_filling"]
pub struct Entity;

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub cake_id: some_rust_type,
    pub filling_id: some_rust_type,
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
        match self {}
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Fruit => Entity::has_many(super::fruit::Entity)
                .from(Column::Id)
                .to(super::fruit::Column::CakeId)
                .into(),
        }
    }
}

impl Related<super::fruit::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Fruit.def()
    }
}

impl Related<super::filling::Entity> for Entity {
    fn to() -> RelationDef {
        super::cake_filling::Relation::Filling.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::cake_filling::Relation::Cake.def().rev())
    }
}

impl Model {
    pub fn find_fruit(&self) -> Select<super::fruit::Entity> {
        Entity::find_related().belongs_to::<Entity>(self)
    }
    pub fn find_filling(&self) -> Select<super::filling::Entity> {
        Entity::find_related().belongs_to::<Entity>(self)
    }
}
