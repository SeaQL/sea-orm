#![cfg(feature = "with-arrow")]
//! cargo t --test arrow_tests --features=with-arrow
//! cargo t --test arrow_tests --features=with-arrow,with-bigdecimal
use arrow::array::*;
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue::NotSet, ArrowSchema, Set, arrow};
use std::sync::Arc;

/// Test entity with all supported primitive types
mod primitive_entity {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "test_arrow", arrow_schema)]
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

// ===========================================================================
// to_arrow tests
// ===========================================================================

#[test]
fn test_to_arrow_basic_primitives() {
    use sea_orm::ArrowSchema;

    let schema = primitive_entity::Entity::arrow_schema();

    let models = vec![
        primitive_entity::ActiveModel {
            id: Set(1),
            tiny: Set(10),
            small: Set(100),
            big: Set(1000),
            tiny_u: Set(5),
            small_u: Set(50),
            uint: Set(500),
            big_u: Set(5000),
            float_val: Set(1.5),
            double_val: Set(10.5),
            name: Set("Alice".to_owned()),
            flag: Set(true),
            nullable_int: Set(Some(42)),
            nullable_name: Set(Some("hello".to_owned())),
        },
        primitive_entity::ActiveModel {
            id: Set(2),
            tiny: Set(20),
            small: Set(200),
            big: Set(2000),
            tiny_u: Set(6),
            small_u: Set(60),
            uint: Set(600),
            big_u: Set(6000),
            float_val: Set(2.5),
            double_val: Set(20.5),
            name: Set("Bob".to_owned()),
            flag: Set(false),
            nullable_int: Set(None),
            nullable_name: Set(None),
        },
    ];

    let batch =
        primitive_entity::ActiveModel::to_arrow(&models, &schema).expect("to_arrow failed");

    assert_eq!(batch.num_rows(), 2);
    assert_eq!(batch.num_columns(), 14);

    // Verify integer columns
    let id_arr = batch
        .column_by_name("id")
        .unwrap()
        .as_any()
        .downcast_ref::<Int32Array>()
        .unwrap();
    assert_eq!(id_arr.value(0), 1);
    assert_eq!(id_arr.value(1), 2);

    let tiny_arr = batch
        .column_by_name("tiny")
        .unwrap()
        .as_any()
        .downcast_ref::<Int8Array>()
        .unwrap();
    assert_eq!(tiny_arr.value(0), 10);
    assert_eq!(tiny_arr.value(1), 20);

    // Verify unsigned
    let tiny_u_arr = batch
        .column_by_name("tiny_u")
        .unwrap()
        .as_any()
        .downcast_ref::<UInt8Array>()
        .unwrap();
    assert_eq!(tiny_u_arr.value(0), 5);

    // Verify floats
    let float_arr = batch
        .column_by_name("float_val")
        .unwrap()
        .as_any()
        .downcast_ref::<Float32Array>()
        .unwrap();
    assert_eq!(float_arr.value(0), 1.5);

    let double_arr = batch
        .column_by_name("double_val")
        .unwrap()
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap();
    assert_eq!(double_arr.value(0), 10.5);

    // Verify strings
    let name_arr = batch
        .column_by_name("name")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert_eq!(name_arr.value(0), "Alice");
    assert_eq!(name_arr.value(1), "Bob");

    // Verify boolean
    let flag_arr = batch
        .column_by_name("flag")
        .unwrap()
        .as_any()
        .downcast_ref::<BooleanArray>()
        .unwrap();
    assert!(flag_arr.value(0));
    assert!(!flag_arr.value(1));

    // Verify nullable: row 0 has values, row 1 has nulls
    let ni_arr = batch
        .column_by_name("nullable_int")
        .unwrap()
        .as_any()
        .downcast_ref::<Int32Array>()
        .unwrap();
    assert!(!ni_arr.is_null(0));
    assert_eq!(ni_arr.value(0), 42);
    assert!(ni_arr.is_null(1));

    let nn_arr = batch
        .column_by_name("nullable_name")
        .unwrap()
        .as_any()
        .downcast_ref::<StringArray>()
        .unwrap();
    assert!(!nn_arr.is_null(0));
    assert_eq!(nn_arr.value(0), "hello");
    assert!(nn_arr.is_null(1));
}

