use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "bakery")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub profit_margin: f64,
    pub manager_id: i32,
    pub cashier_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::worker::Entity",
        from = "Column::ManagerId",
        to = "super::worker::Column::Id"
    )]
    Manager,
    #[sea_orm(
        belongs_to = "super::worker::Entity",
        from = "Column::CashierId",
        to = "super::worker::Column::Id"
    )]
    Cashier,
}

impl ActiveModelBehavior for ActiveModel {}
