use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "applog")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub action: String,
    pub json: Json,
    #[sea_orm(column_type = "JsonBinary")]
    pub jsonb: Json,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
