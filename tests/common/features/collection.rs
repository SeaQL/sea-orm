use super::sea_orm_active_enums::*;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "collection")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(
        column_type = r#"Custom("citext".into())"#,
        cast_select = "text",
        cast_value = "citext"
    )]
    pub name: String,
    pub integers: Vec<i32>,
    pub integers_opt: Option<Vec<i32>>,
    pub teas: Vec<Tea>,
    pub teas_opt: Option<Vec<Tea>>,
    pub colors: Vec<Color>,
    pub colors_opt: Option<Vec<Color>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
