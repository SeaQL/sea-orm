use crate::{ColumnDef, DbErr, TryGetable};
use sea_query::{Nullable, Value, ValueType};

pub trait ActiveEnum: Sized {
    type Value: Into<Value> + ValueType + Nullable + TryGetable;

    fn to_value(&self) -> Self::Value;

    fn try_from_value(v: &Self::Value) -> Result<Self, DbErr>;

    fn db_type() -> ColumnDef;
}

#[cfg(test)]
mod tests {
    use crate as sea_orm;
    use crate::{entity::prelude::*, *};
    use pretty_assertions::assert_eq;

    #[test]
    fn active_enum_1() {
        #[derive(Debug, PartialEq)]
        pub enum Category {
            Big,
            Small,
        }

        impl ActiveEnum for Category {
            type Value = String;

            fn to_value(&self) -> Self::Value {
                match self {
                    Self::Big => "B",
                    Self::Small => "S",
                }
                .to_owned()
            }

            fn try_from_value(v: &Self::Value) -> Result<Self, DbErr> {
                match v.as_ref() {
                    "B" => Ok(Self::Big),
                    "S" => Ok(Self::Small),
                    _ => Err(DbErr::Query(format!(
                        "unexpected value for Category enum: {}",
                        v
                    ))),
                }
            }

            fn db_type() -> ColumnDef {
                ColumnType::String(Some(1)).def()
            }
        }

        #[derive(Debug, PartialEq, DeriveActiveEnum)]
        #[sea_orm(rs_type = "String", db_type = "String(Some(1))")]
        pub enum DeriveCategory {
            #[sea_orm(string_value = "B")]
            Big,
            #[sea_orm(string_value = "S")]
            Small,
        }

        assert_eq!(Category::Big.to_value(), "B".to_owned());
        assert_eq!(Category::Small.to_value(), "S".to_owned());
        assert_eq!(DeriveCategory::Big.to_value(), "B".to_owned());
        assert_eq!(DeriveCategory::Small.to_value(), "S".to_owned());

        assert_eq!(
            Category::try_from_value(&"A".to_owned()).err(),
            Some(DbErr::Query(
                "unexpected value for Category enum: A".to_owned()
            ))
        );
        assert_eq!(
            Category::try_from_value(&"B".to_owned()).ok(),
            Some(Category::Big)
        );
        assert_eq!(
            Category::try_from_value(&"S".to_owned()).ok(),
            Some(Category::Small)
        );
        assert_eq!(
            DeriveCategory::try_from_value(&"A".to_owned()).err(),
            Some(DbErr::Query(
                "unexpected value for DeriveCategory enum: A".to_owned()
            ))
        );
        assert_eq!(
            DeriveCategory::try_from_value(&"B".to_owned()).ok(),
            Some(DeriveCategory::Big)
        );
        assert_eq!(
            DeriveCategory::try_from_value(&"S".to_owned()).ok(),
            Some(DeriveCategory::Small)
        );

        assert_eq!(Category::db_type(), ColumnType::String(Some(1)).def());
        assert_eq!(DeriveCategory::db_type(), ColumnType::String(Some(1)).def());
    }

    #[test]
    fn active_enum_2() {
        #[derive(Debug, PartialEq)]
        pub enum Category {
            Big,
            Small,
        }

        impl ActiveEnum for Category {
            type Value = i32; // FIXME: only support i32 for now

            fn to_value(&self) -> Self::Value {
                match self {
                    Self::Big => 1,
                    Self::Small => 0,
                }
                .to_owned()
            }

            fn try_from_value(v: &Self::Value) -> Result<Self, DbErr> {
                match v {
                    1 => Ok(Self::Big),
                    0 => Ok(Self::Small),
                    _ => Err(DbErr::Query(format!(
                        "unexpected value for Category enum: {}",
                        v
                    ))),
                }
            }

            fn db_type() -> ColumnDef {
                ColumnType::Integer.def()
            }
        }

        #[derive(Debug, PartialEq, DeriveActiveEnum)]
        #[sea_orm(rs_type = "i32", db_type = "Integer")]
        pub enum DeriveCategory {
            #[sea_orm(num_value = 1)]
            Big,
            #[sea_orm(num_value = 0)]
            Small,
        }

        assert_eq!(Category::Big.to_value(), 1);
        assert_eq!(Category::Small.to_value(), 0);
        assert_eq!(DeriveCategory::Big.to_value(), 1);
        assert_eq!(DeriveCategory::Small.to_value(), 0);

        assert_eq!(
            Category::try_from_value(&2).err(),
            Some(DbErr::Query(
                "unexpected value for Category enum: 2".to_owned()
            ))
        );
        assert_eq!(
            Category::try_from_value(&1).ok(),
            Some(Category::Big)
        );
        assert_eq!(
            Category::try_from_value(&0).ok(),
            Some(Category::Small)
        );
        assert_eq!(
            DeriveCategory::try_from_value(&2).err(),
            Some(DbErr::Query(
                "unexpected value for DeriveCategory enum: 2".to_owned()
            ))
        );
        assert_eq!(
            DeriveCategory::try_from_value(&1).ok(),
            Some(DeriveCategory::Big)
        );
        assert_eq!(
            DeriveCategory::try_from_value(&0).ok(),
            Some(DeriveCategory::Small)
        );

        assert_eq!(Category::db_type(), ColumnType::Integer.def());
        assert_eq!(DeriveCategory::db_type(), ColumnType::Integer.def());
    }
}
