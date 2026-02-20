use arrow::array::*;
use arrow::datatypes::i256;
use sea_query::{ColumnType, Value};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur when converting between SeaORM [`Value`]s and Arrow arrays.
#[derive(Debug, thiserror::Error)]
pub enum ArrowError {
    /// The Arrow array type is incompatible with the target SeaORM column type.
    #[error("expected {expected} for column type {col_type}, got Arrow type {actual}")]
    TypeMismatch {
        expected: &'static str,
        col_type: &'static str,
        actual: String,
    },

    /// A value lies outside the representable range for the target type.
    #[error("{0}")]
    OutOfRange(String),

    /// The column type or Arrow data type is not supported for conversion.
    #[error("{0}")]
    Unsupported(String),
}

fn type_err(expected: &'static str, col_type: &'static str, array: &dyn Array) -> ArrowError {
    ArrowError::TypeMismatch {
        expected,
        col_type,
        actual: format!("{:?}", array.data_type()),
    }
}

// ---------------------------------------------------------------------------
// Arrow → Value
// ---------------------------------------------------------------------------

/// Extract a [`Value`] from an Arrow array at the given row index,
/// based on the expected [`ColumnType`] from the entity definition.
///
/// For date/time column types, this produces chrono `Value` variants when
/// the `with-chrono` feature is enabled, or time-crate variants when only
/// `with-time` is enabled.
pub fn arrow_array_to_value(
    array: &dyn Array,
    col_type: &ColumnType,
    row: usize,
) -> Result<Value, ArrowError> {
    if array.is_null(row) {
        return Ok(null_value_for_type(col_type));
    }
    match col_type {
        ColumnType::TinyInteger => {
            let arr = array
                .as_any()
                .downcast_ref::<Int8Array>()
                .ok_or_else(|| type_err("Int8Array", "TinyInteger", array))?;
            Ok(Value::TinyInt(Some(arr.value(row))))
        }
        ColumnType::SmallInteger => {
            let arr = array
                .as_any()
                .downcast_ref::<Int16Array>()
                .ok_or_else(|| type_err("Int16Array", "SmallInteger", array))?;
            Ok(Value::SmallInt(Some(arr.value(row))))
        }
        ColumnType::Integer => {
            let arr = array
                .as_any()
                .downcast_ref::<Int32Array>()
                .ok_or_else(|| type_err("Int32Array", "Integer", array))?;
            Ok(Value::Int(Some(arr.value(row))))
        }
        ColumnType::BigInteger => {
            let arr = array
                .as_any()
                .downcast_ref::<Int64Array>()
                .ok_or_else(|| type_err("Int64Array", "BigInteger", array))?;
            Ok(Value::BigInt(Some(arr.value(row))))
        }
        ColumnType::TinyUnsigned => {
            let arr = array
                .as_any()
                .downcast_ref::<UInt8Array>()
                .ok_or_else(|| type_err("UInt8Array", "TinyUnsigned", array))?;
            Ok(Value::TinyUnsigned(Some(arr.value(row))))
        }
        ColumnType::SmallUnsigned => {
            let arr = array
                .as_any()
                .downcast_ref::<UInt16Array>()
                .ok_or_else(|| type_err("UInt16Array", "SmallUnsigned", array))?;
            Ok(Value::SmallUnsigned(Some(arr.value(row))))
        }
        ColumnType::Unsigned => {
            let arr = array
                .as_any()
                .downcast_ref::<UInt32Array>()
                .ok_or_else(|| type_err("UInt32Array", "Unsigned", array))?;
            Ok(Value::Unsigned(Some(arr.value(row))))
        }
        ColumnType::BigUnsigned => {
            let arr = array
                .as_any()
                .downcast_ref::<UInt64Array>()
                .ok_or_else(|| type_err("UInt64Array", "BigUnsigned", array))?;
            Ok(Value::BigUnsigned(Some(arr.value(row))))
        }
        ColumnType::Float => {
            let arr = array
                .as_any()
                .downcast_ref::<Float32Array>()
                .ok_or_else(|| type_err("Float32Array", "Float", array))?;
            Ok(Value::Float(Some(arr.value(row))))
        }
        ColumnType::Double => {
            let arr = array
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| type_err("Float64Array", "Double", array))?;
            Ok(Value::Double(Some(arr.value(row))))
        }
        ColumnType::String(_) | ColumnType::Text | ColumnType::Char(_) => {
            if let Some(arr) = array.as_any().downcast_ref::<StringArray>() {
                Ok(Value::String(Some(arr.value(row).to_owned())))
            } else if let Some(arr) = array.as_any().downcast_ref::<LargeStringArray>() {
                Ok(Value::String(Some(arr.value(row).to_owned())))
            } else {
                Err(type_err(
                    "StringArray or LargeStringArray",
                    "String/Text",
                    array,
                ))
            }
        }
        ColumnType::Boolean => {
            let arr = array
                .as_any()
                .downcast_ref::<BooleanArray>()
                .ok_or_else(|| type_err("BooleanArray", "Boolean", array))?;
            Ok(Value::Bool(Some(arr.value(row))))
        }
        // Decimal types
        ColumnType::Decimal(_) | ColumnType::Money(_) => arrow_to_decimal(array, row),
        // Date/time types: delegate to feature-gated helpers.
        // Prefer chrono when available; fall back to time crate.
        #[cfg(feature = "with-chrono")]
        ColumnType::Date => arrow_to_chrono_date(array, row),
        #[cfg(feature = "with-chrono")]
        ColumnType::Time => arrow_to_chrono_time(array, row),
        #[cfg(feature = "with-chrono")]
        ColumnType::DateTime | ColumnType::Timestamp => arrow_to_chrono_datetime(array, row),
        #[cfg(feature = "with-chrono")]
        ColumnType::TimestampWithTimeZone => arrow_to_chrono_datetime_utc(array, row),

        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
        ColumnType::Date => arrow_to_time_date(array, row),
        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
        ColumnType::Time => arrow_to_time_time(array, row),
        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
        ColumnType::DateTime | ColumnType::Timestamp => arrow_to_time_datetime(array, row),
        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
        ColumnType::TimestampWithTimeZone => arrow_to_time_datetime_tz(array, row),

        _ => Err(ArrowError::Unsupported(format!(
            "Unsupported column type for Arrow conversion: {col_type:?}"
        ))),
    }
}

