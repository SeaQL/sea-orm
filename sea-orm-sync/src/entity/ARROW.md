# Apache Arrow Type Support in SeaORM

This document explains Apache Arrow's decimal and timestamp formats and their integration with SeaORM.

## Arrow Decimal Types Overview

Apache Arrow provides fixed-point decimal types for representing precise numeric values with a specific precision and scale:

### 1. **Decimal64**
- **Storage**: 64-bit (8 bytes) fixed-point decimal
- **Precision**: Up to 18 decimal digits
- **Format**: `DataType::Decimal64(precision, scale)`
  - `precision` (u8): Total number of decimal digits (1-18)
  - `scale` (i8): Number of digits after decimal point (can be negative)
- **Use case**: Compact precision decimals that fit in an i64 (e.g., prices, exchange rates)
- **Example**: `Decimal64(10, 2)` represents numbers like `12345678.90`

### 2. **Decimal128**
- **Storage**: 128-bit (16 bytes) fixed-point decimal
- **Precision**: Up to 38 decimal digits
- **Format**: `DataType::Decimal128(precision, scale)`
  - `precision` (u8): Total number of decimal digits (1-38)
  - `scale` (i8): Number of digits after decimal point (can be negative)
- **Use case**: Standard precision decimals (e.g., financial calculations requiring > 18 digits)
- **Example**: `Decimal128(20, 4)` represents numbers like `9999999999999999.9999`

### 3. **Decimal256**
- **Storage**: 256-bit (32 bytes) fixed-point decimal
- **Precision**: Up to 76 decimal digits
- **Format**: `DataType::Decimal256(precision, scale)`
  - `precision` (u8): Total number of decimal digits (1-76)
  - `scale` (i8): Number of digits after decimal point (can be negative)
- **Use case**: High-precision scientific calculations, very large numbers
- **Example**: `Decimal256(76, 20)` for extreme precision requirements

### Precision vs Scale

- **Precision**: Total number of significant digits (before + after decimal point)
- **Scale**: Number of digits after the decimal point
  - Positive scale: digits after decimal (e.g., scale=2 → `123.45`)
  - Zero scale: integer (e.g., scale=0 → `12345`)
  - Negative scale: multiplier (e.g., scale=-2 → `12300` stored as `123`)

**Examples**:
```
Decimal64(10, 2)   → Can store: -99999999.99 to 99999999.99 (compact, i64)
Decimal64(5, 0)    → Can store: -99999 to 99999 (integers, compact)
Decimal128(20, 4)  → Can store: -9999999999999999.9999 to 9999999999999999.9999
Decimal128(10, 4)  → Can store: -999999.9999 to 999999.9999
Decimal256(38, 10) → High precision: up to 28 digits before, 10 after decimal
```

## SeaORM Decimal Support

SeaORM supports two Rust decimal libraries:

### 1. **rust_decimal** (feature: `with-rust_decimal`)
- Type: `rust_decimal::Decimal`
- Precision: 28-29 significant digits
- Scale: 0-28
- Storage: 128-bit
- Value type: `Value::Decimal`
- Best for: Most business applications, financial calculations

### 2. **bigdecimal** (feature: `with-bigdecimal`)
- Type: `bigdecimal::BigDecimal`
- Precision: Arbitrary (limited by memory)
- Scale: Arbitrary
- Storage: Variable (uses BigInt internally)
- Value type: `Value::BigDecimal`
- Best for: Arbitrary precision requirements, scientific computing

## Arrow → SeaORM Mapping

### Decimal64Array → rust_decimal::Decimal
- **When**: Feature `with-rust_decimal` is enabled
- **Limitation**: Precision ≤ 18 (fits in i64)
- **Conversion**: Cast i64 to i128, then `Decimal::from_i128_with_scale()`
- **Column Type**: `ColumnType::Decimal(Some((precision, scale)))`

### Decimal64Array → bigdecimal::BigDecimal
- **When**: Feature `with-bigdecimal` is enabled (fallback if rust_decimal not available)
- **Conversion**: Convert i64 via BigInt
- **Column Type**: `ColumnType::Decimal(Some((precision, scale)))`

### Decimal128Array → rust_decimal::Decimal
- **When**: Feature `with-rust_decimal` is enabled
- **Limitation**: Precision ≤ 28, Scale ≤ 28
- **Conversion**: Direct mapping using `i128` to `Decimal::from_i128_with_scale()`
- **Column Type**: `ColumnType::Decimal(Some((precision, scale)))`

