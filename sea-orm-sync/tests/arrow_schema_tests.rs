#![cfg(feature = "with-arrow")]
//! Tests for the DeriveArrowSchema macro.
//!
//! cargo t --test arrow_schema_tests --features=with-arrow

use sea_orm::ArrowSchema;
use sea_orm_arrow::arrow::datatypes::{DataType, Field, Schema, TimeUnit};

// ---------------------------------------------------------------------------
// Entities using #[sea_orm::model] (2.0 format, arrow_schema flag)
// ---------------------------------------------------------------------------

mod basic_entity {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "basic", arrow_schema)]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(column_name = "user_name")]
        pub name: String,
        pub flag: bool,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod float_entity {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "floats", arrow_schema)]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub float_val: f32,
        pub double_val: f64,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod nullable_entity {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "nullable", arrow_schema)]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub required_name: String,
        pub optional_name: Option<String>,
        pub optional_int: Option<i32>,
        #[sea_orm(nullable)]
        pub nullable_via_attr: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod string_variants_entity {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "string_variants", arrow_schema)]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub plain: String,
        #[sea_orm(column_type = "Text")]
        pub text_field: String,
        #[sea_orm(column_type = "Char(Some(10))")]
        pub char_field: String,
        #[sea_orm(column_type = "String(StringLen::N(100))")]
        pub short_string: String,
        #[sea_orm(column_type = "String(StringLen::N(50000))")]
        pub long_string: String,
        #[sea_orm(column_type = "String(StringLen::Max)")]
        pub max_string: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod comment_entity {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "comment_test", arrow_schema)]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(arrow_comment = "The user's display name")]
        pub name: String,
        #[sea_orm(nullable, arrow_comment = "Optional email address")]
        pub email: Option<String>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod timestamp_override_entity {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "timestamp_override", arrow_schema)]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(column_type = "DateTime", arrow_timestamp_unit = "Nanosecond")]
        pub nano_ts: String,
        #[sea_orm(column_type = "DateTime", arrow_timestamp_unit = "Second")]
        pub second_ts: String,
        #[sea_orm(column_type = "DateTime", arrow_timestamp_unit = "Millisecond")]
        pub milli_ts: String,
        #[sea_orm(
            column_type = "DateTime",
            arrow_timestamp_unit = "Nanosecond",
            arrow_timezone = "America/New_York"
        )]
        pub nano_with_tz: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

mod decimal_override_entity {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "decimal_override", arrow_schema)]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(
            column_type = "Decimal(Some((10, 2)))",
            arrow_precision = 20,
            arrow_scale = 4
        )]
        pub overridden: String,
        #[sea_orm(column_type = "Decimal(Some((10, 2)))", arrow_precision = 50)]
        pub large_precision: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// ---------------------------------------------------------------------------
// Entities using old format (manual Relation enum + DeriveArrowSchema)
// ---------------------------------------------------------------------------

mod all_integers_entity {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, DeriveArrowSchema)]
    #[sea_orm(table_name = "all_integers")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub tiny: i8,
        pub small: i16,
        pub big: i64,
        pub tiny_u: u8,
        pub small_u: u16,
        pub uint: u32,
        pub big_u: u64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod column_type_entity {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, DeriveArrowSchema)]
    #[sea_orm(table_name = "column_types")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(column_type = "Text")]
        pub description: String,
        #[sea_orm(column_type = "Boolean")]
        pub active: bool,
        #[sea_orm(column_type = "TinyInteger")]
        pub tiny: i8,
        #[sea_orm(column_type = "SmallInteger")]
        pub small: i16,
        #[sea_orm(column_type = "BigInteger")]
        pub big: i64,
        #[sea_orm(column_type = "TinyUnsigned")]
        pub tiny_u: u8,
        #[sea_orm(column_type = "SmallUnsigned")]
        pub small_u: u16,
        #[sea_orm(column_type = "Unsigned")]
        pub uint: u32,
        #[sea_orm(column_type = "BigUnsigned")]
        pub big_u: u64,
        #[sea_orm(column_type = "Float")]
        pub fval: f32,
        #[sea_orm(column_type = "Double")]
        pub dval: f64,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod skip_entity {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, DeriveArrowSchema)]
    #[sea_orm(table_name = "skip_test")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(column_name = "db_visible", arrow_field = "arrowVisible")]
        pub visible: String,
        #[sea_orm(arrow_skip)]
        pub internal: String,
        #[sea_orm(column_name = "db_also_visible")]
        pub also_visible: bool,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod special_types_entity {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, DeriveArrowSchema)]
    #[sea_orm(table_name = "special_types")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(column_type = "Json")]
        pub json_data: String,
        #[sea_orm(column_type = "JsonBinary")]
        pub jsonb_data: String,
        #[sea_orm(column_type = "Uuid")]
        pub uuid_val: String,
        #[sea_orm(column_type = "Binary(16)")]
        pub bin_val: Vec<u8>,
        #[sea_orm(column_type = "VarBinary(StringLen::N(256))")]
        pub varbin_val: Vec<u8>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod date_time_entity {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, DeriveArrowSchema)]
    #[sea_orm(table_name = "date_time_test")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub date_val: Date,
        pub time_val: Time,
        pub datetime_val: DateTime,
        #[sea_orm(column_type = "Timestamp")]
        pub timestamp_val: String,
        pub timestamptz_val: DateTimeWithTimeZone,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

