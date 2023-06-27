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
        if fields.clone().expect("hello").into_iter().count() != 1 {
            panic!()
        };

        let ty = fields
            .expect("This derive accept only struct")
            .first()
            .expect("The struct should contain one value field")
            .to_owned()
            .ty;
        let name = input.ident;

        Ok(DeriveValueType { name, ty })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_impl_value_type: TokenStream = self.impl_value_type();
        Ok(expanded_impl_value_type)
    }

    fn impl_value_type(&self) -> TokenStream {
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
