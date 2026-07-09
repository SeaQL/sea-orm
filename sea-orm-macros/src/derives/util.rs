use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Field, GenericArgument, Ident, Meta, MetaNameValue, PathArguments, Type, TypePath,
    meta::ParseNestedMeta, punctuated::Punctuated, token::Comma,
};

pub(crate) fn async_token() -> TokenStream {
    if cfg!(feature = "async") {
        quote!(async)
    } else {
        quote!()
    }
}

pub(crate) fn await_token() -> TokenStream {
    if cfg!(feature = "async") {
        quote!(.await)
    } else {
        quote!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CardinalityKind {
    Required,
    Optional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompoundKind {
    BelongsTo(CardinalityKind),
    HasOne,
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
        last_path_segment(type_path).is_ok_and(|segment| {
            matches!(
                segment.ident.to_string().as_str(),
                "BelongsTo" | "HasOne" | "HasMany"
            )
        })
    }

    /// Parses `BelongsTo<Entity>`, `BelongsTo<Option<Entity>>`, `HasOne<Entity>`, and `HasMany<Entity>`.
    pub(crate) fn from_type(type_path: &TypePath) -> syn::Result<Option<Self>> {
        let segment = last_path_segment(type_path)?;

        match segment.ident.to_string().as_str() {
            "BelongsTo" => {
                let PathArguments::AngleBracketed(args) = &segment.arguments else {
                    return Err(syn::Error::new_spanned(
                        type_path,
                        "BelongsTo requires an Entity or Option<Entity> generic argument",
                    ));
                };
                let mut args = args.args.iter();
                let Some(GenericArgument::Type(ty)) = args.next() else {
                    return Err(syn::Error::new_spanned(
                        type_path,
                        "BelongsTo generic argument must be an Entity or Option<Entity>",
                    ));
                };
                if args.next().is_some() {
                    return Err(syn::Error::new_spanned(
                        type_path,
                        "BelongsTo requires an Entity or Option<Entity> generic argument",
                    ));
                }
                let Type::Path(ty_path) = ty else {
                    return Err(syn::Error::new_spanned(
                        ty,
                        "BelongsTo generic argument must be an Entity or Option<Entity>",
                    ));
                };
                let target_segment = last_path_segment(ty_path)?;
                match (
                    target_segment.ident.to_string().as_str(),
                    &target_segment.arguments,
                ) {
                    ("Entity", _) => Ok(Some(Self {
                        kind: CompoundKind::BelongsTo(CardinalityKind::Required),
                        entity: ty_path.clone(),
                    })),
                    ("Option", PathArguments::AngleBracketed(args)) => {
                        let Some(entity) = entity_generic_arg(&args.args) else {
                            return Err(syn::Error::new_spanned(
                                ty,
                                "BelongsTo optional target must be Option<Entity>",
                            ));
                        };
                        Ok(Some(Self {
                            kind: CompoundKind::BelongsTo(CardinalityKind::Optional),
                            entity,
                        }))
                    }
                    _ => Err(syn::Error::new_spanned(
                        ty,
                        "BelongsTo generic argument must be an Entity or Option<Entity>",
                    )),
                }
            }
            "HasOne" => {
                let PathArguments::AngleBracketed(args) = &segment.arguments else {
                    return Err(syn::Error::new_spanned(
                        type_path,
                        "HasOne requires an Entity generic argument",
                    ));
                };
                let Some(entity) = entity_generic_arg(&args.args) else {
                    return Err(syn::Error::new_spanned(
                        type_path,
                        "HasOne requires an Entity generic argument",
                    ));
                };
                Ok(Some(Self {
                    kind: CompoundKind::HasOne,
                    entity,
                }))
            }
            "HasMany" => {
                let PathArguments::AngleBracketed(args) = &segment.arguments else {
                    return Err(syn::Error::new_spanned(
                        type_path,
                        "HasMany requires an Entity generic argument",
                    ));
                };
                let Some(entity) = entity_generic_arg(&args.args) else {
                    return Err(syn::Error::new_spanned(
                        type_path,
                        "HasMany requires an Entity generic argument",
                    ));
                };
                Ok(Some(Self {
                    kind: CompoundKind::HasMany,
                    entity,
                }))
            }
            _ => Ok(None),
        }
    }
}

fn last_path_segment(type_path: &TypePath) -> syn::Result<&syn::PathSegment> {
    type_path
        .path
        .segments
        .last()
        .ok_or_else(|| syn::Error::new_spanned(type_path, "expected path type"))
}

pub(crate) fn is_self_entity(entity: &TypePath) -> bool {
    entity.path.segments.len() == 1
        && last_path_segment(entity).is_ok_and(|segment| segment.ident == "Entity")
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

fn entity_generic_arg(args: &Punctuated<GenericArgument, Comma>) -> Option<TypePath> {
    if args.len() != 1 {
        return None;
    }
    match args.first() {
        Some(GenericArgument::Type(Type::Path(type_path))) if is_entity_type(type_path) => {
            Some(type_path.clone())
        }
        _ => None,
    }
}

fn is_entity_type(type_path: &TypePath) -> bool {
    last_path_segment(type_path).is_ok_and(|segment| segment.ident == "Entity")
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

pub(crate) fn combine_error(acc: &mut Option<syn::Error>, error: syn::Error) {
    if let Some(acc) = acc {
        acc.combine(error);
    } else {
        *acc = Some(error)
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
