use super::sea_orm_active_enums::*;
use sea_orm::entity::prelude::*;

#[sea_orm::compact_model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "collection")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(
        column_type = r#"custom("citext")"#,
        select_as = "text",
        save_as = "citext"
    )]
    pub name: String,
    pub integers: Vec<Option<i32>>,
    pub integers_opt: Option<Vec<Option<i32>>>,
    pub teas: Vec<Option<Tea>>,
    pub teas_opt: Option<Vec<Option<Tea>>>,
    pub colors: Vec<Option<Color>>,
    pub colors_opt: Option<Vec<Option<Color>>>,
    pub uuid: Vec<Option<Uuid>>,
    pub uuid_hyphenated: Vec<Option<uuid::fmt::Hyphenated>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
