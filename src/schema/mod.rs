use crate::DbBackend;

mod builder;
mod entity;
#[cfg(feature = "serde_json")]
mod json;
mod topology;

pub use builder::*;
use topology::*;

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

    /// Creates a schema builder that can apply schema changes to database
    pub fn builder(self) -> SchemaBuilder {
        SchemaBuilder::new(self)
    }
}
