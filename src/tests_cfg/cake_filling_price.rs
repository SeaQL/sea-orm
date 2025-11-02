use crate as sea_orm;
use crate::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(schema_name = "public", table_name = "cake_filling_price")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub cake_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub filling_id: i32,
    #[cfg(feature = "with-rust_decimal")]
    #[sea_orm(extra = "CHECK (price > 0)")]
    pub price: Decimal,
    #[sea_orm(ignore)]
    pub ignored_attr: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::cake_filling::Entity",
        from = "(Column::CakeId, Column::FillingId)",
        to = "(super::cake_filling::Column::CakeId, super::cake_filling::Column::FillingId)"
    )]
    CakeFilling,
    #[sea_orm(
        belongs_to = "super::cake::Entity",
        from = "Column::CakeId",
        to = "super::cake::Column::Id",
        skip_fk
    )]
    Cake,
}

impl Related<super::cake_filling::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CakeFilling.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
