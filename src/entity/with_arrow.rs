use crate::{ColumnType, DbErr};
use arrow::array::*;
use arrow::datatypes::i256;
use sea_query::Value;

/// Extract a [`Value`] from an Arrow array at the given row index,
/// based on the expected [`ColumnType`] from the entity definition.
///
/// For date/time column types, this produces chrono `Value` variants when
/// the `with-chrono` feature is enabled, or time-crate variants when only
/// `with-time` is enabled.
pub(crate) fn arrow_array_to_value(
    array: &dyn Array,
    col_type: &ColumnType,
    row: usize,
) -> Result<Value, DbErr> {
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

        _ => Err(DbErr::Type(format!(
            "Unsupported column type for Arrow conversion: {col_type:?}"
        ))),
    }
}

/// When both `with-chrono` and `with-time` are enabled, this provides the
/// time-crate alternative for date/time columns. Called as a fallback when
/// the chrono Value variant doesn't match the model's field type.
#[cfg(all(feature = "with-chrono", feature = "with-time"))]
pub(crate) fn arrow_array_to_value_alt(
    array: &dyn Array,
    col_type: &ColumnType,
    row: usize,
) -> Result<Option<Value>, DbErr> {
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

/// Convert Arrow Decimal128Array or Decimal256Array to a decimal Value.
/// Prefers rust_decimal when available and precision fits, otherwise bigdecimal.
fn arrow_to_decimal(array: &dyn Array, row: usize) -> Result<Value, DbErr> {
    // Try Decimal128Array first
    if let Some(arr) = array.as_any().downcast_ref::<Decimal128Array>() {
        let value = arr.value(row);
        let precision = arr.precision();
        let scale = arr.scale();
        return decimal128_to_value(value, precision, scale);
    }

    // Try Decimal256Array
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
fn decimal128_to_value(value: i128, precision: u8, scale: i8) -> Result<Value, DbErr> {
    use rust_decimal::Decimal;

    // rust_decimal supports up to 28 digits precision and 28 scale
    if precision > 28 || scale > 28 || scale < 0 {
        // If rust_decimal can't handle it, try bigdecimal as fallback
        #[cfg(feature = "with-bigdecimal")]
        return decimal128_to_bigdecimal(value, scale);

        #[cfg(not(feature = "with-bigdecimal"))]
        return Err(DbErr::Type(format!(
            "Decimal128 with precision={precision}, scale={scale} exceeds rust_decimal limits (max precision=28, scale=0-28). Enable 'with-bigdecimal' feature for arbitrary precision."
        )));
    }

    let decimal = Decimal::from_i128_with_scale(value, scale as u32);
    Ok(Value::Decimal(Some(decimal)))
}

#[cfg(not(feature = "with-rust_decimal"))]
fn decimal128_to_value(value: i128, _precision: u8, scale: i8) -> Result<Value, DbErr> {
    #[cfg(feature = "with-bigdecimal")]
    return decimal128_to_bigdecimal(value, scale);

    #[cfg(not(feature = "with-bigdecimal"))]
    Err(DbErr::Type(
        "Decimal128Array requires 'with-rust_decimal' or 'with-bigdecimal' feature".into(),
    ))
}

#[cfg(feature = "with-bigdecimal")]
fn decimal128_to_bigdecimal(value: i128, scale: i8) -> Result<Value, DbErr> {
    use bigdecimal::{BigDecimal, num_bigint::BigInt};

    let bigint = BigInt::from(value);
    let decimal = BigDecimal::new(bigint, scale as i64);
    Ok(Value::BigDecimal(Some(Box::new(decimal))))
}

fn decimal256_to_value(_value: i256, _precision: u8, _scale: i8) -> Result<Value, DbErr> {
    #[cfg(feature = "with-bigdecimal")]
    {
        use bigdecimal::{
            BigDecimal,
            num_bigint::{BigInt, Sign},
        };

        // Convert i256 to BigInt via byte representation
        let bytes = _value.to_be_bytes();

        // Determine sign and magnitude
        let (sign, magnitude) = if _value.is_negative() {
            // For negative numbers, we need to compute two's complement
            let mut abs_bytes = [0u8; 32];
            let mut carry = true;

            // Invert bits and add 1 (two's complement)
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
            // Positive: strip leading zeros
            let first_nonzero = bytes.iter().position(|&b| b != 0).unwrap_or(31);
            (Sign::Plus, bytes[first_nonzero..].to_vec())
        };

        let bigint = BigInt::from_bytes_be(sign, &magnitude);
        let decimal = BigDecimal::new(bigint, _scale as i64);
        Ok(Value::BigDecimal(Some(Box::new(decimal))))
    }

    #[cfg(not(feature = "with-bigdecimal"))]
    Err(DbErr::Type(
        "Decimal256Array requires 'with-bigdecimal' feature for arbitrary precision support".into(),
    ))
}

#[cfg(feature = "with-chrono")]
fn arrow_to_chrono_date(array: &dyn Array, row: usize) -> Result<Value, DbErr> {
    use chrono::NaiveDate;
    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).expect("valid date");

    if let Some(arr) = array.as_any().downcast_ref::<Date32Array>() {
        let days = arr.value(row);
        let date = epoch
            .checked_add_signed(chrono::Duration::days(days as i64))
            .ok_or_else(|| DbErr::Type(format!("Date32 value {days} out of range")))?;
        Ok(Value::ChronoDate(Some(date)))
    } else if let Some(arr) = array.as_any().downcast_ref::<Date64Array>() {
        let ms = arr.value(row);
        let date = epoch
            .checked_add_signed(chrono::Duration::milliseconds(ms))
            .ok_or_else(|| DbErr::Type(format!("Date64 value {ms} out of range")))?;
        Ok(Value::ChronoDate(Some(date)))
    } else {
        Err(type_err("Date32Array or Date64Array", "Date", array))
    }
}

#[cfg(feature = "with-chrono")]
fn arrow_to_chrono_time(array: &dyn Array, row: usize) -> Result<Value, DbErr> {
    use chrono::NaiveTime;

    if let Some(arr) = array.as_any().downcast_ref::<Time32SecondArray>() {
        let secs = arr.value(row) as u32;
        let t = NaiveTime::from_num_seconds_from_midnight_opt(secs, 0)
            .ok_or_else(|| DbErr::Type(format!("Time32Second value {secs} out of range")))?;
        Ok(Value::ChronoTime(Some(t)))
    } else if let Some(arr) = array.as_any().downcast_ref::<Time32MillisecondArray>() {
        let ms = arr.value(row);
        let secs = (ms / 1_000) as u32;
        let nanos = ((ms % 1_000) * 1_000_000) as u32;
        let t = NaiveTime::from_num_seconds_from_midnight_opt(secs, nanos)
            .ok_or_else(|| DbErr::Type(format!("Time32Millisecond value {ms} out of range")))?;
        Ok(Value::ChronoTime(Some(t)))
    } else if let Some(arr) = array.as_any().downcast_ref::<Time64MicrosecondArray>() {
        let us = arr.value(row);
        let secs = (us / 1_000_000) as u32;
        let nanos = ((us % 1_000_000) * 1_000) as u32;
        let t = NaiveTime::from_num_seconds_from_midnight_opt(secs, nanos)
            .ok_or_else(|| DbErr::Type(format!("Time64Microsecond value {us} out of range")))?;
        Ok(Value::ChronoTime(Some(t)))
    } else if let Some(arr) = array.as_any().downcast_ref::<Time64NanosecondArray>() {
        let ns = arr.value(row);
        let secs = (ns / 1_000_000_000) as u32;
        let nanos = (ns % 1_000_000_000) as u32;
        let t = NaiveTime::from_num_seconds_from_midnight_opt(secs, nanos)
            .ok_or_else(|| DbErr::Type(format!("Time64Nanosecond value {ns} out of range")))?;
        Ok(Value::ChronoTime(Some(t)))
    } else {
        Err(type_err("Time32/Time64 Array", "Time", array))
    }
}

#[cfg(feature = "with-chrono")]
fn arrow_timestamp_to_utc(
    array: &dyn Array,
    row: usize,
) -> Result<chrono::DateTime<chrono::Utc>, DbErr> {
    use chrono::{DateTime, Utc};

    if let Some(arr) = array.as_any().downcast_ref::<TimestampSecondArray>() {
        DateTime::<Utc>::from_timestamp(arr.value(row), 0)
            .ok_or_else(|| DbErr::Type("Timestamp seconds out of range".into()))
    } else if let Some(arr) = array.as_any().downcast_ref::<TimestampMillisecondArray>() {
        DateTime::<Utc>::from_timestamp_millis(arr.value(row))
            .ok_or_else(|| DbErr::Type("Timestamp milliseconds out of range".into()))
    } else if let Some(arr) = array.as_any().downcast_ref::<TimestampMicrosecondArray>() {
        DateTime::<Utc>::from_timestamp_micros(arr.value(row))
            .ok_or_else(|| DbErr::Type("Timestamp microseconds out of range".into()))
    } else if let Some(arr) = array.as_any().downcast_ref::<TimestampNanosecondArray>() {
        let nanos = arr.value(row);
        let secs = nanos.div_euclid(1_000_000_000);
        let nsec = nanos.rem_euclid(1_000_000_000) as u32;
        DateTime::<Utc>::from_timestamp(secs, nsec)
            .ok_or_else(|| DbErr::Type("Timestamp nanoseconds out of range".into()))
    } else {
        Err(type_err(
            "TimestampSecond/Millisecond/Microsecond/NanosecondArray",
            "DateTime/Timestamp",
            array,
        ))
    }
}

#[cfg(feature = "with-chrono")]
fn arrow_to_chrono_datetime(array: &dyn Array, row: usize) -> Result<Value, DbErr> {
    let dt = arrow_timestamp_to_utc(array, row)?;
    Ok(Value::ChronoDateTime(Some(dt.naive_utc())))
}

#[cfg(feature = "with-chrono")]
fn arrow_to_chrono_datetime_utc(array: &dyn Array, row: usize) -> Result<Value, DbErr> {
    let dt = arrow_timestamp_to_utc(array, row)?;
    Ok(Value::ChronoDateTimeUtc(Some(dt)))
}

#[cfg(feature = "with-time")]
fn arrow_to_time_date(array: &dyn Array, row: usize) -> Result<Value, DbErr> {
    // Unix epoch is Julian day 2_440_588
    const EPOCH_JULIAN: i32 = 2_440_588;

    if let Some(arr) = array.as_any().downcast_ref::<Date32Array>() {
        let days = arr.value(row);
        let date = time::Date::from_julian_day(EPOCH_JULIAN + days)
            .map_err(|e| DbErr::Type(format!("Date32 value {days} out of range: {e}")))?;
        Ok(Value::TimeDate(Some(date)))
    } else if let Some(arr) = array.as_any().downcast_ref::<Date64Array>() {
        let ms = arr.value(row);
        let days = (ms / 86_400_000) as i32;
        let date = time::Date::from_julian_day(EPOCH_JULIAN + days)
            .map_err(|e| DbErr::Type(format!("Date64 value {ms} out of range: {e}")))?;
        Ok(Value::TimeDate(Some(date)))
    } else {
        Err(type_err("Date32Array or Date64Array", "Date", array))
    }
}

#[cfg(feature = "with-time")]
fn arrow_to_time_time(array: &dyn Array, row: usize) -> Result<Value, DbErr> {
    if let Some(arr) = array.as_any().downcast_ref::<Time32SecondArray>() {
        let secs = arr.value(row);
        let t = time::Time::from_hms(
            (secs / 3600) as u8,
            ((secs % 3600) / 60) as u8,
            (secs % 60) as u8,
        )
        .map_err(|e| DbErr::Type(format!("Time32Second value {secs} out of range: {e}")))?;
        Ok(Value::TimeTime(Some(t)))
    } else if let Some(arr) = array.as_any().downcast_ref::<Time32MillisecondArray>() {
        let ms = arr.value(row);
        let total_secs = ms / 1_000;
        let nanos = ((ms % 1_000) * 1_000_000) as u32;
        let t = time::Time::from_hms_nano(
            (total_secs / 3600) as u8,
            ((total_secs % 3600) / 60) as u8,
            (total_secs % 60) as u8,
            nanos,
        )
        .map_err(|e| DbErr::Type(format!("Time32Millisecond value {ms} out of range: {e}")))?;
        Ok(Value::TimeTime(Some(t)))
    } else if let Some(arr) = array.as_any().downcast_ref::<Time64MicrosecondArray>() {
        let us = arr.value(row);
        let total_secs = us / 1_000_000;
        let nanos = ((us % 1_000_000) * 1_000) as u32;
        let t = time::Time::from_hms_nano(
            (total_secs / 3600) as u8,
            ((total_secs % 3600) / 60) as u8,
            (total_secs % 60) as u8,
            nanos,
        )
        .map_err(|e| DbErr::Type(format!("Time64Microsecond value {us} out of range: {e}")))?;
        Ok(Value::TimeTime(Some(t)))
    } else if let Some(arr) = array.as_any().downcast_ref::<Time64NanosecondArray>() {
        let ns = arr.value(row);
        let total_secs = ns / 1_000_000_000;
        let nanos = (ns % 1_000_000_000) as u32;
        let t = time::Time::from_hms_nano(
            (total_secs / 3600) as u8,
            ((total_secs % 3600) / 60) as u8,
            (total_secs % 60) as u8,
            nanos,
        )
        .map_err(|e| DbErr::Type(format!("Time64Nanosecond value {ns} out of range: {e}")))?;
        Ok(Value::TimeTime(Some(t)))
    } else {
        Err(type_err("Time32/Time64 Array", "Time", array))
    }
}

#[cfg(feature = "with-time")]
fn arrow_timestamp_to_offset_dt(
    array: &dyn Array,
    row: usize,
) -> Result<time::OffsetDateTime, DbErr> {
    if let Some(arr) = array.as_any().downcast_ref::<TimestampSecondArray>() {
        time::OffsetDateTime::from_unix_timestamp(arr.value(row))
            .map_err(|e| DbErr::Type(format!("Timestamp seconds out of range: {e}")))
    } else if let Some(arr) = array.as_any().downcast_ref::<TimestampMillisecondArray>() {
        let ms = arr.value(row);
        time::OffsetDateTime::from_unix_timestamp_nanos(ms as i128 * 1_000_000)
            .map_err(|e| DbErr::Type(format!("Timestamp milliseconds out of range: {e}")))
    } else if let Some(arr) = array.as_any().downcast_ref::<TimestampMicrosecondArray>() {
        let us = arr.value(row);
        time::OffsetDateTime::from_unix_timestamp_nanos(us as i128 * 1_000)
            .map_err(|e| DbErr::Type(format!("Timestamp microseconds out of range: {e}")))
    } else if let Some(arr) = array.as_any().downcast_ref::<TimestampNanosecondArray>() {
        time::OffsetDateTime::from_unix_timestamp_nanos(arr.value(row) as i128)
            .map_err(|e| DbErr::Type(format!("Timestamp nanoseconds out of range: {e}")))
    } else {
        Err(type_err(
            "TimestampSecond/Millisecond/Microsecond/NanosecondArray",
            "DateTime/Timestamp",
            array,
        ))
    }
}

#[cfg(feature = "with-time")]
fn arrow_to_time_datetime(array: &dyn Array, row: usize) -> Result<Value, DbErr> {
    let odt = arrow_timestamp_to_offset_dt(array, row)?;
    Ok(Value::TimeDateTime(Some(time::PrimitiveDateTime::new(
        odt.date(),
        odt.time(),
    ))))
}

#[cfg(feature = "with-time")]
fn arrow_to_time_datetime_tz(array: &dyn Array, row: usize) -> Result<Value, DbErr> {
    let odt = arrow_timestamp_to_offset_dt(array, row)?;
    Ok(Value::TimeDateTimeWithTimeZone(Some(odt)))
}

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

fn type_err(expected: &str, col_type: &str, array: &dyn Array) -> DbErr {
    DbErr::Type(format!(
        "Expected {expected} for column type {col_type}, got Arrow type {:?}",
        array.data_type()
    ))
}

/// Returns true for ColumnTypes that may need a chrono→time fallback.
pub(crate) fn is_datetime_column(col_type: &ColumnType) -> bool {
    matches!(
        col_type,
        ColumnType::Date
            | ColumnType::Time
            | ColumnType::DateTime
            | ColumnType::Timestamp
            | ColumnType::TimestampWithTimeZone
    )
}
