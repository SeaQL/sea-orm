use sea_query::{SimpleExpr, Value};

// The original `sea_orm::ColumnType` enum was dropped since 0.11.0
// It was replaced by `sea_query::ColumnType`, we reexport it here to keep the `ColumnType` symbol
pub use sea_query::ColumnType;

/// Defines a Column for an Entity
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    pub(crate) col_type: ColumnType,
    pub(crate) null: bool,
    pub(crate) unique: bool,
    pub(crate) indexed: bool,
    pub(crate) default: Option<SimpleExpr>,
    pub(crate) comment: Option<String>,
    pub(crate) unique_key: Option<String>,
}

impl ColumnDef {
    /// Marks the column as `UNIQUE`
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// This column belongs to a unique key
    pub fn unique_key(mut self, key: &str) -> Self {
        self.unique_key = Some(key.into());
        self
    }

    /// Set column comment
    pub fn comment(mut self, v: &str) -> Self {
        self.comment = Some(v.into());
        self
    }

    /// Mark the column as nullable
    pub fn null(self) -> Self {
        self.nullable()
    }

    /// Mark the column as nullable
    pub fn nullable(mut self) -> Self {
        self.null = true;
        self
    }

    /// Set the `indexed` field  to `true`
    pub fn indexed(mut self) -> Self {
        self.indexed = true;
        self
    }

    /// Set the default value
    pub fn default_value<T>(mut self, value: T) -> Self
    where
        T: Into<Value>,
    {
        self.default = Some(value.into().into());
        self
    }

    /// Set the default value or expression of a column
    pub fn default<T>(mut self, default: T) -> Self
    where
        T: Into<SimpleExpr>,
    {
        self.default = Some(default.into());
        self
    }

    /// Get [ColumnType] as reference
    pub fn get_column_type(&self) -> &ColumnType {
        &self.col_type
    }

    /// Get [Option<SimpleExpr>] as reference
    pub fn get_column_default(&self) -> Option<&SimpleExpr> {
        self.default.as_ref()
    }

    /// Returns true if the column is nullable
    pub fn is_null(&self) -> bool {
        self.null
    }

    /// Returns true if the column is unique
    pub fn is_unique(&self) -> bool {
        self.unique
    }
}

#[cfg(test)]
mod tests {
    use crate::tests_cfg::*;

    #[test]
    fn test_col_from_str() {
        use std::str::FromStr;

        assert!(matches!(
            fruit::Column::from_str("id"),
            Ok(fruit::Column::Id)
        ));
        assert!(matches!(
            fruit::Column::from_str("name"),
            Ok(fruit::Column::Name)
        ));
        assert!(matches!(
            fruit::Column::from_str("cake_id"),
            Ok(fruit::Column::CakeId)
        ));
        assert!(matches!(
            fruit::Column::from_str("cakeId"),
            Ok(fruit::Column::CakeId)
        ));
        assert!(matches!(
            fruit::Column::from_str("does_not_exist"),
            Err(crate::ColumnFromStrErr(_))
        ));
    }

