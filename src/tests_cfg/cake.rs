use crate as sea_orm;
use crate::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
#[table = "cake"]
pub struct Entity;

#[derive(Clone, Debug, PartialEq, DeriveModel)]
pub struct Model {
    pub id: i32,
    pub name: String,
}

// can we generate this?
#[derive(Clone, Debug)]
pub struct ActiveModel {
    pub id: Action<i32>,
    pub name: Action<String>,
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

// can we generate this?
impl ActiveModelOf<Model> for ActiveModel {
    fn from_model(m: Model) -> Self {
        Self::from(m)
    }
}

// can we generate this?
impl From<Model> for ActiveModel {
    fn from(m: Model) -> Self {
        Self {
            id: Action::Set(m.id),
            name: Action::Set(m.name),
        }
    }
}

// can we generate this?
impl ActiveModelTrait for ActiveModel {
    type Column = Column;

    fn get(&self, c: Self::Column) -> Action<Value> {
        match c {
            Column::Id => self.id.clone().into_action_value(),
            Column::Name => self.name.clone().into_action_value(),
        }
    }

    fn set(&mut self, c: Self::Column, v: Value) {
        match c {
            Column::Id => self.id = Action::Set(v.unwrap()),
            Column::Name => self.name = Action::Set(v.unwrap()),
        }
    }

    fn unset(&mut self, c: Self::Column) {
        match c {
            Column::Id => self.id = Action::Unset,
            Column::Name => self.name = Action::Unset,
        }
    }
}