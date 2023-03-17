use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "binary")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub binary: Vec<u8>,
    #[sea_orm(column_type = "Binary(BlobSize::Blob(Some(10)))")]
    pub binary_10: Vec<u8>,
    #[sea_orm(column_type = "Binary(BlobSize::Tiny)")]
    pub binary_tiny: Vec<u8>,
    #[sea_orm(column_type = "Binary(BlobSize::Medium)")]
    pub binary_medium: Vec<u8>,
    #[sea_orm(column_type = "Binary(BlobSize::Long)")]
    pub binary_long: Vec<u8>,
    #[sea_orm(column_type = "VarBinary(10)")]
    pub var_binary: Vec<u8>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
