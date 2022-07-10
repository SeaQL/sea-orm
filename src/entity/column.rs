use crate::{EntityName, IdenStatic, Iterable};
use sea_query::{Alias, BinOper, DynIden, Expr, SeaRc, SelectStatement, SimpleExpr, Value};
use std::str::FromStr;

/// Defines a Column for an Entity
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    pub(crate) col_type: ColumnType,
    pub(crate) null: bool,
    pub(crate) unique: bool,
    pub(crate) indexed: bool,
    pub(crate) default_value: Option<Value>,
}

/// The type of column as defined in the SQL format
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    /// `CHAR` type of specified fixed length
    Char(Option<u32>),
    /// `STRING` type for variable string length
    String(Option<u32>),
    /// `TEXT` type used for large pieces of string data and stored out of row in case size is too big
    Text,
    /// `TINYINT` useful for storing one byte of data (range of 0-255)
    TinyInteger,
    /// `SMALLINT` data type stores small whole numbers that range from –32,767 to 32,767
    SmallInteger,
    /// `INTEGER` data types hold numbers that are whole, or without a decimal point
    Integer,
    /// `BIGINT` is a 64-bit representation of an integer taking up 8 bytes of storage and
    /// ranging from -2^63 (-9,223,372,036,854,775,808) to 2^63 (9,223,372,036,854,775,807).
    BigInteger,
    /// `TINYINT UNSIGNED` data type
    TinyUnsigned,
    /// `SMALLINT UNSIGNED` data type
    SmallUnsigned,
    /// `INTEGER UNSIGNED` data type
    Unsigned,
    /// `BIGINT UNSIGNED` data type
    BigUnsigned,
    /// `FLOAT` an approximate-number data type, where values range cannot be represented exactly.
    Float,
    /// `DOUBLE` is a normal-size floating point number where the
    /// total number of digits is specified in size.
    Double,
    /// `DECIMAL` type store numbers that have fixed precision and scale
    Decimal(Option<(u32, u32)>),
    /// `DATETIME` type is used for values that contain both date and time parts.
    DateTime,
    /// `TIMESTAMP` is a temporal data type that holds the combination of date and time.
    Timestamp,
    /// `TIMESTAMP WITH TIME ZONE` (or `TIMESTAMPTZ`) data type stores 8-byte
    /// date values that include timestamp and time zone information in UTC format.
    TimestampWithTimeZone,
    /// `TIME` data type defines a time of a day based on 24-hour clock
    Time,
    /// `DATE` data type stores the calendar date
    Date,
    /// `BINARY` data types contain byte strings—a sequence of octets or bytes.
    Binary,
    /// `BOOLEAN` is the result of a comparison operator
    Boolean,
    /// `MONEY` data type handles monetary data
    Money(Option<(u32, u32)>),
    /// `JSON` represents the JavaScript Object Notation type
    Json,
    /// JSON binary format is structured in the way that permits the server to search for
    /// values within the JSON document directly by key or array index, which is very fast.
    JsonBinary,
    /// A custom implementation of a data type
    Custom(String),
    /// A Universally Unique IDentifier that is specified in  RFC 4122
    Uuid,
    /// `ENUM` data type with name and variants
    Enum(String, Vec<String>),
}

macro_rules! bind_oper {
    ( $op: ident ) => {
        #[allow(missing_docs)]
        fn $op<V>(&self, v: V) -> SimpleExpr
        where
            V: Into<Value>,
        {
            Expr::tbl(self.entity_name(), *self).$op(v)
        }
    };
}

macro_rules! bind_oper_with_enum_casting {
    ( $op: ident, $bin_op: ident ) => {
        #[allow(missing_docs)]
        fn $op<V>(&self, v: V) -> SimpleExpr
        where
            V: Into<Value>,
        {
            let val = Expr::val(v);
            let col_def = self.def();
            let col_type = col_def.get_column_type();
            let expr = match col_type.get_enum_name() {
                Some(enum_name) => val.as_enum(Alias::new(enum_name)),
                None => val.into(),
            };
            Expr::tbl(self.entity_name(), *self).binary(BinOper::$bin_op, expr)
        }
    };
}

