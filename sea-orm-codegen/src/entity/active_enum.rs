use heck::CamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::WithSerde;

#[derive(Clone, Debug)]
pub struct ActiveEnum {
    pub(crate) enum_name: String,
    pub(crate) values: Vec<String>,
}

impl ActiveEnum {
    pub fn impl_active_enum(&self, with_serde: &WithSerde, with_copy_enums: bool) -> TokenStream {
        let enum_name = &self.enum_name;
        let enum_iden = format_ident!("{}", enum_name.to_camel_case());
        let values = &self.values;
        let variants = self.values.iter().map(|v| v.trim()).map(|v| {
            if v.chars().all(|c| c.is_numeric()) {
                format_ident!("_{}", v)
            } else {
                format_ident!("{}", v.to_camel_case())
            }
        });

        let extra_derive = with_serde.extra_derive();
        let copy_derive = if with_copy_enums {
            quote! { , Copy }
        } else {
            quote! {}
        };

        quote! {
            #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum #copy_derive #extra_derive)]
            #[sea_orm(rs_type = "String", db_type = "Enum", enum_name = #enum_name)]
            pub enum #enum_iden {
                #(
                    #[sea_orm(string_value = #values)]
                    #variants,
                )*
            }
        }
    }
}
