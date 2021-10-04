use syn::{punctuated::Punctuated, token::Comma, Field, Meta};

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
    if is_rust_keyword(&string) {
        format!("r#{}", string)
    } else {
        string
    }
}

pub(crate) fn is_rust_keyword<T>(string: T) -> bool
where
    T: ToString,
{
    let string = string.to_string();
    RUST_KEYWORDS.iter().any(|s| s.eq(&string))
}

pub(crate) const RAW_IDENTIFIER: &str = "r#";

pub(crate) const RUST_KEYWORDS: [&str; 52] = [
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "Self", "self", "static", "struct", "super", "trait", "true", "type", "union",
    "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "try", "typeof", "unsized", "virtual", "yield",
];