mod decimal_column_type_entity {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, DeriveArrowSchema)]
    #[sea_orm(table_name = "decimal_column_type")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
        pub price: String,
        #[sea_orm(column_type = "Decimal(Some((20, 4)))")]
        pub amount: String,
        #[sea_orm(column_type = "Decimal(None)")]
        pub default_decimal: String,
        #[sea_orm(column_type = "Money(None)")]
        pub money_val: String,
        #[sea_orm(column_type = "Money(Some((12, 3)))")]
        pub money_custom: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_basic_schema() {
    let schema = basic_entity::Entity::arrow_schema();
    let expected = Schema::new(vec![
        Field::new("id", DataType::Int32, true),
        // column_name = "user_name" should be used instead of "name"
        Field::new("user_name", DataType::Utf8, true),
        Field::new("flag", DataType::Boolean, true),
    ]);
    assert_eq!(schema, expected);
}

#[test]
fn test_all_integer_types() {
    let schema = all_integers_entity::Entity::arrow_schema();
    let fields = schema.fields();
    assert_eq!(fields.len(), 8);
    assert_eq!(fields[0].as_ref(), &Field::new("id", DataType::Int32, true));
    assert_eq!(
        fields[1].as_ref(),
        &Field::new("tiny", DataType::Int8, true)
    );
    assert_eq!(
        fields[2].as_ref(),
        &Field::new("small", DataType::Int16, true)
    );
    assert_eq!(
        fields[3].as_ref(),
        &Field::new("big", DataType::Int64, true)
    );
    assert_eq!(
        fields[4].as_ref(),
        &Field::new("tiny_u", DataType::UInt8, true)
    );
    assert_eq!(
        fields[5].as_ref(),
        &Field::new("small_u", DataType::UInt16, true)
    );
    assert_eq!(
        fields[6].as_ref(),
        &Field::new("uint", DataType::UInt32, true)
    );
    assert_eq!(
        fields[7].as_ref(),
        &Field::new("big_u", DataType::UInt64, true)
    );
}

#[test]
fn test_float_types() {
    let schema = float_entity::Entity::arrow_schema();
    let fields = schema.fields();
    assert_eq!(
        fields[1].as_ref(),
        &Field::new("float_val", DataType::Float32, true)
    );
    assert_eq!(
        fields[2].as_ref(),
        &Field::new("double_val", DataType::Float64, true)
    );
}

#[test]
fn test_nullable_fields() {
    let schema = nullable_entity::Entity::arrow_schema();
    let fields = schema.fields();

    // required_name: not nullable
    assert_eq!(fields[1].name(), "required_name");
    assert!(fields[1].is_nullable());

    // optional_name: Option<String> -> nullable
    assert_eq!(fields[2].name(), "optional_name");
    assert!(fields[2].is_nullable());

    // optional_int: Option<i32> -> nullable
    assert_eq!(fields[3].name(), "optional_int");
    assert!(fields[3].is_nullable());

    // nullable_via_attr: #[sea_orm(nullable)] -> nullable
    assert_eq!(fields[4].name(), "nullable_via_attr");
    assert!(fields[4].is_nullable());
}

