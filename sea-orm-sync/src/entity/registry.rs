use crate::{EntitySchemaInfo, Schema, SchemaBuilder};
use tracing::debug;

#[derive(derive_more::Debug)]
/// Entry registered into the inventory of known entities, used by the
/// entity-first workflow to enumerate every entity and rebuild the schema.
/// You normally don't construct this by hand — `#[derive(DeriveEntityModel)]`
/// emits the right [`register_entity!`] call when the `entity-registry`
/// feature is enabled.
pub struct EntityRegistry {
    /// Module path the entity lives in; use `module_path!()` to set it.
    pub module_path: &'static str,
    /// Builder that produces this entity's schema info given a [`Schema`]
    /// helper for the active backend.
    #[debug(skip)]
    pub schema_info: fn(&Schema) -> EntitySchemaInfo,
}

inventory::collect!(EntityRegistry);

/// Submit an [`EntityRegistry`] entry to the inventory. Wraps
/// `inventory::submit!` so derive macros don't have to depend on `inventory`
/// directly.
pub use inventory::submit as register_entity;

impl EntityRegistry {
    /// Collect every registered entity whose `module_path` starts with
    /// `prefix` (a trailing `*` is allowed and ignored) and add it to a
    /// fresh [`SchemaBuilder`]. Used by
    /// [`DatabaseConnection::get_schema_registry`](crate::DatabaseConnection::get_schema_registry).
    pub fn build_schema(schema: Schema, prefix: &str) -> SchemaBuilder {
        let mut schema = SchemaBuilder::new(schema);
        let mut string;
        let mut prefix = prefix.trim_end_matches("*");
        if !prefix.contains("::") {
            string = format!("{prefix}::");
            prefix = &string;
        }
        if let Some((left, right)) = prefix.split_once("::") {
            if left.contains("-") {
                // convert crate name to module path
                let left = left.replace('-', "_");
                string = format!("{left}::{right}");
                prefix = &string;
            }
        }
        debug!("Registering entities with prefix `{prefix}`");
        for entity in inventory::iter::<crate::EntityRegistry>() {
            if entity.module_path.starts_with(prefix) {
                schema.register_entity((entity.schema_info)(schema.helper()));
                debug!("Registered {}", entity.module_path);
            } else {
                debug!("Skipped {}", entity.module_path);
            }
        }
        schema
    }
}
