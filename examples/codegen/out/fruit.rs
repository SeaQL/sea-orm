use crate as sea_orm;
use crate::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "fruit"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub id: some_rust_type,
    pub name: some_rust_type,
    pub cake_id: some_rust_type,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    Name,
    CakeId,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Id,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Cake,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Custom(s),
            Self::Name => ColumnType::Custom(s),
            Self::CakeId => ColumnType::Custom(s),
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
        }
    }
}

impl Related<super::cake::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Cake.def()
    }
}

impl Model {
    pub fn find_cake(&self) -> Select<super::cake::Entity> {
        Entity::find_related().belongs_to::<Entity>(self)
    }
}

impl ActiveModelBehavior for ActiveModel {}
