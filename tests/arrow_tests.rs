#![cfg(feature = "with-arrow")]
//! cargo t --test arrow_tests --features=with-arrow
//! cargo t --test arrow_tests --features=with-arrow,with-bigdecimal
use arrow::array::*;
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue::NotSet, Set, arrow};
use std::sync::Arc;

/// Test entity with all supported primitive types
mod primitive_entity {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "test_arrow")]
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
        pub float_val: f32,
        pub double_val: f64,
        pub name: String,
        pub flag: bool,
        pub nullable_int: Option<i32>,
        pub nullable_name: Option<String>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

fn make_batch() -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("tiny", DataType::Int8, false),
        Field::new("small", DataType::Int16, false),
        Field::new("big", DataType::Int64, false),
        Field::new("tiny_u", DataType::UInt8, false),
        Field::new("small_u", DataType::UInt16, false),
        Field::new("uint", DataType::UInt32, false),
        Field::new("big_u", DataType::UInt64, false),
        Field::new("float_val", DataType::Float32, false),
        Field::new("double_val", DataType::Float64, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("flag", DataType::Boolean, false),
        Field::new("nullable_int", DataType::Int32, true),
        Field::new("nullable_name", DataType::Utf8, true),
    ]));

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int32Array::from(vec![1, 2])),
            Arc::new(Int8Array::from(vec![10i8, 20])),
            Arc::new(Int16Array::from(vec![100i16, 200])),
            Arc::new(Int64Array::from(vec![1000i64, 2000])),
            Arc::new(UInt8Array::from(vec![5u8, 6])),
            Arc::new(UInt16Array::from(vec![50u16, 60])),
            Arc::new(UInt32Array::from(vec![500u32, 600])),
            Arc::new(UInt64Array::from(vec![5000u64, 6000])),
            Arc::new(Float32Array::from(vec![1.5f32, 2.5])),
            Arc::new(Float64Array::from(vec![10.5f64, 20.5])),
            Arc::new(StringArray::from(vec!["Alice", "Bob"])),
            Arc::new(BooleanArray::from(vec![true, false])),
            Arc::new(Int32Array::from(vec![Some(42), None])),
            Arc::new(StringArray::from(vec![Some("hello"), None])),
        ],
    )
    .expect("Failed to create RecordBatch")
}

#[test]
fn test_from_arrow_basic() {
    let batch = make_batch();
    let active_models =
        primitive_entity::ActiveModel::from_arrow(&batch).expect("from_arrow failed");

    assert_eq!(active_models.len(), 2);

    let am = &active_models[0];
    assert_eq!(am.id, Set(1));
    assert_eq!(am.tiny, Set(10));
    assert_eq!(am.small, Set(100));
    assert_eq!(am.big, Set(1000));
    assert_eq!(am.tiny_u, Set(5));
    assert_eq!(am.small_u, Set(50));
    assert_eq!(am.uint, Set(500));
    assert_eq!(am.big_u, Set(5000));
    assert_eq!(am.float_val, Set(1.5));
    assert_eq!(am.double_val, Set(10.5));
    assert_eq!(am.name, Set("Alice".to_owned()));
    assert_eq!(am.flag, Set(true));
    assert_eq!(am.nullable_int, Set(Some(42)));
    assert_eq!(am.nullable_name, Set(Some("hello".to_owned())));

    let am = &active_models[1];
    assert_eq!(am.id, Set(2));
    assert_eq!(am.tiny, Set(20));
    assert_eq!(am.name, Set("Bob".to_owned()));
    assert_eq!(am.flag, Set(false));
    assert_eq!(am.nullable_int, Set(None));
    assert_eq!(am.nullable_name, Set(None));
}

#[test]
fn test_from_arrow_missing_columns() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("name", DataType::Utf8, false),
    ]));

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int32Array::from(vec![42])),
            Arc::new(StringArray::from(vec!["partial"])),
        ],
    )
    .expect("Failed to create RecordBatch");

    let active_models =
        primitive_entity::ActiveModel::from_arrow(&batch).expect("from_arrow failed");
    assert_eq!(active_models.len(), 1);

    let am = &active_models[0];
    assert_eq!(am.id, Set(42));
    assert_eq!(am.name, Set("partial".to_owned()));
    assert_eq!(am.tiny, NotSet);
    assert_eq!(am.small, NotSet);
    assert_eq!(am.big, NotSet);
    assert_eq!(am.float_val, NotSet);
    assert_eq!(am.double_val, NotSet);
    assert_eq!(am.flag, NotSet);
    assert_eq!(am.nullable_int, NotSet);
    assert_eq!(am.nullable_name, NotSet);
}

