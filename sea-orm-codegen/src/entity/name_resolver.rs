use heck::{CamelCase, SnakeCase};
use inflection::singular;

#[derive(Clone, Debug)]
pub struct NameResolver {
    singularize: bool,
}

impl NameResolver {
    pub fn new(singularize: bool) -> Self {
        Self { singularize }
    }

    pub fn resolve_module_name(&self, name: &str) -> String {
        let name = name.to_snake_case();

        if self.singularize {
            singular(name)
        } else {
            name
        }
    }

    pub fn resolve_entity_name(&self, name: &str) -> String {
        let name = name.to_camel_case();

        if self.singularize {
            singular(name)
        } else {
            name
        }
    }

    pub fn resolve_relation_name(&self, name: &str) -> String {
        let name = name.to_camel_case();

        if self.singularize {
            singular(name)
        } else {
            name
        }
    }
}
