use sea_orm::{entity::prelude::*, ActiveValue, IntoActiveValue};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "worker")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::bakery::Entity")]
    BakeryManager,
    #[sea_orm(has_one = "super::bakery::Entity")]
    BakeryCashier,
}

impl ActiveModelBehavior for ActiveModel {}