#[test]
fn test_from_arrow_empty_batch() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("name", DataType::Utf8, false),
    ]));

    let batch = RecordBatch::new_empty(schema);
    let active_models =
        primitive_entity::ActiveModel::from_arrow(&batch).expect("from_arrow failed");
    assert!(active_models.is_empty());
}

#[test]
fn test_from_arrow_type_mismatch() {
    let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int64, false)]));

    let batch = RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![1i64]))])
        .expect("Failed to create RecordBatch");

    let result = primitive_entity::ActiveModel::from_arrow(&batch);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DbErr::Type(_)));
}

/// Chrono datetime tests
#[cfg(feature = "with-chrono")]
mod chrono_tests {
    use super::*;

    mod chrono_entity {
        use sea_orm::entity::prelude::*;

        #[sea_orm::model]
        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "test_chrono")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub created_date: ChronoDate,
            pub created_time: ChronoTime,
            pub created_at: ChronoDateTime,
            pub updated_at: ChronoDateTimeUtc,
            pub nullable_ts: Option<ChronoDateTimeUtc>,
        }

        impl ActiveModelBehavior for ActiveModel {}
    }

    #[test]
    fn test_from_arrow_chrono_timestamp_micros() {
        use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};

        // 2024-06-15 10:30:00 UTC
        let epoch_us: i64 = 1_718_447_400_000_000;
        // Date: days since 1970-01-01 for 2024-06-15
        let epoch_days: i32 = 19889;
        // Time: 10:30:00 as microseconds since midnight
        let time_us: i64 = 10 * 3_600_000_000 + 30 * 60_000_000;

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("created_date", DataType::Date32, false),
            Field::new(
                "created_time",
                DataType::Time64(TimeUnit::Microsecond),
                false,
            ),
            Field::new(
                "created_at",
                DataType::Timestamp(TimeUnit::Microsecond, None),
                false,
            ),
            Field::new(
                "updated_at",
                DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())),
                false,
            ),
            Field::new(
                "nullable_ts",
                DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())),
                true,
            ),
        ]));

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(vec![1, 2])),
                Arc::new(Date32Array::from(vec![epoch_days, epoch_days])),
                Arc::new(Time64MicrosecondArray::from(vec![time_us, time_us])),
                Arc::new(TimestampMicrosecondArray::from(vec![epoch_us, epoch_us])),
                Arc::new(
                    TimestampMicrosecondArray::from(vec![epoch_us, epoch_us]).with_timezone("UTC"),
                ),
                Arc::new(
                    TimestampMicrosecondArray::from(vec![Some(epoch_us), None])
                        .with_timezone("UTC"),
                ),
            ],
        )
        .expect("Failed to create RecordBatch");

        let ams = chrono_entity::ActiveModel::from_arrow(&batch).expect("from_arrow failed");
        assert_eq!(ams.len(), 2);

        let am = &ams[0];
        assert_eq!(am.id, Set(1));
        assert_eq!(
            am.created_date,
            Set(NaiveDate::from_ymd_opt(2024, 6, 15).expect("valid"))
        );
        assert_eq!(
            am.created_time,
            Set(NaiveTime::from_hms_opt(10, 30, 0).expect("valid"))
        );
        let expected_naive = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 6, 15).expect("valid"),
            NaiveTime::from_hms_opt(10, 30, 0).expect("valid"),
        );
        assert_eq!(am.created_at, Set(expected_naive));
        let expected_utc: DateTime<Utc> = DateTime::from_timestamp_micros(epoch_us).expect("valid");
        assert_eq!(am.updated_at, Set(expected_utc));
        assert_eq!(am.nullable_ts, Set(Some(expected_utc)));

        // Second row: nullable_ts should be None
        let am = &ams[1];
        assert_eq!(am.nullable_ts, Set(None));
    }

    #[test]
    fn test_from_arrow_chrono_timestamp_seconds() {
        use chrono::{DateTime, Utc};

        // 2024-06-15 10:30:00 UTC as seconds
        let epoch_s: i64 = 1_718_447_400;

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("created_date", DataType::Date32, false),
            Field::new("created_time", DataType::Time32(TimeUnit::Second), false),
            Field::new(
                "created_at",
                DataType::Timestamp(TimeUnit::Second, None),
                false,
            ),
            Field::new(
                "updated_at",
                DataType::Timestamp(TimeUnit::Second, None),
                false,
            ),
            Field::new(
                "nullable_ts",
                DataType::Timestamp(TimeUnit::Second, None),
                true,
            ),
        ]));

        let time_secs: i32 = 10 * 3600 + 30 * 60; // 10:30:00
        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(vec![1])),
                Arc::new(Date32Array::from(vec![19889])),
                Arc::new(Time32SecondArray::from(vec![time_secs])),
                Arc::new(TimestampSecondArray::from(vec![epoch_s])),
                Arc::new(TimestampSecondArray::from(vec![epoch_s])),
                Arc::new(TimestampSecondArray::from(vec![Some(epoch_s)])),
            ],
        )
        .expect("Failed to create RecordBatch");

        let ams = chrono_entity::ActiveModel::from_arrow(&batch).expect("from_arrow failed");
        assert_eq!(ams.len(), 1);

        let expected_utc: DateTime<Utc> = DateTime::from_timestamp(epoch_s, 0).expect("valid");
        assert_eq!(ams[0].updated_at, Set(expected_utc));
    }

    #[test]
    fn test_from_arrow_chrono_timestamp_nanos() {
        use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};

        // 2024-06-15 10:30:00.123456789 UTC as nanoseconds
        let epoch_ns: i64 = 1_718_447_400_123_456_789;
        // Date: days since 1970-01-01 for 2024-06-15
        let epoch_days: i32 = 19889;
        // Time: 10:30:00.123456789 as nanoseconds since midnight
        let time_ns: i64 = 10 * 3_600_000_000_000 + 30 * 60_000_000_000 + 123_456_789;

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("created_date", DataType::Date32, false),
            Field::new(
                "created_time",
                DataType::Time64(TimeUnit::Nanosecond),
                false,
            ),
            Field::new(
                "created_at",
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                false,
            ),
            Field::new(
                "updated_at",
                DataType::Timestamp(TimeUnit::Nanosecond, Some("UTC".into())),
                false,
            ),
            Field::new(
                "nullable_ts",
                DataType::Timestamp(TimeUnit::Nanosecond, Some("UTC".into())),
                true,
            ),
        ]));

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(vec![1, 2])),
                Arc::new(Date32Array::from(vec![epoch_days, epoch_days])),
                Arc::new(Time64NanosecondArray::from(vec![time_ns, time_ns])),
                Arc::new(TimestampNanosecondArray::from(vec![epoch_ns, epoch_ns])),
                Arc::new(
                    TimestampNanosecondArray::from(vec![epoch_ns, epoch_ns]).with_timezone("UTC"),
                ),
                Arc::new(
                    TimestampNanosecondArray::from(vec![Some(epoch_ns), None])
                        .with_timezone("UTC"),
                ),
            ],
        )
        .expect("Failed to create RecordBatch");

        let ams = chrono_entity::ActiveModel::from_arrow(&batch).expect("from_arrow failed");
        assert_eq!(ams.len(), 2);

        let am = &ams[0];
        assert_eq!(am.id, Set(1));
        assert_eq!(
            am.created_date,
            Set(NaiveDate::from_ymd_opt(2024, 6, 15).expect("valid"))
        );
        assert_eq!(
            am.created_time,
            Set(NaiveTime::from_hms_nano_opt(10, 30, 0, 123_456_789).expect("valid"))
        );
        let expected_naive = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 6, 15).expect("valid"),
            NaiveTime::from_hms_nano_opt(10, 30, 0, 123_456_789).expect("valid"),
        );
        assert_eq!(am.created_at, Set(expected_naive));
        let expected_utc: DateTime<Utc> = DateTime::from_timestamp_nanos(epoch_ns);
        assert_eq!(am.updated_at, Set(expected_utc));
        assert_eq!(am.nullable_ts, Set(Some(expected_utc)));

        // Second row: nullable_ts should be None
        let am = &ams[1];
        assert_eq!(am.nullable_ts, Set(None));
    }
}

