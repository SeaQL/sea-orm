//! Build `CREATE TABLE`, `CREATE TYPE`, and related schema statements from
//! [`Entity`](crate::EntityTrait) definitions.
//!
//! Use [`Schema::new`] with a [`DbBackend`] to get a helper, then call
//! [`Schema::builder`] for a fluent [`SchemaBuilder`] that emits `sea_query`
//! statements you can execute on a connection. With the `entity-registry`
//! and `schema-sync` feature flags enabled, `db.get_schema_registry(prefix)`
//! syncs all registered entities at once.

use crate::DbBackend;

mod builder;
mod entity;
#[cfg(feature = "serde_json")]
mod json;
mod topology;

pub use builder::*;
use topology::*;

/// Helper that converts an [`EntityTrait`](crate::EntityTrait) into different
/// [`sea_query`] statements (CREATE TABLE / CREATE TYPE / etc) for a given
/// [`DbBackend`].
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
