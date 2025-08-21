use sea_orm::{ActiveValue, IntoActiveValue, entity::prelude::*};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "worker")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::bakery::Entity", via = "Relation::Manager")]
    BakeryManager,
}

impl ActiveModelBehavior for ActiveModel {}
