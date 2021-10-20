use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{punctuated::Punctuated, token::Comma, Lit, LitInt, LitStr, Meta};

enum Error {
    InputNotEnum,
    Syn(syn::Error),
    TT(TokenStream),
}

struct ActiveEnum {
    ident: syn::Ident,
    rs_type: TokenStream,
    db_type: TokenStream,
    is_string: bool,
    variants: Vec<ActiveEnumVariant>,
}

struct ActiveEnumVariant {
    ident: syn::Ident,
    string_value: Option<LitStr>,
    num_value: Option<LitInt>,
}

impl ActiveEnum {
    fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let ident_span = input.ident.span();
        let ident = input.ident;

        let mut rs_type = Err(Error::TT(quote_spanned! {
            ident_span => compile_error!("Missing macro attribute `rs_type`");
        }));
        let mut db_type = Err(Error::TT(quote_spanned! {
            ident_span => compile_error!("Missing macro attribute `db_type`");
        }));
        for attr in input.attrs.iter() {
            if let Some(ident) = attr.path.get_ident() {
                if ident != "sea_orm" {
                    continue;
                }
            } else {
                continue;
            }
            if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
                for meta in list.iter() {
                    if let Meta::NameValue(nv) = meta {
                        if let Some(name) = nv.path.get_ident() {
                            if name == "rs_type" {
                                if let Lit::Str(litstr) = &nv.lit {
                                    rs_type = syn::parse_str::<TokenStream>(&litstr.value())
                                        .map_err(Error::Syn);
                                }
                            } else if name == "db_type" {
                                if let Lit::Str(litstr) = &nv.lit {
                                    db_type = syn::parse_str::<TokenStream>(&litstr.value())
                                        .map_err(Error::Syn);
                                }
                            }
                        }
                    }
                }
            }
        }

        let variant_vec = match input.data {
            syn::Data::Enum(syn::DataEnum { variants, .. }) => variants,
            _ => return Err(Error::InputNotEnum),
        };

        let mut is_string = false;
        let mut is_int = false;
        let mut variants = Vec::new();
        for variant in variant_vec {
            let variant_span = variant.ident.span();
            let mut string_value = None;
            let mut num_value = None;
            for attr in variant.attrs.iter() {
                if let Some(ident) = attr.path.get_ident() {
                    if ident != "sea_orm" {
                        continue;
                    }
                } else {
                    continue;
                }
                if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                {
                    for meta in list {
                        if let Meta::NameValue(nv) = meta {
                            if let Some(name) = nv.path.get_ident() {
                                if name == "string_value" {
                                    if let Lit::Str(lit) = nv.lit {
                                        is_string = true;
                                        string_value = Some(lit);
                                    }
                                } else if name == "num_value" {
                                    if let Lit::Int(lit) = nv.lit {
                                        is_int = true;
                                        num_value = Some(lit);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if is_string && is_int {
                return Err(Error::TT(quote_spanned! {
                    ident_span => compile_error!("All enum variants should specify the same `*_value` macro attribute, either `string_value` or `num_value` but not both");
                }));
            }

            if string_value.is_none() && num_value.is_none() {
                return Err(Error::TT(quote_spanned! {
                    variant_span => compile_error!("Missing macro attribute, either `string_value` or `num_value` should be specified");
                }));
            }

            variants.push(ActiveEnumVariant {
                ident: variant.ident,
                string_value,
                num_value,
            });
        }

        Ok(ActiveEnum {
            ident,
            rs_type: rs_type?,
            db_type: db_type?,
            is_string,
            variants,
        })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_impl_active_enum = self.impl_active_enum();

        Ok(expanded_impl_active_enum)
    }

    fn impl_active_enum(&self) -> TokenStream {
        let Self {
            ident,
            rs_type,
            db_type,
            is_string,
            variants,
        } = self;

        let variant_idents: Vec<syn::Ident> = variants
            .iter()
            .map(|variant| variant.ident.clone())
            .collect();

        let variant_values: Vec<TokenStream> = variants
            .iter()
            .map(|variant| {
                let variant_span = variant.ident.span();

                if let Some(string_value) = &variant.string_value {
                    let string = string_value.value();
                    quote! { #string }
                } else if let Some(num_value) = &variant.num_value {
                    quote! { #num_value }
                } else {
                    quote_spanned! {
                        variant_span => compile_error!("Missing macro attribute, either `string_value` or `num_value` should be specified");
                    }
                }
            })
            .collect();

        let val = if *is_string {
            quote! { v.as_ref() }
        } else {
            quote! { v }
        };

        quote!(
            #[automatically_derived]
            impl sea_orm::ActiveEnum for #ident {
                type Value = #rs_type;

                fn to_value(&self) -> Self::Value {
                    match self {
                        #( Self::#variant_idents => #variant_values, )*
                    }
                    .to_owned()
                }

                fn try_from_value(v: &Self::Value) -> Result<Self, sea_orm::DbErr> {
                    match #val {
                        #( #variant_values => Ok(Self::#variant_idents), )*
                        _ => Err(sea_orm::DbErr::Type(format!(
                            "unexpected value for {} enum: {}",
                            stringify!(#ident),
                            v
                        ))),
                    }
                }

                fn db_type() -> sea_orm::ColumnDef {
                    sea_orm::ColumnType::#db_type.def()
                }
            }

            #[automatically_derived]
            #[allow(clippy::from_over_into)]
            impl Into<sea_query::Value> for #ident {
                fn into(self) -> sea_query::Value {
                    <Self as sea_orm::ActiveEnum>::to_value(&self).into()
                }
            }

            #[automatically_derived]
            impl sea_orm::TryGetable for #ident {
                fn try_get(res: &sea_orm::QueryResult, pre: &str, col: &str) -> Result<Self, sea_orm::TryGetError> {
                    let value = <<Self as sea_orm::ActiveEnum>::Value as sea_orm::TryGetable>::try_get(res, pre, col)?;
                    <Self as sea_orm::ActiveEnum>::try_from_value(&value).map_err(sea_orm::TryGetError::DbErr)
                }
            }

            #[automatically_derived]
            impl sea_query::ValueType for #ident {
                fn try_from(v: sea_query::Value) -> Result<Self, sea_query::ValueTypeErr> {
                    let value = <<Self as sea_orm::ActiveEnum>::Value as sea_query::ValueType>::try_from(v)?;
                    <Self as sea_orm::ActiveEnum>::try_from_value(&value).map_err(|_| sea_query::ValueTypeErr)
                }

                fn type_name() -> String {
                    <<Self as sea_orm::ActiveEnum>::Value as sea_query::ValueType>::type_name()
                }

                fn column_type() -> sea_query::ColumnType {
                    <Self as sea_orm::ActiveEnum>::db_type()
                        .get_column_type()
                        .to_owned()
                        .into()
                }
            }

            #[automatically_derived]
            impl sea_query::Nullable for #ident {
                fn null() -> sea_query::Value {
                    <<Self as sea_orm::ActiveEnum>::Value as sea_query::Nullable>::null()
                }
            }
        )
    }
}

pub fn expand_derive_active_enum(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match ActiveEnum::new(input) {
        Ok(model) => model.expand(),
        Err(Error::InputNotEnum) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive ActiveEnum on enums");
        }),
        Err(Error::TT(token_stream)) => Ok(token_stream),
        Err(Error::Syn(e)) => Err(e),
    }
}
