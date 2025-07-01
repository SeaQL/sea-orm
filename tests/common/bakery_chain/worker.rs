use sea_orm::{entity::prelude::*, ActiveValue, IntoActiveValue};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "worker")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::bakery::Entity",
        from = "Column::Id",
        to = "super::bakery::Column::ManagerId"
    )]
    Bakery,
}

impl Related<super::bakery::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bakery.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
