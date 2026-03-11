//! SeaORM's active enums.

use sea_orm::entity::prelude::*;

/// Tea active enum
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "tea")]
pub enum Tea {
    /// EverydayTea variant
    #[sea_orm(string_value = "EverydayTea")]
    EverydayTea,
    /// BreakfastTea variant
    #[sea_orm(string_value = "BreakfastTea")]
    BreakfastTea,
}
