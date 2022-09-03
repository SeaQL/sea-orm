use crate::DbBackend;

mod entity;

/// This is a helper struct to convert [`EntityTrait`](crate::EntityTrait)
/// into different [`sea_query`](crate::sea_query) statements.
#[derive(Debug)]
pub struct Schema {
    backend: DbBackend,
    schema_name: Option<String>,
}

impl Schema {
    /// Create a helper for a specific database backend
    pub fn new(backend: DbBackend, schema_name: Option<String>) -> Self {
        Self {
            backend,
            schema_name,
        }
    }
}
