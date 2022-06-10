use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "access_log")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub access_token_id: i32,
    pub description: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::access_token::Entity",
        from = "Column::AccessTokenId",
        to = "super::access_token::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    AccessToken,
}

impl Related<super::access_token::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AccessToken.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
