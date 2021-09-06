use crate as sea_orm;
use crate::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
#[sea(schema_name = "public", table_name = "cake_filling_price")]
pub struct Entity;

#[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub cake_id: i32,
    pub filling_id: i32,
    pub price: Decimal,
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
        }
    }
}

impl Related<super::cake_filling::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CakeFilling.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
