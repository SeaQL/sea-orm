use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "applog")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub action: String,
    pub json: Json,
    pub date_time_naive: DateTime,
    pub timestamp_naive: DateTime,
    pub timestamp_tz_timezone: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
