use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "person", schema_name = "test1")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    #[sea_orm(column_type = "Text")]
    pub gender: String,
}

impl ActiveModelBehavior for ActiveModel {}
