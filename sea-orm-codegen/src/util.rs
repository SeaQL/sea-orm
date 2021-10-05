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

pub(crate) const RUST_KEYWORDS: [&str; 52] = [
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "Self", "self", "static", "struct", "super", "trait", "true", "type", "union",
    "unsafe", "use", "where", "while", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "try", "typeof", "unsized", "virtual", "yield",
];
