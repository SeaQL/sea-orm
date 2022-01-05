use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "satellites")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub satellite_name: String,
    pub launch_date: DateTimeUtc,
    pub deployment_date: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
