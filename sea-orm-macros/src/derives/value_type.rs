use proc_macro2::TokenStream;
use quote::quote;
use syn::Type;

struct DeriveValueType {
    name: syn::Ident,
    ty: Type,
}

impl DeriveValueType {
    pub fn new(input: syn::DeriveInput) -> Result<Self, syn::Error> {
        let dat = input.data;
        let fields: Option<syn::punctuated::Punctuated<syn::Field, syn::token::Comma>> = match dat {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }),
                ..
            }) => Some(unnamed),
            _ => None,
        };
        let ty = fields
            .expect("not a struct")
            .first()
            .expect("empty type")
            .to_owned()
            .ty;
        let name = input.ident;

        Ok(DeriveValueType { name, ty })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_impl_entity_name: TokenStream = self.impl_entity_name();
        Ok(expanded_impl_entity_name)
    }

    fn impl_entity_name(&self) -> TokenStream {
        let name = &self.name;
        let ty = &self.ty;

        quote!(
            #[automatically_derived]
            impl From<#name> for Value {
                fn from(source: #name) -> Self {
                    source.0.into()
                }
            }

            #[automatically_derived]
            impl sea_orm::TryGetable for #name {
                fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, idx: I) -> Result<Self, sea_orm::TryGetError> {
                    <#ty as sea_orm::TryGetable>::try_get_by(res, idx).map(|v| #name(v))
                }
            }

            #[automatically_derived]
            impl sea_query::ValueType for #name {
                fn try_from(v: Value) -> Result<Self, sea_query::ValueTypeErr> {
                    <#ty as sea_query::ValueType>::try_from(v).map(|v| #name(v))
                }

                fn type_name() -> String {
                    stringify!(#name).to_owned()
                }

                fn array_type() -> sea_orm::sea_query::ArrayType {
                    <#ty as sea_orm::sea_query::ValueType>::array_type()
                }

                fn column_type() -> sea_orm::sea_query::ColumnType {
                    <#ty as sea_orm::sea_query::ValueType>::column_type()
                }
            }
        )
    }
}

pub fn expand_derive_value_type(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    match DeriveValueType::new(input) {
        Ok(value_type) => value_type.expand(),
        Err(err) => Err(err),
    }
}
