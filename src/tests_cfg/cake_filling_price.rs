use crate as sea_orm;
use crate::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn schema_name(&self) -> Option<&str> {
        Some("public")
    }

    fn table_name(&self) -> &str {
        "cake_filling_price"
    }
}

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub cake_id: i32,
    pub filling_id: i32,
    pub price: Decimal,
    #[sea_orm(ignore)]
    pub ignored_attr: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    CakeId,
    FillingId,
    Price,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    CakeId,
    FillingId,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = (i32, i32);

    fn auto_increment() -> bool {
        false
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    CakeFilling,
    // This `SelfRef` relation was created to test self joins,
    // it's not intended to be used in real life
    SelfRef,
}

impl ColumnTrait for Column {
    type EntityName = Entity;

    fn def(&self) -> ColumnDef {
        match self {
            Self::CakeId => ColumnType::Integer.def(),
            Self::FillingId => ColumnType::Integer.def(),
            Self::Price => ColumnType::Decimal(None).def(),
        }
    }
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::CakeFilling => Entity::belongs_to(super::cake_filling::Entity)
                .from((Column::CakeId, Column::FillingId))
                .to((
                    super::cake_filling::Column::CakeId,
                    super::cake_filling::Column::FillingId,
                ))
                .into(),
            Self::SelfRef => Entity::belongs_to(Entity)
                .from(Column::FillingId)
                .to(Column::CakeId)
                .into(),
        }
    }
}

impl Related<super::cake_filling::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CakeFilling.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
