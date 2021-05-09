use crate::{
    ColumnTrait, ColumnType, EntityTrait, EnumIter, Iden, IdenStatic, Identity, IntoIdentity,
    ModelTrait, QueryResult, Related, RelationDef, RelationTrait, Select, TypeErr, Value,
};

#[derive(Default, Debug, Iden)]
#[iden = "cake"]
pub struct Entity;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Model {
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, Iden, EnumIter)]
pub enum Column {
    Id,
    Name,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Fruit,
}

impl EntityTrait for Entity {
    type Model = Model;

    type Column = Column;

    type Relation = Relation;

    fn primary_key() -> Identity {
        Column::Id.into_identity()
    }
}

// TODO: implement with derive macro
impl ModelTrait for Model {
    type Column = Column;

    fn get(&self, c: Self::Column) -> Value {
        match c {
            Column::Id => self.id.clone().into(),
            Column::Name => self.name.clone().into(),
        }
    }

    fn set(&mut self, c: Self::Column, v: Value) {
        match c {
            Column::Id => self.id = v.unwrap(),
            Column::Name => self.name = v.unwrap(),
        }
    }

    fn from_query_result(row: QueryResult) -> Result<Self, TypeErr> {
        Ok(Self {
            id: row.try_get(Column::Id.as_str())?,
            name: row.try_get(Column::Name.as_str())?,
        })
    }
}

// TODO: implement with derive macro
impl IdenStatic for Column {
    fn as_str(&self) -> &str {
        match self {
            Column::Id => "id",
            Column::Name => "name",
        }
    }
}

impl ColumnTrait for Column {
    type Entity = Entity;

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

impl Entity {
    pub fn find_fruit() -> Select<super::fruit::Entity> {
        Self::find_related()
    }
}
