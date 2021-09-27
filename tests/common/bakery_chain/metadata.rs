use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "metadata")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uuid: Uuid,
    pub key: String,
    pub value: String,
    pub bytes: Vec<u8>,
    pub date: Date,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