#[test]
fn test_to_arrow_not_set_becomes_null() {
    // Use an all-nullable schema so that NotSet → null is accepted by Arrow
    let base = primitive_entity::Entity::arrow_schema();
    let nullable_fields: Vec<Field> = base
        .fields()
        .iter()
        .map(|f| Field::new(f.name(), f.data_type().clone(), true))
        .collect();
    let schema = Schema::new(nullable_fields);

    // ActiveModel with only id and name set; everything else is NotSet
    let models = vec![primitive_entity::ActiveModel {
        id: Set(99),
        name: Set("partial".to_owned()),
        ..Default::default()
    }];

    let batch =
        primitive_entity::ActiveModel::to_arrow(&models, &schema).expect("to_arrow failed");
    assert_eq!(batch.num_rows(), 1);

    // id and name should be present
    let id_arr = batch
        .column_by_name("id")
        .unwrap()
        .as_any()
        .downcast_ref::<Int32Array>()
        .unwrap();
    assert_eq!(id_arr.value(0), 99);

    // NotSet fields should be null
    let tiny_arr = batch
        .column_by_name("tiny")
        .unwrap()
        .as_any()
        .downcast_ref::<Int8Array>()
        .unwrap();
    assert!(tiny_arr.is_null(0));

    let flag_arr = batch
        .column_by_name("flag")
        .unwrap()
        .as_any()
        .downcast_ref::<BooleanArray>()
        .unwrap();
    assert!(flag_arr.is_null(0));
}

#[test]
fn test_to_arrow_empty_slice() {
    let schema = primitive_entity::Entity::arrow_schema();
    let batch =
        primitive_entity::ActiveModel::to_arrow(&[], &schema).expect("to_arrow failed");
    assert_eq!(batch.num_rows(), 0);
    assert_eq!(batch.num_columns(), 14);
}

#[test]
fn test_to_arrow_roundtrip_primitives() {
    let schema = primitive_entity::Entity::arrow_schema();

    let original = vec![
        primitive_entity::ActiveModel {
            id: Set(1),
            tiny: Set(10),
            small: Set(100),
            big: Set(1000),
            tiny_u: Set(5),
            small_u: Set(50),
            uint: Set(500),
            big_u: Set(5000),
            float_val: Set(1.5),
            double_val: Set(10.5),
            name: Set("Alice".to_owned()),
            flag: Set(true),
            nullable_int: Set(Some(42)),
            nullable_name: Set(Some("hello".to_owned())),
        },
        primitive_entity::ActiveModel {
            id: Set(2),
            tiny: Set(20),
            small: Set(200),
            big: Set(2000),
            tiny_u: Set(6),
            small_u: Set(60),
            uint: Set(600),
            big_u: Set(6000),
            float_val: Set(2.5),
            double_val: Set(20.5),
            name: Set("Bob".to_owned()),
            flag: Set(false),
            nullable_int: Set(None),
            nullable_name: Set(None),
        },
    ];

    // to_arrow -> from_arrow roundtrip
    let batch =
        primitive_entity::ActiveModel::to_arrow(&original, &schema).expect("to_arrow failed");
    let roundtripped =
        primitive_entity::ActiveModel::from_arrow(&batch).expect("from_arrow failed");

    assert_eq!(roundtripped.len(), 2);

    // First row
    assert_eq!(roundtripped[0].id, Set(1));
    assert_eq!(roundtripped[0].tiny, Set(10));
    assert_eq!(roundtripped[0].small, Set(100));
    assert_eq!(roundtripped[0].big, Set(1000));
    assert_eq!(roundtripped[0].tiny_u, Set(5));
    assert_eq!(roundtripped[0].small_u, Set(50));
    assert_eq!(roundtripped[0].uint, Set(500));
    assert_eq!(roundtripped[0].big_u, Set(5000));
    assert_eq!(roundtripped[0].float_val, Set(1.5));
    assert_eq!(roundtripped[0].double_val, Set(10.5));
    assert_eq!(roundtripped[0].name, Set("Alice".to_owned()));
    assert_eq!(roundtripped[0].flag, Set(true));
    assert_eq!(roundtripped[0].nullable_int, Set(Some(42)));
    assert_eq!(roundtripped[0].nullable_name, Set(Some("hello".to_owned())));

    // Second row: nullable fields are None
    assert_eq!(roundtripped[1].nullable_int, Set(None));
    assert_eq!(roundtripped[1].nullable_name, Set(None));
}