#[test]
fn test_column_type_overrides() {
    let schema = column_type_entity::Entity::arrow_schema();
    let fields = schema.fields();

    assert_eq!(fields[0].data_type(), &DataType::Int32); // id
    assert_eq!(fields[1].data_type(), &DataType::LargeUtf8); // Text
    assert_eq!(fields[2].data_type(), &DataType::Boolean); // Boolean
    assert_eq!(fields[3].data_type(), &DataType::Int8); // TinyInteger
    assert_eq!(fields[4].data_type(), &DataType::Int16); // SmallInteger
    assert_eq!(fields[5].data_type(), &DataType::Int64); // BigInteger
    assert_eq!(fields[6].data_type(), &DataType::UInt8); // TinyUnsigned
    assert_eq!(fields[7].data_type(), &DataType::UInt16); // SmallUnsigned
    assert_eq!(fields[8].data_type(), &DataType::UInt32); // Unsigned
    assert_eq!(fields[9].data_type(), &DataType::UInt64); // BigUnsigned
    assert_eq!(fields[10].data_type(), &DataType::Float32); // Float
    assert_eq!(fields[11].data_type(), &DataType::Float64); // Double
}

#[test]
fn test_string_variants() {
    let schema = string_variants_entity::Entity::arrow_schema();
    let fields = schema.fields();

    // plain String -> Utf8
    assert_eq!(fields[1].data_type(), &DataType::Utf8);
    // Text -> LargeUtf8
    assert_eq!(fields[2].data_type(), &DataType::LargeUtf8);
    // Char -> Utf8
    assert_eq!(fields[3].data_type(), &DataType::Utf8);
    // String(N(100)) where 100 <= 32767 -> Utf8
    assert_eq!(fields[4].data_type(), &DataType::Utf8);
    // String(N(50000)) where 50000 > 32767 -> LargeUtf8
    assert_eq!(fields[5].data_type(), &DataType::LargeUtf8);
    // String(Max) -> LargeUtf8
    assert_eq!(fields[6].data_type(), &DataType::LargeUtf8);
}

#[test]
fn test_arrow_skip() {
    let schema = skip_entity::Entity::arrow_schema();
    let fields = schema.fields();

    // Should have 3 fields: id, visible, also_visible (internal is skipped)
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0].name(), "id");
    // arrow_field = "arrowVisible" takes priority over column_name = "db_visible"
    assert_eq!(fields[1].name(), "arrowVisible");
    // column_name = "db_also_visible" is used when no arrow_field is set
    assert_eq!(fields[2].name(), "db_also_visible");
}

#[test]
fn test_arrow_comment_metadata() {
    let schema = comment_entity::Entity::arrow_schema();
    let fields = schema.fields();

    // name field has comment metadata
    let name_field = fields[1].as_ref();
    assert_eq!(name_field.name(), "name");
    let metadata = name_field.metadata();
    assert_eq!(
        metadata.get("comment"),
        Some(&"The user's display name".to_string())
    );

    // email field has comment metadata and is nullable
    let email_field = fields[2].as_ref();
    assert_eq!(email_field.name(), "email");
    assert!(email_field.is_nullable());
    let metadata = email_field.metadata();
    assert_eq!(
        metadata.get("comment"),
        Some(&"Optional email address".to_string())
    );
}

#[test]
fn test_special_types() {
    let schema = special_types_entity::Entity::arrow_schema();
    let fields = schema.fields();

    // Json -> Utf8
    assert_eq!(fields[1].data_type(), &DataType::Utf8);
    // JsonBinary -> Utf8
    assert_eq!(fields[2].data_type(), &DataType::Utf8);
    // Uuid -> Binary
    assert_eq!(fields[3].data_type(), &DataType::Binary);
    // Binary(16) -> Binary
    assert_eq!(fields[4].data_type(), &DataType::Binary);
    // VarBinary -> Binary
    assert_eq!(fields[5].data_type(), &DataType::Binary);
}