    #[test]
    #[cfg(feature = "macros")]
    fn entity_model_column_1() {
        use crate::prelude::*;

        mod hello {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
            #[sea_orm(table_name = "hello")]
            pub struct Model {
                #[sea_orm(primary_key)]
                pub id: i32,
                pub one: i32,
                #[sea_orm(unique)]
                pub two: i8,
                #[sea_orm(indexed)]
                pub three: i16,
                #[sea_orm(nullable)]
                pub four: i32,
                #[sea_orm(unique, indexed, nullable)]
                pub five: i64,
                #[sea_orm(unique)]
                pub six: u8,
                #[sea_orm(indexed)]
                pub seven: u16,
                #[sea_orm(nullable)]
                pub eight: u32,
                #[sea_orm(unique, indexed, nullable)]
                pub nine: u64,
                #[sea_orm(default_expr = "Expr::current_timestamp()")]
                pub ten: DateTimeUtc,
                #[sea_orm(default_value = 7)]
                pub eleven: u8,
                #[sea_orm(default_value = "twelve_value")]
                pub twelve: String,
                #[sea_orm(default_expr = "\"twelve_value\"")]
                pub twelve_two: String,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }

        assert_eq!(hello::Column::One.def(), ColumnType::Integer.def());
        assert_eq!(
            hello::Column::Two.def(),
            ColumnType::TinyInteger.def().unique()
        );
        assert_eq!(
            hello::Column::Three.def(),
            ColumnType::SmallInteger.def().indexed()
        );
        assert_eq!(
            hello::Column::Four.def(),
            ColumnType::Integer.def().nullable()
        );
        assert_eq!(
            hello::Column::Five.def(),
            ColumnType::BigInteger.def().unique().indexed().nullable()
        );
        assert_eq!(
            hello::Column::Six.def(),
            ColumnType::TinyUnsigned.def().unique()
        );
        assert_eq!(
            hello::Column::Seven.def(),
            ColumnType::SmallUnsigned.def().indexed()
        );
        assert_eq!(
            hello::Column::Eight.def(),
            ColumnType::Unsigned.def().nullable()
        );
        assert_eq!(
            hello::Column::Nine.def(),
            ColumnType::BigUnsigned.def().unique().indexed().nullable()
        );
        assert_eq!(
            hello::Column::Ten.def(),
            ColumnType::TimestampWithTimeZone
                .def()
                .default(Expr::current_timestamp())
        );
        assert_eq!(
            hello::Column::Eleven.def(),
            ColumnType::TinyUnsigned.def().default(7)
        );
        assert_eq!(
            hello::Column::Twelve.def(),
            ColumnType::String(StringLen::None)
                .def()
                .default("twelve_value")
        );
        assert_eq!(
            hello::Column::TwelveTwo.def(),
            ColumnType::String(StringLen::None)
                .def()
                .default("twelve_value")
        );
    }

    #[test]
    #[cfg(feature = "macros")]
    fn column_name_1() {
        use sea_query::Iden;

        mod hello {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
            #[sea_orm(table_name = "hello")]
            pub struct Model {
                #[sea_orm(primary_key)]
                pub id: i32,
                #[sea_orm(column_name = "ONE")]
                pub one: i32,
                pub two: i32,
                #[sea_orm(column_name = "3")]
                pub three: i32,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }

        assert_eq!(hello::Column::One.to_string().as_str(), "ONE");
        assert_eq!(hello::Column::Two.to_string().as_str(), "two");
        assert_eq!(hello::Column::Three.to_string().as_str(), "3");
    }

    #[test]
    #[cfg(feature = "macros")]
    fn column_name_2() {
        use sea_query::Iden;

        mod hello {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Copy, Clone, Default, Debug, DeriveEntity)]
            pub struct Entity;

            impl EntityName for Entity {
                fn table_name(&self) -> &'static str {
                    "hello"
                }
            }

            #[derive(Clone, Debug, PartialEq, Eq, DeriveModel, DeriveActiveModel)]
            pub struct Model {
                pub id: i32,
                pub one: i32,
                pub two: i32,
                pub three: i32,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
            pub enum Column {
                Id,
                #[sea_orm(column_name = "ONE")]
                One,
                Two,
                #[sea_orm(column_name = "3")]
                Three,
            }

            impl ColumnTrait for Column {
                type EntityName = Entity;

                fn def(&self) -> ColumnDef {
                    match self {
                        Column::Id => ColumnType::Integer.def(),
                        Column::One => ColumnType::Integer.def(),
                        Column::Two => ColumnType::Integer.def(),
                        Column::Three => ColumnType::Integer.def(),
                    }
                }
            }

            #[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
            pub enum PrimaryKey {
                Id,
            }

            impl PrimaryKeyTrait for PrimaryKey {
                type ValueType = i32;

                fn auto_increment() -> bool {
                    true
                }
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }

        assert_eq!(hello::Column::One.to_string().as_str(), "ONE");
        assert_eq!(hello::Column::Two.to_string().as_str(), "two");
        assert_eq!(hello::Column::Three.to_string().as_str(), "3");
    }

    #[test]
    #[cfg(feature = "macros")]
    fn enum_name_1() {
        use sea_query::Iden;

        mod hello {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
            #[sea_orm(table_name = "hello")]
            pub struct Model {
                #[sea_orm(primary_key)]
                pub id: i32,
                #[sea_orm(enum_name = "One1")]
                pub one: i32,
                pub two: i32,
                #[sea_orm(enum_name = "Three3")]
                pub three: i32,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }

        assert_eq!(hello::Column::One1.to_string().as_str(), "one1");
        assert_eq!(hello::Column::Two.to_string().as_str(), "two");
        assert_eq!(hello::Column::Three3.to_string().as_str(), "three3");
    }

    #[test]
    #[cfg(feature = "macros")]
    fn enum_name_2() {
        use sea_query::Iden;

        mod hello {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Copy, Clone, Default, Debug, DeriveEntity)]
            pub struct Entity;

            impl EntityName for Entity {
                fn table_name(&self) -> &'static str {
                    "hello"
                }
            }

            #[derive(Clone, Debug, PartialEq, Eq, DeriveModel, DeriveActiveModel)]
            pub struct Model {
                pub id: i32,
                #[sea_orm(enum_name = "One1")]
                pub one: i32,
                pub two: i32,
                #[sea_orm(enum_name = "Three3")]
                pub three: i32,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
            pub enum Column {
                Id,
                One1,
                Two,
                Three3,
            }

            impl ColumnTrait for Column {
                type EntityName = Entity;

                fn def(&self) -> ColumnDef {
                    match self {
                        Column::Id => ColumnType::Integer.def(),
                        Column::One1 => ColumnType::Integer.def(),
                        Column::Two => ColumnType::Integer.def(),
                        Column::Three3 => ColumnType::Integer.def(),
                    }
                }
            }

            #[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
            pub enum PrimaryKey {
                Id,
            }

            impl PrimaryKeyTrait for PrimaryKey {
                type ValueType = i32;

                fn auto_increment() -> bool {
                    true
                }
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }

        assert_eq!(hello::Column::One1.to_string().as_str(), "one1");
        assert_eq!(hello::Column::Two.to_string().as_str(), "two");
        assert_eq!(hello::Column::Three3.to_string().as_str(), "three3");
    }

    #[test]
    #[cfg(feature = "macros")]
    fn column_name_enum_name_1() {
        use sea_query::Iden;

        #[allow(clippy::enum_variant_names)]
        mod hello {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
            #[sea_orm(table_name = "hello")]
            pub struct Model {
                #[sea_orm(primary_key, column_name = "ID", enum_name = "IdentityColumn")]
                pub id: i32,
                #[sea_orm(column_name = "ONE", enum_name = "One1")]
                pub one: i32,
                pub two: i32,
                #[sea_orm(column_name = "THREE", enum_name = "Three3")]
                pub three: i32,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }

        assert_eq!(hello::Column::IdentityColumn.to_string().as_str(), "ID");
        assert_eq!(hello::Column::One1.to_string().as_str(), "ONE");
        assert_eq!(hello::Column::Two.to_string().as_str(), "two");
        assert_eq!(hello::Column::Three3.to_string().as_str(), "THREE");
    }

    #[test]
    #[cfg(feature = "macros")]
    fn column_name_enum_name_2() {
        use sea_query::Iden;

        mod hello {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Copy, Clone, Default, Debug, DeriveEntity)]
            pub struct Entity;

            impl EntityName for Entity {
                fn table_name(&self) -> &'static str {
                    "hello"
                }
            }

            #[derive(Clone, Debug, PartialEq, Eq, DeriveModel, DeriveActiveModel)]
            pub struct Model {
                #[sea_orm(enum_name = "IdentityCol")]
                pub id: i32,
                #[sea_orm(enum_name = "One1")]
                pub one: i32,
                pub two: i32,
                #[sea_orm(enum_name = "Three3")]
                pub three: i32,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
            pub enum Column {
                #[sea_orm(column_name = "ID")]
                IdentityCol,
                #[sea_orm(column_name = "ONE")]
                One1,
                Two,
                #[sea_orm(column_name = "THREE")]
                Three3,
            }

            impl ColumnTrait for Column {
                type EntityName = Entity;

                fn def(&self) -> ColumnDef {
                    match self {
                        Column::IdentityCol => ColumnType::Integer.def(),
                        Column::One1 => ColumnType::Integer.def(),
                        Column::Two => ColumnType::Integer.def(),
                        Column::Three3 => ColumnType::Integer.def(),
                    }
                }
            }

            #[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
            pub enum PrimaryKey {
                IdentityCol,
            }

            impl PrimaryKeyTrait for PrimaryKey {
                type ValueType = i32;

                fn auto_increment() -> bool {
                    true
                }
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }

        assert_eq!(hello::Column::IdentityCol.to_string().as_str(), "ID");
        assert_eq!(hello::Column::One1.to_string().as_str(), "ONE");
        assert_eq!(hello::Column::Two.to_string().as_str(), "two");
        assert_eq!(hello::Column::Three3.to_string().as_str(), "THREE");
    }

    #[test]
    #[cfg(feature = "macros")]
    fn column_name_enum_name_3() {
        use sea_query::Iden;

        mod my_entity {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
            #[sea_orm(table_name = "my_entity")]
            pub struct Model {
                #[sea_orm(primary_key, enum_name = "IdentityColumn", column_name = "id")]
                pub id: i32,
                #[sea_orm(column_name = "type")]
                pub type_: String,
            }

            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            pub enum Relation {}

            impl ActiveModelBehavior for ActiveModel {}
        }

        assert_eq!(my_entity::Column::IdentityColumn.to_string().as_str(), "id");
        assert_eq!(my_entity::Column::Type.to_string().as_str(), "type");
    }

    #[test]
    #[cfg(feature = "macros")]
    fn column_def_unique_key() {
        use crate as sea_orm;
        use crate::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "my_entity")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            #[sea_orm(column_name = "my_a", unique_key = "my_unique")]
            pub a: String,
            #[sea_orm(unique_key = "my_unique")]
            pub b: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}

        assert_eq!(
            Column::A.def(),
            ColumnType::string(None).def().unique_key("my_unique")
        );
        assert_eq!(
            Column::B.def(),
            ColumnType::string(None).def().unique_key("my_unique")
        );
    }
}
