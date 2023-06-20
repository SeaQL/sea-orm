use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub self_id: Option<i32>,
    pub self_idd: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "Entity", from = "Column::SelfId", to = "Column::Id")]
    SelfRef2,
    #[sea_orm(belongs_to = "Entity", from = "Column::SelfIdd", to = "Column::Id")]
    SelfRef1,
}

impl ActiveModelBehavior for ActiveModel {}