macro_rules! bind_func_no_params {
    ( $func: ident ) => {
        /// See also SeaQuery's method with same name.
        fn $func(&self) -> SimpleExpr {
            Expr::tbl(self.entity_name(), *self).$func()
        }
    };
}

macro_rules! bind_vec_func {
    ( $func: ident ) => {
        #[allow(missing_docs)]
        #[allow(clippy::wrong_self_convention)]
        fn $func<V, I>(&self, v: I) -> SimpleExpr
        where
            V: Into<Value>,
            I: IntoIterator<Item = V>,
        {
            Expr::tbl(self.entity_name(), *self).$func(v)
        }
    };
}

macro_rules! bind_subquery_func {
    ( $func: ident ) => {
        #[allow(clippy::wrong_self_convention)]
        #[allow(missing_docs)]
        fn $func(&self, s: SelectStatement) -> SimpleExpr {
            Expr::tbl(self.entity_name(), *self).$func(s)
        }
    };
}

// LINT: when the operand value does not match column type
/// Wrapper of the identically named method in [`sea_query::Expr`]
pub trait ColumnTrait: IdenStatic + Iterable + FromStr {
    #[allow(missing_docs)]
    type EntityName: EntityName;

    /// Define a column for an Entity
    fn def(&self) -> ColumnDef;

    /// Get the name of the entity the column belongs to
    fn entity_name(&self) -> DynIden {
        SeaRc::new(Self::EntityName::default()) as DynIden
    }

    /// get the name of the entity the column belongs to
    fn as_column_ref(&self) -> (DynIden, DynIden) {
        (self.entity_name(), SeaRc::new(*self) as DynIden)
    }

    bind_oper_with_enum_casting!(eq, Equal);
    bind_oper_with_enum_casting!(ne, NotEqual);
    bind_oper!(gt);
    bind_oper!(gte);
    bind_oper!(lt);
    bind_oper!(lte);

    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Id.between(2, 3))
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` BETWEEN 2 AND 3"
    /// );
    /// ```
    fn between<V>(&self, a: V, b: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        Expr::tbl(self.entity_name(), *self).between(a, b)
    }

    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Id.not_between(2, 3))
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` NOT BETWEEN 2 AND 3"
    /// );
    /// ```
    fn not_between<V>(&self, a: V, b: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        Expr::tbl(self.entity_name(), *self).not_between(a, b)
    }

    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Name.like("cheese"))
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE 'cheese'"
    /// );
    /// ```
    fn like(&self, s: &str) -> SimpleExpr {
        Expr::tbl(self.entity_name(), *self).like(s)
    }

    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Name.not_like("cheese"))
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` NOT LIKE 'cheese'"
    /// );
    /// ```
    fn not_like(&self, s: &str) -> SimpleExpr {
        Expr::tbl(self.entity_name(), *self).not_like(s)
    }

    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Name.starts_with("cheese"))
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE 'cheese%'"
    /// );
    /// ```
    fn starts_with(&self, s: &str) -> SimpleExpr {
        let pattern = format!("{}%", s);
        Expr::tbl(self.entity_name(), *self).like(pattern)
    }

    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Name.ends_with("cheese"))
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE '%cheese'"
    /// );
    /// ```
    fn ends_with(&self, s: &str) -> SimpleExpr {
        let pattern = format!("%{}", s);
        Expr::tbl(self.entity_name(), *self).like(pattern)
    }

    /// ```
    /// use sea_orm::{entity::*, query::*, tests_cfg::cake, DbBackend};
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .filter(cake::Column::Name.contains("cheese"))
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     "SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE '%cheese%'"
    /// );
    /// ```
    fn contains(&self, s: &str) -> SimpleExpr {
        let pattern = format!("%{}%", s);
        Expr::tbl(self.entity_name(), *self).like(pattern)
    }

    bind_func_no_params!(max);
    bind_func_no_params!(min);
    bind_func_no_params!(sum);
    bind_func_no_params!(count);
    bind_func_no_params!(is_null);
    bind_func_no_params!(is_not_null);

    /// Perform an operation if the column is null
    fn if_null<V>(&self, v: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        Expr::tbl(self.entity_name(), *self).if_null(v)
    }

    bind_vec_func!(is_in);
    bind_vec_func!(is_not_in);

    bind_subquery_func!(in_subquery);
    bind_subquery_func!(not_in_subquery);
}

