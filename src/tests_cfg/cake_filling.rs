use crate as sea_orm;
use sea_orm::entity::prelude::*;

#[sea_orm::compact_model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "cake_filling")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub cake_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub filling_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::cake::Entity",
        from = "Column::CakeId",
        to = "super::cake::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Cake,
    #[sea_orm(
        belongs_to = "super::filling::Entity",
        from = "Column::FillingId",
        to = "super::filling::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Filling,
}

impl Related<super::cake::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Cake.def()
    }
}

impl Related<super::filling::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Filling.def()
    }
}

impl Related<super::cake_filling_price::Entity> for Entity {
    fn to() -> RelationDef {
        super::cake_filling_price::Relation::CakeFilling.def().rev()
    }
}

impl ActiveModelBehavior for ActiveModel {}