#[test]
fn test_date_time_column_types() {
    let schema = date_time_entity::Entity::arrow_schema();
    let fields = schema.fields();

    // Date -> Date32
    assert_eq!(fields[1].data_type(), &DataType::Date32);
    // Time -> Time64(Microsecond)
    assert_eq!(
        fields[2].data_type(),
        &DataType::Time64(TimeUnit::Microsecond)
    );
    // DateTime -> Timestamp(Microsecond, None)
    assert_eq!(
        fields[3].data_type(),
        &DataType::Timestamp(TimeUnit::Microsecond, None)
    );
    // Timestamp -> Timestamp(Microsecond, None)
    assert_eq!(
        fields[4].data_type(),
        &DataType::Timestamp(TimeUnit::Microsecond, None)
    );
    // TimestampWithTimeZone -> Timestamp(Microsecond, Some("UTC"))
    assert_eq!(
        fields[5].data_type(),
        &DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into()))
    );
}

#[test]
fn test_timestamp_unit_overrides() {
    let schema = timestamp_override_entity::Entity::arrow_schema();
    let fields = schema.fields();

    // Nanosecond override
    assert_eq!(
        fields[1].data_type(),
        &DataType::Timestamp(TimeUnit::Nanosecond, None)
    );
    // Second override
    assert_eq!(
        fields[2].data_type(),
        &DataType::Timestamp(TimeUnit::Second, None)
    );
    // Millisecond override
    assert_eq!(
        fields[3].data_type(),
        &DataType::Timestamp(TimeUnit::Millisecond, None)
    );
    // Nanosecond + timezone override
    assert_eq!(
        fields[4].data_type(),
        &DataType::Timestamp(TimeUnit::Nanosecond, Some("America/New_York".into()))
    );
}

#[test]
fn test_decimal_column_types() {
    let schema = decimal_column_type_entity::Entity::arrow_schema();
    let fields = schema.fields();

    // Decimal(Some((10, 2))) -> Decimal64(10, 2) (precision ≤ 18)
    assert_eq!(fields[1].data_type(), &DataType::Decimal64(10, 2));
    // Decimal(Some((20, 4))) -> Decimal128(20, 4) (precision > 18)
    assert_eq!(fields[2].data_type(), &DataType::Decimal128(20, 4));
    // Decimal(None) -> Decimal128(38, 10)
    assert_eq!(fields[3].data_type(), &DataType::Decimal128(38, 10));
    // Money(None) -> Decimal128(19, 4) (precision 19 > 18)
    assert_eq!(fields[4].data_type(), &DataType::Decimal128(19, 4));
    // Money(Some((12, 3))) -> Decimal128 (defaults to precision 19)
    assert!(matches!(fields[5].data_type(), DataType::Decimal128(..)));
}

#[test]
fn test_decimal_arrow_precision_override() {
    let schema = decimal_override_entity::Entity::arrow_schema();
    let fields = schema.fields();

    // arrow_precision=20, arrow_scale=4 overrides column_type's (10, 2)
    assert_eq!(fields[1].data_type(), &DataType::Decimal128(20, 4));

    // arrow_precision=50 (>38) -> Decimal256, scale falls back to column_type's 2
    assert_eq!(fields[2].data_type(), &DataType::Decimal256(50, 2));
}

#[test]
fn test_field_count_matches() {
    assert_eq!(basic_entity::Entity::arrow_schema().fields().len(), 3);
    assert_eq!(
        all_integers_entity::Entity::arrow_schema().fields().len(),
        8
    );
    assert_eq!(float_entity::Entity::arrow_schema().fields().len(), 3);
    assert_eq!(nullable_entity::Entity::arrow_schema().fields().len(), 5);
    assert_eq!(skip_entity::Entity::arrow_schema().fields().len(), 3); // 1 skipped
    assert_eq!(comment_entity::Entity::arrow_schema().fields().len(), 3);
}

#[test]
fn test_field_names_preserve_snake_case() {
    let schema = all_integers_entity::Entity::arrow_schema();
    let names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
    assert_eq!(
        names,
        vec![
            "id", "tiny", "small", "big", "tiny_u", "small_u", "uint", "big_u"
        ]
    );
}

// ---------------------------------------------------------------------------
// Chrono type tests (feature-gated)
// ---------------------------------------------------------------------------

#[cfg(feature = "with-chrono")]
mod chrono_schema_tests {
    use super::*;

    // 2.0 format
    mod chrono_entity {
        use sea_orm::entity::prelude::*;

