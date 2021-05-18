use crate::entity::prelude::*;
use crate as sea_orm;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
#[table = "cake"]
pub struct Entity;

#[derive(Clone, Debug, Default, PartialEq, DeriveModel)]
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

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Fruit,
    CakeFilling,
}

impl EntityTrait for Entity {
    type Model = Model;

    type Column = Column;

    type PrimaryKey = PrimaryKey;

    type Relation = Relation;
}

impl ColumnTrait for Column {
    type EntityName = Entity;

    fn def(&self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Integer(None),
            Self::Name => ColumnType::String(None),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Fruit => Entity::has_many(super::fruit::Entity)
                .from(Column::Id)
                .to(super::fruit::Column::CakeId)
                .into(),
            Self::CakeFilling => Entity::has_many(super::cake_filling::Entity)
                .from(Column::Id)
                .to(super::cake_filling::Column::CakeId)
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
        Some(Relation::CakeFilling.def())
    }
}

impl Model {
    pub fn find_fruit(&self) -> Select<super::fruit::Entity> {
        Entity::find_related().belongs_to::<Entity>(self)
    }
}
