use sea_orm::entity::prelude::*;
use sea_orm_macros::DeriveValueType;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "value_type")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub number: Integer,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
pub struct Integer(pub i32);

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
#[sea_orm(column_type = "String(Some(1))", array_type = "String")]
pub struct StringVec(pub Vec<String>);

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
#[sea_orm(column_type = "Boolean", array_type = "Bool")]
pub struct Boolbean(pub String);
