use sea_orm::entity::prelude::*;
use sea_orm_macros::DeriveValueType;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "custom_wrapper")]
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
pub struct StringVec(pub Vec<String>);