/// When both `with-chrono` and `with-time` are enabled, this provides the
/// time-crate alternative for date/time columns. Called as a fallback when
/// the chrono Value variant doesn't match the model's field type.
#[cfg(all(feature = "with-chrono", feature = "with-time"))]
pub fn arrow_array_to_value_alt(
    array: &dyn Array,
    col_type: &ColumnType,
    row: usize,
) -> Result<Option<Value>, ArrowError> {
    if array.is_null(row) {
        return Ok(Some(null_value_for_type_time(col_type)));
    }
    match col_type {
        ColumnType::Date => arrow_to_time_date(array, row).map(Some),
        ColumnType::Time => arrow_to_time_time(array, row).map(Some),
        ColumnType::DateTime | ColumnType::Timestamp => {
            arrow_to_time_datetime(array, row).map(Some)
        }
        ColumnType::TimestampWithTimeZone => arrow_to_time_datetime_tz(array, row).map(Some),
        _ => Ok(None),
    }
}

/// Returns true for ColumnTypes that may need a chrono→time fallback.
pub fn is_datetime_column(col_type: &ColumnType) -> bool {
    matches!(
        col_type,
        ColumnType::Date
            | ColumnType::Time
            | ColumnType::DateTime
            | ColumnType::Timestamp
            | ColumnType::TimestampWithTimeZone
    )
}

// ---------------------------------------------------------------------------
// Decimal helpers
// ---------------------------------------------------------------------------

/// Convert Arrow Decimal128Array or Decimal256Array to a decimal Value.
/// Prefers rust_decimal when available and precision fits, otherwise bigdecimal.
fn arrow_to_decimal(array: &dyn Array, row: usize) -> Result<Value, ArrowError> {
    if let Some(arr) = array.as_any().downcast_ref::<Decimal128Array>() {
        let value = arr.value(row);
        let precision = arr.precision();
        let scale = arr.scale();
        return decimal128_to_value(value, precision, scale);
    }

    if let Some(arr) = array.as_any().downcast_ref::<Decimal256Array>() {
        let value = arr.value(row);
        let precision = arr.precision();
        let scale = arr.scale();
        return decimal256_to_value(value, precision, scale);
    }

    Err(type_err(
        "Decimal128Array or Decimal256Array",
        "Decimal",
        array,
    ))
}

#[cfg(feature = "with-rust_decimal")]
fn decimal128_to_value(value: i128, precision: u8, scale: i8) -> Result<Value, ArrowError> {
    use sea_query::prelude::Decimal;

    if precision > 28 || scale > 28 || scale < 0 {
        #[cfg(feature = "with-bigdecimal")]
        return decimal128_to_bigdecimal(value, scale);

        #[cfg(not(feature = "with-bigdecimal"))]
        return Err(ArrowError::Unsupported(format!(
            "Decimal128 with precision={precision}, scale={scale} exceeds rust_decimal limits \
             (max precision=28, scale=0-28). Enable 'with-bigdecimal' feature for arbitrary precision."
        )));
    }

    let decimal = Decimal::from_i128_with_scale(value, scale as u32);
    Ok(Value::Decimal(Some(decimal)))
}

#[cfg(not(feature = "with-rust_decimal"))]
fn decimal128_to_value(value: i128, _precision: u8, scale: i8) -> Result<Value, ArrowError> {
    #[cfg(feature = "with-bigdecimal")]
    return decimal128_to_bigdecimal(value, scale);

    #[cfg(not(feature = "with-bigdecimal"))]
    Err(ArrowError::Unsupported(
        "Decimal128Array requires 'with-rust_decimal' or 'with-bigdecimal' feature".into(),
    ))
}

#[cfg(feature = "with-bigdecimal")]
fn decimal128_to_bigdecimal(value: i128, scale: i8) -> Result<Value, ArrowError> {
    use sea_query::prelude::bigdecimal::{BigDecimal, num_bigint::BigInt};

    let bigint = BigInt::from(value);
    let decimal = BigDecimal::new(bigint, scale as i64);
    Ok(Value::BigDecimal(Some(Box::new(decimal))))
}