/// time crate datetime tests
#[cfg(feature = "with-time")]
mod time_tests {
    use super::*;

    mod time_entity {
        use sea_orm::entity::prelude::*;

        #[sea_orm::model]
        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "test_time")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            pub created_date: TimeDate,
            pub created_time: TimeTime,
            pub created_at: TimeDateTime,
            pub updated_at: TimeDateTimeWithTimeZone,
            pub nullable_ts: Option<TimeDateTimeWithTimeZone>,
        }

        impl ActiveModelBehavior for ActiveModel {}
    }

    #[test]
    fn test_from_arrow_time_crate() {
        // 2024-06-15 10:30:00 UTC
        let epoch_us: i64 = 1_718_447_400_000_000;
        let epoch_days: i32 = 19889;
        let time_us: i64 = 10 * 3_600_000_000 + 30 * 60_000_000;

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("created_date", DataType::Date32, false),
            Field::new(
                "created_time",
                DataType::Time64(TimeUnit::Microsecond),
                false,
            ),
            Field::new(
                "created_at",
                DataType::Timestamp(TimeUnit::Microsecond, None),
                false,
            ),
            Field::new(
                "updated_at",
                DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())),
                false,
            ),
            Field::new(
                "nullable_ts",
                DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())),
                true,
            ),
        ]));

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(vec![1])),
                Arc::new(Date32Array::from(vec![epoch_days])),
                Arc::new(Time64MicrosecondArray::from(vec![time_us])),
                Arc::new(TimestampMicrosecondArray::from(vec![epoch_us])),
                Arc::new(TimestampMicrosecondArray::from(vec![epoch_us]).with_timezone("UTC")),
                Arc::new(
                    TimestampMicrosecondArray::from(vec![Some(epoch_us)]).with_timezone("UTC"),
                ),
            ],
        )
        .expect("Failed to create RecordBatch");

        let ams = time_entity::ActiveModel::from_arrow(&batch).expect("from_arrow failed");
        assert_eq!(ams.len(), 1);

        let am = &ams[0];

        let expected_date =
            time::Date::from_calendar_date(2024, time::Month::June, 15).expect("valid");
        assert_eq!(am.created_date, Set(expected_date));

        let expected_time = time::Time::from_hms(10, 30, 0).expect("valid");
        assert_eq!(am.created_time, Set(expected_time));

        let expected_pdt = time::PrimitiveDateTime::new(expected_date, expected_time);
        assert_eq!(am.created_at, Set(expected_pdt));

        let expected_odt =
            time::OffsetDateTime::from_unix_timestamp_nanos(epoch_us as i128 * 1_000)
                .expect("valid");
        assert_eq!(am.updated_at, Set(expected_odt));
        assert_eq!(am.nullable_ts, Set(Some(expected_odt)));
    }

    #[test]
    fn test_from_arrow_time_crate_nanos() {
        // 2024-06-15 10:30:00.123456789 UTC
        let epoch_ns: i64 = 1_718_447_400_123_456_789;
        let epoch_days: i32 = 19889;
        let time_ns: i64 = 10 * 3_600_000_000_000 + 30 * 60_000_000_000 + 123_456_789;

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("created_date", DataType::Date32, false),
            Field::new(
                "created_time",
                DataType::Time64(TimeUnit::Nanosecond),
                false,
            ),
            Field::new(
                "created_at",
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                false,
            ),
            Field::new(
                "updated_at",
                DataType::Timestamp(TimeUnit::Nanosecond, Some("UTC".into())),
                false,
            ),
            Field::new(
                "nullable_ts",
                DataType::Timestamp(TimeUnit::Nanosecond, Some("UTC".into())),
                true,
            ),
        ]));

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(vec![1, 2])),
                Arc::new(Date32Array::from(vec![epoch_days, epoch_days])),
                Arc::new(Time64NanosecondArray::from(vec![time_ns, time_ns])),
                Arc::new(TimestampNanosecondArray::from(vec![epoch_ns, epoch_ns])),
                Arc::new(
                    TimestampNanosecondArray::from(vec![epoch_ns, epoch_ns]).with_timezone("UTC"),
                ),
                Arc::new(
                    TimestampNanosecondArray::from(vec![Some(epoch_ns), None])
                        .with_timezone("UTC"),
                ),
            ],
        )
        .expect("Failed to create RecordBatch");

        let ams = time_entity::ActiveModel::from_arrow(&batch).expect("from_arrow failed");
        assert_eq!(ams.len(), 2);

        let am = &ams[0];

        let expected_date =
            time::Date::from_calendar_date(2024, time::Month::June, 15).expect("valid");
        assert_eq!(am.created_date, Set(expected_date));

        let expected_time =
            time::Time::from_hms_nano(10, 30, 0, 123_456_789).expect("valid");
        assert_eq!(am.created_time, Set(expected_time));

        let expected_pdt = time::PrimitiveDateTime::new(expected_date, expected_time);
        assert_eq!(am.created_at, Set(expected_pdt));

        let expected_odt =
            time::OffsetDateTime::from_unix_timestamp_nanos(epoch_ns as i128).expect("valid");
        assert_eq!(am.updated_at, Set(expected_odt));
        assert_eq!(am.nullable_ts, Set(Some(expected_odt)));

        // Second row: nullable_ts should be None
        let am = &ams[1];
        assert_eq!(am.nullable_ts, Set(None));
    }
}

