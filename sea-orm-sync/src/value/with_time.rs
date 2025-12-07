use super::impl_timestamp;
use crate as sea_orm;
use crate::{DbErr, TryGetError, prelude::TimeDateTimeWithTimeZone};
use std::ops::{Deref, DerefMut};

/// A OffsetDateTime mapped to i64 in database
#[derive(derive_more::Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[debug("{_0:?}")]
pub struct TimeUnixTimestamp(pub TimeDateTimeWithTimeZone);

/// A OffsetDateTime mapped to i64 in database, but in milliseconds
#[derive(derive_more::Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[debug("{_0:?}")]
pub struct TimeUnixTimestampMillis(pub TimeDateTimeWithTimeZone);

impl_timestamp!(
    TimeUnixTimestamp,
    TimeDateTimeWithTimeZone,
    from_timestamp,
    to_timestamp
);

impl_timestamp!(
    TimeUnixTimestampMillis,
    TimeDateTimeWithTimeZone,
    from_timestamp_millis,
    to_timestamp_millis
);

fn from_timestamp(ts: i64) -> Option<TimeUnixTimestamp> {
    TimeDateTimeWithTimeZone::from_unix_timestamp(ts)
        .ok()
        .map(TimeUnixTimestamp)
}

fn to_timestamp(ts: TimeUnixTimestamp) -> i64 {
    ts.0.unix_timestamp()
}

fn from_timestamp_millis(ts: i64) -> Option<TimeUnixTimestampMillis> {
    TimeDateTimeWithTimeZone::from_unix_timestamp_nanos(ts as i128 * 1_000_000)
        .ok()
        .map(TimeUnixTimestampMillis)
}

fn to_timestamp_millis(ts: TimeUnixTimestampMillis) -> i64 {
    (ts.0.unix_timestamp_nanos() / 1_000_000) as i64
}
