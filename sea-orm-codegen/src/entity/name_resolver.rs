use heck::{CamelCase, SnakeCase};
use inflection::singular;

pub trait NameResolverClone {
    fn clone_box(&self) -> Box<dyn NameResolver>;
}

impl<T> NameResolverClone for T
where
    T: 'static + NameResolver + Clone
{
    fn clone_box(&self) -> Box<dyn NameResolver> {
        Box::new(self.clone())
    }   
}

impl Clone for Box<dyn NameResolver> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait NameResolver: NameResolverClone + std::fmt::Debug {
    fn resolve_module_name(&self, name: &str) -> String {
        name.to_snake_case()
    }

    fn resolve_entity_name(&self, name: &str) -> String {
        name.to_camel_case()
    }

    fn resolve_relation_name(&self, name: &str) -> String {
        name.to_camel_case()
    }
}

#[derive(Clone, Debug)]
pub struct DefaultNameResolver;

impl NameResolver for DefaultNameResolver {}

#[derive(Clone, Debug)]
pub struct SingularNameResolver;

impl NameResolver for SingularNameResolver {
    fn resolve_module_name(&self, name: &str) -> String {
        singular(name.to_snake_case())
    }

    fn resolve_entity_name(&self, name: &str) -> String {
        singular(name.to_camel_case())
    }

    fn resolve_relation_name(&self, name: &str) -> String {
        singular(name.to_camel_case())
    }
}