fn decimal256_to_value(_value: i256, _precision: u8, _scale: i8) -> Result<Value, ArrowError> {
    #[cfg(feature = "with-bigdecimal")]
    {
        use sea_query::prelude::bigdecimal::{
            BigDecimal,
            num_bigint::{BigInt, Sign},
        };

        let bytes = _value.to_be_bytes();

        let (sign, magnitude) = if _value.is_negative() {
            let mut abs_bytes = [0u8; 32];
            let mut carry = true;

            for i in (0..32).rev() {
                abs_bytes[i] = !bytes[i];
                if carry {
                    if abs_bytes[i] == 255 {
                        abs_bytes[i] = 0;
                    } else {
                        abs_bytes[i] += 1;
                        carry = false;
                    }
                }
            }

            (Sign::Minus, abs_bytes.to_vec())
        } else if _value == i256::ZERO {
            (Sign::NoSign, vec![0])
        } else {
            let first_nonzero = bytes.iter().position(|&b| b != 0).unwrap_or(31);
            (Sign::Plus, bytes[first_nonzero..].to_vec())
        };

        let bigint = BigInt::from_bytes_be(sign, &magnitude);
        let decimal = BigDecimal::new(bigint, _scale as i64);
        return Ok(Value::BigDecimal(Some(Box::new(decimal))));
    }

    #[cfg(not(feature = "with-bigdecimal"))]
    Err(ArrowError::Unsupported(
        "Decimal256Array requires 'with-bigdecimal' feature for arbitrary precision support".into(),
    ))
}

// ---------------------------------------------------------------------------
// Chrono date/time helpers
// ---------------------------------------------------------------------------

#[cfg(feature = "with-chrono")]
fn arrow_to_chrono_date(array: &dyn Array, row: usize) -> Result<Value, ArrowError> {
    use sea_query::prelude::chrono::NaiveDate;
    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).expect("valid date");

    if let Some(arr) = array.as_any().downcast_ref::<Date32Array>() {
        let days = arr.value(row);
        let date = epoch
            .checked_add_signed(sea_query::prelude::chrono::Duration::days(days as i64))
            .ok_or_else(|| ArrowError::OutOfRange(format!("Date32 value {days} out of range")))?;
        Ok(Value::ChronoDate(Some(date)))
    } else if let Some(arr) = array.as_any().downcast_ref::<Date64Array>() {
        let ms = arr.value(row);
        let date = epoch
            .checked_add_signed(sea_query::prelude::chrono::Duration::milliseconds(ms))
            .ok_or_else(|| ArrowError::OutOfRange(format!("Date64 value {ms} out of range")))?;
        Ok(Value::ChronoDate(Some(date)))
    } else {
        Err(type_err("Date32Array or Date64Array", "Date", array))
    }
}

#[cfg(feature = "with-chrono")]
fn arrow_to_chrono_time(array: &dyn Array, row: usize) -> Result<Value, ArrowError> {
    use sea_query::prelude::chrono::NaiveTime;

    if let Some(arr) = array.as_any().downcast_ref::<Time32SecondArray>() {
        let secs = arr.value(row) as u32;
        let t = NaiveTime::from_num_seconds_from_midnight_opt(secs, 0)
            .ok_or_else(|| {
                ArrowError::OutOfRange(format!("Time32Second value {secs} out of range"))
            })?;
        Ok(Value::ChronoTime(Some(t)))
    } else if let Some(arr) = array.as_any().downcast_ref::<Time32MillisecondArray>() {
        let ms = arr.value(row);
        let secs = (ms / 1_000) as u32;
        let nanos = ((ms % 1_000) * 1_000_000) as u32;
        let t = NaiveTime::from_num_seconds_from_midnight_opt(secs, nanos)
            .ok_or_else(|| {
                ArrowError::OutOfRange(format!("Time32Millisecond value {ms} out of range"))
            })?;
        Ok(Value::ChronoTime(Some(t)))
    } else if let Some(arr) = array.as_any().downcast_ref::<Time64MicrosecondArray>() {
        let us = arr.value(row);
        let secs = (us / 1_000_000) as u32;
        let nanos = ((us % 1_000_000) * 1_000) as u32;
        let t = NaiveTime::from_num_seconds_from_midnight_opt(secs, nanos)
            .ok_or_else(|| {
                ArrowError::OutOfRange(format!("Time64Microsecond value {us} out of range"))
            })?;
        Ok(Value::ChronoTime(Some(t)))
    } else if let Some(arr) = array.as_any().downcast_ref::<Time64NanosecondArray>() {
        let ns = arr.value(row);
        let secs = (ns / 1_000_000_000) as u32;
        let nanos = (ns % 1_000_000_000) as u32;
        let t = NaiveTime::from_num_seconds_from_midnight_opt(secs, nanos)
            .ok_or_else(|| {
                ArrowError::OutOfRange(format!("Time64Nanosecond value {ns} out of range"))
            })?;
        Ok(Value::ChronoTime(Some(t)))
    } else {
        Err(type_err("Time32/Time64 Array", "Time", array))
    }
}