impl ColumnType {
    /// instantiate a new [ColumnDef]
    pub fn def(self) -> ColumnDef {
        ColumnDef {
            col_type: self,
            null: false,
            unique: false,
            indexed: false,
            default_value: None,
        }
    }

    pub(crate) fn get_enum_name(&self) -> Option<&String> {
        match self {
            ColumnType::Enum(s, _) => Some(s),
            _ => None,
        }
    }
}

impl ColumnDef {
    /// Marks the column as `UNIQUE`
    pub fn unique(mut self) -> Self {
        self.unique = true;
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
        self.default_value = Some(value.into());
        self
    }

    /// Get [ColumnType] as reference
    pub fn get_column_type(&self) -> &ColumnType {
        &self.col_type
    }
}

impl From<ColumnType> for sea_query::ColumnType {
    fn from(col: ColumnType) -> Self {
        match col {
            ColumnType::Char(s) => sea_query::ColumnType::Char(s),
            ColumnType::String(s) => sea_query::ColumnType::String(s),
            ColumnType::Text => sea_query::ColumnType::Text,
            ColumnType::TinyInteger => sea_query::ColumnType::TinyInteger(None),
            ColumnType::SmallInteger => sea_query::ColumnType::SmallInteger(None),
            ColumnType::Integer => sea_query::ColumnType::Integer(None),
            ColumnType::BigInteger => sea_query::ColumnType::BigInteger(None),
            ColumnType::TinyUnsigned => sea_query::ColumnType::TinyUnsigned(None),
            ColumnType::SmallUnsigned => sea_query::ColumnType::SmallUnsigned(None),
            ColumnType::Unsigned => sea_query::ColumnType::Unsigned(None),
            ColumnType::BigUnsigned => sea_query::ColumnType::BigUnsigned(None),
            ColumnType::Float => sea_query::ColumnType::Float(None),
            ColumnType::Double => sea_query::ColumnType::Double(None),
            ColumnType::Decimal(s) => sea_query::ColumnType::Decimal(s),
            ColumnType::DateTime => sea_query::ColumnType::DateTime(None),
            ColumnType::Timestamp => sea_query::ColumnType::Timestamp(None),
            ColumnType::TimestampWithTimeZone => sea_query::ColumnType::TimestampWithTimeZone(None),
            ColumnType::Time => sea_query::ColumnType::Time(None),
            ColumnType::Date => sea_query::ColumnType::Date,
            ColumnType::Binary => sea_query::ColumnType::Binary(sea_query::BlobSize::Blob(None)),
            ColumnType::Boolean => sea_query::ColumnType::Boolean,
            ColumnType::Money(s) => sea_query::ColumnType::Money(s),
            ColumnType::Json => sea_query::ColumnType::Json,
            ColumnType::JsonBinary => sea_query::ColumnType::JsonBinary,
            ColumnType::Custom(s) => {
                sea_query::ColumnType::Custom(sea_query::SeaRc::new(sea_query::Alias::new(&s)))
            }
            ColumnType::Uuid => sea_query::ColumnType::Uuid,
            ColumnType::Enum(name, variants) => sea_query::ColumnType::Enum(name, variants),
        }
    }
}