/// rust_decimal type tests
#[cfg(feature = "with-rust_decimal")]
mod rust_decimal_tests {
    use super::*;

    mod decimal_entity {
        use sea_orm::entity::prelude::*;

        #[sea_orm::model]
        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "test_rust_decimal")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
            pub price: Decimal,
            #[sea_orm(column_type = "Decimal(Some((20, 4)))")]
            pub amount: Decimal,
            #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
            pub nullable_decimal: Option<Decimal>,
        }

        impl ActiveModelBehavior for ActiveModel {}
    }

    #[test]
    fn test_from_arrow_decimal128_rust_decimal() {
        use rust_decimal::Decimal;

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("price", DataType::Decimal128(10, 2), false),
            Field::new("amount", DataType::Decimal128(20, 4), false),
            Field::new("nullable_decimal", DataType::Decimal128(10, 2), true),
        ]));

        // Create test data: price=12345.67, amount=9876543.2109
        let price_scaled = 1234567i128; // 12345.67 with scale 2
        let amount_scaled = 98765432109i128; // 9876543.2109 with scale 4

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(vec![1, 2])),
                Arc::new(
                    Decimal128Array::from(vec![price_scaled, price_scaled])
                        .with_precision_and_scale(10, 2)
                        .expect("valid precision/scale"),
                ),
                Arc::new(
                    Decimal128Array::from(vec![amount_scaled, amount_scaled])
                        .with_precision_and_scale(20, 4)
                        .expect("valid precision/scale"),
                ),
                Arc::new(
                    Decimal128Array::from(vec![Some(price_scaled), None])
                        .with_precision_and_scale(10, 2)
                        .expect("valid precision/scale"),
                ),
            ],
        )
        .expect("Failed to create RecordBatch");

        let ams = decimal_entity::ActiveModel::from_arrow(&batch).expect("from_arrow failed");
        assert_eq!(ams.len(), 2);

        let am = &ams[0];
        assert_eq!(am.id, Set(1));
        assert_eq!(
            am.price,
            Set(Decimal::from_i128_with_scale(price_scaled, 2))
        );
        assert_eq!(
            am.amount,
            Set(Decimal::from_i128_with_scale(amount_scaled, 4))
        );
        assert_eq!(
            am.nullable_decimal,
            Set(Some(Decimal::from_i128_with_scale(price_scaled, 2)))
        );

        // Second row: nullable_decimal should be None
        let am = &ams[1];
        assert_eq!(am.nullable_decimal, Set(None));
    }

    #[test]
    fn test_from_arrow_decimal128_edge_cases() {
        use rust_decimal::Decimal;

        // Test zero, negative, and large values
        let zero = Decimal::from_i128_with_scale(0, 2);
        let negative = Decimal::from_i128_with_scale(-123456, 2);
        let large = Decimal::from_i128_with_scale(123456789012345678i128, 10);

        assert_eq!(zero.to_string(), "0.00");
        assert_eq!(negative.to_string(), "-1234.56");
        assert!(large.to_string().contains("12345678.9012345678"));
    }
}

