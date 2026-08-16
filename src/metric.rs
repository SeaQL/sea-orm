use std::{sync::Arc, time::Duration};

pub(crate) type Callback = Arc<dyn Fn(&Info<'_>) + Send + Sync>;

#[allow(unused_imports)]
pub(crate) use inner::metric;

#[derive(Debug)]
/// Information about a single query execution, passed to the callback
/// registered via [`DatabaseConnection::set_metric_callback`](crate::DatabaseConnection::set_metric_callback).
pub struct Info<'a> {
    /// Query execution duration
    pub elapsed: Duration,
    /// SQL statement with parameters
    pub statement: &'a crate::Statement,
    /// `true` if the query returned an error
    pub failed: bool,
}

mod inner {
    #[allow(unused_macros)]
    macro_rules! metric {
        ($metric_callback:expr, $stmt:expr, $code:block) => {{
            let _start = $metric_callback.is_some().then(std::time::SystemTime::now);
            let res = $code;
            if let (Some(_start), Some(callback)) = (_start, $metric_callback.as_deref()) {
                let info = crate::metric::Info {
                    elapsed: _start.elapsed().unwrap_or_default(),
                    statement: $stmt,
                    failed: res.is_err(),
                };
                callback(&info);
            }
            res
        }};
    }
    pub(crate) use metric;
}
