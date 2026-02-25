use super::case_style::{CaseStyle, CaseStyleHelpers};
use super::util::camel_case_with_escaped_non_uax31;
use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{Expr, Lit, LitInt, LitStr, UnOp, parse};

struct ActiveEnum {
    ident: syn::Ident,
    enum_name: String,
    rs_type: RsType,
    db_type: DbType,
    is_string: bool,
    variants: Vec<ActiveEnumVariant>,
    variant_idents: Vec<syn::Ident>,
    variant_values: Vec<TokenStream>,
    rename_all: Option<CaseStyle>,
}

enum RsType {
    String,
    Enum,
    Other(TokenStream),
}

impl RsType {
    fn from_attr(
        ident_span: proc_macro2::Span,
        rs_type: Option<String>,
        db_type: &DbType,
    ) -> Result<Self, Error> {
        if db_type.is_enum() {
            match rs_type.as_deref() {
                None => Ok(RsType::Enum),
                Some(value) => RsType::from_database_enum_attr_value(value).ok_or_else(|| {
                    Error::TT(quote_spanned! {
                        ident_span => compile_error!("`db_type = \"Enum\"` only supports `rs_type = \"String\"` or `rs_type = \"Enum\"` (or omit `rs_type`)");
                    })
                }),
            }
        } else {
            let rs_type = match rs_type {
                Some(rs_type) => rs_type,
                None => {
                    return Err(Error::TT(quote_spanned! {
                        ident_span => compile_error!("Missing macro attribute `rs_type`");
                    }));
                }
            };

            if rs_type == "Enum" {
                return Err(Error::TT(quote_spanned! {
                    ident_span => compile_error!("`rs_type = \"Enum\"` requires `db_type = \"Enum\"`");
                }));
            }

            RsType::from_str(&rs_type).map_err(Error::Syn)
        }
    }

    fn from_str(value: &str) -> syn::Result<Self> {
        Ok(Self::Other(syn::parse_str::<TokenStream>(value)?))
    }

    fn from_database_enum_attr_value(value: &str) -> Option<Self> {
        match value {
            "Enum" => Some(Self::Enum),
            "String" => Some(Self::String),
            _ => None,
        }
    }
}

impl quote::ToTokens for RsType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            RsType::String => tokens.extend(quote! { String }),
            RsType::Enum => tokens.extend(quote! { sea_orm::sea_query::Enum }),
            RsType::Other(rs_type) => tokens.extend(rs_type.clone()),
        }
    }
}

enum DbType {
    Enum,
    Other(TokenStream),
}

impl DbType {
    fn from_attr(ident_span: proc_macro2::Span, db_type: Option<String>) -> Result<Self, Error> {
        let db_type = match db_type {
            Some(db_type) => db_type,
            None => {
                return Err(Error::TT(quote_spanned! {
                    ident_span => compile_error!("Missing macro attribute `db_type`");
                }));
            }
        };

        DbType::from_str(&db_type).map_err(Error::Syn)
    }

    fn from_str(value: &str) -> syn::Result<Self> {
        match value {
            "Enum" => Ok(Self::Enum),
            _ => Ok(Self::Other(syn::parse_str::<TokenStream>(value)?)),
        }
    }

    fn is_enum(&self) -> bool {
        matches!(self, DbType::Enum)
    }
}

struct ActiveEnumVariant {
    ident: syn::Ident,
    string_value: Option<LitStr>,
    num_value: Option<LitInt>,
    rename: Option<CaseStyle>,
}

enum Error {
    InputNotEnum,
    Syn(syn::Error),
    TT(TokenStream),
}

