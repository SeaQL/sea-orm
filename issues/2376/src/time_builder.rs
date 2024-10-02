
extern crate chrono;
use chrono::prelude::*;

pub struct TimeBuilder {
    time: DateTime<FixedOffset>,
}

impl TimeBuilder {
    pub fn now() -> Self {
        Self {
            time: Utc::now().fixed_offset(),
        }
    }

    pub fn from_string(str: &str) -> Self {
        Self {
            time: DateTime::parse_from_rfc3339(str).unwrap(),
        }
    }

    pub fn from_i64(timestamp: i64) -> Self {
        let time = DateTime::from_timestamp(timestamp, 0)
            .unwrap()
            .fixed_offset();

        Self { time }
    }

    pub fn to_string(&self) -> String {
        self.time.to_rfc3339()
    }

    pub fn to_i64(&self) -> i64 {
        self.time.timestamp()
    }

    pub fn is_near_now(&self) -> bool {
        self.is_near_offset_sec(1)
    }

    pub fn is_near_offset_sec(&self, offset_sec: i64) -> bool {
        let now = Utc::now();
        let diff = now.signed_duration_since(self.time).num_seconds().abs();
        diff <= offset_sec
    }
}

impl From<TimeBuilder> for String {
    fn from(time: TimeBuilder) -> Self {
        time.to_string()
    }
}

impl From<TimeBuilder> for i64 {
    fn from(time: TimeBuilder) -> Self {
        time.to_i64()
    }
}
