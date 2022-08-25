use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::access_token::Entity")]
    AccessToken,
}

impl Related<super::access_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AccessToken.def()
    }
}

impl Related<super::access_log::Entity> for Entity {
    fn to() -> RelationDef {
        super::access_token::Relation::AccessLog.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::access_token::Relation::User.def().rev())
    }
}

pub struct AccessLogLink;

impl Linked for AccessLogLink {
    type FromEntity = Entity;

    type ToEntity = super::access_log::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            Relation::AccessToken.def(),
            super::access_token::Relation::AccessLog.def(),
        ]
    }
}

impl ActiveModelBehavior for ActiveModel {}
