use crate::{
    ColumnTrait, ColumnType, EntityTrait, Identity, IntoIdentity, ModelTrait, QueryResult, RelationDef,
    RelationTrait, TypeErr, EnumIter, Iden
};

#[derive(Iden, Default, Debug)]
#[iden = "cake"]
pub struct Entity;

#[derive(Debug, Default, PartialEq)]
pub struct Model {
    pub id: i32,
    pub name: String,
}

#[derive(Iden, EnumIter)]
pub enum Column {
    Id,
    Name,
}

#[derive(EnumIter)]
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
        })
    }
}

impl ColumnTrait for Column {
    fn col_type(&self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Integer(None),
            Self::Name => ColumnType::String(None),
        }
    }
}

impl RelationTrait for Relation {
    fn rel_def(&self) -> RelationDef {
        panic!()
    }
}
