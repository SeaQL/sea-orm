use crate as sea_orm;
use crate::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "vendor")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        panic!()
    }
}

impl Related<super::filling::Entity> for Entity {
    fn to() -> RelationDef {
        super::filling::Relation::Vendor.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