#[cfg(feature = "with-chrono")]
fn arrow_timestamp_to_utc(
    array: &dyn Array,
    row: usize,
) -> Result<sea_query::prelude::chrono::DateTime<sea_query::prelude::chrono::Utc>, ArrowError> {
    use sea_query::prelude::chrono::{DateTime, Utc};

    if let Some(arr) = array.as_any().downcast_ref::<TimestampSecondArray>() {
        DateTime::<Utc>::from_timestamp(arr.value(row), 0)
            .ok_or_else(|| ArrowError::OutOfRange("Timestamp seconds out of range".into()))
    } else if let Some(arr) = array.as_any().downcast_ref::<TimestampMillisecondArray>() {
        DateTime::<Utc>::from_timestamp_millis(arr.value(row))
            .ok_or_else(|| ArrowError::OutOfRange("Timestamp milliseconds out of range".into()))
    } else if let Some(arr) = array.as_any().downcast_ref::<TimestampMicrosecondArray>() {
        DateTime::<Utc>::from_timestamp_micros(arr.value(row))
            .ok_or_else(|| ArrowError::OutOfRange("Timestamp microseconds out of range".into()))
    } else if let Some(arr) = array.as_any().downcast_ref::<TimestampNanosecondArray>() {
        let nanos = arr.value(row);
        let secs = nanos.div_euclid(1_000_000_000);
        let nsec = nanos.rem_euclid(1_000_000_000) as u32;
        DateTime::<Utc>::from_timestamp(secs, nsec)
            .ok_or_else(|| ArrowError::OutOfRange("Timestamp nanoseconds out of range".into()))
    } else {
        Err(type_err(
            "TimestampSecond/Millisecond/Microsecond/NanosecondArray",
            "DateTime/Timestamp",
            array,
        ))
    }
}

#[cfg(feature = "with-chrono")]
fn arrow_to_chrono_datetime(array: &dyn Array, row: usize) -> Result<Value, ArrowError> {
    let dt = arrow_timestamp_to_utc(array, row)?;
    Ok(Value::ChronoDateTime(Some(dt.naive_utc())))
}

#[cfg(feature = "with-chrono")]
fn arrow_to_chrono_datetime_utc(array: &dyn Array, row: usize) -> Result<Value, ArrowError> {
    let dt = arrow_timestamp_to_utc(array, row)?;
    Ok(Value::ChronoDateTimeUtc(Some(dt)))
}

// ---------------------------------------------------------------------------
// Time-crate date/time helpers
// ---------------------------------------------------------------------------

#[cfg(feature = "with-time")]
fn arrow_to_time_date(array: &dyn Array, row: usize) -> Result<Value, ArrowError> {
    const EPOCH_JULIAN: i32 = 2_440_588;

    if let Some(arr) = array.as_any().downcast_ref::<Date32Array>() {
        let days = arr.value(row);
        let date = sea_query::prelude::time::Date::from_julian_day(EPOCH_JULIAN + days)
            .map_err(|e| ArrowError::OutOfRange(format!("Date32 value {days} out of range: {e}")))?;
        Ok(Value::TimeDate(Some(date)))
    } else if let Some(arr) = array.as_any().downcast_ref::<Date64Array>() {
        let ms = arr.value(row);
        let days = (ms / 86_400_000) as i32;
        let date = sea_query::prelude::time::Date::from_julian_day(EPOCH_JULIAN + days)
            .map_err(|e| ArrowError::OutOfRange(format!("Date64 value {ms} out of range: {e}")))?;
        Ok(Value::TimeDate(Some(date)))
    } else {
        Err(type_err("Date32Array or Date64Array", "Date", array))
    }
}

#[cfg(feature = "with-time")]
fn arrow_to_time_time(array: &dyn Array, row: usize) -> Result<Value, ArrowError> {
    if let Some(arr) = array.as_any().downcast_ref::<Time32SecondArray>() {
        let secs = arr.value(row);
        let t = sea_query::prelude::time::Time::from_hms(
            (secs / 3600) as u8,
            ((secs % 3600) / 60) as u8,
            (secs % 60) as u8,
        )
        .map_err(|e| {
            ArrowError::OutOfRange(format!("Time32Second value {secs} out of range: {e}"))
        })?;
        Ok(Value::TimeTime(Some(t)))
    } else if let Some(arr) = array.as_any().downcast_ref::<Time32MillisecondArray>() {
        let ms = arr.value(row);
        let total_secs = ms / 1_000;
        let nanos = ((ms % 1_000) * 1_000_000) as u32;
        let t = sea_query::prelude::time::Time::from_hms_nano(
            (total_secs / 3600) as u8,
            ((total_secs % 3600) / 60) as u8,
            (total_secs % 60) as u8,
            nanos,
        )
        .map_err(|e| {
            ArrowError::OutOfRange(format!("Time32Millisecond value {ms} out of range: {e}"))
        })?;
        Ok(Value::TimeTime(Some(t)))
    } else if let Some(arr) = array.as_any().downcast_ref::<Time64MicrosecondArray>() {
        let us = arr.value(row);
        let total_secs = us / 1_000_000;
        let nanos = ((us % 1_000_000) * 1_000) as u32;
        let t = sea_query::prelude::time::Time::from_hms_nano(
            (total_secs / 3600) as u8,
            ((total_secs % 3600) / 60) as u8,
            (total_secs % 60) as u8,
            nanos,
        )
        .map_err(|e| {
            ArrowError::OutOfRange(format!("Time64Microsecond value {us} out of range: {e}"))
        })?;
        Ok(Value::TimeTime(Some(t)))
    } else if let Some(arr) = array.as_any().downcast_ref::<Time64NanosecondArray>() {
        let ns = arr.value(row);
        let total_secs = ns / 1_000_000_000;
        let nanos = (ns % 1_000_000_000) as u32;
        let t = sea_query::prelude::time::Time::from_hms_nano(
            (total_secs / 3600) as u8,
            ((total_secs % 3600) / 60) as u8,
            (total_secs % 60) as u8,
            nanos,
        )
        .map_err(|e| {
            ArrowError::OutOfRange(format!("Time64Nanosecond value {ns} out of range: {e}"))
        })?;
        Ok(Value::TimeTime(Some(t)))
    } else {
        Err(type_err("Time32/Time64 Array", "Time", array))
    }
}

