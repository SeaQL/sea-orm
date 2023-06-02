use super::util::camel_case_with_escaped_non_uax31;
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{parse, punctuated::Punctuated, token::Comma, Expr, Lit, LitInt, LitStr, Meta, UnOp};

enum Error {
    InputNotEnum,
    Syn(syn::Error),
    TT(TokenStream),
}

struct ActiveEnum {
    ident: syn::Ident,
    enum_name: String,
    rs_type: TokenStream,
    db_type: TokenStream,
    is_string: bool,
    variants: Vec<ActiveEnumVariant>,
    display: Option<bool>,
}

enum VariantDisplayMode {
    Unmodified,
    Label,
    Value,
    Custom(String),
}

struct ActiveEnumVariant {
    ident: syn::Ident,
    string_value: Option<LitStr>,
    num_value: Option<LitInt>,
    display: VariantDisplayMode,
}

impl ActiveEnum {
    fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let ident_span = input.ident.span();
        let ident = input.ident;

        let mut enum_name = ident.to_string().to_upper_camel_case();
        let mut rs_type = Err(Error::TT(quote_spanned! {
            ident_span => compile_error!("Missing macro attribute `rs_type`");
        }));
        let mut db_type = Err(Error::TT(quote_spanned! {
            ident_span => compile_error!("Missing macro attribute `db_type`");
        }));
        let mut display = None;
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
                                    let s = litstr.value();
                                    match s.as_ref() {
                                        "Enum" => {
                                            db_type = Ok(quote! {
                                                Enum {
                                                    name: Self::name(),
                                                    variants: Self::iden_values(),
                                                }
                                            })
                                        }
                                        _ => {
                                            db_type = syn::parse_str::<TokenStream>(&s)
                                                .map_err(Error::Syn);
                                        }
                                    }
                                }
                            } else if name == "enum_name" {
                                if let Lit::Str(litstr) = &nv.lit {
                                    enum_name = litstr.value();
                                }
                            } else if name == "display" {
                                if let Lit::Str(litstr) = &nv.lit {
                                    display = Some(litstr.value() == "label");
                                }
                            }
                        }
                    }
                    if let Meta::Path(path) = meta {
                        if let Some(name) = path.get_ident() {
                            if name == "display" || name == "display_label" {
                                display = Some(true);
                            } else if name == "display_value" {
                                display = Some(false);
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
            let mut display = VariantDisplayMode::Unmodified;

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
                                } else if name == "display" {
                                    if let Lit::Str(lit) = nv.lit {
                                        display = VariantDisplayMode::Custom(lit.value());
                                    }
                                }
                            }
                        } else if let Meta::Path(path) = meta {
                            if let Some(name) = path.get_ident() {
                                if name == "display_label" {
                                    display = VariantDisplayMode::Label;
                                } else if name == "display_value" {
                                    display = VariantDisplayMode::Value;
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
                match variant.discriminant {
                    Some((_, Expr::Lit(exprlit))) => {
                        if let Lit::Int(litint) = exprlit.lit {
                            is_int = true;
                            num_value = Some(litint);
                        } else {
                            return Err(Error::TT(quote_spanned! {
                                variant_span => compile_error!("Enum variant discriminant is not an integer");
                            }));
                        }
                    }
                    //rust doesn't provide negative variants in enums as a single LitInt, this workarounds that
                    Some((_, Expr::Unary(exprnlit))) => {
                        if let UnOp::Neg(_) = exprnlit.op {
                            if let Expr::Lit(exprlit) = *exprnlit.expr {
                                if let Lit::Int(litint) = exprlit.lit {
                                    let negative_token = quote! { -#litint };
                                    let litint = parse(negative_token.into()).unwrap();

                                    is_int = true;
                                    num_value = Some(litint);
                                }
                            }
                        } else {
                            return Err(Error::TT(quote_spanned! {
                                variant_span => compile_error!("Only - token is supported in enum variants, not ! and *");
                            }));
                        }
                    }
                    _ => {
                        return Err(Error::TT(quote_spanned! {
                            variant_span => compile_error!("Missing macro attribute, either `string_value` or `num_value` should be specified or specify repr[X] and have a value for every entry");
                        }));
                    }
                }
            }

            variants.push(ActiveEnumVariant {
                ident: variant.ident,
                string_value,
                num_value,
                display,
            });
        }

        Ok(ActiveEnum {
            ident,
            enum_name,
            rs_type: rs_type?,
            db_type: db_type?,
            is_string,
            variants,
            display,
        })
    }

    fn expand(&self) -> syn::Result<TokenStream> {
        let expanded_impl_active_enum = self.impl_active_enum();

        Ok(expanded_impl_active_enum)
    }

    fn impl_active_enum(&self) -> TokenStream {
        let Self {
            ident,
            enum_name,
            rs_type,
            db_type,
            is_string,
            variants,
            display,
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

        let displays: Vec<TokenStream> = variants
            .iter()
            .map(|variant| {
                let enum_ident = variant.ident.clone().to_string();
                let enum_ident = enum_ident.as_str();
                match &variant.display {
                    VariantDisplayMode::Unmodified if *display == Some(true) => {
                        quote!(#enum_ident.into())
                    }
                    VariantDisplayMode::Label => quote!(#enum_ident.into()),
                    VariantDisplayMode::Value | VariantDisplayMode::Unmodified => {
                        quote!(self.to_value().into())
                    }
                    VariantDisplayMode::Custom(x) => quote!(#x.into()),
                }
            })
            .collect();

        let val = if *is_string {
            quote! { v.as_ref() }
        } else {
            quote! { v }
        };

        let enum_name_iden = format_ident!("{}Enum", ident);

        let str_variants: Vec<String> = variants
            .iter()
            .filter_map(|variant| {
                variant
                    .string_value
                    .as_ref()
                    .map(|string_value| string_value.value())
            })
            .collect();

        let impl_enum_variant_iden = if !str_variants.is_empty() {
            let enum_variant_iden = format_ident!("{}Variant", ident);
            let enum_variants: Vec<syn::Ident> = str_variants
                .iter()
                .map(|v| {
                    let v_cleaned = camel_case_with_escaped_non_uax31(v);

                    format_ident!("{}", v_cleaned)
                })
                .collect();

            quote!(
                #[doc = " Generated by sea-orm-macros"]
                #[derive(Debug, Clone, PartialEq, Eq, sea_orm::EnumIter)]
                pub enum #enum_variant_iden {
                    #(
                        #[doc = " Generated by sea-orm-macros"]
                        #enum_variants,
                    )*
                }

                #[automatically_derived]
                impl sea_orm::sea_query::Iden for #enum_variant_iden {
                    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
                        write!(s, "{}", match self {
                            #(
                                Self::#enum_variants => #str_variants,
                            )*
                        }).unwrap();
                    }
                }

                #[automatically_derived]
                impl #ident {
                    #[doc = " Generated by sea-orm-macros"]
                    pub fn iden_values() -> Vec<sea_orm::sea_query::DynIden> {
                        <#enum_variant_iden as sea_orm::strum::IntoEnumIterator>::iter()
                            .map(|v| sea_orm::sea_query::SeaRc::new(v) as sea_orm::sea_query::DynIden)
                            .collect()
                    }
                }
            )
        } else {
            quote!()
        };

        let impl_not_u8 = if cfg!(feature = "postgres-array") {
            quote!(
                #[automatically_derived]
                impl sea_orm::sea_query::value::with_array::NotU8 for #ident {}
            )
        } else {
            quote!()
        };

        quote!(
            #[doc = " Generated by sea-orm-macros"]
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct #enum_name_iden;

            #[automatically_derived]
            impl sea_orm::sea_query::Iden for #enum_name_iden {
                fn unquoted(&self, s: &mut dyn std::fmt::Write) {
                    write!(s, "{}", #enum_name).unwrap();
                }
            }

            #impl_enum_variant_iden

            #[automatically_derived]
            impl sea_orm::ActiveEnum for #ident {
                type Value = #rs_type;

                type ValueVec = Vec<#rs_type>;

                fn name() -> sea_orm::sea_query::DynIden {
                    sea_orm::sea_query::SeaRc::new(#enum_name_iden) as sea_orm::sea_query::DynIden
                }

                fn to_value(&self) -> Self::Value {
                    match self {
                        #( Self::#variant_idents => #variant_values, )*
                    }
                    .to_owned()
                }

                fn try_from_value(v: &Self::Value) -> std::result::Result<Self, sea_orm::DbErr> {
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
                    sea_orm::prelude::ColumnTypeTrait::def(sea_orm::ColumnType::#db_type)
                }
            }

            #[automatically_derived]
            #[allow(clippy::from_over_into)]
            impl Into<sea_orm::sea_query::Value> for #ident {
                fn into(self) -> sea_orm::sea_query::Value {
                    <Self as sea_orm::ActiveEnum>::to_value(&self).into()
                }
            }

            #[automatically_derived]
            impl sea_orm::TryGetable for #ident {
                fn try_get_by<I: sea_orm::ColIdx>(res: &sea_orm::QueryResult, idx: I) -> std::result::Result<Self, sea_orm::TryGetError> {
                    let value = <<Self as sea_orm::ActiveEnum>::Value as sea_orm::TryGetable>::try_get_by(res, idx)?;
                    <Self as sea_orm::ActiveEnum>::try_from_value(&value).map_err(sea_orm::TryGetError::DbErr)
                }
            }

            #[automatically_derived]
            impl sea_orm::sea_query::ValueType for #ident {
                fn try_from(v: sea_orm::sea_query::Value) -> std::result::Result<Self, sea_orm::sea_query::ValueTypeErr> {
                    let value = <<Self as sea_orm::ActiveEnum>::Value as sea_orm::sea_query::ValueType>::try_from(v)?;
                    <Self as sea_orm::ActiveEnum>::try_from_value(&value).map_err(|_| sea_orm::sea_query::ValueTypeErr)
                }

                fn type_name() -> String {
                    <<Self as sea_orm::ActiveEnum>::Value as sea_orm::sea_query::ValueType>::type_name()
                }

                fn array_type() -> sea_orm::sea_query::ArrayType {
                    <<Self as sea_orm::ActiveEnum>::Value as sea_orm::sea_query::ValueType>::array_type()
                }

                fn column_type() -> sea_orm::sea_query::ColumnType {
                    <Self as sea_orm::ActiveEnum>::db_type()
                        .get_column_type()
                        .to_owned()
                        .into()
                }
            }

            #[automatically_derived]
            impl sea_orm::sea_query::Nullable for #ident {
                fn null() -> sea_orm::sea_query::Value {
                    <<Self as sea_orm::ActiveEnum>::Value as sea_orm::sea_query::Nullable>::null()
                }
            }

            #[automatically_derived]
            impl std::fmt::Display for #ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let v: sea_orm::sea_query::Value = match self {
                        #( Self::#variant_idents => #displays, )*
                    };
                    write!(f, "{}", v)
                }
            }

            #impl_not_u8
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
