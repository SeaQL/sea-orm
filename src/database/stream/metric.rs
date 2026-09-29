#[cfg(not(feature = "sync"))]
use std::time::Instant;
use std::{pin::Pin, task::Poll, time::Duration};

use futures_util::Stream;

use crate::{DbErr, QueryResult, Statement};

#[cfg(not(feature = "sync"))]
type PinBoxStream<'a> = Pin<Box<dyn Stream<Item = Result<QueryResult, DbErr>> + 'a + Send>>;
#[cfg(feature = "sync")]
type PinBoxStream<'a> = Box<dyn Iterator<Item = Result<QueryResult, DbErr>> + 'a>;

pub(crate) struct MetricStream<'a> {
    metric_callback: &'a Option<crate::metric::Callback>,
    stmt: &'a Statement,
    elapsed: Option<Duration>,
    #[cfg(not(feature = "sync"))]
    waiting_since: Option<Instant>,
    failed: bool,
    stream: PinBoxStream<'a>,
}

impl<'a> MetricStream<'a> {
    #[allow(dead_code)]
    pub(crate) fn new<S>(
        metric_callback: &'a Option<crate::metric::Callback>,
        stmt: &'a Statement,
        elapsed: Option<Duration>,
        stream: S,
    ) -> Self
    where
        S: Stream<Item = Result<QueryResult, DbErr>> + 'a + Send,
    {
        MetricStream {
            metric_callback,
            stmt,
            elapsed,
            #[cfg(not(feature = "sync"))]
            waiting_since: None,
            failed: false,
            stream: Box::pin(stream),
        }
    }
}

#[cfg(not(feature = "sync"))]
impl Stream for MetricStream<'_> {
    type Item = Result<QueryResult, DbErr>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if let (Some(waiting_since), Some(elapsed)) = (this.waiting_since.take(), &mut this.elapsed)
        {
            *elapsed += waiting_since.elapsed();
        }
        let _start = this
            .metric_callback
            .is_some()
            .then(std::time::SystemTime::now);
        let res = Pin::new(&mut this.stream).poll_next(cx);
        if let (Some(_start), Some(elapsed)) = (_start, &mut this.elapsed) {
            *elapsed += _start.elapsed().unwrap_or_default();
            if res.is_pending() {
                // Include database waits, but not time spent decoding or consuming a row.
                this.waiting_since = Some(Instant::now());
            }
        }
        this.failed |= matches!(res, Poll::Ready(Some(Err(_))));
        res
    }
}

#[cfg(feature = "sync")]
impl Iterator for MetricStream<'_> {
    type Item = Result<QueryResult, DbErr>;

    fn next(&mut self) -> Option<Self::Item> {
        let _start = self
            .metric_callback
            .is_some()
            .then(std::time::SystemTime::now);
        let res = self.stream.next();
        if let (Some(_start), Some(elapsed)) = (_start, &mut self.elapsed) {
            *elapsed += _start.elapsed().unwrap_or_default();
        }
        self.failed |= matches!(res, Some(Err(_)));
        res
    }
}

impl Drop for MetricStream<'_> {
    fn drop(&mut self) {
        if let (Some(callback), Some(elapsed)) = (self.metric_callback.as_deref(), self.elapsed) {
            #[cfg(not(feature = "sync"))]
            let elapsed = elapsed
                + self
                    .waiting_since
                    .map_or(Duration::ZERO, |start| start.elapsed());
            let info = crate::metric::Info {
                elapsed,
                statement: self.stmt,
                failed: self.failed,
            };
            callback(&info);
        }
    }
}

#[cfg(all(test, not(feature = "sync")))]
mod tests;
