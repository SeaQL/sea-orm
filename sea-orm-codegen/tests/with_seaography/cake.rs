//! SeaORM Entity. Generated by sea-orm-codegen 0.1.0

use sea_orm::entity::prelude:: * ;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "cake")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "Text", nullable)]
    pub name: Option<String> ,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::fruit::Entity")]
    Fruit,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::fruit::Entity", to = "Relation::Fruit.def()")]
    Fruit,
    #[sea_orm (entity = "super::filling::Entity", to = "super::cake_filling::Relation::Filling.def()", via = "Some(super::cake_filling::Relation::Cake.def().rev())")]
    Filling
}

impl ActiveModelBehavior for ActiveModel {}

impl seaography::RelationBuilder for Relation {
    fn get_relation(&self, context: & 'static seaography::BuilderContext) -> async_graphql::dynamic::Field {
        let builder = seaography::EntityObjectRelationBuilder { context };
        match self {
            Self::Fruit => builder.get_relation:: <Entity, super::fruit::Entity>("fruit", Self::Fruit.def(),)
        }
    }
}

impl seaography::RelationBuilder for RelatedEntity {
    fn get_relation(&self, context: & 'static seaography::BuilderContext) -> async_graphql::dynamic::Field {
        let builder = seaography::EntityObjectViaRelationBuilder { context };
        match self {
            Self::Fruit => builder.get_relation:: <Entity, super::fruit::Entity>("fruit"),
            Self::Filling => builder.get_relation:: <Entity, super::filling::Entity>("filling")
        }
    }
}