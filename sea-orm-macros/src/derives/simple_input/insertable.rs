use heck::CamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, token::Comma, Field, Generics};

use crate::util::option_type_to_inner_type;

pub(crate) fn impl_insertable(
    input_model_ident: &Ident,
    mut input_model_generics: Generics,
    entity_ident: &Ident,
    fields: &Punctuated<Field, Comma>,
) -> TokenStream {
    input_model_generics
        .lifetimes_mut()
        .into_iter()
        .for_each(|mut lifetime| {
            lifetime.lifetime.ident = format_ident!("_");
        });

    let get_fields = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let column_name = format_ident!("{}", field_name.to_string().to_camel_case());

        if option_type_to_inner_type(&field.ty).is_some() {
            quote!(
                <Self::Entity as sea_orm::entity::EntityTrait>::Column::#column_name => {
                    if let Some(value) = &self.#field_name {
                        sea_orm::ActiveValue::set(value.clone()).into_wrapped_value()
                    } else {
                        sea_orm::ActiveValue::unset()
                    }
                }
            )
        } else {
            quote!(<Self::Entity as sea_orm::entity::EntityTrait>::Column::#column_name => sea_orm::ActiveValue::set(self.#field_name.clone()).into_wrapped_value())
        }
    });

    quote!(
        impl sea_orm::Insertable for #input_model_ident#input_model_generics {
            type Entity = #entity_ident;

            fn take(&mut self, c: <Self::Entity as sea_orm::entity::EntityTrait>::Column) -> sea_orm::ActiveValue<sea_orm::Value> {
                match c {
                    #(#get_fields,)*
                    _ => sea_orm::ActiveValue::unset(),
                }
            }
        }
    )
}
