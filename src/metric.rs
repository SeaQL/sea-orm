use std::{time::Duration, sync::Arc};

pub(crate) type Callback = Arc<dyn Fn(&Info<'_>) + Send + Sync>;

#[derive(Debug)]
/// Query execution infos
pub struct Info<'a> {
    /// Query executiuon duration
    pub elapsed: Duration,
    /// Query data
    pub statement: &'a crate::Statement,
}
