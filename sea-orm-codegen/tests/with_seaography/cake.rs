//! SeaORM Entity. Generated by sea-orm-codegen 0.1.0

use sea_orm::entity::prelude:: * ;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "cake")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text", nullable)]
    pub name: Option<String> ,
    pub base_id: Option<i32> ,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::fruit::Entity")]
    Fruit,
    #[sea_orm(has_one = "Entity")]
    SelfRef ,
}

impl Related<super::fruit::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Fruit.def()
    }
}

impl Related<super::filling::Entity> for Entity {
    fn to() -> RelationDef {
        super::cake_filling::Relation::Filling.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::cake_filling::Relation::Cake.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::fruit::Entity")]
    Fruit,
    #[sea_orm(entity = "Entity", def = "Relation::SelfRef.def()")]
    SelfRef,
    #[sea_orm(entity = "Entity", def = "Relation::SelfRef.def().rev()")]
    SelfRefRev,
    #[sea_orm(entity = "super::filling::Entity")]
    Filling
}
