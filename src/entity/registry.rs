use crate::{EntitySchemaInfo, Schema, SchemaBuilder};

#[derive(derive_more::Debug)]
/// The data structure submitted by your Entity to the Entity Registry.
pub struct EntityRegistry {
    /// Please use `module_path!()`.
    pub module_path: &'static str,
    /// Function that returns schema info for the Entity.
    #[debug(skip)]
    pub schema_info: fn(&Schema) -> EntitySchemaInfo,
}

inventory::collect!(EntityRegistry);

/// Macro to register an Entity
pub use inventory::submit as register_entity;

impl EntityRegistry {
    /// Builds a schema from all the registered entities, filtering by prefix.
    pub fn build_schema(schema: Schema, prefix: &str) -> SchemaBuilder {
        let mut schema = SchemaBuilder::new(schema);
        let mut prefix = prefix.trim_end_matches("*");
        let string;
        if let Some((left, right)) = prefix.split_once("::") {
            if left.contains("-") {
                let left = left.replace('-', "_");
                string = format!("{left}::{right}");
                prefix = &string;
            }
        }
        for entity in inventory::iter::<crate::EntityRegistry>() {
            if entity.module_path.starts_with(prefix) {
                schema.register_entity((entity.schema_info)(schema.helper()));
            }
        }
        schema
    }
}
