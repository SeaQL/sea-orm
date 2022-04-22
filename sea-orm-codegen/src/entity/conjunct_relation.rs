use heck::{CamelCase, SnakeCase};
use proc_macro2::Ident;
use quote::format_ident;

use crate::NameResolver;

#[derive(Clone, Debug)]
pub struct ConjunctRelation {
    pub(crate) via: String,
    pub(crate) to: String,
}

impl ConjunctRelation {
    pub fn resolve_via_module_name(&self, name_resolver: &dyn NameResolver) -> Ident {
        format_ident!("{}", name_resolver.resolve_module_name(&self.via))
    }

    pub fn resolve_to_module_name(&self, name_resolver: &dyn NameResolver) -> Ident {
        format_ident!("{}", name_resolver.resolve_module_name(&self.to))
    }

    pub fn resolve_to_relation_name(&self, name_resolver: &dyn NameResolver) -> Ident {
        format_ident!("{}", name_resolver.resolve_relation_name(&self.to))
    }

    pub fn get_via_snake_case(&self) -> Ident {
        format_ident!("{}", self.via.to_snake_case())
    }

    pub fn get_to_snake_case(&self) -> Ident {
        format_ident!("{}", self.to.to_snake_case())
    }

    pub fn get_to_camel_case(&self) -> Ident {
        format_ident!("{}", self.to.to_camel_case())
    }
}

#[cfg(test)]
mod tests {
    use crate::ConjunctRelation;

    fn setup() -> Vec<ConjunctRelation> {
        vec![
            ConjunctRelation {
                via: "cake_filling".to_owned(),
                to: "cake".to_owned(),
            },
            ConjunctRelation {
                via: "cake_filling".to_owned(),
                to: "filling".to_owned(),
            },
        ]
    }

    #[test]
    fn test_get_via_snake_case() {
        let conjunct_relations = setup();
        let via_vec = vec!["cake_filling", "cake_filling"];
        for (con_rel, via) in conjunct_relations.into_iter().zip(via_vec) {
            assert_eq!(con_rel.get_via_snake_case(), via);
        }
    }

    #[test]
    fn test_get_to_snake_case() {
        let conjunct_relations = setup();
        let to_vec = vec!["cake", "filling"];
        for (con_rel, to) in conjunct_relations.into_iter().zip(to_vec) {
            assert_eq!(con_rel.get_to_snake_case(), to);
        }
    }

    #[test]
    fn test_get_to_camel_case() {
        let conjunct_relations = setup();
        let to_vec = vec!["Cake", "Filling"];
        for (con_rel, to) in conjunct_relations.into_iter().zip(to_vec) {
            assert_eq!(con_rel.get_to_camel_case(), to);
        }
    }
}
