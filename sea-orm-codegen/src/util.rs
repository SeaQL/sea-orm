use sea_query::TableRef;

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

pub(crate) const RUST_KEYWORDS: [&str; 49] = [
    "as", "async", "await", "break", "const", "continue", "dyn", "else", "enum", "extern", "false",
    "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "static", "struct", "super", "trait", "true", "type", "union", "unsafe", "use",
    "where", "while", "abstract", "become", "box", "do", "final", "macro", "override", "priv",
    "try", "typeof", "unsized", "virtual", "yield",
];

pub(crate) const RUST_SPECIAL_KEYWORDS: [&str; 3] = ["crate", "Self", "self"];

pub(crate) fn unpack_table_ref(table_ref: &TableRef) -> String {
    match table_ref {
        TableRef::Table(tbl)
        | TableRef::SchemaTable(_, tbl)
        | TableRef::DatabaseSchemaTable(_, _, tbl)
        | TableRef::TableAlias(tbl, _)
        | TableRef::SchemaTableAlias(_, tbl, _)
        | TableRef::DatabaseSchemaTableAlias(_, _, tbl, _)
        | TableRef::SubQuery(_, tbl)
        | TableRef::ValuesList(_, tbl)
        | TableRef::FunctionCall(_, tbl) => tbl.to_string(),
    }
}
