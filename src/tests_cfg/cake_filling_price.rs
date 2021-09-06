use crate as sea_orm;
use crate::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
#[sea(schema_name = "public", table_name = "cake_filling_price")]
pub struct Entity;

#[derive(
    Clone,
    Debug,
    PartialEq,
    DeriveModel,
    DeriveActiveModel,
    DeriveActiveModelBehavior,
    DeriveModelColumn,
    DeriveModelPrimaryKey,
)]
pub struct Model {
    #[sea(primary_key)]
    pub cake_id: i32,
    #[sea(primary_key)]
    pub filling_id: i32,
    pub price: Decimal,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea(
        belongs_to = "super::cake_filling::Entity",
        from = "(Column::CakeId, Column::FillingId)",
        to = "(
            super::cake_filling::Column::CakeId,
            super::cake_filling::Column::FillingId,
        )"
    )]
    CakeFilling,
}

impl Related<super::cake_filling::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CakeFilling.def()
    }
}
