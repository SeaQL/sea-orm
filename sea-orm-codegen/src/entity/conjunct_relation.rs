use heck::{CamelCase, SnakeCase};
use proc_macro2::Ident;
use quote::format_ident;
use inflection::singular;

#[derive(Clone, Debug)]
pub struct ConjunctRelation {
    pub(crate) via: String,
    pub(crate) to: String,
    pub(crate) singularize: bool
}

impl ConjunctRelation {
    pub fn get_via_snake_case(&self) -> Ident {
        let mut name = self.via.to_snake_case();
        if self.singularize {
            name = singular(name);
        }
        format_ident!("{}", name)
    }

    pub fn get_to_snake_case(&self) -> Ident {
        let mut name = self.to.to_snake_case();
        if self.singularize {
            name = singular(name);
        }
        format_ident!("{}", name)
    }

    pub fn get_to_camel_case(&self) -> Ident {
        let mut name = self.to.to_camel_case();
        if self.singularize {
            name = singular(name);
        }
        format_ident!("{}", name)
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
                singularize: false,
            },
            ConjunctRelation {
                via: "cake_filling".to_owned(),
                to: "filling".to_owned(),
                singularize: false,
            },
            ConjunctRelation {
                via: "cake_fillings".to_owned(),
                to: "fillings".to_owned(),
                singularize: true,
            }
        ]
    }

    #[test]
    fn test_get_via_snake_case() {
        let conjunct_relations = setup();
        let via_vec = vec!["cake_filling", "cake_filling", "cake_filling"];
        for (con_rel, via) in conjunct_relations.into_iter().zip(via_vec) {
            assert_eq!(con_rel.get_via_snake_case(), via);
        }
    }

    #[test]
    fn test_get_to_snake_case() {
        let conjunct_relations = setup();
        let to_vec = vec!["cake", "filling", "filling"];
        for (con_rel, to) in conjunct_relations.into_iter().zip(to_vec) {
            assert_eq!(con_rel.get_to_snake_case(), to);
        }
    }

    #[test]
    fn test_get_to_camel_case() {
        let conjunct_relations = setup();
        let to_vec = vec!["Cake", "Filling", "Filling"];
        for (con_rel, to) in conjunct_relations.into_iter().zip(to_vec) {
            assert_eq!(con_rel.get_to_camel_case(), to);
        }
    }
}