        #[sea_orm::model]
        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "chrono_schema", arrow_schema)]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub date_val: ChronoDate,
            pub time_val: ChronoTime,
            pub datetime_val: ChronoDateTime,
            pub datetime_utc: ChronoDateTimeUtc,
            pub optional_date: Option<ChronoDate>,
        }

        impl ActiveModelBehavior for ActiveModel {}
    }

    // Old format
    mod chrono_override_entity {
        use sea_orm::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel, DeriveArrowSchema)]
        #[sea_orm(table_name = "chrono_override")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            #[sea_orm(arrow_timestamp_unit = "Nanosecond")]
            pub nano_dt: ChronoDateTime,
            #[sea_orm(arrow_timestamp_unit = "Nanosecond", arrow_timezone = "UTC")]
            pub nano_utc: ChronoDateTimeUtc,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    #[test]
    fn test_chrono_date() {
        let schema = chrono_entity::Entity::arrow_schema();
        let fields = schema.fields();

        assert_eq!(fields[1].name(), "date_val");
        assert_eq!(fields[1].data_type(), &DataType::Date32);
        assert!(fields[1].is_nullable());
    }

    #[test]
    fn test_chrono_time() {
        let schema = chrono_entity::Entity::arrow_schema();
        let fields = schema.fields();

        assert_eq!(fields[2].name(), "time_val");
        assert_eq!(
            fields[2].data_type(),
            &DataType::Time64(TimeUnit::Microsecond)
        );
    }

    #[test]
    fn test_chrono_datetime_naive() {
        let schema = chrono_entity::Entity::arrow_schema();
        let fields = schema.fields();

        assert_eq!(fields[3].name(), "datetime_val");
        assert_eq!(
            fields[3].data_type(),
            &DataType::Timestamp(TimeUnit::Microsecond, None)
        );
    }

    #[test]
    fn test_chrono_datetime_utc() {
        let schema = chrono_entity::Entity::arrow_schema();
        let fields = schema.fields();

        assert_eq!(fields[4].name(), "datetime_utc");
        assert_eq!(
            fields[4].data_type(),
            &DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into()))
        );
    }

    #[test]
    fn test_chrono_optional_nullable() {
        let schema = chrono_entity::Entity::arrow_schema();
        let fields = schema.fields();

        assert_eq!(fields[5].name(), "optional_date");
        assert!(fields[5].is_nullable());
        assert_eq!(fields[5].data_type(), &DataType::Date32);
    }

    #[test]
    fn test_chrono_timestamp_unit_override() {
        let schema = chrono_override_entity::Entity::arrow_schema();
        let fields = schema.fields();

        // ChronoDateTime with Nanosecond override, no timezone
        assert_eq!(
            fields[1].data_type(),
            &DataType::Timestamp(TimeUnit::Nanosecond, None)
        );

        // ChronoDateTimeUtc with Nanosecond + explicit UTC
        assert_eq!(
            fields[2].data_type(),
            &DataType::Timestamp(TimeUnit::Nanosecond, Some("UTC".into()))
        );
    }
}

// ---------------------------------------------------------------------------
// Decimal type tests with rust_decimal (feature-gated)
// ---------------------------------------------------------------------------

#[cfg(feature = "with-rust_decimal")]
mod rust_decimal_schema_tests {
    use super::*;

    // Old format
    mod decimal_entity {
        use sea_orm::entity::prelude::*;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel, DeriveArrowSchema)]
        #[sea_orm(table_name = "decimal_schema")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
            pub price: Decimal,
            pub inferred_decimal: Decimal,
            pub optional_decimal: Option<Decimal>,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}
    }

    #[test]
    fn test_decimal_with_column_type() {
        let schema = decimal_entity::Entity::arrow_schema();
        let fields = schema.fields();

        // precision 10 ≤ 18 → Decimal64
        assert_eq!(fields[1].data_type(), &DataType::Decimal64(10, 2));
        assert!(fields[1].is_nullable());
    }

    #[test]
    fn test_decimal_inferred_type() {
        let schema = decimal_entity::Entity::arrow_schema();
        let fields = schema.fields();

        // Inferred from Rust type Decimal -> Decimal128(38, 10) defaults
        assert_eq!(fields[2].data_type(), &DataType::Decimal128(38, 10));
    }

    #[test]
    fn test_decimal_optional_nullable() {
        let schema = decimal_entity::Entity::arrow_schema();
        let fields = schema.fields();

        assert!(fields[3].is_nullable());
        assert_eq!(fields[3].data_type(), &DataType::Decimal128(38, 10));
    }
}