impl ActiveEnum {
    fn new(input: syn::DeriveInput) -> Result<Self, Error> {
        let ident_span = input.ident.span();
        let ident = input.ident;

        let mut enum_name = ident.to_string().to_upper_camel_case();
        let mut rs_type = None;
        let mut db_type = None;
        let mut rename_all = None;

        input
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("sea_orm"))
            .try_for_each(|attr| {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("rs_type") {
                        let litstr: LitStr = meta.value()?.parse()?;
                        rs_type = Some(litstr.value());
                    } else if meta.path.is_ident("db_type") {
                        let litstr: LitStr = meta.value()?.parse()?;
                        db_type = Some(litstr.value());
                    } else if meta.path.is_ident("enum_name") {
                        let litstr: LitStr = meta.value()?.parse()?;
                        enum_name = litstr.value();
                    } else if meta.path.is_ident("rename_all") {
                        rename_all = Some((&meta).try_into()?);
                    } else {
                        return Err(meta.error(format!(
                            "Unknown attribute parameter found: {:?}",
                            meta.path.get_ident()
                        )));
                    }
                    Ok(())
                })
                .map_err(Error::Syn)
            })?;

        let db_type = DbType::from_attr(ident_span, db_type)?;
        let rs_type = RsType::from_attr(ident_span, rs_type, &db_type)?;

        let variant_vec = match input.data {
            syn::Data::Enum(syn::DataEnum { variants, .. }) => variants,
            _ => return Err(Error::InputNotEnum),
        };

        let mut is_string = rename_all.is_some();
        let mut is_int = false;
        let mut variants = Vec::new();

        for variant in variant_vec {
            let variant_span = variant.ident.span();
            let mut string_value = None;
            let mut num_value = None;
            let mut rename_rule = None;

            for attr in variant.attrs.iter() {
                if !attr.path().is_ident("sea_orm") {
                    continue;
                }
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("string_value") {
                        is_string = true;
                        string_value = Some(meta.value()?.parse::<LitStr>()?);
                    } else if meta.path.is_ident("num_value") {
                        is_int = true;
                        num_value = Some(meta.value()?.parse::<LitInt>()?);
                    } else if meta.path.is_ident("display_value") {
                        // This is a placeholder to prevent the `display_value` proc_macro attribute of `DeriveDisplay`
                        // to be considered unknown attribute parameter
                        meta.value()?.parse::<LitStr>()?;
                    } else if meta.path.is_ident("rename") {
                        is_string = true;
                        rename_rule = Some((&meta).try_into()?);
                    } else {
                        return Err(meta.error(format!(
                            "Unknown attribute parameter found: {:?}",
                            meta.path.get_ident()
                        )));
                    }

                    Ok(())
                })
                .map_err(Error::Syn)?;
            }

            if is_string && is_int {
                return Err(Error::TT(quote_spanned! {
                    ident_span => compile_error!("All enum variants should specify the same `*_value` macro attribute, either `string_value` or `num_value` but not both");
                }));
            }

            if string_value.is_none() && num_value.is_none() && rename_rule.or(rename_all).is_none()
            {
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
                            variant_span => compile_error!("Missing macro attribute, either `string_value`, `num_value` or `rename` should be specified or specify repr[X] and have a value for every entry");
                        }));
                    }
                }
            }

            variants.push(ActiveEnumVariant {
                ident: variant.ident,
                string_value,
                num_value,
                rename: rename_rule,
            });
        }

        if db_type.is_enum() && is_int {
            return Err(Error::TT(quote_spanned! {
                ident_span => compile_error!("`db_type = \"Enum\"` does not support `num_value` or numeric discriminants");
            }));
        }

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
                    Ok(quote! { #string })
                } else if let Some(num_value) = &variant.num_value {
                    Ok(quote! { #num_value })
                } else if let Some(rename_rule) = variant.rename.or(rename_all) {
                    let variant_ident = variant.ident.convert_case(Some(rename_rule));
                    Ok(quote! { #variant_ident })
                } else {
                    Err(Error::TT(quote_spanned! {
                        variant_span => compile_error!("Missing macro attribute, either `string_value`, `num_value` or `rename_all` should be specified");
                    }))
                }
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            ident,
            enum_name,
            rs_type,
            db_type,
            is_string,
            variants,
            variant_idents,
            variant_values,
            rename_all,
        })
    }

    fn generate_enum_impls(&self) -> bool {
        self.db_type.is_enum() && matches!(self.rs_type, RsType::Enum)
    }

    fn to_value_impl(&self) -> TokenStream {
        let enum_name = &self.enum_name;
        let variant_idents = &self.variant_idents;
        let variant_values = &self.variant_values;

        if self.generate_enum_impls() {
            quote! {
                let value = match self {
                    #( Self::#variant_idents => #variant_values, )*
                };
                sea_orm::sea_query::Enum {
                    type_name: #enum_name.into(),
                    value: value.into(),
                }
            }
        } else {
            quote! {
                match self {
                    #( Self::#variant_idents => #variant_values, )*
                }
                .to_owned()
            }
        }
    }

    fn value_type_try_from_impl(&self) -> TokenStream {
        if self.generate_enum_impls() {
            quote! {
                use sea_orm::sea_query::{OptionEnum, Value, ValueTypeErr};

                match v {
                    Value::Enum(value) => match value {
                        OptionEnum::Some(value) => <Self as sea_orm::ActiveEnum>::try_from_value(value.as_ref())
                            .map_err(|_| ValueTypeErr),
                        OptionEnum::None(_) => Err(ValueTypeErr),
                    },
                    _ => Err(ValueTypeErr),
                }
            }
        } else {
            quote! {
                use sea_orm::sea_query::{ValueType, ValueTypeErr};

                let value = <<Self as sea_orm::ActiveEnum>::Value as ValueType>::try_from(v)?;
                <Self as sea_orm::ActiveEnum>::try_from_value(&value).map_err(|_| ValueTypeErr)
            }
        }
    }

    fn nullable_impl(&self) -> TokenStream {
        let ident = &self.ident;
        let enum_name = &self.enum_name;
        let nullable_value_impl = if self.generate_enum_impls() {
            quote! {
                use sea_orm::sea_query::{OptionEnum, Value};
                Value::Enum(OptionEnum::None(#enum_name.into()))
            }
        } else {
            quote! {
                use sea_orm::sea_query;
                <<Self as sea_orm::ActiveEnum>::Value as sea_query::Nullable>::null()
            }
        };

        quote! {
            #[automatically_derived]
            #[allow(unexpected_cfgs)]
            impl sea_orm::sea_query::Nullable for #ident {
                fn null() -> sea_orm::sea_query::Value {
                    #nullable_value_impl
                }
            }
        }
    }

    fn value_type_impl(&self) -> TokenStream {
        let ident = &self.ident;
        let value_type_try_from_impl = self.value_type_try_from_impl();
        let enum_name = &self.enum_name;

        let type_name_impl = quote! { stringify!(#ident).to_owned() };

        let value_type_array_type = if self.generate_enum_impls() {
            quote! {
                sea_orm::sea_query::ArrayType::Enum(Box::new(#enum_name.into()))
            }
        } else {
            quote! {
                <<Self as sea_orm::ActiveEnum>::Value as sea_orm::sea_query::ValueType>::array_type()
            }
        };

        let enum_type_name = if self.db_type.is_enum() {
            quote! { Some(#enum_name) }
        } else {
            quote! { None }
        };

        quote! {
            #[automatically_derived]
            #[allow(unexpected_cfgs)]
            impl sea_orm::sea_query::ValueType for #ident {
                fn try_from(v: sea_orm::sea_query::Value) -> std::result::Result<Self, sea_orm::sea_query::ValueTypeErr> {
                    #value_type_try_from_impl
                }

                fn type_name() -> String {
                    #type_name_impl
                }

                fn array_type() -> sea_orm::sea_query::ArrayType {
                    #value_type_array_type
                }

                fn column_type() -> sea_orm::sea_query::ColumnType {
                    <Self as sea_orm::ActiveEnum>::db_type()
                        .get_column_type()
                        .to_owned()
                        .into()
                }

                fn enum_type_name() -> Option<&'static str> {
                    #enum_type_name
                }
            }
        }
    }

    fn try_getable_impl(&self) -> TokenStream {
        let ident = &self.ident;
        let try_get_by_impl = {
            let enum_name = &self.enum_name;
            if self.generate_enum_impls() {
                quote! {
                    let value: String = <String as sea_orm::TryGetable>::try_get_by(res, idx)?;
                    let value = sea_orm::sea_query::Enum {
                        type_name: #enum_name.into(),
                        value: value.into(),
                    };
                    <Self as sea_orm::ActiveEnum>::try_from_value(&value)
                        .map_err(sea_orm::TryGetError::DbErr)
                }
            } else {
                quote! {
                    let value = <<Self as sea_orm::ActiveEnum>::Value as sea_orm::TryGetable>::try_get_by(res, idx)?;
                    <Self as sea_orm::ActiveEnum>::try_from_value(&value)
                        .map_err(sea_orm::TryGetError::DbErr)
                }
            }
        };

        quote! {
            #[automatically_derived]
            impl sea_orm::TryGetable for #ident {
                fn try_get_by<I: sea_orm::ColIdx>(res: &sea_orm::QueryResult, idx: I) -> std::result::Result<Self, sea_orm::TryGetError> {
                    #try_get_by_impl
                }
            }
        }
    }

    fn active_enum_impl(&self) -> TokenStream {
        let ident = &self.ident;
        let enum_name_iden = format_ident!("{}Enum", ident);
        let rs_type = &self.rs_type;
        let variant_idents = &self.variant_idents;
        let variant_values = &self.variant_values;
        let to_value_body = self.to_value_impl();
        let column_type = {
            match &self.db_type {
                DbType::Enum => quote! {
                    Enum {
                        name: <Self as sea_orm::ActiveEnum>::name(),
                        variants: Self::iden_values(),
                    }
                },
                DbType::Other(db_type) => db_type.clone(),
            }
        };

        let val = if self.generate_enum_impls() {
            quote! { v.value.as_ref() }
        } else if self.is_string {
            quote! { <<Self as sea_orm::ActiveEnum>::Value as std::convert::AsRef<str>>::as_ref(v) }
        } else {
            quote! { v }
        };

        quote! {
            #[automatically_derived]
            impl sea_orm::ActiveEnum for #ident {
                type Value = #rs_type;

                type ValueVec = Vec<#rs_type>;

                fn name() -> sea_orm::sea_query::DynIden {
                    #enum_name_iden.into()
                }

                fn to_value(&self) -> <Self as sea_orm::ActiveEnum>::Value {
                    #to_value_body
                }

                fn try_from_value(v: &<Self as sea_orm::ActiveEnum>::Value) -> std::result::Result<Self, sea_orm::DbErr> {
                    match #val {
                        #( #variant_values => Ok(Self::#variant_idents), )*
                        _ => Err(sea_orm::DbErr::Type(format!(
                            "unexpected value for {} enum: {}",
                            stringify!(#ident),
                            #val
                        ))),
                    }
                }

                fn db_type() -> sea_orm::ColumnDef {
                    sea_orm::prelude::ColumnTypeTrait::def(sea_orm::ColumnType::#column_type)
                }
            }
        }
    }

    fn convert_impls(&self) -> TokenStream {
        let ident = &self.ident;

        if self.generate_enum_impls() {
            let enum_name = &self.enum_name;
            let variant_idents = &self.variant_idents;
            let variant_values = &self.variant_values;

            quote! {
                #[automatically_derived]
                impl std::convert::From<#ident> for sea_orm::sea_query::Enum {
                    fn from(source: #ident) -> Self {
                        let value = match source {
                            #( #ident::#variant_idents => #variant_values, )*
                        };
                        Self {
                            type_name: #enum_name.into(),
                            value: value.into(),
                        }
                    }
                }

                #[automatically_derived]
                impl std::convert::From<#ident> for sea_orm::sea_query::Value {
                    fn from(source: #ident) -> Self {
                        let enum_value = sea_orm::sea_query::Enum::from(source);
                        sea_orm::sea_query::Value::from(enum_value)
                    }
                }
            }
        } else {
            quote! {
                #[automatically_derived]
                impl std::convert::From<#ident> for sea_orm::sea_query::Value {
                    fn from(source: #ident) -> Self {
                        <#ident as sea_orm::ActiveEnum>::to_value(&source).into()
                    }
                }
            }
        }
    }

    fn try_getable_array_impl(&self) -> TokenStream {
        let ident = &self.ident;

        if cfg!(feature = "postgres-array") {
            quote!(
                #[automatically_derived]
                impl sea_orm::TryGetableArray for #ident {
                    fn try_get_by<I: sea_orm::ColIdx>(res: &sea_orm::QueryResult, index: I) -> std::result::Result<Vec<Self>, sea_orm::TryGetError> {
                        <<Self as sea_orm::ActiveEnum>::Value as sea_orm::ActiveEnumValue>::try_get_vec_by(res, index)?
                            .into_iter()
                            .map(|value| <Self as sea_orm::ActiveEnum>::try_from_value(&value).map_err(Into::into))
                            .collect()
                    }
                }
            )
        } else {
            quote!()
        }
    }

    fn expand(&self) -> TokenStream {
        let Self {
            ident,
            enum_name,
            variants,
            rename_all,
            ..
        } = self;

        let enum_name_iden = format_ident!("{}Enum", ident);

        let str_variants: Vec<String> = variants
            .iter()
            .filter_map(|variant| {
                variant
                    .string_value
                    .as_ref()
                    .map(|string_value| string_value.value())
                    .or(variant
                        .rename
                        .map(|rename| variant.ident.convert_case(Some(rename))))
                    .or_else(|| rename_all.map(|rule| variant.ident.convert_case(Some(rule))))
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
                impl sea_orm::Iden for #enum_variant_iden {
                    fn unquoted(&self) -> &str {
                        match self {
                            #(
                                Self::#enum_variants => #str_variants,
                            )*
                        }
                    }
                }

                #[automatically_derived]
                impl #ident {
                    #[doc = " Generated by sea-orm-macros"]
                    pub fn iden_values() -> Vec<sea_orm::sea_query::DynIden> {
                        <#enum_variant_iden as sea_orm::strum::IntoEnumIterator>::iter()
                            // TODO: Use DynIden constructor
                            .map(|v| sea_orm::sea_query::SeaRc::new(v) as sea_orm::sea_query::DynIden)
                            .collect()
                    }
                }
            )
        } else {
            quote!()
        };

        let not_u8_impl = if cfg!(feature = "postgres-array") {
            quote!(
                #[automatically_derived]
                impl sea_orm::sea_query::postgres_array::NotU8 for #ident {}
            )
        } else {
            quote!()
        };

        let value_type_impl = self.value_type_impl();
        let convert_impls = self.convert_impls();
        let nullable_impl = self.nullable_impl();
        let impl_try_getable_array = self.try_getable_array_impl();
        let active_enum_impl = self.active_enum_impl();
        let try_getable_impl = self.try_getable_impl();

        quote!(
            #[doc = " Generated by sea-orm-macros"]
            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct #enum_name_iden;

            #[automatically_derived]
            impl sea_orm::Iden for #enum_name_iden {
                fn unquoted(&self) -> &str {
                    #enum_name
                }
            }

            #impl_enum_variant_iden

            #active_enum_impl

            #impl_try_getable_array

            #convert_impls

            #try_getable_impl

            #value_type_impl

            #nullable_impl

            #[automatically_derived]
            impl sea_orm::IntoActiveValue<#ident> for #ident {
                fn into_active_value(self) -> sea_orm::ActiveValue<#ident> {
                    sea_orm::ActiveValue::set(self)
                }
            }

            #not_u8_impl
        )
    }
}

pub fn expand_derive_active_enum(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let ident_span = input.ident.span();

    match ActiveEnum::new(input) {
        Ok(model) => Ok(model.expand()),
        Err(Error::InputNotEnum) => Ok(quote_spanned! {
            ident_span => compile_error!("you can only derive ActiveEnum on enums");
        }),
        Err(Error::TT(token_stream)) => Ok(token_stream),
        Err(Error::Syn(e)) => Err(e),
    }
}
