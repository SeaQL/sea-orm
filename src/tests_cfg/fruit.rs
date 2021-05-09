use crate::{
    ColumnTrait, ColumnType, EntityTrait, EnumIter, Iden, IdenStatic, PrimaryKeyOfModel,
    ModelTrait, QueryResult, RelationDef, RelationTrait, TypeErr, Value, PrimaryKeyTrait
};

#[derive(Default, Debug, Iden)]
#[iden = "fruit"]
pub struct Entity;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Model {
    pub id: i32,
    pub name: String,
    pub cake_id: Option<i32>,
}

#[derive(Copy, Clone, Debug, Iden, EnumIter)]
pub enum Column {
    Id,
    Name,
    CakeId,
}

#[derive(Copy, Clone, Debug, Iden, EnumIter)]
pub enum PrimaryKey {
    Id,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl EntityTrait for Entity {
    type Model = Model;

    type Column = Column;

    type PrimaryKey = PrimaryKey;

    type Relation = Relation;
}

// TODO: implement with derive macro
impl ModelTrait for Model {
    type Column = Column;

    fn get(&self, c: Self::Column) -> Value {
        match c {
            Column::Id => self.id.clone().into(),
            Column::Name => self.name.clone().into(),
            Column::CakeId => self.cake_id.clone().into(),
        }
    }

    fn set(&mut self, c: Self::Column, v: Value) {
        match c {
            Column::Id => self.id = v.unwrap(),
            Column::Name => self.name = v.unwrap(),
            Column::CakeId => self.cake_id = v.unwrap(),
        }
    }

    fn from_query_result(row: QueryResult) -> Result<Self, TypeErr> {
        Ok(Self {
            id: row.try_get(Column::Id.as_str())?,
            name: row.try_get(Column::Name.as_str())?,
            cake_id: row.try_get(Column::CakeId.as_str())?,
        })
    }
}

// TODO: implement with derive macro
impl IdenStatic for Column {
    fn as_str(&self) -> &str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
            Self::CakeId => "cake_id",
        }
    }
}

impl ColumnTrait for Column {
    type Entity = Entity;

    fn def(&self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Integer(None),
            Self::Name => ColumnType::String(None),
            Self::CakeId => ColumnType::Integer(None),
        }
    }
}

// TODO: implement with derive macro
impl IdenStatic for PrimaryKey {
    fn as_str(&self) -> &str {
        match self {
            Self::Id => "id",
        }
    }
}

// TODO: implement with derive macro
impl PrimaryKeyTrait for PrimaryKey {}

// TODO: implement with derive macro
impl PrimaryKeyOfModel<Model> for PrimaryKey {
    fn into_column(self) -> <Model as ModelTrait>::Column {
        match self {
            Self::Id => Column::Id,
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!()
    }
}
