use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "model")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_1: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_2: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_3: f64,
    pub owner: String,
    pub name: String,
    pub description: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