### Decimal128Array → bigdecimal::BigDecimal
- **When**: Feature `with-bigdecimal` is enabled (fallback if rust_decimal fails or not available)
- **Limitation**: None (arbitrary precision)
- **Conversion**: Convert via BigInt
- **Column Type**: `ColumnType::Decimal(Some((precision, scale)))` or `ColumnType::Money(_)`

### Decimal256Array → bigdecimal::BigDecimal
- **When**: Feature `with-bigdecimal` is enabled
- **Required**: BigDecimal for precision > 38
- **Conversion**: Convert via byte array to BigInt, then apply scale
- **Column Type**: `ColumnType::Decimal(Some((precision, scale)))`

## Implementation Strategy

1. **Decimal64Array**:
   - Try `with-rust_decimal` first (always fits, precision ≤ 18)
   - Fallback to `with-bigdecimal` if needed
   - Return type error if neither feature is enabled

2. **Decimal128Array**:
   - Try `with-rust_decimal` first (if precision/scale fit)
   - Fallback to `with-bigdecimal` if needed
   - Return type error if neither feature is enabled

3. **Decimal256Array**:
   - Requires `with-bigdecimal` (rust_decimal can't handle precision > 28)
   - Convert byte representation to BigInt
   - Apply scale to create BigDecimal

4. **Null Handling**:
   - Return `Value::Decimal(None)` or `Value::BigDecimal(None)` for null values

---

# Apache Arrow Timestamp Types Support

Arrow provides several temporal types for representing dates, times, and timestamps with varying precision.

## Arrow Temporal Types Overview

### Date Types

#### 1. **Date32**
- **Storage**: 32-bit signed integer
- **Unit**: Days since Unix epoch (1970-01-01)
- **Format**: `DataType::Date32`
- **Range**: Approximately ±5.8 million years from epoch
- **Use case**: Calendar dates without time component

#### 2. **Date64**
- **Storage**: 64-bit signed integer
- **Unit**: Milliseconds since Unix epoch
- **Format**: `DataType::Date64`
- **Range**: Much larger than Date32
- **Use case**: Dates with millisecond precision (though time is typically zeroed)

### Time Types

#### 1. **Time32**
- **Storage**: 32-bit signed integer
- **Units**: Second or Millisecond
- **Variants**:
  - `Time32(TimeUnit::Second)` - seconds since midnight
  - `Time32(TimeUnit::Millisecond)` - milliseconds since midnight
- **Range**: 0 to 86,399 seconds (00:00:00 to 23:59:59)
- **Use case**: Time of day without date

#### 2. **Time64**
- **Storage**: 64-bit signed integer
- **Units**: Microsecond or Nanosecond
- **Variants**:
  - `Time64(TimeUnit::Microsecond)` - microseconds since midnight
  - `Time64(TimeUnit::Nanosecond)` - nanoseconds since midnight
- **Range**: 0 to 86,399,999,999,999 nanoseconds
- **Use case**: High-precision time of day

### Timestamp Types

**Timestamp** types represent absolute points in time with optional timezone.

- **Storage**: 64-bit signed integer
- **Units**: Second, Millisecond, Microsecond, or Nanosecond
- **Timezone**: Optional timezone string (e.g., "UTC", "America/New_York")
- **Format**: `Timestamp(TimeUnit, Option<String>)`

**Variants**:
```rust
DataType::Timestamp(TimeUnit::Second, None)        // No timezone
DataType::Timestamp(TimeUnit::Millisecond, None)   // No timezone
DataType::Timestamp(TimeUnit::Microsecond, None)   // No timezone
DataType::Timestamp(TimeUnit::Nanosecond, None)    // No timezone

DataType::Timestamp(TimeUnit::Second, Some("UTC".into()))      // With timezone
DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())) // With timezone
DataType::Timestamp(TimeUnit::Nanosecond, Some("UTC".into()))  // With timezone
```

**TimeUnit Precision**:
- **Second**: 1 second precision (1,000,000,000 ns)
- **Millisecond**: 1 millisecond precision (1,000,000 ns)
- **Microsecond**: 1 microsecond precision (1,000 ns)
- **Nanosecond**: 1 nanosecond precision (highest)

## SeaORM Temporal Type Support

SeaORM supports two Rust datetime libraries for temporal types:

### 1. **chrono** (feature: `with-chrono`) - Preferred
- Type Mappings:
  - `chrono::NaiveDate` - Date without timezone
  - `chrono::NaiveTime` - Time without date/timezone
  - `chrono::NaiveDateTime` - DateTime without timezone
  - `chrono::DateTime<Utc>` - DateTime with UTC timezone
- Value types:
  - `Value::ChronoDate(Option<NaiveDate>)`
  - `Value::ChronoTime(Option<NaiveTime>)`
  - `Value::ChronoDateTime(Option<NaiveDateTime>)`
  - `Value::ChronoDateTimeUtc(Option<DateTime<Utc>>)`
- Best for: Most applications needing date/time support

### 2. **time** (feature: `with-time`) - Alternative
- Type Mappings:
  - `time::Date` - Date without timezone
  - `time::Time` - Time without date/timezone
  - `time::PrimitiveDateTime` - DateTime without timezone
  - `time::OffsetDateTime` - DateTime with timezone offset
- Value types:
  - `Value::TimeDate(Option<Date>)`
  - `Value::TimeTime(Option<Time>)`
  - `Value::TimeDateTime(Option<PrimitiveDateTime>)`
  - `Value::TimeDateTimeWithTimeZone(Option<OffsetDateTime>)`
- Best for: Projects preferring the `time` crate ecosystem

### Feature Priority
- **Both enabled**: Prefers `chrono`, with automatic fallback to `time` if type mismatch
- **Only chrono**: Uses chrono types exclusively
- **Only time**: Uses time crate types exclusively

## Arrow → SeaORM Timestamp Mapping

### Date32/Date64Array → Date

**chrono**:
```rust
Date32Array → chrono::NaiveDate
Date64Array → chrono::NaiveDate
```
- Conversion: Calculate days from epoch (1970-01-01) and add/subtract

**time**:
```rust
Date32Array → time::Date
Date64Array → time::Date
```
- Conversion: Julian day calculation (Unix epoch = Julian day 2,440,588)

### Time32/Time64Array → Time

**chrono**:
```rust
Time32(Second)      → chrono::NaiveTime
Time32(Millisecond) → chrono::NaiveTime
Time64(Microsecond) → chrono::NaiveTime
Time64(Nanosecond)  → chrono::NaiveTime
```
- Conversion: Break down time units into (hours, minutes, seconds, nanoseconds)

**time**:
```rust
Time32(Second)      → time::Time
Time32(Millisecond) → time::Time
Time64(Microsecond) → time::Time
Time64(Nanosecond)  → time::Time
```
- Conversion: Extract hours, minutes, seconds, and nanoseconds from total time value

### TimestampArray → DateTime

**Without Timezone**:
- **Arrow**: `Timestamp(TimeUnit, None)`
- **chrono**: `→ chrono::NaiveDateTime`
- **time**: `→ time::PrimitiveDateTime`
- **Column Type**: `ColumnType::DateTime` or `ColumnType::Timestamp`

**With Timezone**:
- **Arrow**: `Timestamp(TimeUnit, Some(tz))`
- **chrono**: `→ chrono::DateTime<Utc>`
- **time**: `→ time::OffsetDateTime`
- **Column Type**: `ColumnType::TimestampWithTimeZone`

### Conversion Details by TimeUnit

| TimeUnit | Conversion Method | Precision |
|----------|-------------------|-----------|
| **Second** | `from_timestamp(secs, 0)` | 1 second |
| **Millisecond** | `from_timestamp_millis(ms)` | 1 millisecond |
| **Microsecond** | `from_timestamp_micros(us)` | 1 microsecond |
| **Nanosecond** | `from_timestamp_nanos(ns)` or `from_timestamp(secs, nsecs)` | 1 nanosecond |

## References

- [Arrow DataType Documentation](https://docs.rs/arrow/latest/arrow/datatypes/enum.DataType.html)
- [Arrow Decimal Module](https://arrow.apache.org/rust/arrow_data/decimal/index.html)
- [Decimal256Type](https://arrow.apache.org/rust/arrow/datatypes/struct.Decimal256Type.html)
- [Arrow Temporal Types](https://arrow.apache.org/docs/python/api/datatypes.html#temporal-types)
