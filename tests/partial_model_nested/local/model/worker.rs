use sea_orm::{ActiveValue, IntoActiveValue, entity::prelude::*};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "worker")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    #[sea_orm(has_many, relation_enum = "BakeryManager", via_rel = "Manager")]
    pub manager_of: HasMany<super::bakery::Entity>,
    #[sea_orm(has_many, relation_enum = "BakeryCashier", via_rel = "Cashier")]
    pub cashier_of: HasMany<super::bakery::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
