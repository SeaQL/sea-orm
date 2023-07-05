use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Lit, Type};

struct DeriveValueType {
    name: syn::Ident,
    ty: Type,
    column_type: TokenStream,
    array_type: TokenStream,
}

impl DeriveValueType {
    pub fn new(input: syn::DeriveInput) -> Option<Self> {
        let dat = input.data;
        let fields: Option<syn::punctuated::Punctuated<syn::Field, syn::token::Comma>> = match dat {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }),
                ..
            }) => Some(unnamed),
            _ => None,
        };

        let field = fields
            .expect("This derive accept only struct")
            .first()
            .expect("The struct should contain one value field")
            .to_owned();

        let name = input.ident;
        let mut col_type = None;
        let mut arr_type = None;

        for attr in input.attrs.iter() {
            if !attr.path().is_ident("sea_orm") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("column_type") {
                    let lit = meta.value()?.parse()?;
                    if let Lit::Str(litstr) = lit {
                        let ty: TokenStream = syn::parse_str(&litstr.value())?;
                        col_type = Some(ty);
                    } else {
                        return Err(meta.error(format!("Invalid column_type {:?}", lit)));
                    }
                } else if meta.path.is_ident("array_type") {
                    let lit = meta.value()?.parse()?;
                    if let Lit::Str(litstr) = lit {
                        let ty: TokenStream = syn::parse_str(&litstr.value())?;
                        arr_type = Some(ty);
                    } else {
                        return Err(meta.error(format!("Invalid array_type {:?}", lit)));
                    }
                } else {
                    // received other attribute
                    return Err(meta.error(format!("Invalid attribute {:?}", meta.path)));
                }

                Ok(())
            })
            .unwrap_or(());
        }

        let ty = field.clone().ty;
        let field_type = quote! { #ty }
            .to_string() //E.g.: "Option < String >"
            .replace(' ', ""); // Remove spaces
        let field_type = if field_type.starts_with("Option<") {
            &field_type[7..(field_type.len() - 1)] // Extract `T` out of `Option<T>`
        } else {
            field_type.as_str()
        };
        let field_span = field.span();

        let column_type =
            crate::derives::sql_type_match::col_type_match(col_type, field_type, field_span);

        let array_type =
            crate::derives::sql_type_match::arr_type_match(arr_type, field_type, field_span);

        Some(DeriveValueType {
            name,
            ty,
            column_type,
            array_type,
        })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_impl_value_type: TokenStream = self.impl_value_type();
        Ok(expanded_impl_value_type)
    }

    fn impl_value_type(&self) -> TokenStream {
        let name = &self.name;
        let field_type = &self.ty;
        let column_type = &self.column_type;
        let array_type = &self.array_type;

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
                    <#field_type as sea_orm::TryGetable>::try_get_by(res, idx).map(|v| #name(v))
                }
            }

            #[automatically_derived]
            impl sea_orm::sea_query::ValueType for #name {
                fn try_from(v: Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
                    <#field_type as sea_orm::sea_query::ValueType>::try_from(v).map(|v| #name(v))
                }

                fn type_name() -> String {
                    stringify!(#name).to_owned()
                }

                fn array_type() -> sea_orm::sea_query::ArrayType {
                    #array_type
                }

                fn column_type() -> sea_orm::sea_query::ColumnType {
                    #column_type
                }
            }
        )
    }
}

pub fn expand_derive_value_type(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let input_span = input.span();
    match DeriveValueType::new(input) {
        Some(model) => model.expand(),
        None => Err(syn::Error::new(input_span, "error")),
    }
}
