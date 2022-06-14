use crate as sea_orm;
use crate::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "vendor")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    #[sea_orm(
        soft_delete_column,
        soft_delete_expr = "Func::current_timestamp()",
        restore_soft_delete_expr = r#"Expr::cust("NULL")"#
    )]
    #[cfg(feature = "with-chrono")]
    pub deleted_at: Option<DateTimeWithTimeZone>,
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
