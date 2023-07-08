pub mod value_type_general {
    use super::*;
    use sea_orm::entity::prelude::*;

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
}

pub mod value_type_pg {
    use super::*;
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "value_type_postgres")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub number: Integer,
        pub str_vec: StringVec,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

use sea_orm::entity::prelude::*;
use sea_orm_macros::DeriveValueType;

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
#[sea_orm(array_type = "Int")]
pub struct Integer(i32);

impl<T> From<T> for Integer
where
    T: Into<i32>,
{
    fn from(v: T) -> Integer {
        Integer(v.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
#[sea_orm(column_type = "Boolean", array_type = "Bool")]
pub struct Boolbean(pub String);

#[derive(Clone, Debug, PartialEq, Eq, DeriveValueType)]
pub struct StringVec(pub Vec<String>);
