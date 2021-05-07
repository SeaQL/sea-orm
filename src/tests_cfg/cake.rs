use crate::{Column, ColumnType, Entity, Identity, IntoIdentity, Relation, RelationDef};
use sea_query::Iden;
use strum::EnumIter;

#[derive(Iden, Default, Debug)]
pub struct Cake;

#[derive(Debug, Default, PartialEq)]
pub struct CakeModel {
    pub id: Option<i32>,
    pub name: Option<String>,
}

#[derive(Iden, EnumIter)]
pub enum CakeColumn {
    Id,
    Name,
}

#[derive(EnumIter)]
pub enum CakeRelation {}

impl Entity for Cake {
    type Model = CakeModel;

    type Column = CakeColumn;

    type Relation = CakeRelation;

    fn primary_key() -> Identity {
        CakeColumn::Id.into_identity()
    }
}

impl Column for CakeColumn {
    fn col_type(&self) -> ColumnType {
        match self {
            Self::Id => ColumnType::Integer(None),
            Self::Name => ColumnType::String(None),
        }
    }
}

impl Relation for CakeRelation {
    fn rel_def(&self) -> RelationDef {
        panic!()
    }
}
