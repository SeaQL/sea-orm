use crate as sea_orm;
use crate::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
#[sea(table_name = "cake_filling")]
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
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea(
        belongs_to = "super::cake::Entity",
        from = "Column::CakeId"
        to = "super::cake::Column::Id"
    )]
    Cake,
    #[sea(
        belongs_to = "super::filling::Entity",
        from = "Column::FillingId"
        to = "super::filling::Column::Id"
    )]
    Filling,
}

impl Related<super::cake_filling_price::Entity> for Entity {
    fn to() -> RelationDef {
        super::cake_filling_price::Relation::CakeFilling.def().rev()
    }
}
