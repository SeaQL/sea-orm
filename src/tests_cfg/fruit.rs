use crate::{
    ColumnTrait, ColumnType, EntityTrait, EnumIter, Iden, Identity, IntoIdentity, ModelTrait,
    QueryResult, RelationDef, RelationTrait, TypeErr,
};

#[derive(Default, Debug, Iden)]
#[iden = "fruit"]
pub struct Entity;

#[derive(Debug, Default, PartialEq)]
pub struct Model {
    pub id: i32,
    pub name: String,
    pub cake_id: Option<i32>,
}

#[derive(Copy, Clone, Iden, EnumIter)]
pub enum Column {
    Id,
    Name,
    CakeId,
}

#[derive(Copy, Clone, EnumIter)]
pub enum Relation {}

impl EntityTrait for Entity {
    type Model = Model;

    type Column = Column;

    type Relation = Relation;

    fn primary_key() -> Identity {
        Column::Id.into_identity()
    }
}

impl ModelTrait for Model {
    fn from_query_result(row: QueryResult) -> Result<Self, TypeErr> {
        Ok(Self {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            cake_id: row.try_get_option("cake_id")?,
        })
    }
}

impl ColumnTrait for Column {
    type Entity = Entity;

    fn col_type(&self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Integer(None),
            Self::Name => ColumnType::String(None),
            Self::CakeId => ColumnType::Integer(None),
        }
    }
}

impl RelationTrait for Relation {
    fn rel_def(&self) -> RelationDef {
        panic!()
    }
}
