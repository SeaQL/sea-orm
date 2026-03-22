use super::impl_timestamp;
use crate as sea_orm;
use crate::{DbErr, TryGetError, prelude::JiffTimestamp};
use std::ops::{Deref, DerefMut};

/// A Jiff timestamp mapped to i64 in database
#[derive(derive_more::Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[debug("{_0:?}")]
pub struct JiffUnixTimestamp(pub JiffTimestamp);

/// A Jiff timestamp mapped to i64 in database, but in milliseconds
#[derive(derive_more::Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[debug("{_0:?}")]
pub struct JiffUnixTimestampMillis(pub JiffTimestamp);

impl_timestamp!(
    JiffUnixTimestamp,
    JiffTimestamp,
    from_timestamp,
    to_timestamp
);

impl_timestamp!(
    JiffUnixTimestampMillis,
    JiffTimestamp,
    from_timestamp_millis,
    to_timestamp_millis
);

fn from_timestamp(ts: i64) -> Option<JiffUnixTimestamp> {
    JiffTimestamp::from_second(ts).ok().map(JiffUnixTimestamp)
}

fn to_timestamp(ts: JiffUnixTimestamp) -> i64 {
    ts.0.as_second()
}

fn from_timestamp_millis(ts: i64) -> Option<JiffUnixTimestampMillis> {
    JiffTimestamp::from_millisecond(ts)
        .ok()
        .map(JiffUnixTimestampMillis)
}

fn to_timestamp_millis(ts: JiffUnixTimestampMillis) -> i64 {
    ts.0.as_millisecond()
}
