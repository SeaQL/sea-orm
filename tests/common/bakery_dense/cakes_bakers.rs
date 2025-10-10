use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "cakes_bakers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub cake_id: i32,
    #[sea_orm(primary_key)]
    pub baker_id: i32,
    #[sea_orm(relation = "Cake", from = "CakeId", to = "Id")]
    pub cake: BelongsTo<super::cake::Entity>,
    #[sea_orm(relation = "Baker", from = "BakerId", to = "Id")]
    pub baker: BelongsTo<super::baker::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
