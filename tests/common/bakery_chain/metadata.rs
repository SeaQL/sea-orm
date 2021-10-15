use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "metadata")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uuid: Uuid,
    pub uuid_ref: Option<Uuid>,
    #[sea_orm(column_name = "type", enum_name = "Type")]
    pub ty: String,
    pub key: String,
    pub value: String,
    pub bytes: Vec<u8>,
    pub date: Option<Date>,
    pub time: Option<Time>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "Entity", from = "Column::UuidRef", to = "Column::Uuid")]
    SelfReferencing,
}

pub struct SelfReferencingLink;

impl Linked for SelfReferencingLink {
    type FromEntity = Entity;

    type ToEntity = Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::SelfReferencing.def()]
    }
}

impl ActiveModelBehavior for ActiveModel {}
