use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "baker")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub contact_details: Json,
    pub bakery_id: Option<i32>,
    #[sea_orm(
        relation = "Bakery",
        from = "BakeryId",
        to = "Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    pub bakery: BelongsTo<super::bakery::Entity>,
    #[sea_orm(relation = "Cake", via = "cakes_bakers::Baker")]
    pub cakes: HasMany<super::baker::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
