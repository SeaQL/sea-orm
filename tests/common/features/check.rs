use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "check")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub pay: String,
    pub amount: f64,
    #[sea_orm(updated_at, nullable, extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(created_at, nullable, extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