#[cfg(feature = "with-time")]
fn arrow_timestamp_to_offset_dt(
    array: &dyn Array,
    row: usize,
) -> Result<sea_query::prelude::time::OffsetDateTime, ArrowError> {
    if let Some(arr) = array.as_any().downcast_ref::<TimestampSecondArray>() {
        sea_query::prelude::time::OffsetDateTime::from_unix_timestamp(arr.value(row))
            .map_err(|e| ArrowError::OutOfRange(format!("Timestamp seconds out of range: {e}")))
    } else if let Some(arr) = array.as_any().downcast_ref::<TimestampMillisecondArray>() {
        let ms = arr.value(row);
        sea_query::prelude::time::OffsetDateTime::from_unix_timestamp_nanos(ms as i128 * 1_000_000)
            .map_err(|e| {
                ArrowError::OutOfRange(format!("Timestamp milliseconds out of range: {e}"))
            })
    } else if let Some(arr) = array.as_any().downcast_ref::<TimestampMicrosecondArray>() {
        let us = arr.value(row);
        sea_query::prelude::time::OffsetDateTime::from_unix_timestamp_nanos(us as i128 * 1_000)
            .map_err(|e| {
                ArrowError::OutOfRange(format!("Timestamp microseconds out of range: {e}"))
            })
    } else if let Some(arr) = array.as_any().downcast_ref::<TimestampNanosecondArray>() {
        sea_query::prelude::time::OffsetDateTime::from_unix_timestamp_nanos(arr.value(row) as i128)
            .map_err(|e| {
                ArrowError::OutOfRange(format!("Timestamp nanoseconds out of range: {e}"))
            })
    } else {
        Err(type_err(
            "TimestampSecond/Millisecond/Microsecond/NanosecondArray",
            "DateTime/Timestamp",
            array,
        ))
    }
}

#[cfg(feature = "with-time")]
fn arrow_to_time_datetime(array: &dyn Array, row: usize) -> Result<Value, ArrowError> {
    let odt = arrow_timestamp_to_offset_dt(array, row)?;
    Ok(Value::TimeDateTime(Some(sea_query::prelude::time::PrimitiveDateTime::new(
        odt.date(),
        odt.time(),
    ))))
}

#[cfg(feature = "with-time")]
fn arrow_to_time_datetime_tz(array: &dyn Array, row: usize) -> Result<Value, ArrowError> {
    let odt = arrow_timestamp_to_offset_dt(array, row)?;
    Ok(Value::TimeDateTimeWithTimeZone(Some(odt)))
}

// ---------------------------------------------------------------------------
// Null value helpers
// ---------------------------------------------------------------------------

fn null_value_for_type(col_type: &ColumnType) -> Value {
    match col_type {
        ColumnType::TinyInteger => Value::TinyInt(None),
        ColumnType::SmallInteger => Value::SmallInt(None),
        ColumnType::Integer => Value::Int(None),
        ColumnType::BigInteger => Value::BigInt(None),
        ColumnType::TinyUnsigned => Value::TinyUnsigned(None),
        ColumnType::SmallUnsigned => Value::SmallUnsigned(None),
        ColumnType::Unsigned => Value::Unsigned(None),
        ColumnType::BigUnsigned => Value::BigUnsigned(None),
        ColumnType::Float => Value::Float(None),
        ColumnType::Double => Value::Double(None),
        ColumnType::String(_) | ColumnType::Text | ColumnType::Char(_) => Value::String(None),
        ColumnType::Boolean => Value::Bool(None),
        #[cfg(feature = "with-rust_decimal")]
        ColumnType::Decimal(_) | ColumnType::Money(_) => Value::Decimal(None),
        #[cfg(all(feature = "with-bigdecimal", not(feature = "with-rust_decimal")))]
        ColumnType::Decimal(_) | ColumnType::Money(_) => Value::BigDecimal(None),
        #[cfg(feature = "with-chrono")]
        ColumnType::Date => Value::ChronoDate(None),
        #[cfg(feature = "with-chrono")]
        ColumnType::Time => Value::ChronoTime(None),
        #[cfg(feature = "with-chrono")]
        ColumnType::DateTime | ColumnType::Timestamp => Value::ChronoDateTime(None),
        #[cfg(feature = "with-chrono")]
        ColumnType::TimestampWithTimeZone => Value::ChronoDateTimeUtc(None),
        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
        ColumnType::Date => Value::TimeDate(None),
        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
        ColumnType::Time => Value::TimeTime(None),
        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
        ColumnType::DateTime | ColumnType::Timestamp => Value::TimeDateTime(None),
        #[cfg(all(feature = "with-time", not(feature = "with-chrono")))]
        ColumnType::TimestampWithTimeZone => Value::TimeDateTimeWithTimeZone(None),
        _ => Value::Int(None),
    }
}

/// Null values for the time crate variants, used by the alt-value fallback path.
#[cfg(all(feature = "with-chrono", feature = "with-time"))]
fn null_value_for_type_time(col_type: &ColumnType) -> Value {
    match col_type {
        ColumnType::Date => Value::TimeDate(None),
        ColumnType::Time => Value::TimeTime(None),
        ColumnType::DateTime | ColumnType::Timestamp => Value::TimeDateTime(None),
        ColumnType::TimestampWithTimeZone => Value::TimeDateTimeWithTimeZone(None),
        _ => null_value_for_type(col_type),
    }
}

