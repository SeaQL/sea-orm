use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "person", schema_name = "test")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub alias: String,
    pub last_nam: String,
    #[sea_orm(column_type = "Text")]
    pub gender: String,
}

impl ActiveModelBehavior for ActiveModel {}
