use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Field, GenericArgument, Ident, Meta, MetaNameValue, PathArguments, Type, TypePath,
    meta::ParseNestedMeta, punctuated::Punctuated, token::Comma,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CardinalityKind {
    Required,
    Optional,
}

impl CardinalityKind {
    pub(crate) fn is_optional(self) -> bool {
        matches!(self, Self::Optional)
    }

    pub(crate) fn has_one_target_type(self, entity: TokenStream) -> TokenStream {
        match self {
            Self::Required => quote!(#entity),
            Self::Optional => quote!(Option<#entity>),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompoundKind {
    HasOne(CardinalityKind),
    HasMany,
}

#[derive(Clone)]
pub(crate) struct CompoundType {
    pub(crate) kind: CompoundKind,
    pub(crate) entity: TypePath,
}

impl CompoundType {
    /// Returns whether the field uses the compound relation wrapper syntax.
    pub(crate) fn matches_type(type_path: &TypePath) -> bool {
        last_path_segment(type_path)
            .is_ok_and(|segment| segment.ident == "HasOne" || segment.ident == "HasMany")
    }

    /// Parses `HasOne<Entity>`, `HasOne<Option<Entity>>`, and `HasMany<Entity>`.
    pub(crate) fn from_type(type_path: &TypePath) -> syn::Result<Option<Self>> {
        let segment = last_path_segment(type_path)?;

        if segment.ident == "HasOne" {
            let PathArguments::AngleBracketed(args) = &segment.arguments else {
                return Err(syn::Error::new_spanned(
                    type_path,
                    "HasOne requires an Entity or Option<Entity> generic argument",
                ));
            };
            let (entity, cardinality) = has_one_target_arg(&args.args, type_path)?;
            return Ok(Some(Self {
                kind: CompoundKind::HasOne(cardinality),
                entity,
            }));
        }

        if segment.ident == "HasMany" {
            let PathArguments::AngleBracketed(args) = &segment.arguments else {
                return Err(syn::Error::new_spanned(
                    type_path,
                    "HasMany requires an Entity generic argument",
                ));
            };
            let Some(entity) = single_entity_arg(&args.args) else {
                return Err(syn::Error::new_spanned(
                    type_path,
                    "HasMany requires an Entity generic argument",
                ));
            };
            return Ok(Some(Self {
                kind: CompoundKind::HasMany,
                entity,
            }));
        }

        Ok(None)
    }

    pub(crate) fn is_has_one(&self) -> bool {
        matches!(self.kind, CompoundKind::HasOne(_))
    }

    pub(crate) fn cardinality(&self) -> Option<CardinalityKind> {
        match self.kind {
            CompoundKind::HasOne(cardinality) => Some(cardinality),
            CompoundKind::HasMany => None,
        }
    }

    pub(crate) fn is_self_entity(&self) -> bool {
        self.entity.path.segments.len() == 1
            && self
                .entity
                .path
                .segments
                .first()
                .is_some_and(|segment| segment.ident == "Entity")
    }
}

pub(crate) fn consume_meta(meta: ParseNestedMeta<'_>) {
    let _ = meta.value().and_then(|v| v.parse::<syn::Expr>());
}

/// Remove ignored fields and compound fields
pub(crate) fn field_not_ignored(field: &Field) -> bool {
    if let Type::Path(type_path) = &field.ty
        && CompoundType::matches_type(type_path)
    {
        return false;
    }

    !field_ignored(field)
}

fn field_ignored(field: &Field) -> bool {
    let mut ignored = false;

    for attr in &field.attrs {
        if !attr.path().is_ident("sea_orm") {
            continue;
        }

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("ignore") {
                ignored = true;
            } else {
                consume_meta(meta);
            }
            Ok(())
        });
    }

    ignored
}

pub(crate) fn validate_has_one_attr(ident: &Ident, compound: &CompoundType) -> syn::Result<()> {
    match compound.kind {
        CompoundKind::HasOne(CardinalityKind::Optional) => Ok(()),
        CompoundKind::HasOne(CardinalityKind::Required) => Err(syn::Error::new_spanned(
            ident,
            "has_one must be paired with HasOne<Option<Entity>>",
        )),
        CompoundKind::HasMany => Err(syn::Error::new_spanned(
            ident,
            "has_one must be paired with HasOne",
        )),
    }
}

fn single_entity_arg(args: &Punctuated<GenericArgument, Comma>) -> Option<TypePath> {
    if args.len() != 1 {
        return None;
    }
    match args.first() {
        Some(GenericArgument::Type(ty)) if is_entity_type(ty) => type_path(ty).ok().cloned(),
        _ => None,
    }
}

pub(crate) fn type_path(ty: &Type) -> syn::Result<&TypePath> {
    let Type::Path(type_path) = ty else {
        return Err(syn::Error::new_spanned(ty, "expected path type"));
    };
    Ok(type_path)
}

fn last_path_segment(type_path: &TypePath) -> syn::Result<&syn::PathSegment> {
    type_path
        .path
        .segments
        .last()
        .ok_or_else(|| syn::Error::new_spanned(type_path, "expected path type"))
}

fn is_entity_type(ty: &Type) -> bool {
    type_path(ty)
        .and_then(last_path_segment)
        .is_ok_and(|segment| segment.ident == "Entity")
}

fn has_one_target_arg(
    args: &Punctuated<GenericArgument, Comma>,
    field_type: &TypePath,
) -> syn::Result<(TypePath, CardinalityKind)> {
    if args.len() != 1 {
        return Err(syn::Error::new_spanned(
            field_type,
            "HasOne requires an Entity or Option<Entity> generic argument",
        ));
    }

    let Some(GenericArgument::Type(ty)) = args.first() else {
        return Err(syn::Error::new_spanned(
            field_type,
            "HasOne generic argument must be an Entity or Option<Entity>",
        ));
    };

    if is_entity_type(ty) {
        return Ok((type_path(ty)?.clone(), CardinalityKind::Required));
    }

    let type_path = type_path(ty).map_err(|_| {
        syn::Error::new_spanned(
            ty,
            "HasOne generic argument must be an Entity or Option<Entity>",
        )
    })?;
    let segment = last_path_segment(type_path)?;
    if segment.ident != "Option" {
        return Err(syn::Error::new_spanned(
            ty,
            "HasOne generic argument must be an Entity or Option<Entity>",
        ));
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return Err(syn::Error::new_spanned(
            ty,
            "HasOne generic argument must be an Entity or Option<Entity>",
        ));
    };
    let Some(entity) = single_entity_arg(&args.args) else {
        return Err(syn::Error::new_spanned(
            ty,
            "HasOne optional target must be Option<Entity>",
        ));
    };
    Ok((entity, CardinalityKind::Optional))
}

pub(crate) fn format_field_ident(field: &Field) -> Ident {
    field.ident.clone().unwrap()
}

pub(crate) fn trim_starting_raw_identifier<T>(string: T) -> String
where
    T: ToString,
{
    string
        .to_string()
        .trim_start_matches(RAW_IDENTIFIER)
        .to_string()
}

pub(crate) fn escape_rust_keyword<T>(string: T) -> String
where
    T: ToString,
{
    let string = string.to_string();
    if RUST_KEYWORDS.iter().any(|s| s.eq(&string)) {
        format!("r#{string}")
    } else if RUST_SPECIAL_KEYWORDS.iter().any(|s| s.eq(&string)) {
        format!("{string}_")
    } else {
        string
    }
}

/// Turn a string to PascalCase while escaping all special characters in ASCII words.
///
/// (camel_case is used here to match naming of heck.)
///
/// In ActiveEnum, string_value will be PascalCased and made
/// an identifier in {Enum}Variant.
///
/// However Rust only allows for XID_Start char followed by
/// XID_Continue characters as identifiers; this causes a few
/// problems:
///
/// - `string_value = ""` will cause a panic;
/// - `string_value` containing only non-alphanumerics will become `""`
///   and cause the above panic;
/// - `string_values`:
///      - `"A B"`
///      - `"A  B"`
///      - `"A_B"`
///      - `"A_ B"`
///
/// All shares the same identifier of `"AB"`;
///
/// This function does the PascelCase conversion with a few special escapes:
/// - Non-Unicode Standard Annex #31 compliant characters will converted to their hex notation;
/// - `"_"` into `"0x5F"`;
/// - `" "` into `"0x20"`;
/// - Empty strings will become special keyword of `"__Empty"`
///
/// Note that this does NOT address:
///
/// - case-sensitivity. String value "ABC" and "abc" remains
///   conflicted after .camel_case().
///
/// Example Conversions:
///
/// ```ignore
/// assert_eq!(camel_case_with_escaped_non_uax31(""), "__Empty");
/// assert_eq!(camel_case_with_escaped_non_uax31(" "), "_0x20");
/// assert_eq!(camel_case_with_escaped_non_uax31("  "), "_0x200x20");
/// assert_eq!(camel_case_with_escaped_non_uax31("_"), "_0x5F");
/// assert_eq!(camel_case_with_escaped_non_uax31("foobar"), "Foobar");
/// assert_eq!(camel_case_with_escaped_non_uax31("foo bar"), "Foo0x20bar");
/// ```
pub(crate) fn camel_case_with_escaped_non_uax31<T>(string: T) -> String
where
    T: ToString,
{
    let additional_chars_to_replace: [char; 2] = ['_', ' '];

    let mut rebuilt = string
        .to_string()
        .chars()
        .enumerate()
        .map(|(pos, char_)| {
            if !additional_chars_to_replace.contains(&char_)
                && match pos {
                    0 => unicode_ident::is_xid_start(char_),
                    _ => unicode_ident::is_xid_continue(char_),
                }
            {
                char_.to_string()
            } else {
                format!("{:#X}", char_ as u32)
            }
        })
        .reduce(
            // Join the "characters" (now strings)
            // back together
            |lhs, rhs| lhs + rhs.as_str(),
        )
        .map_or(
            // if string_value is ""
            // Make sure the default does NOT go through camel_case,
            // as the __ will be removed! The underscores are
            // what guarantees this being special case avoiding
            // all potential conflicts.
            String::from("__Empty"),
            |s| s.to_upper_camel_case(),
        );

    if rebuilt
        .chars()
        .next()
        .map(char::is_numeric)
        .unwrap_or(false)
    {
        rebuilt = String::from("_") + &rebuilt;
    }

    rebuilt
}

pub(crate) const RAW_IDENTIFIER: &str = "r#";

pub(crate) const RUST_KEYWORDS: [&str; 49] = [
    "as", "async", "await", "break", "const", "continue", "dyn", "else", "enum", "extern", "false",
    "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "static", "struct", "super", "trait", "true", "type", "union", "unsafe", "use",
    "where", "while", "abstract", "become", "box", "do", "final", "macro", "override", "priv",
    "try", "typeof", "unsized", "virtual", "yield",
];

pub(crate) const RUST_SPECIAL_KEYWORDS: [&str; 3] = ["crate", "Self", "self"];

pub(crate) trait GetMeta {
    fn exists(&self, k: &str) -> bool;
    fn get_as_kv(&self, k: &str) -> Option<String>;
    fn get_as_kv_with_ident(&self) -> Option<(Ident, String)>;
    fn get_list_args(&self, name: &str) -> Option<Punctuated<Meta, Comma>>;
}

impl GetMeta for Meta {
    fn exists(&self, key: &str) -> bool {
        let Meta::Path(path) = self else {
            return false;
        };
        path.is_ident(key)
    }

    fn get_as_kv(&self, key: &str) -> Option<String> {
        let Meta::NameValue(MetaNameValue {
            path,
            value: syn::Expr::Lit(exprlit),
            ..
        }) = self
        else {
            return None;
        };

        let syn::Lit::Str(litstr) = &exprlit.lit else {
            return None;
        };

        if path.is_ident(key) {
            Some(litstr.value())
        } else {
            None
        }
    }

    fn get_as_kv_with_ident(&self) -> Option<(Ident, String)> {
        let Meta::NameValue(MetaNameValue {
            path,
            value: syn::Expr::Lit(exprlit),
            ..
        }) = self
        else {
            return None;
        };

        let syn::Lit::Str(litstr) = &exprlit.lit else {
            return None;
        };

        path.get_ident()
            .map(|ident| (ident.clone(), litstr.value()))
    }

    fn get_list_args(&self, name: &str) -> Option<Punctuated<Meta, Comma>> {
        match self {
            Meta::List(list) if list.path.is_ident(name) => list
                .parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
                .ok(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_uax31_escape() {
        // Test empty string
        assert_eq!(camel_case_with_escaped_non_uax31(""), "__Empty");

        // Test additional_chars_to_replace (to_camel_case related characters)
        assert_eq!(camel_case_with_escaped_non_uax31(" "), "_0x20");

        // Test additional_chars_to_replace (multiples. ensure distinct from single)
        assert_eq!(camel_case_with_escaped_non_uax31("  "), "_0x200x20");

        // Test additional_chars_to_replace (udnerscores)
        assert_eq!(camel_case_with_escaped_non_uax31("_"), "_0x5F");

        // Test typical use case
        assert_eq!(camel_case_with_escaped_non_uax31("foobar"), "Foobar");

        // Test spaced words distinct from non-spaced
        assert_eq!(camel_case_with_escaped_non_uax31("foo bar"), "Foo0x20bar");

        // Test underscored words distinct from non-spaced and spaced
        assert_eq!(camel_case_with_escaped_non_uax31("foo_bar"), "Foo0x5Fbar");

        // Test leading numeric characters
        assert_eq!(camel_case_with_escaped_non_uax31("1"), "_0x31");

        // Test escaping also works on full string following lead numeric character
        // This was previously a fail condition.
        assert_eq!(
            camel_case_with_escaped_non_uax31("1 2 3"),
            "_0x310x2020x203"
        );

        assert_eq!(camel_case_with_escaped_non_uax31("씨오알엠"), "씨오알엠");

        assert_eq!(camel_case_with_escaped_non_uax31("A_B"), "A0x5Fb");

        assert_eq!(camel_case_with_escaped_non_uax31("AB"), "Ab");
    }
}
