use sea_orm::entity::prelude::*;

#[cfg(feature = "with-chrono")]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "satellite")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub satellite_name: String,
    #[sea_orm(default_value = "2022-01-26 16:24:00")]
    pub launch_date: DateTimeUtc,
    #[sea_orm(default_value = "2022-01-26 16:24:00")]
    pub deployment_date: DateTimeLocal,
}

#[cfg(feature = "with-time")]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "satellite")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub satellite_name: String,
    #[sea_orm(default_value = "2022-01-26 16:24:00 UTC")]
    pub launch_date: DateTimeWithTimeZone,
    #[sea_orm(default_value = "2022-01-26 16:24:00 +1")]
    pub deployment_date: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
