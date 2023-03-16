use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::Ident;
use quote::format_ident;

#[derive(Clone, Debug)]
pub struct PrimaryKey {
    pub(crate) name: String,
}

impl PrimaryKey {
    pub fn get_name_snake_case(&self) -> Ident {
        format_ident!("{}", self.name.to_snake_case())
    }

    pub fn get_name_camel_case(&self) -> Ident {
        format_ident!("{}", self.name.to_upper_camel_case())
    }
}

#[cfg(test)]
mod tests {
    use crate::PrimaryKey;

    fn setup() -> PrimaryKey {
        PrimaryKey {
            name: "cake_id".to_owned(),
        }
    }

    #[test]
    fn test_get_name_snake_case() {
        let primary_key = setup();

        assert_eq!(primary_key.get_name_snake_case(), "cake_id".to_owned());
    }

    #[test]
    fn test_get_name_camel_case() {
        let primary_key = setup();

        assert_eq!(primary_key.get_name_camel_case(), "CakeId".to_owned());
    }
}