impl From<sea_query::ColumnType> for ColumnType {
    fn from(col_type: sea_query::ColumnType) -> Self {
        #[allow(unreachable_patterns)]
        match col_type {
            sea_query::ColumnType::Char(s) => Self::Char(s),
            sea_query::ColumnType::String(s) => Self::String(s),
            sea_query::ColumnType::Text => Self::Text,
            sea_query::ColumnType::TinyInteger(_) => Self::TinyInteger,
            sea_query::ColumnType::SmallInteger(_) => Self::SmallInteger,
            sea_query::ColumnType::Integer(_) => Self::Integer,
            sea_query::ColumnType::BigInteger(_) => Self::BigInteger,
            sea_query::ColumnType::TinyUnsigned(_) => Self::TinyUnsigned,
            sea_query::ColumnType::SmallUnsigned(_) => Self::SmallUnsigned,
            sea_query::ColumnType::Unsigned(_) => Self::Unsigned,
            sea_query::ColumnType::BigUnsigned(_) => Self::BigUnsigned,
            sea_query::ColumnType::Float(_) => Self::Float,
            sea_query::ColumnType::Double(_) => Self::Double,
            sea_query::ColumnType::Decimal(s) => Self::Decimal(s),
            sea_query::ColumnType::DateTime(_) => Self::DateTime,
            sea_query::ColumnType::Timestamp(_) => Self::Timestamp,
            sea_query::ColumnType::TimestampWithTimeZone(_) => Self::TimestampWithTimeZone,
            sea_query::ColumnType::Time(_) => Self::Time,
            sea_query::ColumnType::Date => Self::Date,
            sea_query::ColumnType::Binary(_) => Self::Binary,
            sea_query::ColumnType::Boolean => Self::Boolean,
            sea_query::ColumnType::Money(s) => Self::Money(s),
            sea_query::ColumnType::Json => Self::Json,
            sea_query::ColumnType::JsonBinary => Self::JsonBinary,
            sea_query::ColumnType::Custom(s) => Self::Custom(s.to_string()),
            sea_query::ColumnType::Uuid => Self::Uuid,
            sea_query::ColumnType::Enum(name, variants) => Self::Enum(name, variants),
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        tests_cfg::*, ColumnTrait, Condition, DbBackend, EntityTrait, QueryFilter, QueryTrait,
    };
    use sea_query::Query;

    #[test]
    fn test_in_subquery_1() {
        assert_eq!(
            cake::Entity::find()
                .filter(
                    Condition::any().add(
                        cake::Column::Id.in_subquery(
                            Query::select()
                                .expr(cake::Column::Id.max())
                                .from(cake::Entity)
                                .to_owned()
                        )
                    )
                )
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `cake`.`id`, `cake`.`name` FROM `cake`",
                "WHERE `cake`.`id` IN (SELECT MAX(`cake`.`id`) FROM `cake`)",
            ]
            .join(" ")
        );
    }

    #[test]
    fn test_in_subquery_2() {
        assert_eq!(
            cake::Entity::find()
                .filter(
                    Condition::any().add(
                        cake::Column::Id.in_subquery(
                            Query::select()
                                .column(cake_filling::Column::CakeId)
                                .from(cake_filling::Entity)
                                .to_owned()
                        )
                    )
                )
                .build(DbBackend::MySql)
                .to_string(),
            [
                "SELECT `cake`.`id`, `cake`.`name` FROM `cake`",
                "WHERE `cake`.`id` IN (SELECT `cake_id` FROM `cake_filling`)",
            ]
            .join(" ")
        );
    }

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
        use crate::entity::*;

        mod hello {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
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
    }

    #[test]
    #[cfg(feature = "macros")]
    fn column_name_1() {
        use sea_query::Iden;

        mod hello {
            use crate as sea_orm;
            use crate::entity::prelude::*;

            #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
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
                fn table_name(&self) -> &str {
                    "hello"
                }
            }

            #[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
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

            #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
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
                fn table_name(&self) -> &str {
                    "hello"
                }
            }

            #[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
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

            #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
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
                fn table_name(&self) -> &str {
                    "hello"
                }
            }

            #[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
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

            #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
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
}
