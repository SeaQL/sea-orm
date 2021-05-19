use heck::SnakeCase;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, Meta};

fn get_entity_attr(attrs: &[Attribute]) -> Option<syn::Lit> {
    for attr in attrs {
        let name_value = match attr.parse_meta() {
            Ok(Meta::NameValue(nv)) => nv,
            _ => continue,
        };
        if name_value.path.is_ident("table") {
            return Some(name_value.lit);
        }
    }
    None
}

pub fn expend_derive_entity(ident: Ident, attrs: Vec<Attribute>) -> syn::Result<TokenStream> {
    let entity_name = match get_entity_attr(&attrs) {
        Some(lit) => quote! { #lit },
        None => {
            let normalized = ident.to_string().to_snake_case();
            quote! { #normalized }
        }
    };

    Ok(quote!(
        impl sea_orm::EntityName for #ident {}

        impl sea_orm::IdenStatic for #ident {
            fn as_str(&self) -> &str {
                #entity_name
            }
        }

        impl sea_orm::Iden for #ident {
            fn unquoted(&self, s: &mut dyn std::fmt::Write) {
                write!(s, "{}", self.as_str()).unwrap();
            }
        }

        impl EntityTrait for #ident {
            type Model = Model;

            type Column = Column;

            type PrimaryKey = PrimaryKey;

            type Relation = Relation;
        }
    ))
}