/// Chrono datetime tests
#[cfg(feature = "with-chrono")]
mod chrono_tests {
    use super::*;

    mod chrono_entity {
        use sea_orm::entity::prelude::*;

        #[sea_orm::model]
        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "test_chrono", arrow_schema)]
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
    fn test_to_arrow_chrono_roundtrip() {
        use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
        use sea_orm::ArrowSchema;

        let schema = chrono_entity::Entity::arrow_schema();

        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let time = NaiveTime::from_hms_opt(10, 30, 0).unwrap();
        let naive_dt = NaiveDateTime::new(date, time);
        let utc_dt: DateTime<Utc> = DateTime::from_timestamp_micros(1_718_447_400_000_000).unwrap();

        let models = vec![
            chrono_entity::ActiveModel {
                id: Set(1),
                created_date: Set(date),
                created_time: Set(time),
                created_at: Set(naive_dt),
                updated_at: Set(utc_dt),
                nullable_ts: Set(Some(utc_dt)),
            },
            chrono_entity::ActiveModel {
                id: Set(2),
                created_date: Set(date),
                created_time: Set(time),
                created_at: Set(naive_dt),
                updated_at: Set(utc_dt),
                nullable_ts: Set(None),
            },
        ];

        let batch =
            chrono_entity::ActiveModel::to_arrow(&models, &schema).expect("to_arrow failed");
        assert_eq!(batch.num_rows(), 2);

        // Verify Date32
        let date_arr = batch
            .column_by_name("created_date")
            .unwrap()
            .as_any()
            .downcast_ref::<Date32Array>()
            .unwrap();
        assert_eq!(date_arr.value(0), 19889); // 2024-06-15

        // Verify Time64
        let time_arr = batch
            .column_by_name("created_time")
            .unwrap()
            .as_any()
            .downcast_ref::<Time64MicrosecondArray>()
            .unwrap();
        let expected_time_us: i64 = 10 * 3_600_000_000 + 30 * 60_000_000;
        assert_eq!(time_arr.value(0), expected_time_us);

        // Verify Timestamp (naive)
        let ts_arr = batch
            .column_by_name("created_at")
            .unwrap()
            .as_any()
            .downcast_ref::<TimestampMicrosecondArray>()
            .unwrap();
        assert_eq!(ts_arr.value(0), 1_718_447_400_000_000);

        // Verify Timestamp with timezone
        let ts_utc_arr = batch
            .column_by_name("updated_at")
            .unwrap()
            .as_any()
            .downcast_ref::<TimestampMicrosecondArray>()
            .unwrap();
        assert_eq!(ts_utc_arr.value(0), 1_718_447_400_000_000);

        // Verify nullable timestamp: row 0 present, row 1 null
        let nullable_arr = batch
            .column_by_name("nullable_ts")
            .unwrap()
            .as_any()
            .downcast_ref::<TimestampMicrosecondArray>()
            .unwrap();
        assert!(!nullable_arr.is_null(0));
        assert!(nullable_arr.is_null(1));

        // Full roundtrip
        let roundtripped =
            chrono_entity::ActiveModel::from_arrow(&batch).expect("from_arrow failed");
        assert_eq!(roundtripped.len(), 2);
        assert_eq!(roundtripped[0].id, Set(1));
        assert_eq!(roundtripped[0].created_date, Set(date));
        assert_eq!(roundtripped[0].created_time, Set(time));
        assert_eq!(roundtripped[0].created_at, Set(naive_dt));
        assert_eq!(roundtripped[0].updated_at, Set(utc_dt));
        assert_eq!(roundtripped[0].nullable_ts, Set(Some(utc_dt)));
        assert_eq!(roundtripped[1].nullable_ts, Set(None));
    }

    #[test]
    fn test_to_arrow_chrono_nanosecond_schema() {
        use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};

        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let time = NaiveTime::from_hms_nano_opt(10, 30, 0, 123_456_789).unwrap();
        let naive_dt = NaiveDateTime::new(date, time);
        let utc_dt: DateTime<Utc> = DateTime::from_timestamp_nanos(1_718_447_400_123_456_789);

        let models = vec![chrono_entity::ActiveModel {
            id: Set(1),
            created_date: Set(date),
            created_time: Set(time),
            created_at: Set(naive_dt),
            updated_at: Set(utc_dt),
            nullable_ts: Set(Some(utc_dt)),
        }];

        // Use a nanosecond schema
        let schema = Schema::new(vec![
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
        ]);

        let batch =
            chrono_entity::ActiveModel::to_arrow(&models, &schema).expect("to_arrow failed");

        let time_arr = batch
            .column_by_name("created_time")
            .unwrap()
            .as_any()
            .downcast_ref::<Time64NanosecondArray>()
            .unwrap();
        let expected_time_ns: i64 =
            10 * 3_600_000_000_000 + 30 * 60_000_000_000 + 123_456_789;
        assert_eq!(time_arr.value(0), expected_time_ns);

        let ts_arr = batch
            .column_by_name("created_at")
            .unwrap()
            .as_any()
            .downcast_ref::<TimestampNanosecondArray>()
            .unwrap();
        assert_eq!(ts_arr.value(0), 1_718_447_400_123_456_789);
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
                    TimestampNanosecondArray::from(vec![Some(epoch_ns), None]).with_timezone("UTC"),
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
        #[sea_orm(table_name = "test_time", arrow_schema)]
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
                    TimestampNanosecondArray::from(vec![Some(epoch_ns), None]).with_timezone("UTC"),
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

        let expected_time = time::Time::from_hms_nano(10, 30, 0, 123_456_789).expect("valid");
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

    #[test]
    fn test_to_arrow_time_crate_roundtrip() {
        use sea_orm::ArrowSchema;

        let schema = time_entity::Entity::arrow_schema();

        let date =
            time::Date::from_calendar_date(2024, time::Month::June, 15).expect("valid");
        let time_val = time::Time::from_hms(10, 30, 0).expect("valid");
        let pdt = time::PrimitiveDateTime::new(date, time_val);
        let odt =
            time::OffsetDateTime::from_unix_timestamp_nanos(1_718_447_400_000_000_000)
                .expect("valid");

        let models = vec![
            time_entity::ActiveModel {
                id: Set(1),
                created_date: Set(date),
                created_time: Set(time_val),
                created_at: Set(pdt),
                updated_at: Set(odt),
                nullable_ts: Set(Some(odt)),
            },
            time_entity::ActiveModel {
                id: Set(2),
                created_date: Set(date),
                created_time: Set(time_val),
                created_at: Set(pdt),
                updated_at: Set(odt),
                nullable_ts: Set(None),
            },
        ];

        let batch =
            time_entity::ActiveModel::to_arrow(&models, &schema).expect("to_arrow failed");
        assert_eq!(batch.num_rows(), 2);

        // Verify Date32
        let date_arr = batch
            .column_by_name("created_date")
            .unwrap()
            .as_any()
            .downcast_ref::<Date32Array>()
            .unwrap();
        assert_eq!(date_arr.value(0), 19889); // 2024-06-15

        // Verify Time64
        let time_arr = batch
            .column_by_name("created_time")
            .unwrap()
            .as_any()
            .downcast_ref::<Time64MicrosecondArray>()
            .unwrap();
        let expected_time_us: i64 = 10 * 3_600_000_000 + 30 * 60_000_000;
        assert_eq!(time_arr.value(0), expected_time_us);

        // Verify nullable: row 0 present, row 1 null
        let nullable_arr = batch
            .column_by_name("nullable_ts")
            .unwrap()
            .as_any()
            .downcast_ref::<TimestampMicrosecondArray>()
            .unwrap();
        assert!(!nullable_arr.is_null(0));
        assert!(nullable_arr.is_null(1));

        // Full roundtrip
        let roundtripped =
            time_entity::ActiveModel::from_arrow(&batch).expect("from_arrow failed");
        assert_eq!(roundtripped.len(), 2);
        assert_eq!(roundtripped[0].id, Set(1));
        assert_eq!(roundtripped[0].created_date, Set(date));
        assert_eq!(roundtripped[0].created_time, Set(time_val));
        assert_eq!(roundtripped[0].created_at, Set(pdt));
        assert_eq!(roundtripped[0].updated_at, Set(odt));
        assert_eq!(roundtripped[0].nullable_ts, Set(Some(odt)));
        assert_eq!(roundtripped[1].nullable_ts, Set(None));
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
        #[sea_orm(table_name = "test_rust_decimal", arrow_schema)]
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

    #[test]
    fn test_to_arrow_rust_decimal_roundtrip() {
        use rust_decimal::Decimal;
        use sea_orm::ArrowSchema;

        let schema = decimal_entity::Entity::arrow_schema();

        let price = Decimal::new(1234567, 2); // 12345.67
        let amount = Decimal::new(98765432109, 4); // 9876543.2109

        let models = vec![
            decimal_entity::ActiveModel {
                id: Set(1),
                price: Set(price),
                amount: Set(amount),
                nullable_decimal: Set(Some(price)),
            },
            decimal_entity::ActiveModel {
                id: Set(2),
                price: Set(price),
                amount: Set(amount),
                nullable_decimal: Set(None),
            },
        ];

        let batch =
            decimal_entity::ActiveModel::to_arrow(&models, &schema).expect("to_arrow failed");
        assert_eq!(batch.num_rows(), 2);

        // Verify Decimal128 column
        let price_arr = batch
            .column_by_name("price")
            .unwrap()
            .as_any()
            .downcast_ref::<Decimal128Array>()
            .unwrap();
        assert_eq!(price_arr.value(0), 1234567);
        assert_eq!(price_arr.precision(), 10);
        assert_eq!(price_arr.scale(), 2);

        // Verify nullable decimal: row 0 present, row 1 null
        let nullable_arr = batch
            .column_by_name("nullable_decimal")
            .unwrap()
            .as_any()
            .downcast_ref::<Decimal128Array>()
            .unwrap();
        assert!(!nullable_arr.is_null(0));
        assert!(nullable_arr.is_null(1));

        // Full roundtrip
        let roundtripped =
            decimal_entity::ActiveModel::from_arrow(&batch).expect("from_arrow failed");
        assert_eq!(roundtripped.len(), 2);
        assert_eq!(roundtripped[0].price, Set(price));
        assert_eq!(roundtripped[0].amount, Set(amount));
        assert_eq!(roundtripped[0].nullable_decimal, Set(Some(price)));
        assert_eq!(roundtripped[1].nullable_decimal, Set(None));
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
        #[sea_orm(table_name = "test_bigdecimal", arrow_schema)]
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
        use bigdecimal::{BigDecimal, num_bigint::BigInt};

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

    #[test]
    #[cfg(not(feature = "with-rust_decimal"))]
    fn test_to_arrow_bigdecimal_roundtrip() {
        use bigdecimal::{BigDecimal, num_bigint::BigInt};
        use sea_orm::ArrowSchema;

        let schema = decimal_entity::Entity::arrow_schema();

        let price = BigDecimal::new(BigInt::from(1234567i64), 2); // 12345.67
        let amount = BigDecimal::new(BigInt::from(98765432109i64), 4); // 9876543.2109

        let models = vec![
            decimal_entity::ActiveModel {
                id: Set(1),
                price: Set(price.clone()),
                amount: Set(amount.clone()),
                nullable_decimal: Set(Some(price.clone())),
            },
            decimal_entity::ActiveModel {
                id: Set(2),
                price: Set(price.clone()),
                amount: Set(amount.clone()),
                nullable_decimal: Set(None),
            },
        ];

        let batch =
            decimal_entity::ActiveModel::to_arrow(&models, &schema).expect("to_arrow failed");
        assert_eq!(batch.num_rows(), 2);

        // Verify Decimal128 column
        let price_arr = batch
            .column_by_name("price")
            .unwrap()
            .as_any()
            .downcast_ref::<Decimal128Array>()
            .unwrap();
        assert_eq!(price_arr.value(0), 1234567);
        assert_eq!(price_arr.precision(), 10);
        assert_eq!(price_arr.scale(), 2);

        // Verify nullable: row 0 present, row 1 null
        let nullable_arr = batch
            .column_by_name("nullable_decimal")
            .unwrap()
            .as_any()
            .downcast_ref::<Decimal128Array>()
            .unwrap();
        assert!(!nullable_arr.is_null(0));
        assert!(nullable_arr.is_null(1));

        // Full roundtrip
        let roundtripped =
            decimal_entity::ActiveModel::from_arrow(&batch).expect("from_arrow failed");
        assert_eq!(roundtripped.len(), 2);
        assert_eq!(roundtripped[0].price, Set(price.clone()));
        assert_eq!(roundtripped[0].amount, Set(amount));
        assert_eq!(roundtripped[0].nullable_decimal, Set(Some(price)));
        assert_eq!(roundtripped[1].nullable_decimal, Set(None));
    }
}
