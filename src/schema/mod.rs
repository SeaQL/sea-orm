use crate::DbBackend;

mod entity;

/// This is a helper struct to convert [`EntityTrait`](crate::EntityTrait)
/// into different [`sea_query`](crate::sea_query) statements.
#[derive(Debug)]
pub struct Schema {
    backend: DbBackend,
}

impl Schema {
    /// Create a helper for a specific database backend
    pub fn new(backend: DbBackend) -> Self {
        Self { backend }
    }
}
