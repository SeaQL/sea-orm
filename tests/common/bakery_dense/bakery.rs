use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "bakery")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub profit_margin: f64,
    #[sea_orm(relation)]
    pub bakers: HasMany<super::baker::Entity>,
    #[sea_orm(relation)]
    pub orders: HasMany<super::order::Entity>,
    #[sea_orm(relation)]
    pub cakes: HasMany<super::cake::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
