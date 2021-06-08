use heck::{SnakeCase, CamelCase};
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
        format_ident!("{}", self.name.to_camel_case())
    }
}
