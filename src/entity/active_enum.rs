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
    fn active_enum_string() {
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
    fn active_enum_derive_signed_integers() {
        macro_rules! test_int {
            ($ident: ident, $rs_type: expr, $db_type: expr, $col_def: ident) => {
                #[derive(Debug, PartialEq, DeriveActiveEnum)]
                #[sea_orm(rs_type = $rs_type, db_type = $db_type)]
                pub enum $ident {
                    #[sea_orm(num_value = 1)]
                    Big,
                    #[sea_orm(num_value = 0)]
                    Small,
                    #[sea_orm(num_value = -10)]
                    Negative,
                }

                assert_eq!($ident::Big.to_value(), 1);
                assert_eq!($ident::Small.to_value(), 0);
                assert_eq!($ident::Negative.to_value(), -10);

                assert_eq!($ident::try_from_value(&1).ok(), Some($ident::Big));
                assert_eq!($ident::try_from_value(&0).ok(), Some($ident::Small));
                assert_eq!($ident::try_from_value(&-10).ok(), Some($ident::Negative));
                assert_eq!(
                    $ident::try_from_value(&2).err(),
                    Some(DbErr::Query(format!(
                        "unexpected value for {} enum: 2",
                        stringify!($ident)
                    )))
                );

                assert_eq!($ident::db_type(), ColumnType::$col_def.def());
            };
        }

        test_int!(I8, "i8", "TinyInteger", TinyInteger);
        test_int!(I16, "i16", "SmallInteger", SmallInteger);
        test_int!(I32, "i32", "Integer", Integer);
        test_int!(I64, "i64", "BigInteger", BigInteger);
    }

    #[test]
    fn active_enum_derive_unsigned_integers() {
        macro_rules! test_uint {
            ($ident: ident, $rs_type: expr, $db_type: expr, $col_def: ident) => {
                #[derive(Debug, PartialEq, DeriveActiveEnum)]
                #[sea_orm(rs_type = $rs_type, db_type = $db_type)]
                pub enum $ident {
                    #[sea_orm(num_value = 1)]
                    Big,
                    #[sea_orm(num_value = 0)]
                    Small,
                }

                assert_eq!($ident::Big.to_value(), 1);
                assert_eq!($ident::Small.to_value(), 0);

                assert_eq!($ident::try_from_value(&1).ok(), Some($ident::Big));
                assert_eq!($ident::try_from_value(&0).ok(), Some($ident::Small));
                assert_eq!(
                    $ident::try_from_value(&2).err(),
                    Some(DbErr::Query(format!(
                        "unexpected value for {} enum: 2",
                        stringify!($ident)
                    )))
                );

                assert_eq!($ident::db_type(), ColumnType::$col_def.def());
            };
        }

        test_uint!(U8, "u8", "TinyInteger", TinyInteger);
        test_uint!(U16, "u16", "SmallInteger", SmallInteger);
        test_uint!(U32, "u32", "Integer", Integer);
        test_uint!(U64, "u64", "BigInteger", BigInteger);
    }
}
