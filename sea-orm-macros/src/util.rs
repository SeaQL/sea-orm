use heck::CamelCase;
use quote::format_ident;
use syn::{punctuated::Punctuated, token::Comma, Field, Ident, Meta};
use unicode_ident;

pub(crate) fn field_not_ignored(field: &Field) -> bool {
    for attr in field.attrs.iter() {
        if let Some(ident) = attr.path.get_ident() {
            if ident != "sea_orm" {
                continue;
            }
        } else {
            continue;
        }

        if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
            for meta in list.iter() {
                if let Meta::Path(path) = meta {
                    if let Some(name) = path.get_ident() {
                        if name == "ignore" {
                            return false;
                        }
                    }
                }
            }
        }
    }
    true
}

pub(crate) fn format_field_ident(field: Field) -> Ident {
    format_ident!("{}", field.ident.unwrap().to_string())
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
        format!("r#{}", string)
    } else if RUST_SPECIAL_KEYWORDS.iter().any(|s| s.eq(&string)) {
        format!("{}_", string)
    } else {
        string
    }
}

/// string_value in ActiveEnum will be CamelCased and become
/// an identifier in {Enum}Variant.
/// However Rust only allows for XID_Start char followed by
/// XID_Continue characters as identifiers; this causes a few
/// problems:
///
/// - string_value = "" will cause a panic;
/// - string_value containing only non-alphanumerics will become ""
///   and cause the above panic;
/// - string_values:
///      - "AB"
///      - "A B"
///      - "A  B"
///      - "A_B"
///      - "A_ B"
///   shares the same identifier of "AB";
///
/// What this does NOT address
/// - case-sensitivity. String value "ABC" and "abc" remains
///   conflicted after .camel_case().
pub(crate) fn camel_case_with_escaped_non_xid<T>(string: T) -> String
where
    T: ToString,
{   
    let additional_chars_to_replace: [char; 2] = ['_', ' '];

    string
    .to_string()
    .chars()
    .into_iter()
    .enumerate()
    .map(
        | (pos, char_) | {
            if  !additional_chars_to_replace.contains(&char_)
                && match pos {
                    0 => unicode_ident::is_xid_start(char_) ,
                    _ => unicode_ident::is_xid_continue(char_),
                } {
                    char_.to_string()
                } else {
                    format!("{:#X}", char_ as u32)
                }
        }
    )
    .reduce(
        // Join the "characters" (now strings)
        // back together
        | lhs, rhs | {
            lhs + rhs.as_str()
        }
    )
    .map_or(
        // if string_value is ""
        // Make sure the default does NOT go through camel_case,
        // as the __ will be removed! The underscores are
        // what guarantees this being special case avoiding
        // all potential conflicts.
        String::from("__Empty"),
        |s| s.to_camel_case(),
    )
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
