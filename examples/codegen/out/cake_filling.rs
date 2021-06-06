use crate as sea_orm;
use crate::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "cake_filling"
    }
}

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
        match self {
            Self::CakeId => ColumnType::Custom(s),
            Self::FillingId => ColumnType::Custom(s),
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

impl Related<super::cake::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Cake.def()
    }
}
impl Related<super::filling::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Filling.def()
    }
}

impl Model {
    pub fn find_cake(&self) -> Select<super::cake::Entity> {
        Entity::find_related().belongs_to::<Entity>(self)
    }
    pub fn find_filling(&self) -> Select<super::filling::Entity> {
        Entity::find_related().belongs_to::<Entity>(self)
    }
}

impl ActiveModelBehavior for ActiveModel {}
