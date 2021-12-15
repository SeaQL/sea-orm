use std::{time::Duration, sync::Arc};

pub(crate) type Callback = Arc<dyn Fn(&Info<'_>) + Send + Sync>;

pub(crate) use inner::{metric, metric_ok};

#[derive(Debug)]
/// Query execution infos
pub struct Info<'a> {
    /// Query executiuon duration
    pub elapsed: Duration,
    /// Query data
    pub statement: &'a crate::Statement,
    /// Query execution failed
    pub failed: bool,
}

mod inner {
    macro_rules! metric {
        ($metric_callback:expr, $stmt:expr, $code:block) => {
            {
                let _start = std::time::SystemTime::now();
                let res = $code;
                if let Some(callback) = $metric_callback.as_deref() {
                    let info = crate::metric::Info {
                        elapsed: _start.elapsed().unwrap_or_default(),
                        statement: $stmt,
                        failed: res.is_err(),
                    };
                    callback(&info);
                }
                res
            }
        };
    }
    pub(crate) use metric;
    macro_rules! metric_ok {
        ($metric_callback:expr, $stmt:expr, $code:block) => {
            {
                let _start = std::time::SystemTime::now();
                let res = $code;
                if let Some(callback) = $metric_callback.as_deref() {
                    let info = crate::metric::Info {
                        elapsed: _start.elapsed().unwrap_or_default(),
                        statement: $stmt,
                        failed: false,
                    };
                    callback(&info);
                }
                res
            }
        };
    }
    pub(crate) use metric_ok;
}
