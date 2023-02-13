use sea_orm::entity::prelude::*;

#[cfg(feature = "custom-migrations-table-name")]
use lazy_static::lazy_static;
#[cfg(feature = "custom-migrations-table-name")]
use std::env::var;

#[cfg(feature = "custom-migrations-table-name")]
lazy_static! {
    pub static ref MIGRATIONS_TABLE_NAME: String = match var("SEA_ORM_MIGRATIONS_TABLE_NAME") {
        Ok(value) => value,
        Err(_) => String::from("seaql_migrations"),
    };
}

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    #[cfg(feature = "custom-migrations-table-name")]
    fn table_name(&self) -> &str {
        MIGRATIONS_TABLE_NAME.as_str()
    }

    #[cfg(not(feature = "custom-migrations-table-name"))]
    fn table_name(&self) -> &str {
        "seaql_migrations"
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub version: String,
    pub applied_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    Version,
    AppliedAt,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    Version,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = String;

    fn auto_increment() -> bool {
        false
    }
}

impl ColumnTrait for Column {
    type EntityName = Entity;

    fn def(&self) -> ColumnDef {
        match self {
            Self::Version => ColumnType::String(None).def(),
            Self::AppliedAt => ColumnType::BigInteger.def(),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
