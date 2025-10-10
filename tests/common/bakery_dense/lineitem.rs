use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "lineitem")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub price: Decimal,
    pub quantity: i32,
    pub order_id: i32,
    pub cake_id: i32,
    #[sea_orm(relation = "Order", from = "OrderId", to = "Id")]
    pub order: BelongsTo<super::order::Entity>,
    #[sea_orm(relation = "Cake", from = "CakeId", to = "Id")]
    pub cake: BelongsTo<super::cake::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
