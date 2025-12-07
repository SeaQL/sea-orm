use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "bakery")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub profit_margin: f64,
    pub manager_id: i32,
    pub cashier_id: i32,
    #[sea_orm(belongs_to, relation_enum = "Manager", from = "manager_id", to = "id")]
    pub manager: HasOne<super::worker::Entity>,
    #[sea_orm(belongs_to, relation_enum = "Cashier", from = "cashier_id", to = "id")]
    pub cashier: HasOne<super::worker::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
