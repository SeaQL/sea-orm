#![cfg(feature = "with-arrow")]
//! cargo t --test arrow_tests
//! cargo t --test arrow_tests --no-default-features --features=macros
//! cargo t --test arrow_tests --no-default-features --features=macros,with-chrono
//! cargo t --test arrow_tests --no-default-features --features=macros,with-time
use arrow::array::*;
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue::NotSet, Set, arrow};
use std::sync::Arc;

/// Test entity with all supported primitive types
mod primitive_entity {
    use sea_orm::entity::prelude::*;

    #[derive(Copy, Clone, Default, Debug, DeriveEntity)]
    #[sea_orm(table_name = "test_arrow")]
    pub struct Entity;

    #[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
    pub struct Model {
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

    #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
    pub enum Column {
        Id,
        Tiny,
        Small,
        Big,
        TinyU,
        SmallU,
        Uint,
        BigU,
        FloatVal,
        DoubleVal,
        Name,
        Flag,
        NullableInt,
        NullableName,
    }

    impl ColumnTrait for Column {
        type EntityName = Entity;

        fn def(&self) -> ColumnDef {
            match self {
                Column::Id => ColumnType::Integer.def(),
                Column::Tiny => ColumnType::TinyInteger.def(),
                Column::Small => ColumnType::SmallInteger.def(),
                Column::Big => ColumnType::BigInteger.def(),
                Column::TinyU => ColumnType::TinyUnsigned.def(),
                Column::SmallU => ColumnType::SmallUnsigned.def(),
                Column::Uint => ColumnType::Unsigned.def(),
                Column::BigU => ColumnType::BigUnsigned.def(),
                Column::FloatVal => ColumnType::Float.def(),
                Column::DoubleVal => ColumnType::Double.def(),
                Column::Name => ColumnType::String(StringLen::None).def(),
                Column::Flag => ColumnType::Boolean.def(),
                Column::NullableInt => ColumnType::Integer.def().nullable(),
                Column::NullableName => ColumnType::String(StringLen::None).def().nullable(),
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

        #[derive(Copy, Clone, Default, Debug, DeriveEntity)]
        #[sea_orm(table_name = "test_chrono")]
        pub struct Entity;

        #[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
        pub struct Model {
            pub id: i32,
            pub created_date: ChronoDate,
            pub created_time: ChronoTime,
            pub created_at: ChronoDateTime,
            pub updated_at: ChronoDateTimeUtc,
            pub nullable_ts: Option<ChronoDateTimeUtc>,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
        pub enum Column {
            Id,
            CreatedDate,
            CreatedTime,
            CreatedAt,
            UpdatedAt,
            NullableTs,
        }

        impl ColumnTrait for Column {
            type EntityName = Entity;

            fn def(&self) -> ColumnDef {
                match self {
                    Column::Id => ColumnType::Integer.def(),
                    Column::CreatedDate => ColumnType::Date.def(),
                    Column::CreatedTime => ColumnType::Time.def(),
                    Column::CreatedAt => ColumnType::DateTime.def(),
                    Column::UpdatedAt => ColumnType::TimestampWithTimeZone.def(),
                    Column::NullableTs => ColumnType::TimestampWithTimeZone.def().nullable(),
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
}

/// time crate datetime tests
#[cfg(feature = "with-time")]
mod time_tests {
    use super::*;

    mod time_entity {
        use sea_orm::entity::prelude::*;

        #[derive(Copy, Clone, Default, Debug, DeriveEntity)]
        #[sea_orm(table_name = "test_time")]
        pub struct Entity;

        #[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
        pub struct Model {
            pub id: i32,
            pub created_date: TimeDate,
            pub created_time: TimeTime,
            pub created_at: TimeDateTime,
            pub updated_at: TimeDateTimeWithTimeZone,
            pub nullable_ts: Option<TimeDateTimeWithTimeZone>,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
        pub enum Column {
            Id,
            CreatedDate,
            CreatedTime,
            CreatedAt,
            UpdatedAt,
            NullableTs,
        }

        impl ColumnTrait for Column {
            type EntityName = Entity;

            fn def(&self) -> ColumnDef {
                match self {
                    Column::Id => ColumnType::Integer.def(),
                    Column::CreatedDate => ColumnType::Date.def(),
                    Column::CreatedTime => ColumnType::Time.def(),
                    Column::CreatedAt => ColumnType::DateTime.def(),
                    Column::UpdatedAt => ColumnType::TimestampWithTimeZone.def(),
                    Column::NullableTs => ColumnType::TimestampWithTimeZone.def().nullable(),
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
}
