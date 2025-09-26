use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "order")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Decimal(Some((16, 4)))")]
    pub total: Decimal,
    pub bakery_id: i32,
    pub customer_id: i32,
    pub placed_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::bakery::Entity",
        from = "Column::BakeryId",
        to = "super::bakery::Column::Id"
    )]
    Bakery,
    #[sea_orm(
        belongs_to = "super::customer::Entity",
        from = "Column::CustomerId",
        to = "super::customer::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Customer,
    #[sea_orm(has_many = "super::lineitem::Entity")]
    Lineitem,
}

impl Related<super::bakery::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bakery.def()
    }
}

impl Related<super::customer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Customer.def()
    }
}

impl Related<super::lineitem::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Lineitem.def()
    }
}

pub struct ToCustomer;
impl Linked for ToCustomer {
    type FromEntity = Entity;
    type ToEntity = super::customer::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Customer.def()]
    }
}

pub struct ToLineitem;
impl Linked for ToLineitem {
    type FromEntity = Entity;
    type ToEntity = super::lineitem::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Lineitem.def()]
    }
}

impl ActiveModelBehavior for ActiveModel {}