// ---------------------------------------------------------------------------
// Value → Arrow
// ---------------------------------------------------------------------------

/// Convert a slice of optional [`Value`]s to an Arrow array matching the
/// target [`DataType`](arrow::datatypes::DataType).
///
/// `None` entries (from `ActiveValue::NotSet`) become null in the array.
/// `Some(Value::Variant(None))` (SQL NULL) also become null.
pub fn values_to_arrow_array(
    values: &[Option<Value>],
    data_type: &arrow::datatypes::DataType,
) -> Result<std::sync::Arc<dyn Array>, ArrowError> {
    use arrow::datatypes::{DataType, TimeUnit};
    use std::sync::Arc;

    match data_type {
        DataType::Int8 => {
            let arr: Int8Array = values
                .iter()
                .map(|v| match v {
                    Some(Value::TinyInt(inner)) => *inner,
                    _ => None,
                })
                .collect();
            Ok(Arc::new(arr))
        }
        DataType::Int16 => {
            let arr: Int16Array = values
                .iter()
                .map(|v| match v {
                    Some(Value::SmallInt(inner)) => *inner,
                    _ => None,
                })
                .collect();
            Ok(Arc::new(arr))
        }
        DataType::Int32 => {
            let arr: Int32Array = values
                .iter()
                .map(|v| match v {
                    Some(Value::Int(inner)) => *inner,
                    _ => None,
                })
                .collect();
            Ok(Arc::new(arr))
        }
        DataType::Int64 => {
            let arr: Int64Array = values
                .iter()
                .map(|v| match v {
                    Some(Value::BigInt(inner)) => *inner,
                    _ => None,
                })
                .collect();
            Ok(Arc::new(arr))
        }
        DataType::UInt8 => {
            let arr: UInt8Array = values
                .iter()
                .map(|v| match v {
                    Some(Value::TinyUnsigned(inner)) => *inner,
                    _ => None,
                })
                .collect();
            Ok(Arc::new(arr))
        }
        DataType::UInt16 => {
            let arr: UInt16Array = values
                .iter()
                .map(|v| match v {
                    Some(Value::SmallUnsigned(inner)) => *inner,
                    _ => None,
                })
                .collect();
            Ok(Arc::new(arr))
        }
        DataType::UInt32 => {
            let arr: UInt32Array = values
                .iter()
                .map(|v| match v {
                    Some(Value::Unsigned(inner)) => *inner,
                    _ => None,
                })
                .collect();
            Ok(Arc::new(arr))
        }
        DataType::UInt64 => {
            let arr: UInt64Array = values
                .iter()
                .map(|v| match v {
                    Some(Value::BigUnsigned(inner)) => *inner,
                    _ => None,
                })
                .collect();
            Ok(Arc::new(arr))
        }
        DataType::Float32 => {
            let arr: Float32Array = values
                .iter()
                .map(|v| match v {
                    Some(Value::Float(inner)) => *inner,
                    _ => None,
                })
                .collect();
            Ok(Arc::new(arr))
        }
        DataType::Float64 => {
            let arr: Float64Array = values
                .iter()
                .map(|v| match v {
                    Some(Value::Double(inner)) => *inner,
                    _ => None,
                })
                .collect();
            Ok(Arc::new(arr))
        }
        DataType::Boolean => {
            let arr: BooleanArray = values
                .iter()
                .map(|v| match v {
                    Some(Value::Bool(inner)) => *inner,
                    _ => None,
                })
                .collect();
            Ok(Arc::new(arr))
        }
        DataType::Utf8 => {
            let strs: Vec<Option<&str>> = values
                .iter()
                .map(|v| match v {
                    Some(Value::String(Some(s))) => Some(s.as_str()),
                    _ => None,
                })
                .collect();
            Ok(Arc::new(StringArray::from(strs)))
        }
        DataType::LargeUtf8 => {
            let strs: Vec<Option<&str>> = values
                .iter()
                .map(|v| match v {
                    Some(Value::String(Some(s))) => Some(s.as_str()),
                    _ => None,
                })
                .collect();
            Ok(Arc::new(LargeStringArray::from(strs)))
        }
        DataType::Binary => {
            let bufs: Vec<Option<&[u8]>> = values
                .iter()
                .map(|v| match v {
                    Some(Value::Bytes(Some(b))) => Some(b.as_slice()),
                    _ => None,
                })
                .collect();
            Ok(Arc::new(BinaryArray::from(bufs)))
        }
        DataType::Date32 => {
            let arr: Date32Array = values.iter().map(|v| extract_date32(v)).collect();
            Ok(Arc::new(arr))
        }
        DataType::Time32(unit) => {
            let vals: Vec<Option<i32>> = values.iter().map(|v| extract_time32(v, unit)).collect();
            let arr: Arc<dyn Array> = match unit {
                TimeUnit::Second => Arc::new(Time32SecondArray::from(vals)),
                TimeUnit::Millisecond => Arc::new(Time32MillisecondArray::from(vals)),
                _ => {
                    return Err(ArrowError::Unsupported(format!(
                        "Unsupported Time32 unit: {unit:?}"
                    )));
                }
            };
            Ok(arr)
        }
        DataType::Time64(unit) => {
            let vals: Vec<Option<i64>> = values.iter().map(|v| extract_time64(v, unit)).collect();
            let arr: Arc<dyn Array> = match unit {
                TimeUnit::Microsecond => Arc::new(Time64MicrosecondArray::from(vals)),
                TimeUnit::Nanosecond => Arc::new(Time64NanosecondArray::from(vals)),
                _ => {
                    return Err(ArrowError::Unsupported(format!(
                        "Unsupported Time64 unit: {unit:?}"
                    )));
                }
            };
            Ok(arr)
        }
        DataType::Timestamp(unit, tz) => {
            let vals: Vec<Option<i64>> =
                values.iter().map(|v| extract_timestamp(v, unit)).collect();
            let arr: Arc<dyn Array> = match unit {
                TimeUnit::Second => {
                    let mut a = TimestampSecondArray::from(vals);
                    if let Some(tz) = tz {
                        a = a.with_timezone(tz.as_ref());
                    }
                    Arc::new(a)
                }
                TimeUnit::Millisecond => {
                    let mut a = TimestampMillisecondArray::from(vals);
                    if let Some(tz) = tz {
                        a = a.with_timezone(tz.as_ref());
                    }
                    Arc::new(a)
                }
                TimeUnit::Microsecond => {
                    let mut a = TimestampMicrosecondArray::from(vals);
                    if let Some(tz) = tz {
                        a = a.with_timezone(tz.as_ref());
                    }
                    Arc::new(a)
                }
                TimeUnit::Nanosecond => {
                    let mut a = TimestampNanosecondArray::from(vals);
                    if let Some(tz) = tz {
                        a = a.with_timezone(tz.as_ref());
                    }
                    Arc::new(a)
                }
            };
            Ok(arr)
        }
        DataType::Decimal128(precision, scale) => {
            let arr: Decimal128Array = values
                .iter()
                .map(|v| extract_decimal128(v, *scale))
                .collect();
            let arr = arr
                .with_precision_and_scale(*precision, *scale)
                .map_err(|e| {
                    ArrowError::Unsupported(format!("Invalid Decimal128 precision/scale: {e}"))
                })?;
            Ok(Arc::new(arr))
        }
        DataType::Decimal256(precision, scale) => {
            let arr: Decimal256Array = values
                .iter()
                .map(|v| extract_decimal256(v, *scale))
                .collect();
            let arr = arr
                .with_precision_and_scale(*precision, *scale)
                .map_err(|e| {
                    ArrowError::Unsupported(format!("Invalid Decimal256 precision/scale: {e}"))
                })?;
            Ok(Arc::new(arr))
        }
        _ => Err(ArrowError::Unsupported(format!(
            "Unsupported Arrow DataType for to_arrow: {data_type:?}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// Date extraction helpers
// ---------------------------------------------------------------------------

fn extract_date32(v: &Option<Value>) -> Option<i32> {
    #[cfg(feature = "with-chrono")]
    if let Some(Value::ChronoDate(Some(d))) = v {
        let epoch = sea_query::prelude::chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        return Some((*d - epoch).num_days() as i32);
    }
    #[cfg(feature = "with-time")]
    if let Some(Value::TimeDate(Some(d))) = v {
        return Some(d.to_julian_day() - 2_440_588);
    }
    let _ = v;
    None
}

// ---------------------------------------------------------------------------
// Time extraction helpers
// ---------------------------------------------------------------------------

fn extract_time32(v: &Option<Value>, unit: &arrow::datatypes::TimeUnit) -> Option<i32> {
    use arrow::datatypes::TimeUnit;

    #[cfg(feature = "with-chrono")]
    if let Some(Value::ChronoTime(Some(t))) = v {
        use sea_query::prelude::chrono::Timelike;
        let secs = t.num_seconds_from_midnight() as i32;
        return match unit {
            TimeUnit::Second => Some(secs),
            TimeUnit::Millisecond => {
                let ms = (t.nanosecond() / 1_000_000) as i32;
                Some(secs * 1_000 + ms)
            }
            _ => None,
        };
    }
    #[cfg(feature = "with-time")]
    if let Some(Value::TimeTime(Some(t))) = v {
        let secs = (t.hour() as i32) * 3600 + (t.minute() as i32) * 60 + (t.second() as i32);
        return match unit {
            TimeUnit::Second => Some(secs),
            TimeUnit::Millisecond => {
                let ms = (t.nanosecond() / 1_000_000) as i32;
                Some(secs * 1_000 + ms)
            }
            _ => None,
        };
    }
    let _ = (v, unit);
    None
}

fn extract_time64(v: &Option<Value>, unit: &arrow::datatypes::TimeUnit) -> Option<i64> {
    use arrow::datatypes::TimeUnit;

    #[cfg(feature = "with-chrono")]
    if let Some(Value::ChronoTime(Some(t))) = v {
        use sea_query::prelude::chrono::Timelike;
        let secs = t.num_seconds_from_midnight() as i64;
        let nanos = (t.nanosecond() % 1_000_000_000) as i64;
        return match unit {
            TimeUnit::Microsecond => Some(secs * 1_000_000 + nanos / 1_000),
            TimeUnit::Nanosecond => Some(secs * 1_000_000_000 + nanos),
            _ => None,
        };
    }
    #[cfg(feature = "with-time")]
    if let Some(Value::TimeTime(Some(t))) = v {
        let secs = (t.hour() as i64) * 3600 + (t.minute() as i64) * 60 + (t.second() as i64);
        let nanos = t.nanosecond() as i64;
        return match unit {
            TimeUnit::Microsecond => Some(secs * 1_000_000 + nanos / 1_000),
            TimeUnit::Nanosecond => Some(secs * 1_000_000_000 + nanos),
            _ => None,
        };
    }
    let _ = (v, unit);
    None
}

// ---------------------------------------------------------------------------
// Timestamp extraction helpers
// ---------------------------------------------------------------------------

fn extract_timestamp(v: &Option<Value>, unit: &arrow::datatypes::TimeUnit) -> Option<i64> {
    use arrow::datatypes::TimeUnit;

    #[cfg(feature = "with-chrono")]
    {
        if let Some(Value::ChronoDateTime(Some(dt))) = v {
            let utc = dt.and_utc();
            return Some(match unit {
                TimeUnit::Second => utc.timestamp(),
                TimeUnit::Millisecond => utc.timestamp_millis(),
                TimeUnit::Microsecond => utc.timestamp_micros(),
                TimeUnit::Nanosecond => utc.timestamp_nanos_opt().unwrap_or(0),
            });
        }
        if let Some(Value::ChronoDateTimeUtc(Some(dt))) = v {
            return Some(match unit {
                TimeUnit::Second => dt.timestamp(),
                TimeUnit::Millisecond => dt.timestamp_millis(),
                TimeUnit::Microsecond => dt.timestamp_micros(),
                TimeUnit::Nanosecond => dt.timestamp_nanos_opt().unwrap_or(0),
            });
        }
    }
    #[cfg(feature = "with-time")]
    {
        if let Some(Value::TimeDateTime(Some(dt))) = v {
            let odt = dt.assume_utc();
            return Some(offset_dt_to_timestamp(&odt, unit));
        }
        if let Some(Value::TimeDateTimeWithTimeZone(Some(dt))) = v {
            return Some(offset_dt_to_timestamp(dt, unit));
        }
    }
    let _ = (v, unit);
    None
}

#[cfg(feature = "with-time")]
fn offset_dt_to_timestamp(dt: &sea_query::prelude::time::OffsetDateTime, unit: &arrow::datatypes::TimeUnit) -> i64 {
    use arrow::datatypes::TimeUnit;
    match unit {
        TimeUnit::Second => dt.unix_timestamp(),
        TimeUnit::Millisecond => (dt.unix_timestamp_nanos() / 1_000_000) as i64,
        TimeUnit::Microsecond => (dt.unix_timestamp_nanos() / 1_000) as i64,
        TimeUnit::Nanosecond => dt.unix_timestamp_nanos() as i64,
    }
}

// ---------------------------------------------------------------------------
// Decimal extraction helpers
// ---------------------------------------------------------------------------

fn extract_decimal128(v: &Option<Value>, target_scale: i8) -> Option<i128> {
    #[cfg(feature = "with-rust_decimal")]
    if let Some(Value::Decimal(Some(d))) = v {
        let mantissa = d.mantissa();
        let current_scale = d.scale() as i8;
        let scale_diff = target_scale - current_scale;
        return if scale_diff >= 0 {
            Some(mantissa * 10i128.pow(scale_diff as u32))
        } else {
            Some(mantissa / 10i128.pow((-scale_diff) as u32))
        };
    }
    #[cfg(feature = "with-bigdecimal")]
    if let Some(Value::BigDecimal(Some(d))) = v {
        return bigdecimal_to_i128(d, target_scale);
    }
    let _ = (v, target_scale);
    None
}

#[cfg(feature = "with-bigdecimal")]
fn bigdecimal_to_i128(d: &sea_query::prelude::bigdecimal::BigDecimal, target_scale: i8) -> Option<i128> {
    use sea_query::prelude::bigdecimal::ToPrimitive;

    let rescaled = d.clone().with_scale(target_scale as i64);
    let (digits, _) = rescaled.into_bigint_and_exponent();
    digits.to_i128()
}

fn extract_decimal256(v: &Option<Value>, target_scale: i8) -> Option<i256> {
    #[cfg(feature = "with-bigdecimal")]
    if let Some(Value::BigDecimal(Some(d))) = v {
        return bigdecimal_to_i256(d, target_scale);
    }
    #[cfg(feature = "with-rust_decimal")]
    if let Some(Value::Decimal(Some(d))) = v {
        let mantissa = d.mantissa();
        let current_scale = d.scale() as i8;
        let scale_diff = target_scale - current_scale;
        let scaled = if scale_diff >= 0 {
            mantissa * 10i128.pow(scale_diff as u32)
        } else {
            mantissa / 10i128.pow((-scale_diff) as u32)
        };
        return Some(i256::from_i128(scaled));
    }
    let _ = (v, target_scale);
    None
}

#[cfg(feature = "with-bigdecimal")]
fn bigdecimal_to_i256(d: &sea_query::prelude::bigdecimal::BigDecimal, target_scale: i8) -> Option<i256> {
    let rescaled = d.clone().with_scale(target_scale as i64);
    let (digits, _) = rescaled.into_bigint_and_exponent();
    bigint_to_i256(&digits)
}

#[cfg(feature = "with-bigdecimal")]
fn bigint_to_i256(bi: &sea_query::prelude::bigdecimal::num_bigint::BigInt) -> Option<i256> {
    use sea_query::prelude::bigdecimal::num_bigint::Sign;

    let (sign, bytes) = bi.to_bytes_be();
    if bytes.len() > 32 {
        return None;
    }

    let mut buf = [0u8; 32];
    let start = 32 - bytes.len();
    buf[start..].copy_from_slice(&bytes);

    let val = i256::from_be_bytes(buf);
    match sign {
        Sign::Minus => Some(val.wrapping_neg()),
        _ => Some(val),
    }
}
