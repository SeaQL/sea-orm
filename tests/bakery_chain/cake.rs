use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "cake"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub id: i32,
    pub name: String,
    pub price: f32,
    pub bakery_id: Option<i32>,
    pub lineitem_id: Option<i32>,
    pub best_before: String,
    pub produced_at: String,
    pub gluten_free: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Id,
    Name,
    Price,
    BakeryId,
    LineitemId,
    BestBefore,
    ProducedAt,
    GlutenFree,
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
    Lineitem,
}

impl ColumnTrait for Column {
    type EntityName = Entity;

    fn def(&self) -> ColumnDef {
        match self {
            Self::Id => ColumnType::Integer.def(),
            Self::Name => ColumnType::String(None).def(),
            Self::Price => ColumnType::Money(Some((19, 4))).def(),
            Self::BakeryId => ColumnType::Integer.def(),
            Self::LineitemId => ColumnType::Integer.def(),
            Self::BestBefore => ColumnType::Date.def(),
            Self::ProducedAt => ColumnType::Timestamp.def(),
            Self::GlutenFree => ColumnType::Boolean.def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Bakery => Entity::belongs_to(super::bakery::Entity)
                .from(Column::BakeryId)
                .to(super::bakery::Column::Id)
                .into(),
            Self::Lineitem => Entity::belongs_to(super::lineitem::Entity)
                .from(Column::LineitemId)
                .to(super::lineitem::Column::Id)
                .into(),
        }
    }
}

impl Related<super::bakery::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bakery.def()
    }
}

impl Related<super::baker::Entity> for Entity {
    fn to() -> RelationDef {
        super::cakes_bakers::Relation::Baker.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::cakes_bakers::Relation::Cake.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
