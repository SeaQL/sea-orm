use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "post1")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub uuid: i32,
    pub name: String,
    #[sea_orm(column_type = "Text")]
    pub text: String,
}

impl ActiveModelBehavior for ActiveModel {}
