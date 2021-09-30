use heck::{CamelCase, KebabCase, MixedCase, ShoutySnakeCase, SnakeCase};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{__private::ext::RepToTokensExt, quote};
use syn::spanned::Spanned;

const ATTR_NAME: &str = "sea_orm";

macro_rules! assert_attribute {
    ($e:expr, $err:expr, $input:expr) => {
        if !$e {
            return Err(syn::Error::new_spanned($input, $err));
        }
    };
}

macro_rules! fail {
    ($t:expr, $m:expr) => {
        return Err(syn::Error::new_spanned($t, $m));
    };
}

macro_rules! try_set {
    ($i:ident, $v:expr, $t:expr) => {
        match $i {
            None => $i = Some($v),
            Some(_) => fail!($t, "duplicate attribute"),
        }
    };
}

pub struct TypeName {
    pub val: String,
    pub span: Span,
}

impl TypeName {
    pub fn get(&self) -> TokenStream {
        let val = &self.val;
        quote! { #val }
    }
}

#[derive(Copy, Clone)]
pub enum RenameAll {
    Lower,
    Snake,
    Upper,
    ScreamingSnake,
    Kebab,
    Camel,
    Pascal,
}

pub struct ContainerAttributes {
    pub transparent: bool,
    pub type_name: Option<TypeName>,
    pub rename_all: Option<RenameAll>,
    pub repr: Option<Ident>,
    pub cast: bool,
}

pub struct ChildAttributes {
    pub rename: Option<String>,
    pub default: bool,
}

pub fn parse_container_attributes(input: &[syn::Attribute]) -> syn::Result<ContainerAttributes> {
    let mut transparent = None;
    let mut repr = None;
    let mut type_name = None;
    let mut rename_all = None;
    let mut cast = false;

    for attr in input
        .iter()
        .filter(|a| a.path.is_ident(ATTR_NAME) || a.path.is_ident("repr"))
    {
        let meta = attr
            .parse_meta()
            .map_err(|e| syn::Error::new_spanned(attr, e))?;
        match meta {
            syn::Meta::List(list) if list.path.is_ident(ATTR_NAME) => {
                let mut nested_items = list.nested.iter().peekable();
                while let Some(value) = nested_items.next() {
                    match value {
                        syn::NestedMeta::Meta(meta) => match meta {
                            syn::Meta::Path(p) if p.is_ident("transparent") => {
                                try_set!(transparent, true, value)
                            }

                            syn::Meta::NameValue(syn::MetaNameValue {
                                path,
                                lit: syn::Lit::Str(val),
                                ..
                            }) if path.is_ident("rename_all") => {
                                let val = match &*val.value() {
                                    "lowercase" => RenameAll::Lower,
                                    "snake_case" => RenameAll::Snake,
                                    "UPPERCASE" => RenameAll::Upper,
                                    "SCREAMING_SNAKE_CASE" => RenameAll::ScreamingSnake,
                                    "kebab-case" => RenameAll::Kebab,
                                    "camelCase" => RenameAll::Camel,
                                    "PascalCase" => RenameAll::Pascal,
                                    _ => fail!(meta, "unexpected value for rename_all"),
                                };

                                try_set!(rename_all, val, value)
                            }

                            syn::Meta::NameValue(syn::MetaNameValue {
                                path,
                                lit: syn::Lit::Str(val),
                                ..
                            }) if path.is_ident("type_name") => {
                                try_set!(
                                    type_name,
                                    TypeName {
                                        val: val.value(),
                                        span: value.span(),
                                    },
                                    value
                                );

                                if let Some(syn::NestedMeta::Meta(syn::Meta::Path(syn::Path {
                                    segments,
                                    ..
                                }))) = nested_items.peek()
                                {
                                    if segments
                                        .next()
                                        .and_then(|segment| segment.first())
                                        .map(|segment| segment.ident == "cast")
                                        .unwrap_or(false)
                                    {
                                        cast = true;
                                        nested_items.next();
                                    }
                                }
                            }

                            syn::Meta::NameValue(syn::MetaNameValue {
                                path,
                                lit: syn::Lit::Str(val),
                                ..
                            }) if path.is_ident("rename") => {
                                try_set!(
                                    type_name,
                                    TypeName {
                                        val: val.value(),
                                        span: value.span(),
                                    },
                                    value
                                )
                            }

                            u => fail!(u, "unexpected attribute"),
                        },
                        u => fail!(u, "unexpected attribute"),
                    }
                }
            }
            syn::Meta::List(list) if list.path.is_ident("repr") => {
                if list.nested.len() != 1 {
                    fail!(&list.nested, "expected one value")
                }
                match list.nested.first().unwrap() {
                    syn::NestedMeta::Meta(syn::Meta::Path(p)) if p.get_ident().is_some() => {
                        try_set!(repr, p.get_ident().unwrap().clone(), list);
                    }
                    u => fail!(u, "unexpected value"),
                }
            }
            _ => {}
        }
    }

    Ok(ContainerAttributes {
        transparent: transparent.unwrap_or(false),
        repr,
        type_name,
        rename_all,
        cast,
    })
}

pub fn parse_child_attributes(input: &[syn::Attribute]) -> syn::Result<ChildAttributes> {
    let mut rename = None;
    let mut default = false;

    for attr in input.iter().filter(|a| a.path.is_ident(ATTR_NAME)) {
        let meta = attr
            .parse_meta()
            .map_err(|e| syn::Error::new_spanned(attr, e))?;

        if let syn::Meta::List(list) = meta {
            for value in list.nested.iter() {
                match value {
                    syn::NestedMeta::Meta(meta) => match meta {
                        syn::Meta::NameValue(syn::MetaNameValue {
                            path,
                            lit: syn::Lit::Str(val),
                            ..
                        }) if path.is_ident("rename") => try_set!(rename, val.value(), value),
                        syn::Meta::Path(path) if path.is_ident("default") => default = true,
                        u => fail!(u, "unexpected attribute"),
                    },
                    u => fail!(u, "unexpected attribute"),
                }
            }
        }
    }

    Ok(ChildAttributes { rename, default })
}

pub fn check_transparent_attributes(
    input: &syn::DeriveInput,
    field: &syn::Field,
) -> syn::Result<ContainerAttributes> {
    let attributes = parse_container_attributes(&input.attrs)?;

    assert_attribute!(
        attributes.rename_all.is_none(),
        "unexpected #[sea_orm(rename_all = ..)]",
        field
    );

    let ch_attributes = parse_child_attributes(&field.attrs)?;

    assert_attribute!(
        ch_attributes.rename.is_none(),
        "unexpected #[sea_orm(rename = ..)]",
        field
    );

    Ok(attributes)
}

pub fn check_enum_attributes(input: &syn::DeriveInput) -> syn::Result<ContainerAttributes> {
    let attributes = parse_container_attributes(&input.attrs)?;

    assert_attribute!(
        !attributes.transparent,
        "unexpected #[sea_orm(transparent)]",
        input
    );

    Ok(attributes)
}

pub fn check_weak_enum_attributes(
    input: &syn::DeriveInput,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
) -> syn::Result<ContainerAttributes> {
    let attributes = check_enum_attributes(input)?;

    assert_attribute!(attributes.repr.is_some(), "expected #[repr(..)]", input);

    assert_attribute!(
        attributes.rename_all.is_none(),
        "unexpected #[sea_orm(c = ..)]",
        input
    );

    for variant in variants {
        let attributes = parse_child_attributes(&variant.attrs)?;

        assert_attribute!(
            attributes.rename.is_none(),
            "unexpected #[sea_orm(rename = ..)]",
            variant
        );
    }

    Ok(attributes)
}

pub fn check_strong_enum_attributes(
    input: &syn::DeriveInput,
    _variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
) -> syn::Result<ContainerAttributes> {
    let attributes = check_enum_attributes(input)?;

    assert_attribute!(attributes.repr.is_none(), "unexpected #[repr(..)]", input);

    Ok(attributes)
}

pub fn check_struct_attributes(
    input: &syn::DeriveInput,
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> syn::Result<ContainerAttributes> {
    let attributes = parse_container_attributes(&input.attrs)?;

    assert_attribute!(
        !attributes.transparent,
        "unexpected #[sea_orm(transparent)]",
        input
    );

    assert_attribute!(
        attributes.rename_all.is_none(),
        "unexpected #[sea_orm(rename_all = ..)]",
        input
    );

    assert_attribute!(attributes.repr.is_none(), "unexpected #[repr(..)]", input);

    for field in fields {
        let attributes = parse_child_attributes(&field.attrs)?;

        assert_attribute!(
            attributes.rename.is_none(),
            "unexpected #[sea_orm(rename = ..)]",
            field
        );
    }

    Ok(attributes)
}

pub(crate) fn rename_all(s: &str, pattern: RenameAll) -> String {
    match pattern {
        RenameAll::Lower => s.to_lowercase(),
        RenameAll::Snake => s.to_snake_case(),
        RenameAll::Upper => s.to_uppercase(),
        RenameAll::ScreamingSnake => s.to_shouty_snake_case(),
        RenameAll::Kebab => s.to_kebab_case(),
        RenameAll::Camel => s.to_mixed_case(),
        RenameAll::Pascal => s.to_camel_case(),
    }
}