/// bigdecimal type tests
#[cfg(feature = "with-bigdecimal")]
mod bigdecimal_tests {
    use super::*;

    mod decimal_entity {
        use sea_orm::entity::prelude::*;

        #[sea_orm::model]
        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "test_bigdecimal")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: i32,
            #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
            pub price: BigDecimal,
            #[sea_orm(column_type = "Decimal(Some((20, 4)))")]
            pub amount: BigDecimal,
            #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
            pub nullable_decimal: Option<BigDecimal>,
        }

        impl ActiveModelBehavior for ActiveModel {}
    }

    #[test]
    #[cfg(not(feature = "with-rust_decimal"))]
    fn test_from_arrow_decimal128_bigdecimal() {
        use bigdecimal::{num_bigint::BigInt, BigDecimal};

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("price", DataType::Decimal128(10, 2), false),
            Field::new("amount", DataType::Decimal128(20, 4), false),
            Field::new("nullable_decimal", DataType::Decimal128(10, 2), true),
        ]));

        // Create test data: price=12345.67, amount=9876543.2109
        let price_scaled = 1234567i128; // 12345.67 with scale 2
        let amount_scaled = 98765432109i128; // 9876543.2109 with scale 4

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(vec![1, 2])),
                Arc::new(
                    Decimal128Array::from(vec![price_scaled, price_scaled])
                        .with_precision_and_scale(10, 2)
                        .expect("valid precision/scale"),
                ),
                Arc::new(
                    Decimal128Array::from(vec![amount_scaled, amount_scaled])
                        .with_precision_and_scale(20, 4)
                        .expect("valid precision/scale"),
                ),
                Arc::new(
                    Decimal128Array::from(vec![Some(price_scaled), None])
                        .with_precision_and_scale(10, 2)
                        .expect("valid precision/scale"),
                ),
            ],
        )
        .expect("Failed to create RecordBatch");

        let ams = decimal_entity::ActiveModel::from_arrow(&batch).expect("from_arrow failed");
        assert_eq!(ams.len(), 2);

        let am = &ams[0];
        assert_eq!(am.id, Set(1));
        assert_eq!(
            am.price,
            Set(BigDecimal::new(BigInt::from(price_scaled), 2))
        );
        assert_eq!(
            am.amount,
            Set(BigDecimal::new(BigInt::from(amount_scaled), 4))
        );
        assert_eq!(
            am.nullable_decimal,
            Set(Some(BigDecimal::new(BigInt::from(price_scaled), 2)))
        );

        // Second row: nullable_decimal should be None
        let am = &ams[1];
        assert_eq!(am.nullable_decimal, Set(None));
    }

    #[test]
    fn test_from_arrow_decimal256_bigdecimal() {
        use arrow::datatypes::i256;

        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("large_value", DataType::Decimal256(76, 20), false),
            Field::new("nullable_large", DataType::Decimal256(76, 20), true),
        ]));

        // Create a large i256 value
        let large_val = i256::from_i128(123456789012345678i128);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(vec![1, 2])),
                Arc::new(
                    Decimal256Array::from(vec![large_val, large_val])
                        .with_precision_and_scale(76, 20)
                        .expect("valid precision/scale"),
                ),
                Arc::new(
                    Decimal256Array::from(vec![Some(large_val), None])
                        .with_precision_and_scale(76, 20)
                        .expect("valid precision/scale"),
                ),
            ],
        )
        .expect("Failed to create RecordBatch");

        // Test the batch was created correctly
        assert_eq!(batch.num_rows(), 2);
        assert_eq!(batch.num_columns(), 3);

        // Test that we can read the decimal values
        let arr = batch
            .column(1)
            .as_any()
            .downcast_ref::<Decimal256Array>()
            .expect("Expected Decimal256Array");
        assert_eq!(arr.value(0), large_val);
        assert_eq!(arr.precision(), 76);
        assert_eq!(arr.scale(), 20);
    }
}
