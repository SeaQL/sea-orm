use crate as sea_orm;
use crate::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "cake"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub id: some_rust_type,
    pub name: some_rust_type,
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
    CakeFilling,
    Fruit,
}

impl ColumnTrait for Column {
    type EntityName = Entity;
    fn def(&self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Custom(s),
            Self::Name => ColumnType::Custom(s),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::CakeFilling => Entity::has_Many(super::cake_filling::Entity)
                .from(Column::Id)
                .to(super::cake_filling::Column::CakeId)
                .into(),
            Self::Fruit => Entity::has_Many(super::fruit::Entity)
                .from(Column::Id)
                .to(super::fruit::Column::CakeId)
                .into(),
        }
    }
}

impl Related<super::cake_filling::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CakeFilling.def()
    }
}
impl Related<super::fruit::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Fruit.def()
    }
}

impl Model {
    pub fn find_cake_filling(&self) -> Select<super::cake_filling::Entity> {
        Entity::find_related().belongs_to::<Entity>(self)
    }
    pub fn find_fruit(&self) -> Select<super::fruit::Entity> {
        Entity::find_related().belongs_to::<Entity>(self)
    }
}

impl ActiveModelBehavior for ActiveModel {}
