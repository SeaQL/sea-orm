use super::impl_timestamp;
use crate as sea_orm;
use crate::{DbErr, TryGetError, prelude::ChronoDateTimeUtc};
use std::ops::{Deref, DerefMut};

/// A `DateTime<Utc>` stored as a Unix timestamp (seconds) `i64` in the database.
#[derive(derive_more::Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[debug("{_0:?}")]
pub struct ChronoUnixTimestamp(pub ChronoDateTimeUtc);

/// A `DateTime<Utc>` stored as a Unix timestamp in milliseconds (`i64`) in the database.
#[derive(derive_more::Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[debug("{_0:?}")]
pub struct ChronoUnixTimestampMillis(pub ChronoDateTimeUtc);

impl_timestamp!(
    ChronoUnixTimestamp,
    ChronoDateTimeUtc,
    from_timestamp,
    to_timestamp
);
super::impl_serde_with_i64!(ChronoUnixTimestamp, from_timestamp, to_timestamp);

impl_timestamp!(
    ChronoUnixTimestampMillis,
    ChronoDateTimeUtc,
    from_timestamp_millis,
    to_timestamp_millis
);
super::impl_serde_with_i64!(
    ChronoUnixTimestampMillis,
    from_timestamp_millis,
    to_timestamp_millis
);

fn from_timestamp(ts: i64) -> Option<ChronoUnixTimestamp> {
    ChronoDateTimeUtc::from_timestamp(ts, 0).map(ChronoUnixTimestamp)
}

fn to_timestamp(ts: ChronoUnixTimestamp) -> i64 {
    ts.0.timestamp()
}

fn from_timestamp_millis(ts: i64) -> Option<ChronoUnixTimestampMillis> {
    ChronoDateTimeUtc::from_timestamp_millis(ts).map(ChronoUnixTimestampMillis)
}

fn to_timestamp_millis(ts: ChronoUnixTimestampMillis) -> i64 {
    ts.0.timestamp_millis()
}
