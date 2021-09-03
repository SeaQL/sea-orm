use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "baker"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub id: i32,
    pub name: String,
    pub contact_details: Json,
    pub bakery_id: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    Name,
    ContactDetails,
    BakeryId,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Id,
}

impl PrimaryKeyTrait for PrimaryKey {
    fn auto_increment() -> bool {
        true
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Bakery,
}

impl ColumnTrait for Column {
    type EntityName = Entity;

    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::Integer.def(),
            Self::Name => ColumnType::String(None).def(),
            Self::ContactDetails => ColumnType::Json.def(),
            Self::BakeryId => ColumnType::Integer.def().null(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Bakery => Entity::belongs_to(super::bakery::Entity)
                .from(Column::BakeryId)
                .to(super::bakery::Column::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .into(),
        }
    }
}

impl Related<super::bakery::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bakery.def()
    }
}

impl Related<super::cake::Entity> for Entity {
    fn to() -> RelationDef {
        super::cakes_bakers::Relation::Cake.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::cakes_bakers::Relation::Baker.def().rev())
    }
}

pub struct BakedForCustomer;

impl Linked for BakedForCustomer {
    type FromEntity = Entity;

    type ToEntity = super::customer::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            super::cakes_bakers::Relation::Baker.def().rev(),
            super::cakes_bakers::Relation::Cake.def(),
            super::lineitem::Relation::Cake.def().rev(),
            super::lineitem::Relation::Order.def(),
            super::order::Relation::Customer.def(),
        ]
    }
}

impl ActiveModelBehavior for ActiveModel {}
