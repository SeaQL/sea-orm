use std::time::Duration;

use once_cell::sync::OnceCell;

type Callback = Box<dyn Fn(&Info<'_>) + Send + Sync>;

static METRIC: OnceCell<Callback> = OnceCell::new();

#[derive(Debug)]
/// Query execution infos
pub struct Info<'a> {
    /// Query executiuon duration
    pub elapsed: Duration,
    /// Query data
    pub statement: &'a crate::Statement,
}

/// Sets a new metric callback, returning it if already set
pub fn set_callback<F>(callback: F) -> Result<(), Callback>
where
    F: Fn(&Info<'_>) + Send + Sync + 'static,
{
    METRIC.set(Box::new(callback))
}

pub(crate) fn get_callback() -> Option<&'static Callback> {
    METRIC.get()
}
