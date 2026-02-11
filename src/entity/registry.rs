use crate::{EntitySchemaInfo, Schema, SchemaBuilder};
use semver::{Version, VersionReq};
use tracing::debug;

#[derive(derive_more::Debug)]
/// The data structure submitted by your Entity to the Entity Registry.
pub struct EntityRegistry {
    /// Please use `module_path!()`.
    pub module_path: &'static str,
    /// Please use `option_env!("CARGO_PKG_VERSION")`
    pub module_version: Option<&'static str>,
    /// Function that returns schema info for the Entity.
    #[debug(skip)]
    pub schema_info: fn(&Schema) -> EntitySchemaInfo,
}

inventory::collect!(EntityRegistry);

/// Macro to register an Entity
pub use inventory::submit as register_entity;

impl EntityRegistry {
    /// Builds a schema from all the registered entities, filtering by module prefix and crate version.
    pub fn build_schema(schema: Schema, prefix: &str, version_spec: Option<&str>) -> SchemaBuilder {
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
        if let Some(spec) = version_spec {
            debug!("Registering entities with prefix `{prefix}` and version `{spec}`");
        } else {
            debug!("Registering entities with prefix `{prefix}`");
        }
        for entity in inventory::iter::<crate::EntityRegistry>() {
            if entity.module_path.starts_with(prefix)
                && version_matches(entity.module_version, version_spec)
            {
                schema.register_entity((entity.schema_info)(schema.helper()));
                if let Some(version) = entity.module_version {
                    debug!("Registered {} ({})", entity.module_path, version);
                } else {
                    debug!("Registered {}", entity.module_path);
                }
            } else {
                if let Some(version) = entity.module_version {
                    debug!("Skipped {} ({})", entity.module_path, version);
                } else {
                    debug!("Skipped {}", entity.module_path);
                }
            }
        }
        schema
    }
}

fn version_matches(version: Option<&str>, version_spec: Option<&str>) -> bool {
    match (version, version_spec) {
        (Some(version), Some(version_spec)) => VersionReq::parse(version_spec)
            .unwrap()
            .matches(&Version::parse(version).unwrap()),

        // Either module version or version spec not given
        _ => true,
    }
}
