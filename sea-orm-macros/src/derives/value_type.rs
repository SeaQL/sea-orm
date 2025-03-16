use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Lit, Type};

struct DeriveValueType {
    name: syn::Ident,
    ty: Type,
    column_type: TokenStream,
    array_type: TokenStream,
}

enum Error {
    InputNotStruct,
    NotTupleStruct,
    Syn(syn::Error),
}

impl DeriveValueType {
    pub fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let fields = match input.data {
            syn::Data::Struct(syn::DataStruct {
                fields: syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }),
                ..
            }) => unnamed,
            _ => return Err(Error::InputNotStruct),
        };

        let Some(field) = fields.into_iter().next() else {
            return Err(Error::NotTupleStruct);
        };

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
            .map_err(Error::Syn)?;
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

        Ok(DeriveValueType {
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
            impl std::convert::From<#name> for sea_orm::Value {
                fn from(source: #name) -> Self {
                    source.0.into()
                }
            }

            #[automatically_derived]
            impl sea_orm::TryGetable for #name {
                fn try_get_by<I: sea_orm::ColIdx>(res: &sea_orm::QueryResult, idx: I)
                    -> std::result::Result<Self, sea_orm::TryGetError> {
                    <#field_type as sea_orm::TryGetable>::try_get_by(res, idx).map(|v| #name(v))
                }
            }

            #[automatically_derived]
            impl sea_orm::sea_query::ValueType for #name {
                fn try_from(v: sea_orm::Value) -> std::result::Result<Self, sea_orm::sea_query::ValueTypeErr> {
                    <#field_type as sea_orm::sea_query::ValueType>::try_from(v).map(|v| #name(v))
                }

                fn type_name() -> std::string::String {
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
        Ok(model) => model.expand(),
        Err(Error::InputNotStruct) => Ok(quote_spanned! {
            input_span => compile_error!("you can only derive `DeriveValueType` on tuple struct");
        }),
        Err(Error::NotTupleStruct) => Ok(quote_spanned! {
            input_span => compile_error!("you can only derive `DeriveValueType` on tuple struct with one member. e.g. `MyType(pub i32)`");
        }),
        Err(Error::Syn(e)) => Err(e),
    }
}
