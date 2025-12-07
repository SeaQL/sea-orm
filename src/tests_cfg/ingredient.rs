use crate as sea_orm;
use sea_orm::entity::prelude::*;

#[cfg(feature = "with-json")]
use serde::{Deserialize, Serialize};

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[cfg_attr(feature = "with-json", derive(Serialize, Deserialize))]
#[sea_orm(table_name = "ingredient")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub filling_id: Option<i32>,
    pub ingredient_id: Option<i32>,
    #[sea_orm(belongs_to, from = "filling_id", to = "id")]
    pub filling: HasOne<super::filling::Entity>,
    #[sea_orm(
        self_ref,
        relation_enum = "Ingredient",
        from = "IngredientId",
        to = "Id"
    )]
    pub ingredient: HasOne<Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
